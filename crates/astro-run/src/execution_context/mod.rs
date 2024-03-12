mod builder;
mod condition_matcher;

pub use self::builder::ExecutionContextBuilder;
use crate::{
  AstroRunSignal, Condition, Context, Error, Job, JobId, JobRunResult, Result, RunResult, Runner,
  SharedPluginDriver, Signal, SignalManager, Step, StepRunResult, StreamExt, Workflow, WorkflowLog,
  WorkflowRunResult, WorkflowState, WorkflowStateEvent,
};
use std::sync::Arc;
use tokio::time;

#[derive(Clone)]
pub struct ExecutionContext {
  runner: Arc<Box<dyn Runner>>,
  plugin_driver: SharedPluginDriver,
  signal_manager: SignalManager,
  condition_matcher: condition_matcher::ConditionMatcher,
}

impl ExecutionContext {
  pub fn builder() -> ExecutionContextBuilder {
    ExecutionContextBuilder::new()
  }

  pub async fn run(&self, step: Step) -> StepRunResult {
    let step_id = step.id.clone();
    let timeout = step.timeout;

    let started_at = chrono::Utc::now();

    let event = crate::RunStepEvent {
      payload: step.clone(),
      workflow_event: self.condition_matcher.event.clone(),
    };
    self.plugin_driver.on_run_step(event.clone());
    self.runner.on_run_step(event.clone());

    // Queued
    let event = WorkflowStateEvent::StepStateUpdated {
      id: step_id.clone(),
      state: WorkflowState::Queued,
    };
    self.plugin_driver.on_state_change(event.clone());
    self.runner.on_state_change(event);

    // Job signal
    let job_signal = self
      .signal_manager
      .get_signal(&step.id.job_id())
      .expect("Missing job signal");

    // Step signal
    let signal = AstroRunSignal::new();

    let mut receiver = match self
      .runner
      .run(Context {
        id: step_id.to_string(),
        signal: signal.clone(),
        command: step.into(),
        event: self.condition_matcher.event.clone(),
      })
      .await
    {
      Ok(receiver) => receiver,
      Err(err) => {
        let completed_at = chrono::Utc::now();
        let duration = completed_at - started_at;
        log::error!(
          "Step {:?} failed with error {:?} in {} seconds",
          step_id,
          err,
          duration.num_seconds()
        );

        let event = WorkflowStateEvent::StepStateUpdated {
          id: step_id.clone(),
          state: WorkflowState::Failed,
        };
        self.plugin_driver.on_state_change(event.clone());
        self.runner.on_state_change(event);

        let result = StepRunResult {
          id: step_id,
          state: WorkflowState::Failed,
          exit_code: Some(1),
          started_at: Some(started_at),
          completed_at: Some(completed_at),
        };

        self.plugin_driver.on_step_completed(result.clone());
        self.runner.on_step_completed(result.clone());

        return result;
      }
    };

    let event = WorkflowStateEvent::StepStateUpdated {
      id: step_id.clone(),
      state: WorkflowState::InProgress,
    };
    self.plugin_driver.on_state_change(event.clone());
    self.runner.on_state_change(event);

    loop {
      tokio::select! {
        // Timeout
        _ = time::sleep(timeout) => {
          // Ignore error
          signal.timeout().ok();
        }
        s = job_signal.recv() => {
          match s {
            Signal::Cancel => {
              signal.cancel().ok();
            }
            Signal::Timeout => {
              signal.timeout().ok();
            }
          }
        }
        received = receiver.next() => {
          if let Some(log) = received {
            let log = WorkflowLog {
              step_id: step_id.clone(),
              log_type: log.log_type,
              message: log.message,
              time: chrono::Utc::now(),
            };

            self.plugin_driver.on_log(log.clone());
            self.runner.on_log(log);
          } else {
            break;
          }
        }
      }
    }

    let res = receiver
      .result()
      // NOTE: This should never happen
      .ok_or(Error::internal_runtime_error(
        "Missing result from runner. This is a bug in the runner implementation.",
      ))
      .unwrap();

    let completed_at = chrono::Utc::now();
    let duration = completed_at - started_at;
    log::trace!(
      "Step {:?} finished with result {:?} in {} seconds",
      step_id,
      res,
      duration.num_seconds()
    );

    let res = match res {
      RunResult::Succeeded => StepRunResult {
        id: step_id.clone(),
        state: WorkflowState::Succeeded,
        exit_code: None,
        started_at: Some(started_at),
        completed_at: Some(completed_at),
      },
      RunResult::Failed { exit_code } => StepRunResult {
        id: step_id.clone(),
        state: WorkflowState::Failed,
        exit_code: Some(exit_code),
        started_at: Some(started_at),
        completed_at: Some(completed_at),
      },
      RunResult::Cancelled => StepRunResult {
        id: step_id.clone(),
        state: WorkflowState::Cancelled,
        exit_code: None,
        started_at: Some(started_at),
        completed_at: Some(completed_at),
      },
    };

    let event = WorkflowStateEvent::StepStateUpdated {
      id: step_id.clone(),
      state: res.state.clone(),
    };
    self.plugin_driver.on_state_change(event.clone());
    self.runner.on_state_change(event);

    self.plugin_driver.on_step_completed(res.clone());
    self.runner.on_step_completed(res.clone());

    log::trace!("Step {:?} completed", step_id);

    res
  }

  pub async fn is_match(&self, condition: &Condition) -> bool {
    self.condition_matcher.is_match(condition).await
  }

  pub fn on_run_workflow(&self, workflow: Workflow) {
    let event = crate::RunWorkflowEvent {
      payload: workflow,
      workflow_event: self.condition_matcher.event.clone(),
    };
    self.plugin_driver.on_run_workflow(event.clone());
    self.runner.on_run_workflow(event);
  }

  pub fn on_run_job(&self, job: Job) {
    self
      .signal_manager
      .register_signal(job.id.clone(), AstroRunSignal::new());

    let event = crate::RunJobEvent {
      payload: job,
      workflow_event: self.condition_matcher.event.clone(),
    };
    self.plugin_driver.on_run_job(event.clone());
    self.runner.on_run_job(event);
  }

  pub fn on_state_change(&self, event: WorkflowStateEvent) {
    self.plugin_driver.on_state_change(event.clone());
    self.runner.on_state_change(event);
  }

  pub fn on_job_completed(&self, result: JobRunResult) {
    self.signal_manager.unregister_signal(&result.id);

    self.plugin_driver.on_job_completed(result.clone());
    self.runner.on_job_completed(result);
  }

  pub fn on_workflow_completed(&self, result: WorkflowRunResult) {
    self.plugin_driver.on_workflow_completed(result.clone());
    self.runner.on_workflow_completed(result);
  }

  pub fn cancel_job(&self, job_id: &JobId) -> Result<()> {
    self.signal_manager.cancel_job(job_id)
  }
}

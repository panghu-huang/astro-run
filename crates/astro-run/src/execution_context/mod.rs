mod builder;

use self::builder::ExecutionContextBuilder;
use crate::{
  AstroRunSharedState, AstroRunSignal, Command, Context, Error, Job, JobRunResult, RunResult,
  Runner, StepRunResult, StreamExt, Workflow, WorkflowLog, WorkflowRunResult, WorkflowState,
  WorkflowStateEvent,
};
use std::sync::Arc;
use tokio::time;

#[derive(Clone)]
pub struct ExecutionContext {
  // pub workflow_shared: WorkflowShared,
  runner: Arc<Box<dyn Runner>>,
  shared_state: AstroRunSharedState,
}

impl ExecutionContext {
  pub fn builder() -> ExecutionContextBuilder {
    ExecutionContextBuilder::new()
  }

  pub async fn run(&self, command: Command) -> StepRunResult {
    let step_id = command.id.clone();
    let timeout = command.timeout;

    let plugin_manager = self.shared_state.plugins();

    let started_at = chrono::Utc::now();

    plugin_manager.on_run_step(command.clone());
    self.runner.on_run_step(command.clone());

    let event = WorkflowStateEvent::StepStateUpdated {
      id: step_id.clone(),
      state: WorkflowState::InProgress,
    };
    plugin_manager.on_state_change(event.clone());
    self.runner.on_state_change(event);

    let signal = AstroRunSignal::new();

    let mut receiver = match self.runner.run(Context {
      id: step_id.to_string(),
      signal: signal.clone(),
      command,
    }) {
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
        plugin_manager.on_state_change(event.clone());
        self.runner.on_state_change(event);

        let result = StepRunResult {
          id: step_id,
          state: WorkflowState::Failed,
          exit_code: Some(1),
          started_at: Some(started_at),
          completed_at: Some(completed_at),
        };

        plugin_manager.on_step_completed(result.clone());
        self.runner.on_step_completed(result.clone());

        return result;
      }
    };

    loop {
      tokio::select! {
        // Timeout
        _ = time::sleep(timeout) => {
          signal.timeout();
        }
        // _ = async {} => {
        //   signal.timeout();
        // }
        received = receiver.next() => {
          if let Some(log) = received {
            let log = WorkflowLog {
              step_id: step_id.clone(),
              log_type: log.log_type,
              message: log.message,
              time: chrono::Utc::now(),
            };

            plugin_manager.on_log(log.clone());
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
    plugin_manager.on_state_change(event.clone());
    self.runner.on_state_change(event);

    plugin_manager.on_step_completed(res.clone());
    self.runner.on_step_completed(res.clone());

    log::trace!("Step {:?} completed", step_id);

    res
  }

  pub fn on_run_workflow(&self, workflow: Workflow) {
    self.shared_state.on_run_workflow(workflow.clone());
    self.runner.on_run_workflow(workflow);
  }

  pub fn on_run_job(&self, job: Job) {
    self.shared_state.on_run_job(job.clone());
    self.runner.on_run_job(job);
  }

  pub fn on_state_change(&self, event: WorkflowStateEvent) {
    self.shared_state.on_state_change(event.clone());
    self.runner.on_state_change(event);
  }

  pub fn on_job_completed(&self, result: JobRunResult) {
    self.shared_state.on_job_completed(result.clone());
    self.runner.on_job_completed(result);
  }

  pub fn on_workflow_completed(&self, result: WorkflowRunResult) {
    self.shared_state.on_workflow_completed(result.clone());
    self.runner.on_workflow_completed(result);
  }
}

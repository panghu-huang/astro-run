mod builder;
// mod workflow_shared;

use self::builder::ExecutionContextBuilder;
use crate::{AstroRunSharedState, Job, StepRunResult, Workflow};
use astro_run_shared::{
  Command, Context, Error, RunResult, Runner, StreamExt, WorkflowLog, WorkflowState,
  WorkflowStateEvent,
};
use std::sync::Arc;

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

  pub async fn run(&self, command: Command) -> astro_run_shared::Result<StepRunResult> {
    let step_id = command.id.clone();

    let plugin_manager = self.shared_state.plugins();

    let started_at = chrono::Utc::now();
    plugin_manager.on_state_change(WorkflowStateEvent::StepStateUpdated {
      id: step_id.clone(),
      state: WorkflowState::InProgress,
    });

    let mut receiver = match self.runner.run(Context { command }) {
      Ok(receiver) => receiver,
      Err(err) => {
        let ended_at = chrono::Utc::now();
        let duration = ended_at - started_at;
        log::error!(
          "Step {:?} failed with error {:?} in {} seconds",
          step_id,
          err,
          duration.num_seconds()
        );

        plugin_manager.on_state_change(WorkflowStateEvent::StepStateUpdated {
          id: step_id.clone(),
          state: WorkflowState::Failed,
        });

        return Ok(StepRunResult {
          state: WorkflowState::Failed,
          exit_code: Some(1),
          started_at: Some(started_at),
          ended_at: Some(ended_at),
        });
      }
    };

    while let Some(log) = receiver.next().await {
      let log = WorkflowLog {
        step_id: step_id.clone(),
        log_type: log.log_type,
        message: log.message,
        time: chrono::Utc::now(),
      };

      plugin_manager.on_log(log);
    }

    let res = receiver.result().ok_or(Error::internal_runtime_error(
      "Missing result from runner. This is a bug in the runner implementation.",
    ))?;

    let ended_at = chrono::Utc::now();
    let duration = ended_at - started_at;
    log::info!(
      "Step {:?} finished with result {:?} in {} seconds",
      step_id,
      res,
      duration.num_seconds()
    );

    let res = match res {
      RunResult::Succeeded => StepRunResult {
        state: WorkflowState::Succeeded,
        exit_code: None,
        started_at: Some(started_at),
        ended_at: Some(ended_at),
      },
      RunResult::Failed { exit_code } => StepRunResult {
        state: WorkflowState::Failed,
        exit_code: Some(exit_code),
        started_at: Some(started_at),
        ended_at: Some(ended_at),
      },
      RunResult::Cancelled => StepRunResult {
        state: WorkflowState::Cancelled,
        exit_code: None,
        started_at: Some(started_at),
        ended_at: Some(ended_at),
      },
    };

    Ok(res)
  }

  pub fn on_run_workflow(&self, workflow: Workflow) {
    self.shared_state.on_run_workflow(workflow);
  }

  pub fn on_run_job(&self, job: Job) {
    self.shared_state.on_run_job(job);
  }

  pub fn on_state_change(&self, event: WorkflowStateEvent) {
    self.shared_state.on_state_change(event);
  }
}

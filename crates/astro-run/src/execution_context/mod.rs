mod builder;
mod workflow_shared;

use self::{builder::ExecutionContextBuilder, workflow_shared::WorkflowShared};
use crate::{AstroRunSharedState, StepRunResult};
use astro_run_shared::{
  Command, Context, Error, RunResult, Runner, StreamExt, WorkflowLog, WorkflowState,
  WorkflowStateEvent,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ExecutionContext {
  pub workflow_shared: WorkflowShared,
  runner: Arc<Box<dyn Runner>>,
  shared_state: AstroRunSharedState,
}

impl ExecutionContext {
  pub fn builder() -> ExecutionContextBuilder {
    ExecutionContextBuilder::new()
  }

  pub async fn run(&self, command: Command) -> astro_run_shared::Result<StepRunResult> {
    let (workflow_id, job_key, step_number) = command.id.clone();

    let plugin_manager = self.shared_state.plugins();

    let started_at = chrono::Utc::now();
    plugin_manager.on_state_change(WorkflowStateEvent::StepStateUpdated {
      workflow_id: workflow_id.clone(),
      job_id: job_key.clone(),
      number: step_number,
      state: WorkflowState::InProgress,
    });

    let mut receiver = match self.runner.run(Context { command }) {
      Ok(receiver) => receiver,
      Err(err) => {
        let ended_at = chrono::Utc::now();
        let duration = ended_at - started_at;
        log::error!(
          "Step {} of job {} in workflow {} failed with error {:?} in {} seconds",
          step_number,
          job_key,
          workflow_id,
          err,
          duration.num_seconds()
        );

        plugin_manager.on_state_change(WorkflowStateEvent::StepStateUpdated {
          workflow_id: workflow_id.clone(),
          job_id: job_key.clone(),
          number: step_number,
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
        workflow_id: workflow_id.clone(),
        job_key: job_key.clone(),
        step_number,
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
      "Step {} of job {} in workflow {} finished with result {:?} in {} seconds",
      step_number,
      job_key,
      workflow_id,
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
}

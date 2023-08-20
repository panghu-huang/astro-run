use crate::{
  context, stream::StreamReceiver, Job, JobRunResult, Step, StepRunResult, Workflow, WorkflowLog,
  WorkflowLogType, WorkflowRunResult, WorkflowStateEvent,
};
pub use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Clone, PartialEq)]
pub enum RunResult {
  Succeeded,
  Failed { exit_code: i32 },
  Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Log {
  pub log_type: WorkflowLogType,
  pub message: String,
}

impl Log {
  pub fn log(message: impl Into<String>) -> Self {
    Self {
      log_type: WorkflowLogType::Log,
      message: message.into(),
    }
  }

  pub fn error(message: impl Into<String>) -> Self {
    Self {
      log_type: WorkflowLogType::Error,
      message: message.into(),
    }
  }

  pub fn is_error(&self) -> bool {
    self.log_type == WorkflowLogType::Error
  }
}

pub type RunResponse = crate::Result<StreamReceiver>;

pub trait Runner: Send + Sync {
  fn on_run_workflow(&self, _workflow: Workflow) {}
  fn on_run_job(&self, _job: Job) {}
  fn on_run_step(&self, _step: Step) {}
  fn on_step_completed(&self, _result: StepRunResult) {}
  fn on_job_completed(&self, _result: JobRunResult) {}
  fn on_workflow_completed(&self, _result: WorkflowRunResult) {}
  fn on_state_change(&self, _event: WorkflowStateEvent) {}
  fn on_log(&self, _log: WorkflowLog) {}
  fn run(&self, config: context::Context) -> RunResponse;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_log() {
    let log = Log::log("test");
    assert_eq!(log.log_type, WorkflowLogType::Log);
    assert_eq!(log.message, "test");
    assert!(!log.is_error());

    let log = Log::error("test");
    assert_eq!(log.log_type, WorkflowLogType::Error);
    assert_eq!(log.message, "test");
    assert!(log.is_error());
  }
}

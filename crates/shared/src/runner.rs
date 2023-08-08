use crate::{context, stream::StreamReceiver, WorkflowLogType};
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
  fn run(&self, config: context::Context) -> RunResponse;
}

use crate::WorkflowLogType;
pub use std::process::ExitStatus;

#[derive(Debug, Clone)]
pub struct Log {
  pub log_type: WorkflowLogType,
  pub message: String,
}

impl Log {
  pub fn log(message: String) -> Self {
    Self {
      log_type: WorkflowLogType::Log,
      message,
    }
  }

  pub fn error(message: String) -> Self {
    Self {
      log_type: WorkflowLogType::Error,
      message,
    }
  }

  pub fn is_error(&self) -> bool {
    self.log_type == WorkflowLogType::Error
  }
}

#[derive(Debug, Clone)]
pub enum StreamResponse {
  Log(Log),
  End(ExitStatus),
}

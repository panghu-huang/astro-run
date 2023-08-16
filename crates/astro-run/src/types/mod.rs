mod envs;
mod error;
mod id;
mod results;
mod workflow_event;
mod workflow_state;
mod workflow_state_event;

pub use envs::*;
pub use error::*;
pub use id::*;
pub use results::*;
pub use workflow_event::*;
pub use workflow_state::*;
pub use workflow_state_event::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowLogType {
  Error,
  Log,
}

impl ToString for WorkflowLogType {
  fn to_string(&self) -> String {
    match self {
      WorkflowLogType::Error => "error".to_string(),
      WorkflowLogType::Log => "log".to_string(),
    }
  }
}

impl From<String> for WorkflowLogType {
  fn from(s: String) -> Self {
    match s.as_str() {
      "error" => WorkflowLogType::Error,
      "log" => WorkflowLogType::Log,
      _ => WorkflowLogType::Log,
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowLog {
  pub step_id: StepId,
  pub log_type: WorkflowLogType,
  pub message: String,
  pub time: chrono::DateTime<chrono::Utc>,
}

impl Default for WorkflowLog {
  fn default() -> Self {
    WorkflowLog {
      step_id: StepId::default(),
      log_type: WorkflowLogType::Log,
      message: "".to_string(),
      time: chrono::Utc::now(),
    }
  }
}

impl WorkflowLog {
  pub fn is_error(&self) -> bool {
    self.log_type == WorkflowLogType::Error
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_workflow_log_type() {
    assert_eq!(
      WorkflowLogType::Error,
      WorkflowLogType::from("error".to_string())
    );
    assert_eq!(
      WorkflowLogType::Log,
      WorkflowLogType::from("log".to_string())
    );
    assert_eq!(
      WorkflowLogType::Log,
      WorkflowLogType::from("unknown".to_string())
    );
  }

  #[test]
  fn test_workflow_log_type_to_string() {
    assert_eq!("error", WorkflowLogType::Error.to_string());
    assert_eq!("log", WorkflowLogType::Log.to_string());
  }
}

mod envs;
mod workflow_event;
mod workflow_state;
mod workflow_state_event;

pub use envs::*;
pub use workflow_event::*;
pub use workflow_state::*;
pub use workflow_state_event::*;

use serde::{Deserialize, Serialize};

pub type Id = String;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Secret {
  pub key: String,
  pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Volume {
  pub from: String,
  pub to: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowLogType {
  Error,
  Log,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowLog {
  pub workflow_id: Id,
  pub job_key: String,
  pub step_number: usize,
  pub log_type: WorkflowLogType,
  pub message: String,
  pub time: chrono::DateTime<chrono::Utc>,
}

impl Default for WorkflowLog {
  fn default() -> Self {
    WorkflowLog {
      workflow_id: "".to_string(),
      job_key: "".to_string(),
      step_number: 0,
      log_type: WorkflowLogType::Log,
      message: "".to_string(),
      time: chrono::Utc::now(),
    }
  }
}

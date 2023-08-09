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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkflowId(pub Id);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JobId(pub Id, pub Id);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StepId(pub Id, pub Id, pub usize);

impl WorkflowId {
  pub fn new(id: impl Into<String>) -> Self {
    WorkflowId(id.into())
  }
}

impl JobId {
  pub fn new(workflow_id: impl Into<String>, job_id: impl Into<String>) -> Self {
    JobId(workflow_id.into(), job_id.into())
  }
}

impl StepId {
  pub fn new(workflow_id: impl Into<String>, job_id: impl Into<String>, step_id: usize) -> Self {
    StepId(workflow_id.into(), job_id.into(), step_id)
  }
}

impl Default for WorkflowId {
  fn default() -> Self {
    WorkflowId("".to_string())
  }
}

impl Default for JobId {
  fn default() -> Self {
    JobId("".to_string(), "".to_string())
  }
}

impl Default for StepId {
  fn default() -> Self {
    StepId("".to_string(), "".to_string(), 0)
  }
}
// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
// pub struct Secret {
//   pub key: String,
//   pub value: String,
// }

// #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
// pub struct Volume {
//   pub from: String,
//   pub to: String,
// }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowLogType {
  Error,
  Log,
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_workflow_id() {
    let workflow_id = WorkflowId::new("test");
    assert_eq!(workflow_id, WorkflowId("test".to_string()));
  }

  #[test]
  fn test_job_id() {
    let job_id = JobId::new("test", "test");
    assert_eq!(job_id, JobId("test".to_string(), "test".to_string()));
  }

  #[test]
  fn test_step_id() {
    let step_id = StepId::new("test", "test", 0);
    assert_eq!(step_id, StepId("test".to_string(), "test".to_string(), 0));
  }
}

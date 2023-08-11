mod envs;
mod error;
mod results;
mod workflow_event;
mod workflow_state;
mod workflow_state_event;

pub use envs::*;
pub use error::*;
pub use results::*;
pub use workflow_event::*;
pub use workflow_state::*;
pub use workflow_state_event::*;

use serde::{Deserialize, Serialize};

pub type Id = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct WorkflowId(Id);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct JobId(Id, Id);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct StepId(Id, Id, usize);

impl WorkflowId {
  pub fn new(id: impl Into<String>) -> Self {
    WorkflowId(id.into())
  }

  pub fn inner(&self) -> Id {
    self.0.clone()
  }
}

impl JobId {
  pub fn new(workflow_id: impl Into<String>, job_id: impl Into<String>) -> Self {
    JobId(workflow_id.into(), job_id.into())
  }

  pub fn workflow_id(&self) -> WorkflowId {
    WorkflowId(self.0.clone())
  }

  pub fn job_key(&self) -> Id {
    self.1.clone()
  }
}

impl StepId {
  pub fn new(workflow_id: impl Into<String>, job_id: impl Into<String>, step_id: usize) -> Self {
    StepId(workflow_id.into(), job_id.into(), step_id)
  }

  pub fn workflow_id(&self) -> WorkflowId {
    WorkflowId(self.0.clone())
  }

  pub fn job_id(&self) -> JobId {
    JobId(self.0.clone(), self.1.clone())
  }

  pub fn job_key(&self) -> Id {
    self.1.clone()
  }

  pub fn step_number(&self) -> usize {
    self.2
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
    let job_id = JobId::new("workflow", "job");
    assert_eq!(job_id, JobId("workflow".to_string(), "job".to_string()));
    assert_eq!(job_id.workflow_id(), WorkflowId("workflow".to_string()));
    assert_eq!(job_id.job_key(), "job".to_string());
  }

  #[test]
  fn test_step_id() {
    let step_id = StepId::new("workflow", "job", 1);
    assert_eq!(
      step_id,
      StepId("workflow".to_string(), "job".to_string(), 1)
    );
    assert_eq!(step_id.workflow_id(), WorkflowId("workflow".to_string()));
    assert_eq!(
      step_id.job_id(),
      JobId("workflow".to_string(), "job".to_string())
    );
    assert_eq!(step_id.job_key(), "job".to_string());
    assert_eq!(step_id.step_number(), 1);
  }
}

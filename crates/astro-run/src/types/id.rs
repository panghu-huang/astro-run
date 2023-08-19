use crate::Error;
use serde::{Deserialize, Serialize};

pub type Id = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct WorkflowId(Id);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq, Default)]
pub struct JobId(Id, Id);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq, Default)]
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

impl ToString for WorkflowId {
  fn to_string(&self) -> String {
    self.0.clone()
  }
}

impl ToString for JobId {
  fn to_string(&self) -> String {
    format!("{}/{}", self.0, self.1)
  }
}

impl ToString for StepId {
  fn to_string(&self) -> String {
    format!("{}/{}/{}", self.0, self.1, self.2)
  }
}

impl TryFrom<&str> for WorkflowId {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    if value.is_empty() {
      Err(Error::internal_runtime_error("WorkflowId cannot be empty"))
    } else {
      Ok(WorkflowId(value.to_string()))
    }
  }
}

impl TryFrom<&str> for JobId {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 2 {
      Err(Error::internal_runtime_error(
        "JobId must be in the format of <workflow_id>/<job_key>",
      ))
    } else {
      Ok(JobId(parts[0].to_string(), parts[1].to_string()))
    }
  }
}

impl TryFrom<&str> for StepId {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 3 {
      Err(Error::internal_runtime_error(
        "StepId must be in the format of <workflow_id>/<job_key>/<step_number>",
      ))
    } else {
      let step_number = parts[2]
        .parse::<usize>()
        .map_err(|_| Error::internal_runtime_error("Step number must be a number"))?;
      Ok(StepId(
        parts[0].to_string(),
        parts[1].to_string(),
        step_number,
      ))
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

  #[test]
  fn test_workflow_id_to_string() {
    let workflow_id = WorkflowId::new("test");
    assert_eq!(workflow_id.to_string(), "test".to_string());
  }

  #[test]
  fn test_job_id_to_string() {
    let job_id = JobId::new("workflow", "job");
    assert_eq!(job_id.to_string(), "workflow/job".to_string());
  }

  #[test]
  fn test_step_id_to_string() {
    let step_id = StepId::new("workflow", "job", 1);
    assert_eq!(step_id.to_string(), "workflow/job/1".to_string());
  }

  #[test]
  fn test_workflow_id_try_from() {
    let workflow_id = WorkflowId::try_from("test").unwrap();
    assert_eq!(workflow_id, WorkflowId("test".to_string()));
  }

  #[test]
  fn test_job_id_try_from() {
    let job_id = JobId::try_from("workflow/job").unwrap();
    assert_eq!(job_id, JobId("workflow".to_string(), "job".to_string()));
  }

  #[test]
  fn test_step_id_try_from() {
    let step_id = StepId::try_from("workflow/job/1").unwrap();
    assert_eq!(
      step_id,
      StepId("workflow".to_string(), "job".to_string(), 1)
    );
  }

  #[test]
  fn test_workflow_id_try_from_empty() {
    let workflow_id = WorkflowId::try_from("");
    assert!(workflow_id.is_err());
  }

  #[test]
  fn test_job_id_try_from_empty() {
    let job_id = JobId::try_from("");
    assert!(job_id.is_err());
  }

  #[test]
  fn test_step_id_try_from_empty() {
    let step_id = StepId::try_from("");
    assert!(step_id.is_err());
  }

  #[test]
  fn test_job_id_try_from_invalid() {
    let job_id = JobId::try_from("workflow");
    assert!(job_id.is_err());
  }

  #[test]
  fn test_step_id_try_from_invalid() {
    let step_id = StepId::try_from("workflow/job");
    assert!(step_id.is_err());
  }
}

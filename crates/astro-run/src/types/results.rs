use crate::{Id, JobId, StepId, WorkflowId, WorkflowState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Time = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepRunResult {
  pub id: StepId,
  pub state: WorkflowState,
  pub exit_code: Option<i32>,
  pub started_at: Option<Time>,
  pub completed_at: Option<Time>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobRunResult {
  pub id: JobId,
  pub state: WorkflowState,
  pub started_at: Option<Time>,
  pub completed_at: Option<Time>,
  pub steps: Vec<StepRunResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowRunResult {
  pub id: WorkflowId,
  pub state: WorkflowState,
  pub started_at: Option<Time>,
  pub completed_at: Option<Time>,
  pub jobs: HashMap<Id, JobRunResult>,
}

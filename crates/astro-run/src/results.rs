use astro_run_shared::{Id, WorkflowState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Time = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepRunResult {
  pub state: WorkflowState,
  pub exit_code: Option<i32>,
  pub started_at: Option<Time>,
  pub ended_at: Option<Time>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobRunResult {
  pub state: WorkflowState,
  pub started_at: Option<Time>,
  pub ended_at: Option<Time>,
  pub steps: Vec<StepRunResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowRunResult {
  pub state: WorkflowState,
  pub started_at: Option<Time>,
  pub ended_at: Option<Time>,
  pub jobs: HashMap<Id, JobRunResult>,
}

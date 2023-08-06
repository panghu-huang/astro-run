use crate::{job::Job, UserWorkflow, WorkflowTriggerEvents};
use astro_run_shared::{Id, WorkflowEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workflow {
  pub id: Id,
  pub name: Option<String>,
  pub event: WorkflowEvent,
  pub on: Option<WorkflowTriggerEvents>,
  pub jobs: HashMap<String, Job>,
}

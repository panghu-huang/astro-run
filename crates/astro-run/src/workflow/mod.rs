mod builder;
mod job;
mod parser;

use crate::{ExecutionContext, WorkflowTriggerEvents};
use astro_run_shared::{Id, WorkflowEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Step = astro_run_shared::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workflow {
  pub id: Id,
  pub name: Option<String>,
  pub event: WorkflowEvent,
  pub on: Option<WorkflowTriggerEvents>,
  pub jobs: HashMap<String, job::Job>,
}

impl Workflow {
  pub async fn run(&self, ctx: ExecutionContext) -> astro_run_shared::Result<()> {
    for job in self.jobs.values() {
      job.run(ctx.clone()).await?;
    }

    Ok(())
  }

  pub fn builder() -> builder::WorkflowBuilder {
    builder::WorkflowBuilder::new()
  }
}

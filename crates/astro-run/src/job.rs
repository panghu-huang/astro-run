use crate::{step::Step, ExecutionContext, WorkflowTriggerEvents};
use astro_run_shared::Id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
  pub id: (Id, Id),
  pub name: Option<String>,
  pub steps: Vec<Step>,
  pub on: Option<WorkflowTriggerEvents>,
  /// For workflow run
  pub depends_on: Option<Vec<String>>,
  pub working_dirs: Option<Vec<String>>,
}

impl Job {
  pub async fn run(&self, ctx: ExecutionContext) -> astro_run_shared::Result<()> {
    for step in &self.steps {
      step.run(&ctx).await?;
    }

    Ok(())
  }
}

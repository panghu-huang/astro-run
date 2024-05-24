use crate::{Command, Condition, ContainerOptions, EnvironmentVariables, ExecutionContext, StepId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Step {
  pub id: StepId,
  pub name: Option<String>,
  pub on: Option<Condition>,
  pub container: Option<ContainerOptions>,
  pub run: String,
  pub continue_on_error: bool,
  pub environments: EnvironmentVariables,
  pub secrets: Vec<String>,
  pub timeout: Duration,
}

impl Step {
  pub async fn should_skip(&self, ctx: &ExecutionContext) -> bool {
    if let Some(on) = &self.on {
      !ctx.is_match(on).await
    } else {
      false
    }
  }
}

impl From<Step> for Command {
  fn from(step: Step) -> Command {
    Command {
      id: step.id,
      name: step.name,
      container: step.container,
      run: step.run,
      continue_on_error: step.continue_on_error,
      environments: step.environments,
      secrets: step.secrets,
      timeout: step.timeout,
    }
  }
}

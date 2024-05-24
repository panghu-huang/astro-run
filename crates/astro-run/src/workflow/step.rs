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
  fn from(val: Step) -> Self {
    Command {
      id: val.id,
      name: val.name,
      container: val.container,
      run: val.run,
      continue_on_error: val.continue_on_error,
      environments: val.environments,
      secrets: val.secrets,
      timeout: val.timeout,
    }
  }
}

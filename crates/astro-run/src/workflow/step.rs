use crate::{Command, Condition, ContainerOptions, EnvironmentVariables, StepId};
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

impl Into<Command> for Step {
  fn into(self) -> Command {
    Command {
      id: self.id,
      name: self.name,
      container: self.container,
      run: self.run,
      continue_on_error: self.continue_on_error,
      environments: self.environments,
      secrets: self.secrets,
      timeout: self.timeout,
    }
  }
}

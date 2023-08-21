use crate::{ContainerOptions, EnvironmentVariables, StepId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Command {
  pub id: StepId,
  pub name: Option<String>,
  pub container: Option<ContainerOptions>,
  pub run: String,
  pub continue_on_error: bool,
  pub environments: EnvironmentVariables,
  pub secrets: Vec<String>,
  pub timeout: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
  pub id: String,
  pub command: Command,
  // cancel signal
}

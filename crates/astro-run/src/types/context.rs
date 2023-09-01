use crate::{AstroRunSignal, ContainerOptions, EnvironmentVariables, StepId, WorkflowEvent};
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

#[derive(Debug, Clone)]
pub struct Context {
  pub id: String,
  pub signal: AstroRunSignal,
  pub command: Command,
  pub event: Option<WorkflowEvent>,
}

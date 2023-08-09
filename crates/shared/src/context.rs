use crate::{EnvironmentVariables, StepId};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Secret {
  /// The name of the secret
  pub key: String,
  /// The name of the environment variable to set
  pub env: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Volume {
  /// The name of the volume
  pub key: String,
  /// The path to mount the volume to
  pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
  pub id: StepId,
  pub name: Option<String>,
  pub image: Option<String>,
  pub run: String,
  pub working_directories: Vec<String>,
  pub continue_on_error: bool,
  pub environments: EnvironmentVariables,
  pub secrets: Vec<Secret>,
  pub volumes: Vec<Volume>,
  pub timeout: Duration,
  pub security_opts: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
  pub command: Command,
  // cancel signal
}

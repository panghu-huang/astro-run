use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum EnvironmentVariable {
  String(String),
  Number(f64),
  Boolean(bool),
}

pub type EnvironmentVariables = HashMap<String, EnvironmentVariable>;

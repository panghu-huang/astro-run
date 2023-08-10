use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum EnvironmentVariable {
  String(String),
  Number(f64),
  Boolean(bool),
}

impl ToString for EnvironmentVariable {
  fn to_string(&self) -> String {
    match self {
      EnvironmentVariable::String(s) => s.to_string(),
      EnvironmentVariable::Number(n) => n.to_string(),
      EnvironmentVariable::Boolean(b) => b.to_string(),
    }
  }
}

pub type EnvironmentVariables = HashMap<String, EnvironmentVariable>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_to_string() {
    assert_eq!(
      EnvironmentVariable::String("test".to_string()).to_string(),
      "test".to_string()
    );
    assert_eq!(
      EnvironmentVariable::Number(1.0).to_string(),
      "1".to_string()
    );
    assert_eq!(
      EnvironmentVariable::Boolean(true).to_string(),
      "true".to_string()
    );
  }
}

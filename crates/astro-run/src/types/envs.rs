use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum EnvironmentVariable {
  String(String),
  Number(f64),
  Boolean(bool),
}

impl std::fmt::Display for EnvironmentVariable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EnvironmentVariable::String(s) => write!(f, "{}", s),
      EnvironmentVariable::Number(n) => write!(f, "{}", n),
      EnvironmentVariable::Boolean(b) => write!(f, "{}", b),
    }
  }
}

impl From<String> for EnvironmentVariable {
  fn from(s: String) -> Self {
    EnvironmentVariable::String(s)
  }
}

impl From<&str> for EnvironmentVariable {
  fn from(s: &str) -> Self {
    EnvironmentVariable::String(s.to_string())
  }
}

impl From<f64> for EnvironmentVariable {
  fn from(n: f64) -> Self {
    EnvironmentVariable::Number(n)
  }
}

impl From<bool> for EnvironmentVariable {
  fn from(b: bool) -> Self {
    EnvironmentVariable::Boolean(b)
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

  #[test]
  fn test_from() {
    assert_eq!(
      EnvironmentVariable::from("test".to_string()),
      EnvironmentVariable::String("test".to_string())
    );
    assert_eq!(
      EnvironmentVariable::from("test"),
      EnvironmentVariable::String("test".to_string())
    );
    assert_eq!(
      EnvironmentVariable::from(1.0),
      EnvironmentVariable::Number(1.0)
    );
    assert_eq!(
      EnvironmentVariable::from(true),
      EnvironmentVariable::Boolean(true)
    );
  }
}

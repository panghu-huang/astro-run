use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct ConditionPayload {
  pub event: String,
  pub branch: String,
  pub paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PushCondition {
  pub branches: Option<Vec<String>>,
  // pub tags: Option<Vec<String>>,
  pub paths: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PullRequestCondition {
  // pub types: Option<Vec<String>>,
  /// Pull request base branches
  pub branches: Option<Vec<String>>,
  pub paths: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ConditionConfig {
  pub push: Option<PushCondition>,
  pub pull_request: Option<PullRequestCondition>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Condition {
  /// Event names. For example: `push`, `pull_request`
  Event(Vec<String>),
  /// Condition config
  Config(ConditionConfig),
}

impl PushCondition {
  pub fn is_match(&self, payload: &ConditionPayload) -> bool {
    if let Some(branches) = &self.branches {
      if !is_match_patterns(&vec![payload.branch.clone()], &branches) {
        return false;
      }
    }

    if let Some(paths) = &self.paths {
      if !is_match_patterns(&payload.paths, paths) {
        return false;
      }
    }

    true
  }
}

impl PullRequestCondition {
  pub fn is_match(&self, payload: &ConditionPayload) -> bool {
    if let Some(branches) = &self.branches {
      if !is_match_patterns(&vec![payload.branch.clone()], &branches) {
        return false;
      }
    }

    if let Some(paths) = &self.paths {
      if !is_match_patterns(&payload.paths, paths) {
        return false;
      }
    }

    true
  }
}

impl Condition {
  pub fn is_match(&self, payload: &ConditionPayload) -> bool {
    log::trace!("Matching condition {:#?} with payload {:#?}", self, payload);
    match self {
      Condition::Event(events) => events.contains(&payload.event),
      Condition::Config(config) => match &payload.event {
        event if event == "push" => {
          if let Some(push) = &config.push {
            push.is_match(payload)
          } else {
            false
          }
        }
        event if event == "pull_request" => {
          if let Some(pull_request) = &config.pull_request {
            pull_request.is_match(payload)
          } else {
            false
          }
        }
        _ => false,
      },
    }
  }
}

fn is_match_patterns(values: &Vec<String>, patterns: &Vec<String>) -> bool {
  for value in values {
    for pattern in patterns {
      match glob::Pattern::new(pattern) {
        Ok(pattern) => {
          if pattern.matches(value) {
            return true;
          }
        }
        Err(err) => {
          log::error!("Invalid glob pattern: {}", err);
          return false;
        }
      }
    }
  }

  false
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_match_workflow_paths() {
    let paths = vec![
      "src/main.rs".to_string(),
      "src/lib.rs".to_string(),
      "src/runner/main.rs".to_string(),
      "src/runner/lib.rs".to_string(),
    ];

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/main.rs".to_string()]),
      true
    );

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/runner/main.rs".to_string()]),
      true
    );

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/runner/*.rs".to_string()]),
      true
    );

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/runner/**/*.rs".to_string()]),
      true
    );

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/runner/**/main.rs".to_string()]),
      true
    );

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/runner/**/lib.rs".to_string()]),
      true
    );

    // Negative tests
    assert_eq!(
      is_match_patterns(&paths, &vec!["scripts/**/lib.rs".to_string()]),
      false
    );

    assert_eq!(
      is_match_patterns(&paths, &vec!["src/runner/**/lib.js".to_string()]),
      false
    );
  }

  #[test]
  fn is_match_branches() {
    let branches = vec!["master".to_string(), "develop".to_string()];

    assert_eq!(
      is_match_patterns(&vec!["master".to_string()], &branches),
      true
    );

    assert_eq!(
      is_match_patterns(&vec!["develop".to_string()], &branches),
      true
    );

    assert_eq!(
      is_match_patterns(&vec!["feature/branch".to_string()], &branches),
      false
    );
  }

  #[test]
  fn is_match_features() {
    let features = vec!["feature/*".to_string()];

    assert_eq!(
      is_match_patterns(&vec!["feature/branch".to_string()], &features),
      true
    );

    assert_eq!(
      is_match_patterns(&vec!["feature/branch/branch".to_string()], &features),
      true
    );

    assert_eq!(
      is_match_patterns(&vec!["feature".to_string()], &features),
      false
    );

    assert_eq!(
      is_match_patterns(&vec!["feature-branch".to_string()], &features),
      false
    );
  }

  #[test]
  fn test_pull_request_condition() {
    let condition = PullRequestCondition {
      branches: Some(vec!["master".to_string()]),
      paths: Some(vec!["src/main.rs".to_string()]),
    };

    let payload = ConditionPayload {
      event: "pull_request".to_string(),
      branch: "master".to_string(),
      paths: vec!["src/main.rs".to_string()],
    };

    assert_eq!(condition.is_match(&payload), true);

    let payload = ConditionPayload {
      event: "pull_request".to_string(),
      branch: "main".to_string(),
      paths: vec!["src/main.rs".to_string()],
    };

    assert_eq!(condition.is_match(&payload), false);
  }

  #[test]
  fn test_push_condition() {
    let condition = PushCondition {
      branches: Some(vec!["master".to_string()]),
      paths: Some(vec!["src/main.rs".to_string()]),
    };

    let payload = ConditionPayload {
      event: "push".to_string(),
      branch: "master".to_string(),
      paths: vec!["src/main.rs".to_string()],
    };

    assert_eq!(condition.is_match(&payload), true);
  }

  #[test]
  fn test_condition() {
    let condition = Condition::Config(ConditionConfig {
      push: Some(PushCondition {
        branches: Some(vec!["master".to_string()]),
        paths: Some(vec!["src/main.rs".to_string()]),
      }),
      pull_request: Some(PullRequestCondition {
        branches: Some(vec!["master".to_string()]),
        paths: Some(vec!["src/main.rs".to_string()]),
      }),
    });

    let payload = ConditionPayload {
      event: "push".to_string(),
      branch: "master".to_string(),
      paths: vec!["src/main.rs".to_string()],
    };

    assert_eq!(condition.is_match(&payload), true);
  }

  #[test]
  fn test_events_condition() {
    let push = Condition::Event(vec!["push".to_string()]);
    let pull_request = Condition::Event(vec!["pull_request".to_string()]);

    let payload = ConditionPayload {
      event: "push".to_string(),
      branch: "master".to_string(),
      paths: vec!["src/main.rs".to_string()],
    };

    assert_eq!(push.is_match(&payload), true);
    assert_eq!(pull_request.is_match(&payload), false);
  }

  #[test]
  fn test_invalid_event() {
    let pull_request = Condition::Event(vec!["pull_request".to_string()]);

    let payload = ConditionPayload {
      event: "invalid".to_string(),
      branch: "".to_string(),
      paths: vec![],
    };

    assert_eq!(pull_request.is_match(&payload), false);
  }

  #[test]
  fn test_invalid_payload_event() {
    let condition = Condition::Config(ConditionConfig {
      push: Some(PushCondition {
        branches: Some(vec!["master".to_string()]),
        paths: Some(vec!["src/main.rs".to_string()]),
      }),
      pull_request: None,
    });

    let payload = ConditionPayload {
      event: "invalid".to_string(),
      branch: "master".to_string(),
      paths: vec!["src/main.rs".to_string()],
    };

    assert_eq!(condition.is_match(&payload), false);
  }

  #[test]
  fn test_invalid_glob_pattern() {
    let v = is_match_patterns(
      &vec!["a/b".to_string()],
      &vec![
        "a**/b".to_string(), // Invalid glob pattern
      ],
    );

    assert_eq!(v, false);
  }
}

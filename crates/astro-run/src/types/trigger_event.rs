use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggerEvent {
  /// push / pull_request
  pub event: String,
  pub repo_owner: String,
  pub repo_name: String,
  pub pr_number: Option<u64>,
  pub sha: String,
  pub branch: String,
  /// refs/heads/master / refs/tags/v1.0.0 / refs/pull/1/merge
  pub ref_name: String,
}

impl Default for TriggerEvent {
  fn default() -> Self {
    Self {
      event: "push".to_string(),
      repo_owner: "panghu-huang".to_string(),
      repo_name: "astro-run".to_string(),
      ref_name: "refs/heads/main".to_string(),
      branch: "main".to_string(),
      sha: "123456".to_string(),
      pr_number: None,
    }
  }
}

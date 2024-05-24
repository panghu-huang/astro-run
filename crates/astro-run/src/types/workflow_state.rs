use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
  Pending,
  Queued,
  InProgress,
  Succeeded,
  Failed,
  Cancelled,
  Skipped,
}

impl WorkflowState {
  pub fn is_terminal(&self) -> bool {
    matches!(
      self,
      WorkflowState::Succeeded
        | WorkflowState::Failed
        | WorkflowState::Cancelled
        | WorkflowState::Skipped
    )
  }

  pub fn is_in_progress(&self) -> bool {
    matches!(self, WorkflowState::InProgress)
  }

  pub fn is_queued(&self) -> bool {
    matches!(self, WorkflowState::Queued)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_terminal() {
    assert!(!WorkflowState::Pending.is_terminal());
    assert!(!WorkflowState::Queued.is_terminal());
    assert!(!WorkflowState::InProgress.is_terminal());
    assert!(WorkflowState::Succeeded.is_terminal());
    assert!(WorkflowState::Failed.is_terminal());
    assert!(WorkflowState::Cancelled.is_terminal());
    assert!(WorkflowState::Skipped.is_terminal());
  }

  #[test]
  fn test_is_in_progress() {
    assert!(!WorkflowState::Pending.is_in_progress());
    assert!(!WorkflowState::Queued.is_in_progress());
    assert!(WorkflowState::InProgress.is_in_progress());
    assert!(!WorkflowState::Succeeded.is_in_progress());
    assert!(!WorkflowState::Failed.is_in_progress());
    assert!(!WorkflowState::Cancelled.is_in_progress());
    assert!(!WorkflowState::Skipped.is_in_progress());
  }

  #[test]
  fn test_is_queued() {
    assert!(!WorkflowState::Pending.is_queued());
    assert!(WorkflowState::Queued.is_queued());
    assert!(!WorkflowState::InProgress.is_queued());
    assert!(!WorkflowState::Succeeded.is_queued());
    assert!(!WorkflowState::Failed.is_queued());
    assert!(!WorkflowState::Cancelled.is_queued());
    assert!(!WorkflowState::Skipped.is_queued());
  }
}

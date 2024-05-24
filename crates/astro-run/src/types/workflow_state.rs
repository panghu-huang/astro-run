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
    assert_eq!(WorkflowState::Pending.is_terminal(), false);
    assert_eq!(WorkflowState::Queued.is_terminal(), false);
    assert_eq!(WorkflowState::InProgress.is_terminal(), false);
    assert_eq!(WorkflowState::Succeeded.is_terminal(), true);
    assert_eq!(WorkflowState::Failed.is_terminal(), true);
    assert_eq!(WorkflowState::Cancelled.is_terminal(), true);
    assert_eq!(WorkflowState::Skipped.is_terminal(), true);
  }

  #[test]
  fn test_is_in_progress() {
    assert_eq!(WorkflowState::Pending.is_in_progress(), false);
    assert_eq!(WorkflowState::Queued.is_in_progress(), false);
    assert_eq!(WorkflowState::InProgress.is_in_progress(), true);
    assert_eq!(WorkflowState::Succeeded.is_in_progress(), false);
    assert_eq!(WorkflowState::Failed.is_in_progress(), false);
    assert_eq!(WorkflowState::Cancelled.is_in_progress(), false);
    assert_eq!(WorkflowState::Skipped.is_in_progress(), false);
  }

  #[test]
  fn test_is_queued() {
    assert_eq!(WorkflowState::Pending.is_queued(), false);
    assert_eq!(WorkflowState::Queued.is_queued(), true);
    assert_eq!(WorkflowState::InProgress.is_queued(), false);
    assert_eq!(WorkflowState::Succeeded.is_queued(), false);
    assert_eq!(WorkflowState::Failed.is_queued(), false);
    assert_eq!(WorkflowState::Cancelled.is_queued(), false);
    assert_eq!(WorkflowState::Skipped.is_queued(), false);
  }
}

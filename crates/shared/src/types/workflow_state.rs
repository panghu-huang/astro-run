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
    match self {
      WorkflowState::Succeeded
      | WorkflowState::Failed
      | WorkflowState::Cancelled
      | WorkflowState::Skipped => true,
      _ => false,
    }
  }

  pub fn is_in_progress(&self) -> bool {
    match self {
      WorkflowState::InProgress => true,
      _ => false,
    }
  }

  pub fn is_queued(&self) -> bool {
    match self {
      WorkflowState::Queued => true,
      _ => false,
    }
  }
}

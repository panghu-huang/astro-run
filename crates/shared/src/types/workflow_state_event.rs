use super::{workflow_state::WorkflowState, Id};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WorkflowStateEvent {
  WorkflowStateUpdated {
    workflow_id: Id,
    state: WorkflowState,
  },
  JobStateUpdated {
    workflow_id: Id,
    job_id: String,
    state: WorkflowState,
  },
  StepStateUpdated {
    workflow_id: Id,
    job_id: String,
    number: usize,
    state: WorkflowState,
  },
}

impl WorkflowStateEvent {
  pub fn workflow_state_updated(workflow_id: Id, state: WorkflowState) -> Self {
    WorkflowStateEvent::WorkflowStateUpdated { workflow_id, state }
  }

  pub fn job_state_updated(workflow_id: Id, job_id: String, state: WorkflowState) -> Self {
    WorkflowStateEvent::JobStateUpdated {
      workflow_id,
      job_id,
      state,
    }
  }

  pub fn step_state_updated(
    workflow_id: Id,
    job_id: String,
    number: usize,
    state: WorkflowState,
  ) -> Self {
    WorkflowStateEvent::StepStateUpdated {
      workflow_id,
      job_id,
      number,
      state,
    }
  }
}

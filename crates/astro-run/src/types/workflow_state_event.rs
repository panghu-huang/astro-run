use super::{workflow_state::WorkflowState, JobId, StepId, WorkflowId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WorkflowStateEvent {
  WorkflowStateUpdated {
    id: WorkflowId,
    state: WorkflowState,
  },
  JobStateUpdated {
    id: JobId,
    state: WorkflowState,
  },
  StepStateUpdated {
    id: StepId,
    state: WorkflowState,
  },
}

mod plugin;
mod plugin_driver;

use serde::{Deserialize, Serialize};
use crate::{
  Action, Error, Job, JobRunResult, Step, StepRunResult, UserActionStep, Workflow, WorkflowEvent,
  WorkflowLog, WorkflowRunResult, WorkflowStateEvent,
};
pub use plugin::*;
pub use plugin_driver::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunEvent<T> {
  pub payload: T,
  pub workflow_event: Option<WorkflowEvent>,
}

pub type RunWorkflowEvent = RunEvent<Workflow>;

pub type RunJobEvent = RunEvent<Job>;

pub type RunStepEvent = RunEvent<Step>;

pub type HookNoopResult = Result<(), Error>;

pub type HookBeforeRunStepResult = Result<Step, Error>;

pub type HookResolveActionResult = Result<Option<Box<dyn Action>>, Error>;

#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
  fn name(&self) -> &'static str;
  async fn on_resolve_dynamic_action(&self, _step: UserActionStep) -> HookResolveActionResult {
    Ok(None)
  }
  async fn on_run_workflow(&self, _event: RunWorkflowEvent) -> HookNoopResult {
    Ok(())
  }
  async fn on_run_job(&self, _event: RunJobEvent) -> HookNoopResult {
    Ok(())
  }
  async fn on_before_run_step(&self, step: Step) -> HookBeforeRunStepResult {
    Ok(step)
  }
  async fn on_run_step(&self, _event: RunStepEvent) -> HookNoopResult {
    Ok(())
  }
  async fn on_state_change(&self, _event: WorkflowStateEvent) -> HookNoopResult {
    Ok(())
  }
  async fn on_log(&self, _log: WorkflowLog) -> HookNoopResult {
    Ok(())
  }
  async fn on_step_completed(&self, _result: StepRunResult) -> HookNoopResult {
    Ok(())
  }
  async fn on_job_completed(&self, _result: JobRunResult) -> HookNoopResult {
    Ok(())
  }
  async fn on_workflow_completed(&self, _result: WorkflowRunResult) -> HookNoopResult {
    Ok(())
  }
}

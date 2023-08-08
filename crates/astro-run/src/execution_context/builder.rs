use super::{workflow_shared::WorkflowShared, ExecutionContext};
use crate::shared_state::{AstroRunSharedState, SharedState};
use astro_run_shared::{Error, Result, Runner};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ExecutionContextBuilder {
  runner: Option<Arc<Box<dyn Runner>>>,
  shared_state: Option<AstroRunSharedState>,
}

impl ExecutionContextBuilder {
  pub fn new() -> Self {
    ExecutionContextBuilder {
      runner: None,
      shared_state: None,
    }
  }

  pub fn runner(mut self, runner: Arc<Box<dyn Runner>>) -> Self {
    self.runner = Some(runner);
    self
  }

  pub fn shared_state(mut self, shared_state: AstroRunSharedState) -> Self {
    self.shared_state = Some(shared_state);
    self
  }

  pub fn build(self) -> Result<ExecutionContext> {
    let runner = self.runner.ok_or(Error::init_error(
      "Runner is not set in execution context builder",
    ))?;

    let shared_state = self
      .shared_state
      .unwrap_or_else(|| Arc::new(Mutex::new(SharedState::new())));

    let ctx = ExecutionContext {
      runner,
      workflow_shared: WorkflowShared::default(),
      shared_state,
    };

    Ok(ctx)
  }
}

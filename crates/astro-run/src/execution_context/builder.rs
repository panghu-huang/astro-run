use super::{workflow_shared::WorkflowShared, ExecutionContext, ExecutionContextInner};
use crate::PluginManager;
use astro_run_shared::{Error, Result, Runner};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct ExecutionContextBuilder {
  runner: Option<Arc<Box<dyn Runner>>>,
  plugin_manager: Option<PluginManager>,
}

impl ExecutionContextBuilder {
  pub fn new() -> Self {
    ExecutionContextBuilder {
      runner: None,
      plugin_manager: None,
    }
  }

  pub fn runner(mut self, runner: Arc<Box<dyn Runner>>) -> Self {
    self.runner = Some(runner);
    self
  }

  pub fn plugin_manager(mut self, plugin_manager: PluginManager) -> Self {
    self.plugin_manager = Some(plugin_manager);
    self
  }

  pub fn build(self) -> Result<ExecutionContext> {
    let runner = self.runner.ok_or(Error::init_error(
      "Runner is not set in execution context builder",
    ))?;

    let plugin_manager = self.plugin_manager.unwrap_or(PluginManager::new());

    let ctx = ExecutionContext {
      runner,
      inner: Arc::new(Mutex::new(ExecutionContextInner { plugin_manager })),
      workflow_shared: WorkflowShared::default(),
    };

    Ok(ctx)
  }
}

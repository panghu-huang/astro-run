use crate::{AstroRunPlugin, ExecutionContext, PluginManager, UserWorkflow};
use astro_run_shared::{Result, Runner};
use parking_lot::Mutex;
use std::sync::Arc;

struct AstroRunInner {
  plugin_manager: PluginManager,
}

pub struct AstroRun {
  runner: Arc<Box<dyn Runner>>,
  inner: Arc<Mutex<AstroRunInner>>,
}

impl AstroRun {
  pub fn register_plugin(&self, plugin: AstroRunPlugin) {
    let mut inner = self.inner.lock();
    inner.plugin_manager.register(plugin);
  }

  pub fn unregister_plugin(&self, plugin_name: &'static str) {
    let mut inner = self.inner.lock();
    inner.plugin_manager.unregister(plugin_name);
  }

  pub fn parse_workflow(&self, workflow: &str) -> Result<UserWorkflow> {
    UserWorkflow::from_str(workflow)
  }

  pub fn execution_context(&self) -> Result<ExecutionContext> {
    let plugin_manager = self.inner.lock().plugin_manager.clone();
    ExecutionContext::builder()
      .runner(self.runner.clone())
      .plugin_manager(plugin_manager)
      // TODO: Set working directory
      .build()
  }
}

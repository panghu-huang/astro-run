use crate::{AstroRunPlugin, ExecutionContext, PluginManager};
use astro_run_shared::Runner;
use std::sync::Arc;

pub struct AstroRun {
  runner: Arc<Box<dyn Runner>>,
  plugin_manager: PluginManager,
}

impl AstroRun {
  pub fn builder() -> AstroRunBuilder {
    AstroRunBuilder::new()
  }

  pub fn register_plugin(&self, plugin: AstroRunPlugin) {
    self.plugin_manager.register(plugin);
  }

  pub fn unregister_plugin(&self, plugin_name: &'static str) {
    self.plugin_manager.unregister(plugin_name);
  }

  pub fn execution_context(&self) -> ExecutionContext {
    let plugin_manager = self.plugin_manager.clone();
    ExecutionContext::builder()
      .runner(self.runner.clone())
      .plugin_manager(plugin_manager)
      .build()
      .unwrap()
  }
}

pub struct AstroRunBuilder {
  runner: Option<Box<dyn Runner>>,
  plugin_manager: PluginManager,
}

impl AstroRunBuilder {
  pub fn new() -> Self {
    AstroRunBuilder {
      runner: None,
      plugin_manager: PluginManager::new(),
    }
  }

  pub fn runner(mut self, runner: Box<dyn Runner>) -> Self {
    self.runner = Some(runner);
    self
  }

  pub fn plugin(self, plugin: AstroRunPlugin) -> Self {
    self.plugin_manager.register(plugin);
    self
  }

  pub fn build(self) -> AstroRun {
    let runner = self.runner.unwrap();

    AstroRun {
      runner: Arc::new(runner),
      plugin_manager: self.plugin_manager,
    }
  }
}

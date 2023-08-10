use crate::{shared_state::AstroRunSharedState, AstroRunPlugin, ExecutionContext, Runner};
use std::sync::Arc;

pub struct AstroRun {
  runner: Arc<Box<dyn Runner>>,
  shared_state: AstroRunSharedState,
}

impl AstroRun {
  pub fn builder() -> AstroRunBuilder {
    AstroRunBuilder::new()
  }

  pub fn register_plugin(&self, plugin: AstroRunPlugin) -> &Self {
    self.shared_state.register_plugin(plugin);

    self
  }

  pub fn unregister_plugin(&self, plugin_name: &'static str) -> &Self {
    self.shared_state.unregister_plugin(plugin_name);

    self
  }

  pub fn execution_context(&self) -> ExecutionContext {
    let shared_state = self.shared_state.clone();
    ExecutionContext::builder()
      .runner(self.runner.clone())
      .shared_state(shared_state)
      .build()
      .unwrap()
  }
}

pub struct AstroRunBuilder {
  runner: Option<Box<dyn Runner>>,
  shared_state: AstroRunSharedState,
}

impl AstroRunBuilder {
  pub fn new() -> Self {
    AstroRunBuilder {
      runner: None,
      shared_state: AstroRunSharedState::new(),
    }
  }

  pub fn runner(mut self, runner: Box<dyn Runner>) -> Self {
    self.runner = Some(runner);
    self
  }

  pub fn plugin(self, plugin: AstroRunPlugin) -> Self {
    self.shared_state.register_plugin(plugin);
    self
  }

  pub fn build(self) -> AstroRun {
    let runner = self.runner.unwrap();

    AstroRun {
      runner: Arc::new(runner),
      shared_state: self.shared_state,
    }
  }
}

use crate::{AstroRunPlugin, PluginManager};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
struct SharedState {
  plugins: PluginManager,
}

impl SharedState {
  pub fn new() -> Self {
    SharedState {
      plugins: PluginManager::new(),
    }
  }
}

#[derive(Clone)]
pub struct AstroRunSharedState(Arc<Mutex<SharedState>>);

impl AstroRunSharedState {
  pub fn new() -> Self {
    AstroRunSharedState(Arc::new(Mutex::new(SharedState::new())))
  }

  pub fn register_plugin(&self, plugin: AstroRunPlugin) {
    self.0.lock().plugins.register(plugin);
  }

  pub fn unregister_plugin(&self, plugin_name: &'static str) {
    self.0.lock().plugins.unregister(plugin_name);
  }

  pub fn plugins(&self) -> PluginManager {
    self.0.lock().plugins.clone()
  }
}

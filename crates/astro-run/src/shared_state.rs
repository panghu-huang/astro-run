use crate::PluginManager;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedState {
  pub plugins: PluginManager,
}

impl SharedState {
  pub fn new() -> Self {
    SharedState {
      plugins: PluginManager::new(),
    }
  }
}

pub type AstroRunSharedState = Arc<Mutex<SharedState>>;

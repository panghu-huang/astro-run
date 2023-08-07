use parking_lot::Mutex;
use std::sync::Arc;

use astro_run_shared::{WorkflowLog, WorkflowStateEvent};

type OnStateChange = dyn Fn(WorkflowStateEvent) -> ();
type OnLog = dyn Fn(WorkflowLog) -> ();

pub trait Plugin: Send {
  fn on_state_change(&self, event: WorkflowStateEvent) -> ();
  fn on_log(&self, log: WorkflowLog) -> ();
}

pub struct PluginBuilder {
  name: &'static str,
  on_state_change: Option<Box<OnStateChange>>,
  on_log: Option<Box<OnLog>>,
}

impl PluginBuilder {
  pub fn new(name: &'static str) -> Self {
    PluginBuilder {
      name,
      on_state_change: None,
      on_log: None,
    }
  }

  pub fn on_state_change<T>(mut self, on_state_change: T) -> Self
  where
    T: Fn(WorkflowStateEvent) -> () + 'static,
  {
    self.on_state_change = Some(Box::new(on_state_change));
    self
  }

  pub fn on_log<T>(mut self, on_log: T) -> Self
  where
    T: Fn(WorkflowLog) -> () + 'static,
  {
    self.on_log = Some(Box::new(on_log));
    self
  }

  pub fn build(self) -> AstroRunPlugin {
    AstroRunPlugin {
      name: self.name,
      on_state_change: self.on_state_change,
      on_log: self.on_log,
    }
  }
}

pub struct AstroRunPlugin {
  name: &'static str,
  on_state_change: Option<Box<OnStateChange>>,
  on_log: Option<Box<OnLog>>,
}

#[derive(Clone)]
pub struct PluginManager {
  plugins: Arc<Mutex<Vec<AstroRunPlugin>>>,
}

impl PluginManager {
  pub fn new() -> Self {
    PluginManager {
      plugins: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn register(&self, plugin: AstroRunPlugin) {
    self.plugins.lock().push(plugin);
  }

  pub fn unregister(&self, name: &'static str) {
    self.plugins.lock().retain(|plugin| plugin.name != name);
  }

  pub fn on_state_change(&self, event: &WorkflowStateEvent) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      if let Some(on_state_change) = &plugin.on_state_change {
        on_state_change(event.clone());
      }
    }
  }

  pub fn on_log(&self, log: &WorkflowLog) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      if let Some(on_log) = &plugin.on_log {
        on_log(log.clone());
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use astro_run_shared::{WorkflowState, WorkflowStateEvent};

  #[test]
  fn plugin_manager_register() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test").build();

    plugin_manager.register(plugin);

    assert_eq!(plugin_manager.plugins.lock().len(), 1);
  }

  #[test]
  fn plugin_manager_unregister() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test").build();

    plugin_manager.register(plugin);
    plugin_manager.unregister("test");

    assert_eq!(plugin_manager.plugins.lock().len(), 0);
  }

  #[test]
  fn plugin_manager_on_state_change() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test")
      .on_state_change(|event| {
        if let WorkflowStateEvent::WorkflowStateUpdated { workflow_id, state } = event {
          assert_eq!(workflow_id, "test");
          assert_eq!(state, WorkflowState::Cancelled);
        } else {
          panic!("Unexpected event type");
        }
      })
      .build();

    plugin_manager.register(plugin);
    plugin_manager.on_state_change(&WorkflowStateEvent::WorkflowStateUpdated {
      workflow_id: "test".to_string(),
      state: WorkflowState::Cancelled,
    });
  }

  #[test]
  fn plugin_manager_on_log() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test")
      .on_log(|log| {
        assert_eq!(log.message, "test");
      })
      .build();

    plugin_manager.register(plugin);
    plugin_manager.on_log(&WorkflowLog {
      message: "test".to_string(),
      ..Default::default()
    });
  }
}

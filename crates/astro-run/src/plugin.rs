use astro_run_shared::{WorkflowLog, WorkflowStateEvent};

type OnStateChange = fn(event: &WorkflowStateEvent) -> ();
type OnLog = fn(log: &WorkflowLog) -> ();

pub trait Plugin: Send {
  fn on_state_change(&self, event: &WorkflowStateEvent) -> ();
  fn on_log(&self, log: &WorkflowLog) -> ();
}

pub struct PluginBuilder {
  name: &'static str,
  on_state_change: Option<OnStateChange>,
  on_log: Option<OnLog>,
}

impl PluginBuilder {
  pub fn new(name: &'static str) -> Self {
    PluginBuilder {
      name,
      on_state_change: None,
      on_log: None,
    }
  }

  pub fn on_state_change(mut self, on_state_change: OnStateChange) -> Self {
    self.on_state_change = Some(on_state_change);
    self
  }

  pub fn on_log(mut self, on_log: OnLog) -> Self {
    self.on_log = Some(on_log);
    self
  }

  pub fn build(self) -> AstroRunPlugin {
    AstroRunPlugin {
      name: self.name,
      on_state_change: Box::new(self.on_state_change),
      on_log: Box::new(self.on_log),
    }
  }
}

#[derive(Clone)]
pub struct AstroRunPlugin {
  name: &'static str,
  on_state_change: Box<Option<OnStateChange>>,
  on_log: Box<Option<OnLog>>,
}

#[derive(Clone)]
pub struct PluginManager {
  plugins: Vec<AstroRunPlugin>,
}

impl PluginManager {
  pub fn new() -> Self {
    PluginManager { plugins: vec![] }
  }

  pub fn register(&mut self, plugin: AstroRunPlugin) {
    self.plugins.push(plugin);
  }

  pub fn unregister(&mut self, name: &'static str) {
    self.plugins.retain(|plugin| plugin.name != name);
  }

  pub fn on_state_change(&self, event: &WorkflowStateEvent) {
    for plugin in &self.plugins {
      if let Some(on_state_change) = plugin.on_state_change.as_ref() {
        on_state_change(event);
      }
    }
  }

  pub fn on_log(&self, log: &WorkflowLog) {
    for plugin in &self.plugins {
      if let Some(on_log) = plugin.on_log.as_ref() {
        on_log(log);
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
    let mut plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test").build();

    plugin_manager.register(plugin);

    assert_eq!(plugin_manager.plugins.len(), 1);
  }

  #[test]
  fn plugin_manager_unregister() {
    let mut plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test").build();

    plugin_manager.register(plugin);
    plugin_manager.unregister("test");

    assert_eq!(plugin_manager.plugins.len(), 0);
  }

  #[test]
  fn plugin_manager_on_state_change() {
    let mut plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test")
      .on_state_change(|event| {
        if let WorkflowStateEvent::WorkflowStateUpdated { workflow_id, state } = event {
          assert_eq!(workflow_id, "test");
          assert_eq!(state, &WorkflowState::Cancelled);
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
    let mut plugin_manager = PluginManager::new();
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

use astro_run_shared::{EnvironmentVariable, WorkflowEvent};
use std::collections::HashMap;

#[derive(Clone)]
pub struct WorkflowShared {
  // workflow state controller
  // workflow log controller
  // job working directories ??
  pub event: Option<WorkflowEvent>,
  pub environments: HashMap<String, EnvironmentVariable>,
}

impl WorkflowShared {
  pub fn set_environment(&mut self, key: String, value: EnvironmentVariable) {
    self.environments.insert(key, value);
  }

  pub fn get_environment(&self, key: String) -> Option<EnvironmentVariable> {
    self.environments.get(&key).cloned()
  }

  pub fn set_event(&mut self, event: WorkflowEvent) {
    self.event = Some(event);
  }

  pub fn get_event(&self) -> Option<WorkflowEvent> {
    self.event.clone()
  }
}

impl Default for WorkflowShared {
  fn default() -> Self {
    WorkflowShared {
      environments: HashMap::new(),
      event: None,
    }
  }
}

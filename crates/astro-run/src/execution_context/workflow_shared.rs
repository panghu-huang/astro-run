use astro_run_shared::EnvironmentVariable;
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone)]
pub struct WorkflowShared {
  // workflow state controller
  // workflow log controller
  pub working_directory: PathBuf,
  pub cache_directory: PathBuf,
  pub environments: HashMap<String, EnvironmentVariable>,
}

impl WorkflowShared {
  pub fn set_environment(&mut self, key: String, value: EnvironmentVariable) {
    self.environments.insert(key, value);
  }

  pub fn get_environment(&self, key: String) -> Option<EnvironmentVariable> {
    self.environments.get(&key).cloned()
  }

  pub fn set_working_directory(&mut self, path: PathBuf) {
    self.working_directory = path;
  }

  pub fn set_cache_directory(&mut self, path: PathBuf) {
    self.cache_directory = path;
  }
}

impl Default for WorkflowShared {
  fn default() -> Self {
    WorkflowShared {
      working_directory: PathBuf::new(),
      cache_directory: PathBuf::new(),
      environments: HashMap::new(),
    }
  }
}

use crate::{
  Action, ActionDriver, ExecutionContext, ExecutionContextBuilder, GithubAuthorization, JobId,
  Plugin, PluginDriver, Result, Runner, SharedActionDriver, SharedPluginDriver, SignalManager,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct AstroRun {
  runner: Arc<Box<dyn Runner>>,
  github_auth: Option<GithubAuthorization>,
  plugin_driver: SharedPluginDriver,
  action_driver: SharedActionDriver,
  signal_manager: SignalManager,
}

impl AstroRun {
  pub fn builder() -> AstroRunBuilder {
    AstroRunBuilder::new()
  }

  pub fn cancel_job(&self, job_id: &JobId) -> Result<()> {
    self.signal_manager.cancel_job(&job_id)
  }

  pub fn execution_context(&self) -> ExecutionContextBuilder {
    // let shared_state = self.shared_state.clone();
    let mut builder = ExecutionContext::builder()
      .runner(self.runner.clone())
      .plugin_driver(self.plugin_driver());

    if let Some(github_auth) = &self.github_auth {
      builder = builder.github_auth(github_auth.clone());
    }

    builder
  }

  pub(crate) fn plugin_driver(&self) -> SharedPluginDriver {
    Arc::clone(&self.plugin_driver)
  }

  pub(crate) fn action_driver(&self) -> SharedActionDriver {
    Arc::clone(&self.action_driver)
  }
}

pub struct AstroRunBuilder {
  runner: Option<Box<dyn Runner>>,
  plugins: Vec<Box<dyn Plugin>>,
  actions: HashMap<String, Box<dyn Action>>,
  github_auth: Option<GithubAuthorization>,
}

impl AstroRunBuilder {
  pub fn new() -> Self {
    AstroRunBuilder {
      runner: None,
      github_auth: None,
      plugins: vec![],
      actions: HashMap::new(),
    }
  }

  pub fn runner<T>(mut self, runner: T) -> Self
  where
    T: Runner + 'static,
  {
    self.runner = Some(Box::new(runner));
    self
  }

  pub fn plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
    self.plugins.push(Box::new(plugin));

    self
  }

  pub fn action(mut self, name: impl Into<String>, action: impl Action + 'static) -> Self {
    self.actions.insert(name.into(), Box::new(action));

    self
  }

  pub fn github_personal_token(mut self, token: impl Into<String>) -> Self {
    self.github_auth = Some(GithubAuthorization::PersonalAccessToken(token.into()));
    self
  }

  pub fn github_app(mut self, app_id: u64, private_key: impl Into<String>) -> Self {
    self.github_auth = Some(GithubAuthorization::GithubApp {
      app_id,
      private_key: private_key.into(),
    });

    self
  }

  pub fn build(self) -> AstroRun {
    let runner = self.runner.unwrap();

    AstroRun {
      runner: Arc::new(runner),
      plugin_driver: Arc::new(PluginDriver::new(self.plugins)),
      action_driver: Arc::new(ActionDriver::new(self.actions)),
      signal_manager: SignalManager::new(),
      github_auth: self.github_auth,
    }
  }
}

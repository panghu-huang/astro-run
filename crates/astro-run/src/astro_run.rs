use crate::{
  shared_state::AstroRunSharedState, Action, Actions, AstroRunPlugin, ExecutionContext,
  ExecutionContextBuilder, GithubAuthorization, JobId, PluginManager, Result, Runner,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AstroRun {
  runner: Arc<Box<dyn Runner>>,
  github_auth: Option<GithubAuthorization>,
  pub(crate) shared_state: AstroRunSharedState,
}

impl AstroRun {
  pub fn builder() -> AstroRunBuilder {
    AstroRunBuilder::new()
  }

  pub fn cancel(&self, job_id: &JobId) -> Result<()> {
    self.shared_state.cancel(job_id)
  }

  pub fn register_plugin(&self, plugin: AstroRunPlugin) -> &Self {
    self.shared_state.register_plugin(plugin);

    self
  }

  pub fn unregister_plugin(&self, plugin_name: &'static str) -> &Self {
    self.shared_state.unregister_plugin(plugin_name);

    self
  }

  pub fn register_action<T>(&self, name: impl Into<String>, action: T) -> &Self
  where
    T: crate::actions::Action + 'static,
  {
    self.shared_state.register_action(name, action);

    self
  }

  pub fn unregister_action(&self, name: &str) -> &Self {
    self.shared_state.unregister_action(name);

    self
  }

  pub fn plugins(&self) -> PluginManager {
    self.shared_state.plugins()
  }

  pub fn actions(&self) -> Actions {
    self.shared_state.actions()
  }

  pub fn execution_context(&self) -> ExecutionContextBuilder {
    let shared_state = self.shared_state.clone();
    let mut builder = ExecutionContext::builder()
      .runner(self.runner.clone())
      .shared_state(shared_state);

    if let Some(github_auth) = &self.github_auth {
      builder = builder.github_auth(github_auth.clone());
    }

    builder
  }
}

pub struct AstroRunBuilder {
  runner: Option<Box<dyn Runner>>,
  shared_state: AstroRunSharedState,
  github_auth: Option<GithubAuthorization>,
}

impl AstroRunBuilder {
  pub fn new() -> Self {
    AstroRunBuilder {
      runner: None,
      github_auth: None,
      shared_state: AstroRunSharedState::new(),
    }
  }

  pub fn runner<T>(mut self, runner: T) -> Self
  where
    T: Runner + 'static,
  {
    self.runner = Some(Box::new(runner));
    self
  }

  pub fn plugin(self, plugin: AstroRunPlugin) -> Self {
    self.shared_state.register_plugin(plugin);
    self
  }

  pub fn action(self, name: impl Into<String>, action: impl Action + 'static) -> Self {
    self.shared_state.register_action(name, action);
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
      shared_state: self.shared_state,
      github_auth: self.github_auth,
    }
  }
}

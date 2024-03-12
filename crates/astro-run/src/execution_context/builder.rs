use super::condition_matcher::ConditionMatcher;
use crate::{
  Error, ExecutionContext, GithubAuthorization, Runner, SharedPluginDriver, SignalManager,
  WorkflowEvent,
};
use std::sync::Arc;

pub struct ExecutionContextBuilder {
  runner: Option<Arc<Box<dyn Runner>>>,
  plugin_driver: Option<SharedPluginDriver>,
  event: Option<WorkflowEvent>,
  github_auth: Option<GithubAuthorization>,
}

impl ExecutionContextBuilder {
  pub fn new() -> Self {
    ExecutionContextBuilder {
      runner: None,
      plugin_driver: None,
      event: None,
      github_auth: None,
    }
  }

  pub fn runner(mut self, runner: Arc<Box<dyn Runner>>) -> Self {
    self.runner = Some(runner);
    self
  }

  pub fn plugin_driver(mut self, plugin_driver: SharedPluginDriver) -> Self {
    self.plugin_driver = Some(plugin_driver);

    self
  }

  pub fn event(mut self, event: WorkflowEvent) -> Self {
    self.event = Some(event);
    self
  }

  pub fn github_auth(mut self, github_auth: GithubAuthorization) -> Self {
    self.github_auth = Some(github_auth);
    self
  }

  pub fn build(self) -> ExecutionContext {
    let runner = self
      .runner
      .ok_or(Error::init_error(
        "Runner is not set in execution context builder",
      ))
      .unwrap();

    let plugin_driver = self
      .plugin_driver
      .ok_or(Error::init_error(
        "Plugin driver is not set in execution context builder",
      ))
      .unwrap();

    let ctx = ExecutionContext {
      runner,
      plugin_driver,
      condition_matcher: ConditionMatcher::new(self.event, self.github_auth),
      signal_manager: SignalManager::new(),
    };

    ctx
  }
}

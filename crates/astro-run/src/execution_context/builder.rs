use super::condition_matcher::ConditionMatcher;
use crate::{
  ContextPayload, Error, ExecutionContext, GithubAuthorization, Runner, SharedPluginDriver,
  SignalManager, WorkflowEvent,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

#[derive(Default)]
pub struct ExecutionContextBuilder {
  runner: Option<Arc<Box<dyn Runner>>>,
  plugin_driver: Option<SharedPluginDriver>,
  signal_manager: Option<SignalManager>,
  event: Option<WorkflowEvent>,
  github_auth: Option<GithubAuthorization>,
  payload: Option<Box<dyn ContextPayload>>,
}

impl ExecutionContextBuilder {
  pub fn new() -> ExecutionContextBuilder {
    ExecutionContextBuilder {
      runner: None,
      plugin_driver: None,
      signal_manager: None,
      event: None,
      github_auth: None,
      payload: None,
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

  pub fn signal_manager(mut self, signal_manager: SignalManager) -> Self {
    self.signal_manager = Some(signal_manager);
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

  pub fn payload<P>(mut self, payload: P) -> Self
  where
    P: ContextPayload + 'static,
  {
    self.payload = Some(Box::new(payload) as Box<dyn ContextPayload>);
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

    let signal_manager = self
      .signal_manager
      .ok_or(Error::init_error(
        "Signal manager is not set in execution context builder",
      ))
      .unwrap();

    let payload = self.payload;

    ExecutionContext {
      runner,
      signal_manager,
      plugin_driver,
      condition_matcher: ConditionMatcher::new(self.event, self.github_auth),
      payload,
    }
  }
}

use super::condition_matcher::ConditionMatcher;
use crate::{
  shared_state::AstroRunSharedState, Error, ExecutionContext, GithubAuthorization, Runner,
  WorkflowEvent,
};
use std::sync::Arc;

pub struct ExecutionContextBuilder {
  runner: Option<Arc<Box<dyn Runner>>>,
  shared_state: Option<AstroRunSharedState>,
  event: Option<WorkflowEvent>,
  github_auth: Option<GithubAuthorization>,
}

impl ExecutionContextBuilder {
  pub fn new() -> Self {
    ExecutionContextBuilder {
      runner: None,
      shared_state: None,
      event: None,
      github_auth: None,
    }
  }

  pub fn runner(mut self, runner: Arc<Box<dyn Runner>>) -> Self {
    self.runner = Some(runner);
    self
  }

  pub fn shared_state(mut self, shared_state: AstroRunSharedState) -> Self {
    self.shared_state = Some(shared_state);
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

    let shared_state = self
      .shared_state
      .unwrap_or_else(|| AstroRunSharedState::new());

    let ctx = ExecutionContext {
      runner,
      shared_state,
      condition_matcher: ConditionMatcher::new(self.event, self.github_auth),
    };

    ctx
  }
}

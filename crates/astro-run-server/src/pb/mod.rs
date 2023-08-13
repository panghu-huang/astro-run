#![cfg(not(tarpaulin_include))]
#![allow(dead_code, non_snake_case)]
mod astro;
mod results;

use std::time::Duration;

pub use astro::*;

impl Into<astro_run::WorkflowState> for WorkflowState {
  fn into(self) -> astro_run::WorkflowState {
    match self {
      WorkflowState::Pending => astro_run::WorkflowState::Pending,
      WorkflowState::Queued => astro_run::WorkflowState::Queued,
      WorkflowState::InProgress => astro_run::WorkflowState::InProgress,
      WorkflowState::Succeeded => astro_run::WorkflowState::Succeeded,
      WorkflowState::Failed => astro_run::WorkflowState::Failed,
      WorkflowState::Cancelled => astro_run::WorkflowState::Cancelled,
      WorkflowState::Skipped => astro_run::WorkflowState::Skipped,
    }
  }
}

impl TryInto<astro_run::ContainerOptions> for Container {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::ContainerOptions, Self::Error> {
    Ok(astro_run::ContainerOptions {
      name: self.name,
      volumes: Some(vec![]),
      security_opts: Some(vec![]),
    })
  }
}

impl TryFrom<astro_run::ContainerOptions> for Container {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::ContainerOptions) -> Result<Self, Self::Error> {
    Ok(Container { name: value.name })
  }
}

impl TryInto<astro_run::Command> for Command {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::Command, Self::Error> {
    Ok(astro_run::Command {
      id: astro_run::StepId::try_from(self.id.as_str())?,
      name: self.name,
      container: self.container.map(|c| c.try_into()).transpose()?,
      run: self.run,
      continue_on_error: self.continue_on_error,
      environments: astro_run::EnvironmentVariables::default(),
      secrets: vec![],
      timeout: Duration::from_secs(60 * 60),
    })
  }
}

impl TryFrom<astro_run::Command> for Command {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Command) -> Result<Self, Self::Error> {
    Ok(Command {
      id: value.id.to_string(),
      name: value.name,
      container: value.container.map(|c| c.try_into()).transpose()?,
      run: value.run,
      continue_on_error: value.continue_on_error,
    })
  }
}

impl TryInto<astro_run::Context> for Context {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::Context, Self::Error> {
    Ok(astro_run::Context {
      id: self.id,
      command: self
        .command
        .ok_or(astro_run::Error::internal_runtime_error(
          "Command is missing",
        ))?
        .try_into()?,
    })
  }
}

impl TryFrom<astro_run::Context> for Context {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Context) -> Result<Self, Self::Error> {
    Ok(Context {
      id: value.id,
      command: Some(value.command.try_into()?),
    })
  }
}

impl From<astro_run::RunResult> for report_run_completed_request::Result {
  fn from(value: astro_run::RunResult) -> Self {
    match value {
      astro_run::RunResult::Cancelled => report_run_completed_request::Result::Cancelled(0),
      astro_run::RunResult::Succeeded => report_run_completed_request::Result::Succeeded(0),
      astro_run::RunResult::Failed { exit_code } => {
        report_run_completed_request::Result::Failed(exit_code)
      }
    }
  }
}

impl Into<astro_run::RunResult> for report_run_completed_request::Result {
  fn into(self) -> astro_run::RunResult {
    match self {
      report_run_completed_request::Result::Cancelled(_) => astro_run::RunResult::Cancelled,
      report_run_completed_request::Result::Failed(exit_code) => {
        astro_run::RunResult::Failed { exit_code }
      }
      report_run_completed_request::Result::Succeeded(_) => astro_run::RunResult::Succeeded,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_into_workflow_state() {
    let state = WorkflowState::Pending;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::Pending);

    let state = WorkflowState::Queued;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::Queued);

    let state = WorkflowState::InProgress;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::InProgress);

    let state = WorkflowState::Succeeded;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::Succeeded);

    let state = WorkflowState::Failed;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::Failed);

    let state = WorkflowState::Cancelled;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::Cancelled);

    let state = WorkflowState::Skipped;
    let astro_state: astro_run::WorkflowState = state.into();

    assert_eq!(astro_state, astro_run::WorkflowState::Skipped);
  }
}

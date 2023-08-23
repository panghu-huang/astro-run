mod pb;
#[cfg(feature = "astro-run-remote-runner")]
mod remote_runner_events;
mod results;
#[cfg(feature = "astro-run-server")]
mod server_events;
mod utils;
mod workflows;

pub use pb::*;
use std::{collections::HashMap, time::Duration};
pub use tonic;

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
      volumes: Some(self.volumes),
      security_opts: Some(self.security_opts),
    })
  }
}

impl TryFrom<astro_run::ContainerOptions> for Container {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::ContainerOptions) -> Result<Self, Self::Error> {
    Ok(Container {
      name: value.name,
      volumes: value.volumes.unwrap_or_default(),
      security_opts: value.security_opts.unwrap_or_default(),
    })
  }
}

impl TryInto<astro_run::EnvironmentVariable> for environment_variable::Value {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::EnvironmentVariable, Self::Error> {
    let env = match self {
      environment_variable::Value::String(value) => astro_run::EnvironmentVariable::String(value),
      environment_variable::Value::Number(value) => {
        astro_run::EnvironmentVariable::Number(value as f64)
      }
      environment_variable::Value::Boolean(value) => astro_run::EnvironmentVariable::Boolean(value),
    };

    Ok(env)
  }
}

impl TryFrom<astro_run::EnvironmentVariable> for environment_variable::Value {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::EnvironmentVariable) -> Result<Self, Self::Error> {
    let env = match value {
      astro_run::EnvironmentVariable::String(value) => environment_variable::Value::String(value),
      astro_run::EnvironmentVariable::Number(value) => {
        environment_variable::Value::Number(value as f32)
      }
      astro_run::EnvironmentVariable::Boolean(value) => environment_variable::Value::Boolean(value),
    };

    Ok(env)
  }
}

impl TryInto<astro_run::Command> for Command {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::Command, Self::Error> {
    let mut environments: astro_run::EnvironmentVariables = HashMap::new();
    for (key, env) in self.environments {
      let value = env
        .value
        .ok_or(astro_run::Error::internal_runtime_error(
          "Environment variable value is missing",
        ))?
        .try_into()?;
      environments.insert(key, value);
    }

    Ok(astro_run::Command {
      id: astro_run::StepId::try_from(self.id.as_str())?,
      name: self.name,
      container: self.container.map(|c| c.try_into()).transpose()?,
      run: self.run,
      continue_on_error: self.continue_on_error,
      environments,
      secrets: self.secrets,
      timeout: Duration::from_secs(60 * 60),
    })
  }
}

impl TryFrom<astro_run::Command> for Command {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Command) -> Result<Self, Self::Error> {
    let mut environments = HashMap::new();

    for (key, value) in value.environments {
      let value = environment_variable::Value::try_from(value)?;
      environments.insert(key, EnvironmentVariable { value: Some(value) });
    }

    Ok(Command {
      id: value.id.to_string(),
      name: value.name,
      container: value.container.map(|c| c.try_into()).transpose()?,
      run: value.run,
      continue_on_error: value.continue_on_error,
      environments,
      secrets: value.secrets,
      timeout: value.timeout.as_secs(),
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

impl TryInto<astro_run::WorkflowLog> for WorkflowLog {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::WorkflowLog, Self::Error> {
    let time = utils::convert_timestamp_to_datetime(&self.time)?;

    Ok(astro_run::WorkflowLog {
      step_id: astro_run::StepId::try_from(self.step_id.as_str())?,
      message: self.message,
      log_type: astro_run::WorkflowLogType::from(self.log_type),
      time: time.ok_or(astro_run::Error::internal_runtime_error(
        "Failed to convert timestamp to datetime",
      ))?,
    })
  }
}

impl TryFrom<astro_run::WorkflowLog> for WorkflowLog {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::WorkflowLog) -> Result<Self, Self::Error> {
    let time = utils::convert_datetime_to_timestamp(&Some(value.time))?;

    Ok(WorkflowLog {
      step_id: value.step_id.to_string(),
      message: value.message,
      log_type: value.log_type.to_string(),
      time,
    })
  }
}

impl TryInto<astro_run::WorkflowStateEvent> for WorkflowStateEvent {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::WorkflowStateEvent, Self::Error> {
    let state: astro_run::WorkflowState = WorkflowState::from_i32(self.state)
      .ok_or(astro_run::Error::internal_runtime_error(format!(
        "Invalid WorkflowState value: {}",
        self.state
      )))?
      .into();
    let event = match self.r#type.as_str() {
      "workflow" => {
        let id = astro_run::WorkflowId::try_from(self.id.as_str())?;
        astro_run::WorkflowStateEvent::WorkflowStateUpdated { id, state }
      }
      "job" => {
        let id = astro_run::JobId::try_from(self.id.as_str())?;
        astro_run::WorkflowStateEvent::JobStateUpdated { id, state }
      }
      "step" => {
        let id = astro_run::StepId::try_from(self.id.as_str())?;
        astro_run::WorkflowStateEvent::StepStateUpdated { id, state }
      }
      _ => {
        return Err(astro_run::Error::internal_runtime_error(format!(
          "Invalid WorkflowStateEvent type: {}",
          self.r#type
        )))
      }
    };

    Ok(event)
  }
}

impl TryFrom<astro_run::WorkflowStateEvent> for WorkflowStateEvent {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::WorkflowStateEvent) -> Result<Self, Self::Error> {
    let res = match value {
      astro_run::WorkflowStateEvent::WorkflowStateUpdated { id, state } => {
        let id = id.to_string();
        let state = state as i32;
        WorkflowStateEvent {
          r#type: "workflow".to_string(),
          id,
          state,
        }
      }
      astro_run::WorkflowStateEvent::JobStateUpdated { id, state } => {
        let id = id.to_string();
        let state = state as i32;
        WorkflowStateEvent {
          r#type: "job".to_string(),
          id,
          state,
        }
      }
      astro_run::WorkflowStateEvent::StepStateUpdated { id, state } => {
        let id = id.to_string();
        let state = state as i32;
        WorkflowStateEvent {
          r#type: "step".to_string(),
          id,
          state,
        }
      }
    };

    Ok(res)
  }
}

impl TryInto<astro_run_scheduler::RunnerMetadata> for RunnerMetadata {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run_scheduler::RunnerMetadata, Self::Error> {
    Ok(astro_run_scheduler::RunnerMetadata {
      id: self.id,
      version: self.version,
      os: self.os,
      arch: self.arch,
      support_docker: self.support_docker,
      support_host: self.support_host,
      max_runs: self.max_runs,
    })
  }
}

impl TryFrom<astro_run_scheduler::RunnerMetadata> for RunnerMetadata {
  type Error = astro_run::Error;

  fn try_from(value: astro_run_scheduler::RunnerMetadata) -> Result<Self, Self::Error> {
    Ok(RunnerMetadata {
      id: value.id,
      version: value.version,
      os: value.os,
      arch: value.arch,
      support_docker: value.support_docker,
      support_host: value.support_host,
      max_runs: value.max_runs,
    })
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

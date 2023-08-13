use super::{utils::*, *};
use std::collections::HashMap;

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

impl TryInto<astro_run::StepRunResult> for StepRunResult {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::StepRunResult, Self::Error> {
    let started_at = convert_timestamp_to_datetime(&self.started_at)?;
    let completed_at = convert_timestamp_to_datetime(&self.completed_at)?;

    Ok(astro_run::StepRunResult {
      id: astro_run::StepId::try_from(self.id.as_str())?,
      state: WorkflowState::from_i32(self.state)
        .ok_or(astro_run::Error::internal_runtime_error(format!(
          "Invalid WorkflowState value: {}",
          self.state
        )))?
        .into(),
      exit_code: self.exit_code,
      started_at,
      completed_at,
    })
  }
}

impl TryFrom<astro_run::StepRunResult> for StepRunResult {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::StepRunResult) -> Result<Self, Self::Error> {
    let started_at = convert_datetime_to_timestamp(&value.started_at)?;
    let completed_at = convert_datetime_to_timestamp(&value.completed_at)?;

    Ok(StepRunResult {
      id: value.id.to_string(),
      state: value.state as i32,
      exit_code: value.exit_code,
      started_at,
      completed_at,
    })
  }
}

impl TryInto<astro_run::JobRunResult> for JobRunResult {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::JobRunResult, Self::Error> {
    let started_at = convert_timestamp_to_datetime(&self.started_at)?;
    let completed_at = convert_timestamp_to_datetime(&self.completed_at)?;

    let mut steps = Vec::new();
    for step in self.steps {
      steps.push(step.try_into()?);
    }

    Ok(astro_run::JobRunResult {
      id: astro_run::JobId::try_from(self.id.as_str())?,
      state: WorkflowState::from_i32(self.state)
        .ok_or(astro_run::Error::internal_runtime_error(format!(
          "Invalid WorkflowState value: {}",
          self.state
        )))?
        .into(),
      started_at,
      completed_at,
      steps,
    })
  }
}

impl TryFrom<astro_run::JobRunResult> for JobRunResult {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::JobRunResult) -> Result<Self, Self::Error> {
    let started_at = convert_datetime_to_timestamp(&value.started_at)?;
    let completed_at = convert_datetime_to_timestamp(&value.completed_at)?;

    let mut steps = Vec::new();
    for step in value.steps {
      steps.push(step.try_into()?);
    }

    Ok(JobRunResult {
      id: value.id.to_string(),
      state: value.state as i32,
      started_at,
      completed_at,
      steps,
    })
  }
}

impl TryInto<astro_run::WorkflowRunResult> for WorkflowRunResult {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::WorkflowRunResult, Self::Error> {
    let started_at = convert_timestamp_to_datetime(&self.started_at)?;
    let completed_at = convert_timestamp_to_datetime(&self.completed_at)?;

    let mut jobs = HashMap::new();
    for (key, job) in self.jobs {
      jobs.insert(key, job.try_into()?);
    }

    Ok(astro_run::WorkflowRunResult {
      id: astro_run::WorkflowId::try_from(self.id.as_str())?,
      state: WorkflowState::from_i32(self.state)
        .ok_or(astro_run::Error::internal_runtime_error(format!(
          "Invalid WorkflowState value: {}",
          self.state
        )))?
        .into(),
      started_at,
      completed_at,
      jobs,
    })
  }
}

impl TryFrom<astro_run::WorkflowRunResult> for WorkflowRunResult {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::WorkflowRunResult) -> Result<Self, Self::Error> {
    let started_at = convert_datetime_to_timestamp(&value.started_at)?;
    let completed_at = convert_datetime_to_timestamp(&value.completed_at)?;

    let mut jobs = HashMap::new();
    for (key, job) in value.jobs {
      jobs.insert(key, job.try_into()?);
    }

    Ok(WorkflowRunResult {
      id: value.id.to_string(),
      state: value.state as i32,
      started_at,
      completed_at,
      jobs,
    })
  }
}

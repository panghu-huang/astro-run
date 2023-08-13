use super::*;
use std::{collections::HashMap, str::FromStr};

fn convert_timestamp_to_datetime(
  timestamp: &Option<prost_types::Timestamp>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, astro_run::Error> {
  let res = match timestamp {
    Some(t) => Some(
      chrono::DateTime::from_str(&t.to_string())
        .map_err(|_| astro_run::Error::internal_runtime_error("Invalid timestamp"))?,
    ),
    None => None,
  };

  Ok(res)
}

fn convert_datetime_to_timestamp(
  datetime: &Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Option<prost_types::Timestamp>, astro_run::Error> {
  let res = match datetime {
    Some(t) => Some(
      prost_types::Timestamp::from_str(&t.to_rfc3339())
        .map_err(|_| astro_run::Error::internal_runtime_error("Invalid timestamp"))?,
    ),
    None => None,
  };

  Ok(res)
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

impl TryFrom<astro_run::JobRunResult> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::JobRunResult) -> Result<Self, Self::Error> {
    let result = JobRunResult::try_from(value)?;

    Ok(Event {
      event_name: "job_completed".to_string(),
      payload: Some(event::Payload::JobCompletedEvent(result)),
    })
  }
}

impl TryFrom<astro_run::WorkflowRunResult> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::WorkflowRunResult) -> Result<Self, Self::Error> {
    let result = WorkflowRunResult::try_from(value)?;

    Ok(Event {
      event_name: "workflow_completed".to_string(),
      payload: Some(event::Payload::WorkflowCompletedEvent(result)),
    })
  }
}

impl TryFrom<astro_run::Error> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Error) -> Result<Self, Self::Error> {
    Ok(Event {
      event_name: "error".to_string(),
      payload: Some(event::Payload::Error(value.to_string())),
    })
  }
}

impl TryFrom<astro_run::Context> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Context) -> Result<Self, Self::Error> {
    let ctx = value.try_into()?;

    Ok(Event {
      event_name: "run".to_string(),
      payload: Some(event::Payload::Run(ctx)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert_datatime_to_timestamp() {
    let datetime = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap();
    let timestamp = convert_datetime_to_timestamp(&Some(datetime.into())).unwrap();
    assert_eq!(timestamp.unwrap().seconds, 1609459200);
  }

  #[test]
  fn test_convert_timestamp_to_datetime() {
    let timestamp = prost_types::Timestamp {
      seconds: 1609459200,
      nanos: 0,
    };
    let datetime = convert_timestamp_to_datetime(&Some(timestamp)).unwrap();
    assert_eq!(datetime.unwrap().to_rfc3339(), "2021-01-01T00:00:00+00:00");
  }

  #[test]
  fn test_convert_step_run_result() {
    let step_run_result = StepRunResult {
      id: "workflow/job/1".to_string(),
      state: 3,
      exit_code: Some(0),
      started_at: Some(prost_types::Timestamp {
        seconds: 1609459200,
        nanos: 0,
      }),
      completed_at: Some(prost_types::Timestamp {
        seconds: 1609459200,
        nanos: 0,
      }),
    };

    let step_run_result: astro_run::StepRunResult = step_run_result.try_into().unwrap();
    assert_eq!(step_run_result.id.to_string(), "workflow/job/1");

    let state: astro_run::WorkflowState = step_run_result.state.try_into().unwrap();
    assert_eq!(state, astro_run::WorkflowState::Succeeded);
    assert_eq!(step_run_result.exit_code.unwrap(), 0);
    assert_eq!(
      step_run_result.started_at.unwrap().to_rfc3339(),
      "2021-01-01T00:00:00+00:00"
    );
    assert_eq!(
      step_run_result.completed_at.unwrap().to_rfc3339(),
      "2021-01-01T00:00:00+00:00"
    );
  }

  #[test]
  fn test_convert_job_run_result() {
    let job_run_result = JobRunResult {
      id: "workflow/job".to_string(),
      state: 3,
      started_at: Some(prost_types::Timestamp {
        seconds: 1609459200,
        nanos: 0,
      }),
      completed_at: Some(prost_types::Timestamp {
        seconds: 1609459200,
        nanos: 0,
      }),
      steps: vec![StepRunResult {
        id: "workflow/job/1".to_string(),
        state: 3,
        exit_code: Some(0),
        started_at: Some(prost_types::Timestamp {
          seconds: 1609459200,
          nanos: 0,
        }),
        completed_at: Some(prost_types::Timestamp {
          seconds: 1609459200,
          nanos: 0,
        }),
      }],
    };

    let job_run_result: astro_run::JobRunResult = job_run_result.try_into().unwrap();
    assert_eq!(job_run_result.id.to_string(), "workflow/job");

    let state: astro_run::WorkflowState = job_run_result.state.try_into().unwrap();
    assert_eq!(state, astro_run::WorkflowState::Succeeded);
    assert_eq!(
      job_run_result.started_at.unwrap().to_rfc3339(),
      "2021-01-01T00:00:00+00:00"
    );
    assert_eq!(
      job_run_result.completed_at.unwrap().to_rfc3339(),
      "2021-01-01T00:00:00+00:00"
    );
    assert_eq!(job_run_result.steps.len(), 1);
  }
}

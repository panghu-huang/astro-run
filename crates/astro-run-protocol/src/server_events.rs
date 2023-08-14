use super::astro_run_server::{event, Event};
use super::*;

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

impl TryFrom<astro_run::Job> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Job) -> Result<Self, Self::Error> {
    let job = value.try_into()?;

    Ok(Event {
      event_name: "run_job".to_string(),
      payload: Some(event::Payload::RunJobEvent(job)),
    })
  }
}

impl TryFrom<astro_run::Workflow> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Workflow) -> Result<Self, Self::Error> {
    let workflow = value.try_into()?;

    Ok(Event {
      event_name: "run_workflow".to_string(),
      payload: Some(event::Payload::RunWorkflowEvent(workflow)),
    })
  }
}

impl TryFrom<astro_run::WorkflowLog> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::WorkflowLog) -> Result<Self, Self::Error> {
    let log = value.try_into()?;

    Ok(Event {
      event_name: "log".to_string(),
      payload: Some(event::Payload::LogEvent(log)),
    })
  }
}

impl TryFrom<astro_run::WorkflowStateEvent> for Event {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::WorkflowStateEvent) -> Result<Self, Self::Error> {
    let event = value.try_into()?;

    Ok(Event {
      event_name: "workflow_state_change".to_string(),
      payload: Some(event::Payload::WorkflowStateEvent(event)),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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

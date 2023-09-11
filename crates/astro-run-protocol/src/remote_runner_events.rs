use super::astro_run_remote_runner::{event, run_response, Event, RunResponse};
use super::*;

impl RunResponse {
  pub fn log(id: String, log: astro_run::Log) -> astro_run::Result<Self> {
    let workflow_log = WorkflowLog {
      step_id: id.clone(),
      message: log.message,
      log_type: log.log_type.to_string(),
      time: utils::convert_datetime_to_timestamp(&Some(chrono::Utc::now()))?,
    };

    Ok(Self {
      id,
      payload: Some(run_response::Payload::Log(workflow_log)),
    })
  }

  pub fn end(id: String, result: astro_run::RunResult) -> astro_run::Result<Self> {
    let result = RunResult {
      result: Some(result.into()),
    };

    Ok(Self {
      id,
      payload: Some(run_response::Payload::Result(result)),
    })
  }
}

impl Event {
  pub fn new_signal(id: String, signal: astro_run::Signal) -> Self {
    Self {
      event_name: "signal".to_string(),
      payload: Some(event::Payload::SignalEvent(Signal {
        id,
        action: signal.to_string(),
      })),
    }
  }
}

impl TryFrom<astro_run::WorkflowLog> for Event {
  type Error = astro_run::Error;

  fn try_from(log: astro_run::WorkflowLog) -> Result<Self, Self::Error> {
    let workflow_log = WorkflowLog {
      step_id: log.step_id.to_string(),
      message: log.message,
      log_type: log.log_type.to_string(),
      time: utils::convert_datetime_to_timestamp(&Some(chrono::Utc::now()))?,
    };

    Ok(Self {
      event_name: "log".to_string(),
      payload: Some(event::Payload::LogEvent(workflow_log)),
    })
  }
}

impl TryFrom<astro_run::StepRunResult> for Event {
  type Error = astro_run::Error;

  fn try_from(result: astro_run::StepRunResult) -> Result<Self, Self::Error> {
    let event = event::Payload::StepCompletedEvent(result.try_into()?);

    Ok(Self {
      event_name: "step_completed".to_string(),
      payload: Some(event),
    })
  }
}

impl TryFrom<astro_run::JobRunResult> for Event {
  type Error = astro_run::Error;

  fn try_from(result: astro_run::JobRunResult) -> Result<Self, Self::Error> {
    let event = event::Payload::JobCompletedEvent(result.try_into()?);

    Ok(Self {
      event_name: "job_completed".to_string(),
      payload: Some(event),
    })
  }
}

impl TryFrom<astro_run::WorkflowRunResult> for Event {
  type Error = astro_run::Error;

  fn try_from(result: astro_run::WorkflowRunResult) -> Result<Self, Self::Error> {
    let event = event::Payload::WorkflowCompletedEvent(result.try_into()?);

    Ok(Self {
      event_name: "workflow_completed".to_string(),
      payload: Some(event),
    })
  }
}

impl TryFrom<astro_run::WorkflowStateEvent> for Event {
  type Error = astro_run::Error;

  fn try_from(event: astro_run::WorkflowStateEvent) -> Result<Self, Self::Error> {
    let event = event::Payload::WorkflowStateEvent(event.try_into()?);

    Ok(Self {
      event_name: "workflow_state_change".to_string(),
      payload: Some(event),
    })
  }
}

impl TryFrom<astro_run::RunStepEvent> for Event {
  type Error = astro_run::Error;

  fn try_from(event: astro_run::RunStepEvent) -> Result<Self, Self::Error> {
    let event = event::Payload::RunStepEvent(event.try_into()?);

    Ok(Self {
      event_name: "run_step".to_string(),
      payload: Some(event),
    })
  }
}

impl TryFrom<astro_run::RunJobEvent> for Event {
  type Error = astro_run::Error;

  fn try_from(event: astro_run::RunJobEvent) -> Result<Self, Self::Error> {
    let event = event::Payload::RunJobEvent(event.try_into()?);

    Ok(Self {
      event_name: "run_job".to_string(),
      payload: Some(event),
    })
  }
}

impl TryFrom<astro_run::RunWorkflowEvent> for Event {
  type Error = astro_run::Error;

  fn try_from(event: astro_run::RunWorkflowEvent) -> Result<Self, Self::Error> {
    let event = event::Payload::RunWorkflowEvent(event.try_into()?);

    Ok(Self {
      event_name: "run_workflow".to_string(),
      payload: Some(event),
    })
  }
}

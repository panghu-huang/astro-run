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
  pub fn new_log(log: astro_run::WorkflowLog) -> astro_run::Result<Self> {
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

  pub fn new_job_completed(result: astro_run::JobRunResult) -> astro_run::Result<Self> {
    let event = event::Payload::JobCompletedEvent(result.try_into()?);

    Ok(Self {
      event_name: "job_completed".to_string(),
      payload: Some(event),
    })
  }

  pub fn new_workflow_completed(result: astro_run::WorkflowRunResult) -> astro_run::Result<Self> {
    let event = event::Payload::WorkflowCompletedEvent(result.try_into()?);

    Ok(Self {
      event_name: "workflow_completed".to_string(),
      payload: Some(event),
    })
  }

  pub fn new_state_change(event: astro_run::WorkflowStateEvent) -> astro_run::Result<Self> {
    let event = event::Payload::WorkflowStateEvent(event.try_into()?);

    Ok(Self {
      event_name: "workflow_state_change".to_string(),
      payload: Some(event),
    })
  }

  pub fn new_run_job(job: astro_run::Job) -> astro_run::Result<Self> {
    let job = job.try_into()?;

    Ok(Self {
      event_name: "run_job".to_string(),
      payload: Some(event::Payload::RunJobEvent(job)),
    })
  }

  pub fn new_run_workflow(workflow: astro_run::Workflow) -> astro_run::Result<Self> {
    let workflow = workflow.try_into()?;

    Ok(Self {
      event_name: "run_workflow".to_string(),
      payload: Some(event::Payload::RunWorkflowEvent(workflow)),
    })
  }
}

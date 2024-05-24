mod builder;
mod job;
mod parser;
mod step;

pub use self::job::Job;
pub use self::step::Step;
use crate::{
  Condition, Error, ExecutionContext, Id, JobRunResult, WorkflowId, WorkflowRunResult,
  WorkflowState, WorkflowStateEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Sender};

// Job key, JobRunResult
type Result = (Id, JobRunResult);

pub trait Payload: Send + Sync {
  fn try_from_string(payload: &str) -> crate::Result<Self>
  where
    Self: Sized;
  fn try_into_string(&self) -> crate::Result<String>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workflow {
  pub id: WorkflowId,
  pub name: Option<String>,
  pub on: Option<Condition>,
  pub jobs: HashMap<String, Job>,
  pub payload: Option<String>,
}

impl Workflow {
  pub async fn run(&self, ctx: ExecutionContext) -> WorkflowRunResult {
    if self.should_skip(&ctx).await {
      ctx
        .call_on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
          id: self.id.clone(),
          state: WorkflowState::Skipped,
        })
        .await;

      return WorkflowRunResult {
        id: self.id.clone(),
        state: WorkflowState::Skipped,
        started_at: None,
        completed_at: None,
        jobs: HashMap::new(),
      };
    }

    let started_at = chrono::Utc::now();

    let mut workflow_state = WorkflowState::InProgress;
    // Dispatch run workflow event
    ctx.call_on_run_workflow(self.clone()).await;
    ctx
      .call_on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
        id: self.id.clone(),
        state: workflow_state.clone(),
      })
      .await;

    let (sender, mut receiver) = channel::<Result>(10);

    let mut waiting_jobs: Vec<(Id, Job)> = vec![];
    let mut job_results: HashMap<String, JobRunResult> = HashMap::new();

    for (key, job) in self.jobs.iter() {
      let key = key.clone();
      let job = job.clone();

      if !job.depends_on.is_empty() {
        for depends_on_key in &job.depends_on {
          // In the user config, there are checks, so this is unlikely to occur here
          #[cfg(not(tarpaulin_include))]
          if !self.jobs.contains_key(depends_on_key) {
            log::error!(
              "Job {} depends on job {} which does not exist",
              key,
              depends_on_key
            );
            workflow_state = WorkflowState::Failed;
            break;
          }
        }

        waiting_jobs.push((key, job));
        continue;
      }

      self.run_job(key.clone(), job.clone(), ctx.clone(), sender.clone());
    }

    let total_jobs = self.jobs.len();

    // If there are no jobs to run, we are done
    while let Some((key, job_result)) = receiver.recv().await {
      if job_result.state == WorkflowState::Failed {
        workflow_state = WorkflowState::Failed;
      } else if job_result.state == WorkflowState::Cancelled {
        workflow_state = WorkflowState::Cancelled;
      }

      waiting_jobs.retain(|(k, _)| k != &key);
      job_results.insert(key, job_result);

      if job_results.len() == total_jobs {
        if workflow_state == WorkflowState::InProgress {
          workflow_state = WorkflowState::Succeeded;
        }
        break;
      }

      for (job_id, job) in waiting_jobs.iter() {
        if !job.depends_on.is_empty() {
          let mut all_finished = true;
          for depends_on_key in &job.depends_on {
            if !job_results.contains_key(depends_on_key) {
              all_finished = false;
              break;
            }
          }

          if all_finished {
            self.run_job(job_id.clone(), job.clone(), ctx.clone(), sender.clone());
          }
        }
      }
    }

    let completed_at = chrono::Utc::now();

    log::trace!(
      "Duration: {:?}ms",
      completed_at.timestamp_millis() - started_at.timestamp_millis()
    );

    ctx
      .call_on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
        id: self.id.clone(),
        state: workflow_state.clone(),
      })
      .await;

    let result = WorkflowRunResult {
      id: self.id.clone(),
      state: workflow_state,
      started_at: Some(started_at),
      completed_at: Some(completed_at),
      jobs: job_results,
    };

    ctx.call_on_workflow_completed(result.clone()).await;

    result
  }

  fn run_job(&self, key: Id, job: job::Job, context: ExecutionContext, sender: Sender<Result>) {
    tokio::spawn(async move {
      let result = job.run(context).await;

      if let Err(err) = sender.send((key.clone(), result)).await {
        log::error!("Failed to send job result for job {}: {}", key, err);
      }
    });
  }

  pub async fn should_skip(&self, ctx: &ExecutionContext) -> bool {
    if let Some(on) = &self.on {
      !ctx.is_match(on).await
    } else {
      false
    }
  }

  pub fn payload<T>(&self) -> crate::Result<T>
  where
    T: Payload,
  {
    if let Some(payload) = &self.payload {
      T::try_from_string(payload)
    } else {
      Err(Error::error("Payload is not set for this workflow"))
    }
  }

  pub fn update_payload<T>(&mut self, payload: &T) -> crate::Result<()>
  where
    T: Payload,
  {
    self.payload = Some(payload.try_into_string()?);
    Ok(())
  }

  pub fn clear_payload(&mut self) {
    self.payload = None;
  }

  pub fn builder() -> builder::WorkflowBuilder {
    builder::WorkflowBuilder::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{async_trait, AstroRun, Context, Error, Result, RunResponse, Runner};

  struct TestRunner;

  #[async_trait]
  impl Runner for TestRunner {
    async fn run(&self, _ctx: Context) -> RunResponse {
      unreachable!("TestRunner should not be called")
    }
  }

  #[astro_run_test::test]
  async fn test_workflow_payload() {
    #[derive(Debug)]
    struct WorkflowPayload(String);

    impl crate::Payload for WorkflowPayload {
      fn try_into_string(&self) -> Result<String> {
        Ok(self.0.clone())
      }

      fn try_from_string(payload: &str) -> Result<Self> {
        Ok(WorkflowPayload(payload.to_owned()))
      }
    }

    let workflow = r#"
      jobs:
        test:
          steps:
            - run: echo "Hello World"
      "#;

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let mut workflow = Workflow::builder()
      .config(workflow)
      .payload(WorkflowPayload("Hello World".to_string()))
      .build(&astro_run)
      .await
      .unwrap();

    let payload = workflow.payload::<WorkflowPayload>().unwrap();

    assert_eq!(payload.0, "Hello World");

    workflow
      .update_payload(&WorkflowPayload("Hello World Updated".to_string()))
      .unwrap();

    let payload = workflow.payload::<WorkflowPayload>().unwrap();

    assert_eq!(payload.0, "Hello World Updated");

    workflow.clear_payload();

    let result = workflow.payload::<WorkflowPayload>();

    assert_eq!(
      result.unwrap_err(),
      Error::error("Payload is not set for this workflow")
    );
  }

  #[astro_run_test::test]
  async fn test_parse_workflow_payload_error() {
    #[derive(Debug)]
    struct WorkflowPayload;

    impl crate::Payload for WorkflowPayload {
      fn try_into_string(&self) -> Result<String> {
        Ok("".to_string())
      }

      fn try_from_string(_payload: &str) -> Result<Self> {
        Err(Error::error("Payload error"))
      }
    }

    let workflow = r#"
        jobs:
          test:
            steps:
              - run: echo "Hello World"
        "#;

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let workflow = Workflow::builder()
      .config(workflow)
      .payload(WorkflowPayload)
      .build(&astro_run)
      .await
      .unwrap();

    let result = workflow.payload::<WorkflowPayload>();

    assert_eq!(result.unwrap_err(), Error::error("Payload error"));
  }
}

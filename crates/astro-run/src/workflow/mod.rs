mod builder;
mod job;
mod parser;

pub use self::job::Job;
use crate::{
  ExecutionContext, Id, JobRunResult, WorkflowAPIEvent, WorkflowId, WorkflowRunResult,
  WorkflowState, WorkflowStateEvent, WorkflowTriggerEvents,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Sender};

pub type Step = crate::Command;

// Job key, JobRunResult
type Result = (Id, JobRunResult);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workflow {
  pub id: WorkflowId,
  pub name: Option<String>,
  pub event: WorkflowAPIEvent,
  pub on: Option<WorkflowTriggerEvents>,
  pub jobs: HashMap<String, Job>,
}

impl Workflow {
  pub async fn run(&self, ctx: ExecutionContext) -> WorkflowRunResult {
    let started_at = chrono::Utc::now();

    let mut workflow_state = WorkflowState::InProgress;
    // Dispatch run workflow event
    ctx.on_run_workflow(self.clone());
    ctx.on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
      id: self.id.clone(),
      state: workflow_state.clone(),
    });

    let (sender, mut receiver) = channel::<Result>(10);

    let mut waiting_jobs: Vec<(Id, Job)> = vec![];
    let mut job_results: HashMap<String, JobRunResult> = HashMap::new();

    for (key, job) in self.jobs.iter() {
      let key = key.clone();
      let job = job.clone();

      if !job.depends_on.is_empty() {
        for depends_on_key in &job.depends_on {
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

    log::info!(
      "Duration: {:?}ms",
      completed_at.timestamp_millis() - started_at.timestamp_millis()
    );

    ctx.on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
      id: self.id.clone(),
      state: workflow_state.clone(),
    });

    let result = WorkflowRunResult {
      id: self.id.clone(),
      state: workflow_state,
      started_at: Some(started_at),
      completed_at: Some(completed_at),
      jobs: job_results,
    };

    ctx.on_workflow_completed(result.clone());

    result
  }

  fn run_job(&self, key: Id, job: job::Job, context: ExecutionContext, sender: Sender<Result>) {
    let _res = tokio::spawn(async move {
      let result = job.run(context).await;

      sender.send((key.clone(), result)).await.map_err(|_| {
        crate::Error::internal_runtime_error(format!("Failed to send result for job {}", key))
      })?;

      Ok::<(), crate::Error>(())
    });
  }

  pub fn builder() -> builder::WorkflowBuilder {
    builder::WorkflowBuilder::new()
  }
}

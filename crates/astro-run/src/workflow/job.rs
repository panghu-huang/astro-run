use super::Step;
use crate::{
  Condition, ExecutionContext, JobId, JobRunResult, StepRunResult, WorkflowState,
  WorkflowStateEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
  pub id: JobId,
  pub name: Option<String>,
  pub on: Option<Condition>,
  pub steps: Vec<Step>,
  /// For workflow run
  pub depends_on: Vec<String>,
  pub working_directories: Vec<String>,
}

impl Job {
  pub async fn run(&self, ctx: ExecutionContext) -> JobRunResult {
    if let Some(on) = &self.on {
      if !ctx.is_match(on).await {
        ctx.on_state_change(WorkflowStateEvent::JobStateUpdated {
          id: self.id.clone(),
          state: WorkflowState::Skipped,
        });

        return JobRunResult {
          id: self.id.clone(),
          state: WorkflowState::Skipped,
          started_at: None,
          completed_at: None,
          steps: vec![],
        };
      }
    }

    let started_at = chrono::Utc::now();
    let mut job_state = WorkflowState::InProgress;

    // Dispatch run job event
    ctx.on_run_job(self.clone());
    ctx.on_state_change(WorkflowStateEvent::JobStateUpdated {
      id: self.id.clone(),
      state: job_state.clone(),
    });

    let mut steps = Vec::new();

    for step in self.steps.iter().cloned() {
      let mut skipped = match job_state {
        WorkflowState::Failed => !step.continue_on_error,
        WorkflowState::Cancelled | WorkflowState::Skipped => true,
        _ => false,
      };

      if !skipped {
        if let Some(on) = &step.on {
          if !ctx.is_match(on).await {
            skipped = true;
          }
        }
      }

      if skipped {
        log::trace!("Step {} is skipped", step.id.to_string());

        ctx.on_state_change(WorkflowStateEvent::StepStateUpdated {
          id: step.id.clone(),
          state: WorkflowState::Skipped,
        });

        steps.push(StepRunResult {
          id: step.id.clone(),
          state: WorkflowState::Skipped,
          exit_code: None,
          started_at: None,
          completed_at: None,
        });
        continue;
      }

      let result = ctx.run(step).await;

      match result.state {
        WorkflowState::Failed => {
          job_state = WorkflowState::Failed;
        }
        WorkflowState::Cancelled => {
          job_state = WorkflowState::Cancelled;
        }
        _ => {}
      }

      steps.push(result);
    }

    if job_state.is_in_progress() {
      job_state = WorkflowState::Succeeded;
    }

    let completed_at = chrono::Utc::now();

    ctx.on_state_change(WorkflowStateEvent::JobStateUpdated {
      id: self.id.clone(),
      state: job_state.clone(),
    });

    let result = JobRunResult {
      id: self.id.clone(),
      state: job_state,
      started_at: Some(started_at),
      completed_at: Some(completed_at),
      steps,
    };

    ctx.on_job_completed(result.clone());

    result
  }
}

mod default;

use astro_run::{Context, Job, JobRunResult, Step, StepRunResult, Workflow, WorkflowRunResult};
pub use default::DefaultScheduler;

#[derive(Debug, Clone, Default)]
pub struct RunnerMetadata {
  pub id: String,
  pub os: String,
  pub arch: String,
  pub support_docker: bool,
  pub support_host: bool,
  pub version: String,
  pub max_runs: i32,
}

#[astro_run::async_trait]
pub trait Scheduler: Send + Sync {
  fn on_run_workflow(&self, _workflow: Workflow) {}
  fn on_run_job(&self, _job: Job) {}
  fn on_run_step(&self, _step: Step) {}
  fn on_step_completed(&self, _result: StepRunResult) {}
  fn on_job_completed(&self, _result: JobRunResult) {}
  fn on_workflow_completed(&self, _result: WorkflowRunResult) {}
  async fn schedule<'a, 'b: 'a>(
    &'b self,
    runners: &'a [RunnerMetadata],
    ctx: &Context,
  ) -> Option<&'a RunnerMetadata>;
}

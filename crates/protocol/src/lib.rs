use serde::{Deserialize, Serialize};

mod common;
pub use tonic;

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RunEvent {
  RunStep(astro_run::RunStepEvent),
  RunJob(astro_run::RunJobEvent),
  RunWorkflow(astro_run::RunWorkflowEvent),
  StepCompleted(astro_run::StepRunResult),
  JobCompleted(astro_run::JobRunResult),
  WorkflowCompleted(astro_run::WorkflowRunResult),
  StepLog(astro_run::WorkflowLog),
  StateChange(astro_run::WorkflowStateEvent),
  Signal(SignalEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RunResponse {
  Log {
    step_id: astro_run::StepId,
    log: astro_run::Log,
  },
  Result {
    step_id: astro_run::StepId,
    result: astro_run::RunResult,
  },
}

impl RunResponse {
  pub fn log(step_id: astro_run::StepId, log: astro_run::Log) -> Self {
    Self::Log { step_id, log }
  }

  pub fn result(step_id: astro_run::StepId, result: astro_run::RunResult) -> Self {
    Self::Result { step_id, result }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalEvent {
  pub step_id: astro_run::StepId,
  pub signal: astro_run::Signal,
}

pub use astro_run_scheduler::RunnerMetadata;

pub use astro_run::{Command, Context};

pub mod remote_runner {
  mod remote_runner_inner {
    include!("proto/remote_runner.RemoteRunner.rs");
  }

  pub use remote_runner_inner::remote_runner_client::RemoteRunnerClient;
  pub use remote_runner_inner::remote_runner_server::RemoteRunner as RemoteRunnerExt;
  pub use remote_runner_inner::remote_runner_server::RemoteRunnerServer;
}

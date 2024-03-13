use crate::{
  stream::StreamReceiver, Context, JobRunResult, PluginNoopResult, StepRunResult, WorkflowLog,
  WorkflowLogType, WorkflowRunResult, WorkflowStateEvent,
};
pub use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Clone, PartialEq)]
pub enum RunResult {
  Succeeded,
  Failed { exit_code: i32 },
  Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Log {
  pub log_type: WorkflowLogType,
  pub message: String,
}

impl Log {
  pub fn log(message: impl Into<String>) -> Self {
    Self {
      log_type: WorkflowLogType::Log,
      message: message.into(),
    }
  }

  pub fn error(message: impl Into<String>) -> Self {
    Self {
      log_type: WorkflowLogType::Error,
      message: message.into(),
    }
  }

  pub fn is_error(&self) -> bool {
    self.log_type == WorkflowLogType::Error
  }
}

pub type RunResponse = crate::Result<StreamReceiver>;

/// # Runner
/// The `Runner` trait provides the most fundamental deconstruction of a runner. You can implement the `run` method to customize your own runner.
///
/// The `run` method is asynchronous. Before `run` is executed, the `Step` status is `WorkflowState::Pending`.
///
/// During the execution of `run`, the status is set to `WorkflowState::Queued`.
/// At this point, you can handle scheduling logic related to the runner. **(Please avoid executing step's runtime logic within the asynchronous `run` method. It is highly recommended to use a separate thread to process individual steps.)**
///
/// After `run` has completed, the status becomes `WorkflowState::InProgress`. This is when the step is truly running. The `run` method returns a stream result that implements the `Stream` trait, allowing dynamic log updates.
///
/// ## Example
///
/// ```rust
/// struct Runner;
///
/// #[astro_run::async_trait]
/// impl astro_run::Runner for Runner {
///   async fn run(&self, ctx: astro_run::Context) -> astro_run::RunResponse {
///     let (tx, rx) = astro_run::stream();
///
///     tokio::task::spawn(async move {
///       // Send running log
///       tx.log(ctx.command.run);
///
///       // Send success log
///       tx.end(astro_run::RunResult::Succeeded);
///     });
///
///     Ok(rx)
///   }
/// }
/// ```
///
#[async_trait::async_trait]
pub trait Runner: Send + Sync {
  async fn on_run_workflow(&self, _event: crate::RunWorkflowEvent) -> PluginNoopResult {
    Ok(())
  }
  async fn on_run_job(&self, _event: crate::RunJobEvent) -> PluginNoopResult {
    Ok(())
  }
  async fn on_run_step(&self, _event: crate::RunStepEvent) -> PluginNoopResult {
    Ok(())
  }
  async fn on_step_completed(&self, _result: StepRunResult) -> PluginNoopResult {
    Ok(())
  }
  async fn on_job_completed(&self, _result: JobRunResult) -> PluginNoopResult {
    Ok(())
  }
  async fn on_workflow_completed(&self, _result: WorkflowRunResult) -> PluginNoopResult {
    Ok(())
  }
  async fn on_state_change(&self, _event: WorkflowStateEvent) -> PluginNoopResult {
    Ok(())
  }
  async fn on_log(&self, _log: WorkflowLog) -> PluginNoopResult {
    Ok(())
  }
  async fn run(&self, config: Context) -> RunResponse;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_log() {
    let log = Log::log("test");
    assert_eq!(log.log_type, WorkflowLogType::Log);
    assert_eq!(log.message, "test");
    assert!(!log.is_error());

    let log = Log::error("test");
    assert_eq!(log.log_type, WorkflowLogType::Error);
    assert_eq!(log.message, "test");
    assert!(log.is_error());
  }
}

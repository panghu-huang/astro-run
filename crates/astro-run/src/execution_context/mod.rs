mod builder;
mod workflow_shared;

use self::{builder::ExecutionContextBuilder, workflow_shared::WorkflowShared};
use crate::PluginManager;
use astro_run_shared::{Command, Id, Runner, StreamExt, StreamResponse, WorkflowLog};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

struct ExecutionContextInner {
  plugin_manager: PluginManager,
}

/// Execution result of a command
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ExecuteResult {
  Success(i32),
  Error { exit_code: i32, error: String },
  ExitWithoutStatus,
}

#[derive(Clone)]
pub struct ExecutionContext {
  pub workflow_shared: WorkflowShared,
  runner: Arc<Box<dyn Runner>>,
  inner: Arc<Mutex<ExecutionContextInner>>,
}

impl ExecutionContext {
  pub fn builder() -> ExecutionContextBuilder {
    ExecutionContextBuilder::new()
  }

  pub async fn run(
    &self,
    workflow_id: Id,
    job_key: String,
    step_number: usize,
    command: Command,
  ) -> astro_run_shared::Result<ExecuteResult> {
    let inner = self.inner.lock();
    let plugin_manager = inner.plugin_manager.clone();
    let mut stream = self.runner.run(command)?;

    drop(inner);

    let mut error = None;
    while let Some(res) = stream.next().await {
      match res {
        StreamResponse::Log(log) => {
          if log.is_error() {
            error = Some(log.message.clone());
          }

          let log = WorkflowLog {
            workflow_id: workflow_id.clone(),
            job_key: job_key.clone(),
            step_number,
            log_type: log.log_type,
            message: log.message,
            time: chrono::Utc::now(),
          };

          plugin_manager.on_log(&log);
        }
        StreamResponse::End(status) => {
          if status.success() {
            return Ok(ExecuteResult::Success(status.code().unwrap_or(0)));
          }
          return Ok(ExecuteResult::Error {
            exit_code: status.code().unwrap_or(1),
            error: error.unwrap_or_else(|| "Unknown error".to_string()),
          });
        }
      }
    }

    Ok(ExecuteResult::ExitWithoutStatus)
  }
}

mod builder;
mod workflow_shared;

use self::{builder::ExecutionContextBuilder, workflow_shared::WorkflowShared};
use crate::PluginManager;
use astro_run_shared::{Command, Config, Error, RunResult, Runner, StreamExt, WorkflowLog};
use parking_lot::Mutex;
use std::sync::Arc;

struct ExecutionContextInner {
  plugin_manager: PluginManager,
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

  pub async fn run(&self, command: Command) -> astro_run_shared::Result<RunResult> {
    let (workflow_id, job_key, step_number) = command.id.clone();

    let inner = self.inner.lock();
    let plugin_manager = inner.plugin_manager.clone();
    let mut receiver = self.runner.run(Config { command })?;

    drop(inner);

    while let Some(log) = receiver.next().await {
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

    let res = receiver.result().ok_or(Error::internal_runtime_error(
      "Missing result from runner. This is a bug in the runner implementation.",
    ))?;

    Ok(res)
  }
}

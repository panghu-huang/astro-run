use crate::executor::Executor;
use astro_run::{
  stream, Context, Error, Result, RunResponse, RunResult, Runner, WorkflowEvent, WorkflowId,
};
use parking_lot::Mutex;
use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};

struct RunnerState {
  workflow_events: HashMap<WorkflowId, WorkflowEvent>,
}

pub struct DockerRunner {
  working_directory: PathBuf,
  state: Arc<Mutex<RunnerState>>,
}

impl DockerRunner {
  pub fn builder() -> DockerRunnerBuilder {
    DockerRunnerBuilder::new()
  }
}

impl Runner for DockerRunner {
  fn on_run_workflow(&self, workflow: astro_run::Workflow) {
    let mut state = self.state.lock();

    if let Some(event) = workflow.event {
      state.workflow_events.insert(workflow.id, event);
    }
  }

  fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) {
    if let Err(err) = self.cleanup_workflow_working_directory(result) {
      log::error!("DockerRunner: cleanup error: {}", err);
    }
  }

  fn run(&self, ctx: Context) -> RunResponse {
    let (sender, receiver) = stream();

    let executor = Executor {
      working_directory: self.working_directory.clone(),
    };
    let event = self
      .state
      .lock()
      .workflow_events
      .get(&ctx.command.id.workflow_id())
      .cloned();

    tokio::spawn(async move {
      if let Err(err) = executor.execute(sender.clone(), event, ctx).await {
        log::error!("DockerRunner: execute error: {}", err);
        if !sender.is_ended() {
          sender.end(RunResult::Failed { exit_code: 1 });
        }
      }
    });

    Ok(receiver)
  }
}

impl DockerRunner {
  fn cleanup_workflow_working_directory(&self, result: astro_run::WorkflowRunResult) -> Result<()> {
    log::info!("DockerRunner: workflow completed: {:?}", result);
    let event = self.state.lock().workflow_events.get(&result.id).cloned();

    let mut directory = self.working_directory.clone();

    if let Some(event) = event {
      directory = directory.join(&event.repo_owner).join(&event.repo_name);
    }
    
    directory = directory.join(&result.id.inner());

    fs::remove_dir_all(directory)?;

    Ok(()) as Result<()>
  }
}

pub struct DockerRunnerBuilder {
  working_directory: Option<PathBuf>,
}

impl DockerRunnerBuilder {
  pub fn new() -> Self {
    Self {
      working_directory: None,
    }
  }

  pub fn working_directory(mut self, working_directory: PathBuf) -> Self {
    self.working_directory = Some(working_directory);
    self
  }

  pub fn build(self) -> Result<DockerRunner> {
    let working_directory = self.working_directory.map(|i| Ok(i)).unwrap_or_else(|| {
      #[allow(deprecated)]
      env::home_dir()
        .map(|home| home.join("astro-run"))
        .ok_or_else(|| Error::init_error("DockerRunnerBuilder: working_directory is required"))
    })?;

    let runner = DockerRunner {
      working_directory,
      state: Arc::new(Mutex::new(RunnerState {
        workflow_events: HashMap::new(),
      })),
    };

    Ok(runner)
  }
}

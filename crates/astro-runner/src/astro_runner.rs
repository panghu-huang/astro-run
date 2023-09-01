use crate::executors::{DockerExecutor, Executor, HostExecutor};
use astro_run::{
  stream, Context, Error, Result, RunResponse, RunResult, Runner, WorkflowEvent, WorkflowId,
};
use parking_lot::Mutex;
use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};

struct RunnerState {
  workflow_events: HashMap<WorkflowId, WorkflowEvent>,
}

pub struct AstroRunner {
  working_directory: PathBuf,
  state: Arc<Mutex<RunnerState>>,
}

impl AstroRunner {
  pub fn builder() -> AstroRunnerBuilder {
    AstroRunnerBuilder::new()
  }
}

impl Runner for AstroRunner {
  fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) {
    if let Err(err) = self.cleanup_workflow_working_directory(result) {
      log::error!("AstroRunner: cleanup error: {}", err);
    }
  }

  fn run(&self, ctx: Context) -> RunResponse {
    let (sender, receiver) = stream();

    let executor = self.create_executor(&ctx);

    let event = ctx.event.clone();
    if let Some(event) = &ctx.event {
      self
        .state
        .lock()
        .workflow_events
        .insert(ctx.command.id.workflow_id(), event.clone());
    }

    tokio::spawn(async move {
      if let Err(err) = executor.execute(ctx, sender.clone(), event).await {
        log::error!("AstroRunner: execute error: {}", err);
      }

      if !sender.is_ended() {
        sender.end(RunResult::Failed { exit_code: 1 });
      }
    });

    Ok(receiver)
  }
}

impl AstroRunner {
  fn create_executor(&self, ctx: &Context) -> Box<dyn Executor> {
    let os_name = std::env::consts::OS;
    let architecture = std::env::consts::ARCH;
    let container = ctx.command.container.clone();
    if let Some(container) = container {
      // Example: host/windows
      let host_name = format!("host/{}", os_name);
      // Example: host/windows-x86_64, host/linux-x86_64
      let host_name_with_arch = format!("host/{}-{}", os_name, architecture);

      if container.name == host_name_with_arch || container.name == host_name {
        let executor = HostExecutor {
          working_directory: self.working_directory.clone(),
        };

        return Box::new(executor);
      }
    }

    let executor = DockerExecutor {
      working_directory: self.working_directory.clone(),
    };

    Box::new(executor)
  }

  fn cleanup_workflow_working_directory(&self, result: astro_run::WorkflowRunResult) -> Result<()> {
    log::trace!("AstroRunner: workflow completed: {:?}", result);
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

pub struct AstroRunnerBuilder {
  working_directory: Option<PathBuf>,
}

impl AstroRunnerBuilder {
  pub fn new() -> Self {
    Self {
      working_directory: None,
    }
  }

  pub fn working_directory(mut self, working_directory: PathBuf) -> Self {
    self.working_directory = Some(working_directory);
    self
  }

  pub fn build(self) -> Result<AstroRunner> {
    let working_directory = self.working_directory.map(|i| Ok(i)).unwrap_or_else(|| {
      #[allow(deprecated)]
      env::home_dir()
        .map(|home| home.join("astro-run"))
        .ok_or_else(|| Error::init_error("AstroRunnerBuilder: working_directory is required"))
    })?;

    let runner = AstroRunner {
      working_directory,
      state: Arc::new(Mutex::new(RunnerState {
        workflow_events: HashMap::new(),
      })),
    };

    Ok(runner)
  }
}

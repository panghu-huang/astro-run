use crate::{
  executors::{DockerExecutor, Executor, HostExecutor},
  Plugin, PluginDriver, SharedPluginDriver,
};
use astro_run::{
  stream, Context, Error, PluginNoopResult, Result, RunResponse, RunResult, Runner, WorkflowEvent,
  WorkflowId,
};
use parking_lot::Mutex;
use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};

struct RunnerState {
  workflow_events: HashMap<WorkflowId, WorkflowEvent>,
}

#[derive(Clone)]
pub struct AstroRunner {
  working_directory: PathBuf,
  state: Arc<Mutex<RunnerState>>,
  plugin_driver: SharedPluginDriver,
}

impl AstroRunner {
  pub fn builder() -> AstroRunnerBuilder {
    AstroRunnerBuilder::new()
  }
}

#[astro_run::async_trait]
impl Runner for AstroRunner {
  async fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) -> PluginNoopResult {
    if let Err(err) = self.cleanup_workflow_working_directory(result) {
      log::error!("AstroRunner: cleanup error: {}", err);
    }

    Ok(())
  }

  async fn run(&self, ctx: Context) -> RunResponse {
    let (sender, receiver) = stream();

    let ctx = self.plugin_driver.on_before_run(ctx).await;

    let executor = self.create_executor(&ctx);

    let event = ctx.event.clone();
    if let Some(event) = &ctx.event {
      self
        .state
        .lock()
        .workflow_events
        .insert(ctx.command.id.workflow_id(), event.clone());
    }

    let plugins = Arc::clone(&self.plugin_driver);

    tokio::spawn(async move {
      if let Err(err) = executor.execute(ctx.clone(), sender.clone(), event).await {
        log::error!("AstroRunner: execute error: {}", err);
      }

      if !sender.is_ended() {
        sender.end(RunResult::Failed { exit_code: 1 });
      }

      plugins.on_after_run(ctx).await;
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
    let event = self.state.lock().workflow_events.get(&result.id).cloned();

    let mut directory = self.working_directory.clone();

    if let Some(event) = event {
      directory = directory.join(&event.repo_owner).join(&event.repo_name);
    }

    directory = directory.join(&result.id.inner());

    if directory.exists() {
      fs::remove_dir_all(directory)?;
    }

    Ok(()) as Result<()>
  }
}

pub struct AstroRunnerBuilder {
  working_directory: Option<PathBuf>,
  plugins: Vec<Box<dyn Plugin>>,
}

impl AstroRunnerBuilder {
  pub fn new() -> Self {
    Self {
      working_directory: None,
      plugins: vec![],
    }
  }

  pub fn plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
    self.plugins.push(Box::new(plugin));

    self
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
      plugin_driver: Arc::new(PluginDriver::new(self.plugins)),
    };

    Ok(runner)
  }
}

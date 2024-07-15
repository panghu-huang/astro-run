use astro_run::{Context, Error, Plugin, PluginDriver, Runner, SharedPluginDriver, StepId};
use astro_run_protocol::remote_runner::RemoteRunnerExt;
use astro_run_protocol::remote_runner::RemoteRunnerServer;
use astro_run_protocol::tonic;
use astro_run_protocol::{ProtocolEvent, RunResponse, RunnerMetadata};
use parking_lot::Mutex;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

pub struct AstroRunRemoteRunnerServer {
  id: String,
  max_runs: i32,
  support_docker: bool,
  support_host: bool,
  runner: Arc<Box<dyn Runner>>,
  plugin_driver: SharedPluginDriver,
  signals: Arc<Mutex<HashMap<StepId, astro_run::AstroRunSignal>>>,
}

impl AstroRunRemoteRunnerServer {
  pub fn builder() -> AstroRunRemoteRunnerServerBuilder {
    AstroRunRemoteRunnerServerBuilder::new()
  }

  pub async fn serve(self, addr: &str) -> Result<(), Error> {
    let addr = addr
      .parse()
      .map_err(|_| Error::internal_runtime_error("Failed to parse address"))?;

    let service = RemoteRunnerServer::new(self);

    tonic::transport::Server::builder()
      .add_service(service)
      .serve(addr)
      .await
      .map_err(|e| Error::internal_runtime_error(e.to_string()))?;

    Ok(())
  }

  fn run(
    runner: Arc<Box<dyn Runner>>,
    sender: mpsc::Sender<Result<RunResponse, tonic::Status>>,
    ctx: astro_run::Context,
  ) {
    let id = ctx.id.clone();

    tokio::spawn(async move {
      let mut stream = runner.run(ctx).await?;

      while let Some(log) = stream.next().await {
        if let Err(err) = sender.send(Ok(RunResponse::log(id.clone(), log))).await {
          log::error!("Cannot send log to client: {}", err);
        }
      }

      let result = stream.result().ok_or(Error::internal_runtime_error(
        "Cannot get result from runner".to_string(),
      ))?;

      if let Err(err) = sender
        .send(
          Ok(RunResponse::result(id, result))
        )
        .await
      {
        log::error!("Cannot send result to client: {}", err);
      }

      Ok(()) as Result<(), Error>
    });
  }
}

#[tonic::async_trait]
impl RemoteRunnerExt for AstroRunRemoteRunnerServer {
  type RunStream = ReceiverStream<tonic::Result<RunResponse>>;

  async fn run(
    &self,
    request: tonic::Request<Context>,
  ) -> Result<tonic::Response<Self::RunStream>, tonic::Status> {
    let (tx, rx) = mpsc::channel(30);

    let context = request.into_inner();

    self
      .signals
      .lock()
      .insert(context.id.clone(), context.signal.clone());

    let runner = self.runner.clone();

    Self::run(runner, tx, context);

    Ok(tonic::Response::new(ReceiverStream::new(rx)))
  }

  async fn send_event(
    &self,
    request: tonic::Request<ProtocolEvent>,
  ) -> Result<tonic::Response<astro_run_protocol::Empty>, tonic::Status> {
    let event = request.into_inner();

    match event {
      ProtocolEvent::WorkflowCompleted(result) => {
        self
          .plugin_driver
          .on_workflow_completed(result.clone())
          .await;

        if let Err(err) = self.runner.on_workflow_completed(result).await {
          log::error!("Failed to handle workflow completed event: {}", err);
        }
      }
      ProtocolEvent::JobCompleted(result) => {
        self.plugin_driver.on_job_completed(result.clone()).await;
        if let Err(err) = self.runner.on_job_completed(result).await {
          log::error!("Failed to handle job completed event: {}", err);
        }
      }
      ProtocolEvent::StepCompleted(result) => {
        // Remove signal once step is completed
        let step_id = &result.id;

        self.signals.lock().remove(step_id);

        // Dispatch event to plugins and runner
        self.plugin_driver.on_step_completed(result.clone()).await;
        if let Err(err) = self.runner.on_step_completed(result).await {
          log::error!("Failed to handle step completed event: {}", err);
        }
      }
      ProtocolEvent::StepLog(log) => {
        self.plugin_driver.on_log(log.clone()).await;
        if let Err(err) = self.runner.on_log(log).await {
          log::error!("Failed to handle log event: {}", err);
        }
      }
      ProtocolEvent::StateChange(event) => {
        self.plugin_driver.on_state_change(event.clone()).await;

        if let Err(err) = self.runner.on_state_change(event).await {
          log::error!("Failed to handle state event: {}", err);
        }
      }
      ProtocolEvent::RunStep(event) => {
        self.plugin_driver.on_run_step(event.clone()).await;

        if let Err(err) = self.runner.on_run_step(event).await {
          log::error!("Failed to handle run step event: {}", err);
        }
      }
      ProtocolEvent::RunJob(event) => {
        self.plugin_driver.on_run_job(event.clone()).await;

        if let Err(err) = self.runner.on_run_job(event).await {
          log::error!("Failed to handle run job event: {}", err);
        }
      }
      ProtocolEvent::RunWorkflow(event) => {
        self.plugin_driver.on_run_workflow(event.clone()).await;

        if let Err(err) = self.runner.on_run_workflow(event).await {
          log::error!("Failed to handle run workflow event: {}", err);
        }
      }
      ProtocolEvent::Signal(signal) => {
        log::trace!("Received signal: {:?}", signal);
        let astro_run_signal = self.signals.lock().get(&signal.step_id).cloned();

        if let Some(astro_run_signal) = astro_run_signal {
          match signal.signal {
            astro_run::Signal::Cancel => {
              astro_run_signal.cancel().ok();
            }
            astro_run::Signal::Timeout => {
              astro_run_signal.timeout().ok();
            }
          }
        } else {
          log::trace!("Signal {} is not found", signal.step_id);
        }
      }
    }

    Ok(tonic::Response::new(astro_run_protocol::Empty {}))
  }

  async fn get_runner_metadata(
    &self,
    _req: tonic::Request<astro_run_protocol::Empty>,
  ) -> Result<tonic::Response<RunnerMetadata>, tonic::Status> {
    let metadata = RunnerMetadata {
      id: self.id.clone(),
      max_runs: self.max_runs,
      support_docker: self.support_docker,
      support_host: self.support_host,
      os: env::consts::OS.to_string(),
      arch: env::consts::ARCH.to_string(),
      version: crate::VERSION.to_string(),
    };

    Ok(tonic::Response::new(metadata))
  }

  async fn call_before_run_step_hook(
    &self,
    req: tonic::Request<astro_run::Command>,
  ) -> Result<tonic::Response<astro_run_protocol::Command>, tonic::Status> {
    let command = req.into_inner();
    let step: astro_run::Step = command.into();

    // Call before run step hook
    let step = self.plugin_driver.on_before_run_step(step).await;

    // Call runner before run step hook
    let step = match self.runner.on_before_run_step(step).await {
      Ok(step) => step,
      Err(err) => {
        return Err(tonic::Status::internal(format!(
          "Failed to call before run step hook: {}",
          err
        )))
      }
    };

    Ok(tonic::Response::new(step.into()))
  }
}

pub struct AstroRunRemoteRunnerServerBuilder {
  id: Option<String>,
  runner: Option<Arc<Box<dyn Runner>>>,
  max_runs: i32,
  support_docker: Option<bool>,
  support_host: bool,
  plugins: Vec<Box<dyn Plugin>>,
}

impl AstroRunRemoteRunnerServerBuilder {
  fn new() -> Self {
    Self {
      id: None,
      runner: None,
      max_runs: 5,
      support_docker: None,
      support_host: true,
      plugins: vec![],
    }
  }

  pub fn id(mut self, id: impl Into<String>) -> Self {
    self.id = Some(id.into());
    self
  }

  pub fn max_runs(mut self, max_runs: i32) -> Self {
    self.max_runs = max_runs;
    self
  }

  pub fn support_docker(mut self, support_docker: bool) -> Self {
    self.support_docker = Some(support_docker);
    self
  }

  pub fn support_host(mut self, support_host: bool) -> Self {
    self.support_host = support_host;
    self
  }

  pub fn plugin<P>(mut self, plugin: P) -> Self
  where
    P: Plugin + 'static,
  {
    self.plugins.push(Box::new(plugin));

    self
  }

  pub fn runner<T>(mut self, runner: T) -> Self
  where
    T: Runner + 'static,
  {
    self.runner = Some(Arc::new(Box::new(runner)));
    self
  }

  pub fn build(self) -> Result<AstroRunRemoteRunnerServer, Error> {
    let runner = self
      .runner
      .ok_or_else(|| Error::init_error("Runner is not set"))?;

    let id = self.id.ok_or_else(|| Error::init_error("Id is not set"))?;

    let support_docker = self.support_docker.unwrap_or_else(|| {
      log::trace!("Support docker is not set, Checking if docker is installed and running");

      // Check if docker is installed and running
      std::process::Command::new("docker")
        .arg("ps")
        .status()
        .map_or(false, |status| status.success())
    });

    Ok(AstroRunRemoteRunnerServer {
      id,
      max_runs: self.max_runs,
      support_docker,
      support_host: self.support_host,
      runner,
      plugin_driver: Arc::new(PluginDriver::new(self.plugins)),
      signals: Arc::new(Mutex::new(HashMap::new())),
    })
  }
}

use astro_run::{Error, Plugin, PluginDriver, Runner, SharedPluginDriver};
use astro_run_protocol::{
  astro_run_remote_runner::{self, event::Payload as EventPayload, RunResponse, SendEventResponse},
  tonic, AstroRunRemoteRunner, RunnerMetadata,
};
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
  signals: Arc<Mutex<HashMap<String, astro_run::AstroRunSignal>>>,
}

impl AstroRunRemoteRunnerServer {
  pub fn builder() -> AstroRunRemoteRunnerServerBuilder {
    AstroRunRemoteRunnerServerBuilder::new()
  }

  pub async fn serve(self, addr: &str) -> Result<(), Error> {
    let addr = addr
      .parse()
      .map_err(|_| Error::internal_runtime_error("Failed to parse address"))?;

    let service = astro_run_protocol::AstroRunRemoteRunnerServer::new(self);

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
        if let Err(err) = sender
          .send(
            RunResponse::log(id.clone(), log)
              .map_err(|e| tonic::Status::internal(format!("Cannot send log to client: {}", e))),
          )
          .await
        {
          log::error!("Cannot send log to client: {}", err);
        }
      }

      let result = stream.result().ok_or(Error::internal_runtime_error(
        "Cannot get result from runner".to_string(),
      ))?;

      if let Err(err) = sender
        .send(
          RunResponse::end(id, result)
            .map_err(|e| tonic::Status::internal(format!("Cannot send result to client: {}", e))),
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
impl AstroRunRemoteRunner for AstroRunRemoteRunnerServer {
  type RunStream = ReceiverStream<Result<RunResponse, tonic::Status>>;

  async fn run(
    &self,
    request: tonic::Request<astro_run_protocol::Context>,
  ) -> Result<tonic::Response<Self::RunStream>, tonic::Status> {
    let (tx, rx) = mpsc::channel(30);

    let context = request.into_inner();
    let context: astro_run::Context = context
      .try_into()
      .map_err(|e| tonic::Status::internal(format!("Failed to convert context: {}", e)))?;

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
    request: tonic::Request<astro_run_remote_runner::Event>,
  ) -> Result<tonic::Response<astro_run_remote_runner::SendEventResponse>, tonic::Status> {
    let event = request.into_inner();
    let event = event
      .payload
      .ok_or_else(|| tonic::Status::internal("Payload is empty".to_string()))?;

    match event {
      EventPayload::WorkflowCompletedEvent(event) => {
        let result: astro_run::WorkflowRunResult = event.try_into().map_err(|e| {
          tonic::Status::internal(format!("Failed to convert workflow completed event: {}", e))
        })?;

        self
          .plugin_driver
          .on_workflow_completed(result.clone())
          .await;
        self.runner.on_workflow_completed(result);
      }
      EventPayload::JobCompletedEvent(event) => {
        let result: astro_run::JobRunResult = event.try_into().map_err(|e| {
          tonic::Status::internal(format!("Failed to convert job completed event: {}", e))
        })?;

        self.plugin_driver.on_job_completed(result.clone()).await;
        self.runner.on_job_completed(result);
      }
      EventPayload::StepCompletedEvent(event) => {
        let result: astro_run::StepRunResult = event.try_into().map_err(|e| {
          tonic::Status::internal(format!("Failed to convert step completed event: {}", e))
        })?;

        // Remove signal once step is completed
        let step_id = result.id.to_string();
        self.signals.lock().remove(&step_id);

        // Dispatch event to plugins and runner
        self.plugin_driver.on_step_completed(result.clone()).await;
        self.runner.on_step_completed(result);
      }
      EventPayload::LogEvent(event) => {
        let log: astro_run::WorkflowLog = event
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert log event: {}", e)))?;

        self.plugin_driver.on_log(log.clone());
        self.runner.on_log(log);
      }
      EventPayload::WorkflowStateEvent(event) => {
        let event: astro_run::WorkflowStateEvent = event
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert state event: {}", e)))?;

        self.plugin_driver.on_state_change(event.clone());
        self.runner.on_state_change(event);
      }
      EventPayload::RunStepEvent(event) => {
        let event: astro_run::RunStepEvent = event.try_into().map_err(|e| {
          tonic::Status::internal(format!("Failed to convert run step event: {}", e))
        })?;

        self.plugin_driver.on_run_step(event.clone());
        self.runner.on_run_step(event);
      }
      EventPayload::RunJobEvent(event) => {
        let event: astro_run::RunJobEvent = event
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert job: {}", e)))?;

        self.plugin_driver.on_run_job(event.clone());
        self.runner.on_run_job(event);
      }
      EventPayload::RunWorkflowEvent(event) => {
        let event: astro_run::RunWorkflowEvent = event
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert workflow: {}", e)))?;

        self.plugin_driver.on_run_workflow(event.clone());
        self.runner.on_run_workflow(event);
      }
      EventPayload::SignalEvent(signal) => {
        log::trace!("Received signal: {:?}", signal);
        let astro_run_signal = self.signals.lock().get(&signal.id).cloned();

        if let Some(astro_run_signal) = astro_run_signal {
          match astro_run::Signal::from(signal.action.as_str()) {
            astro_run::Signal::Cancel => {
              astro_run_signal.cancel().ok();
            }
            astro_run::Signal::Timeout => {
              astro_run_signal.timeout().ok();
            }
          }
        } else {
          log::trace!("Signal {} is not found", signal.id);
        }
      }
    }

    Ok(tonic::Response::new(SendEventResponse {}))
  }

  async fn get_runner_metadata(
    &self,
    _req: tonic::Request<astro_run_remote_runner::ConnectRequest>,
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
  pub fn new() -> Self {
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
      .ok_or_else(|| Error::internal_runtime_error("Runner is not set".to_string()))?;

    let id = self
      .id
      .ok_or_else(|| Error::internal_runtime_error("Id is not set".to_string()))?;

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

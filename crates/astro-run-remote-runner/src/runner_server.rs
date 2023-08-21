use astro_run::{Error, PluginManager, Runner};
use astro_run_protocol::{
  astro_run_remote_runner::{self, event::Payload as EventPayload, RunResponse, SendEventResponse},
  tonic, AstroRunRemoteRunner, RunnerMetadata,
};
use std::{env, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

pub struct AstroRunRemoteRunnerServer {
  id: String,
  max_runs: i32,
  support_docker: bool,
  support_host: bool,
  runner: Arc<Box<dyn Runner>>,
  plugins: PluginManager,
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
      let mut stream = runner.run(ctx).unwrap();

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

        self.plugins.on_workflow_completed(result.clone());
        self.runner.on_workflow_completed(result);
      }
      EventPayload::JobCompletedEvent(event) => {
        let result: astro_run::JobRunResult = event.try_into().map_err(|e| {
          tonic::Status::internal(format!("Failed to convert job completed event: {}", e))
        })?;

        self.plugins.on_job_completed(result.clone());
        self.runner.on_job_completed(result);
      }
      EventPayload::StepCompletedEvent(event) => {
        let result: astro_run::StepRunResult = event.try_into().map_err(|e| {
          tonic::Status::internal(format!("Failed to convert step completed event: {}", e))
        })?;

        self.plugins.on_step_completed(result.clone());
        self.runner.on_step_completed(result);
      }
      EventPayload::LogEvent(event) => {
        let log: astro_run::WorkflowLog = event
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert log event: {}", e)))?;

        self.plugins.on_log(log.clone());
        self.runner.on_log(log);
      }
      EventPayload::WorkflowStateEvent(event) => {
        let event: astro_run::WorkflowStateEvent = event
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert state event: {}", e)))?;

        self.plugins.on_state_change(event.clone());
        self.runner.on_state_change(event);
      }
      EventPayload::RunStepEvent(step) => {
        let step: astro_run::Step = step
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert step: {}", e)))?;

        self.plugins.on_run_step(step.clone());
        self.runner.on_run_step(step);
      }
      EventPayload::RunJobEvent(job) => {
        let job: astro_run::Job = job
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert job: {}", e)))?;

        self.plugins.on_run_job(job.clone());
        self.runner.on_run_job(job);
      }
      EventPayload::RunWorkflowEvent(workflow) => {
        let workflow: astro_run::Workflow = workflow
          .try_into()
          .map_err(|e| tonic::Status::internal(format!("Failed to convert workflow: {}", e)))?;

        self.plugins.on_run_workflow(workflow.clone());
        self.runner.on_run_workflow(workflow);
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
  support_docker: bool,
  support_host: bool,
}

impl AstroRunRemoteRunnerServerBuilder {
  pub fn new() -> Self {
    Self {
      id: None,
      runner: None,
      max_runs: 1,
      support_docker: false,
      support_host: false,
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
    self.support_docker = support_docker;
    self
  }

  pub fn support_host(mut self, support_host: bool) -> Self {
    self.support_host = support_host;
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

    Ok(AstroRunRemoteRunnerServer {
      id,
      max_runs: self.max_runs,
      support_docker: self.support_docker,
      support_host: self.support_host,
      runner,
      plugins: PluginManager::new(),
    })
  }
}

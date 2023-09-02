use astro_run::{stream, Context, Error, RunResponse, Runner, StreamSender, WorkflowLogType};
use astro_run_protocol::{
  astro_run_server, tonic, AstroRunService, AstroRunServiceServer, RunnerMetadata,
};
use astro_run_scheduler::{DefaultScheduler, Scheduler};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone)]
struct Client {
  metadata: astro_run_scheduler::RunnerMetadata,
  sender: mpsc::Sender<Result<astro_run_server::Event, Status>>,
}

#[derive(Clone)]
struct RunningClient {
  sender: StreamSender,
  completed_token: mpsc::Sender<()>,
}

struct SharedState {
  /// Run ID -> Client
  running: HashMap<String, RunningClient>,
  /// Runner clients (Runner ID -> Client)
  clients: HashMap<String, Client>,
}

#[derive(Clone)]
pub struct AstroRunServer {
  state: Arc<Mutex<SharedState>>,
  scheduler: Arc<Box<dyn Scheduler>>,
}

#[tonic::async_trait]
impl AstroRunService for AstroRunServer {
  type SubscribeEventsStream = ReceiverStream<Result<astro_run_server::Event, Status>>;

  async fn subscribe_events(
    &self,
    request: Request<RunnerMetadata>,
  ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
    let req = request.into_inner();
    let metadata: astro_run_scheduler::RunnerMetadata =
      req.try_into().map_err(|err: astro_run::Error| {
        Status::invalid_argument(format!("Failed to convert metadata: {}", err.to_string()))
      })?;

    if metadata.version != crate::VERSION {
      return Err(Status::invalid_argument(format!(
        "Version mismatch: {} != {}",
        metadata.version,
        crate::VERSION
      )));
    }

    let (tx, rx) = mpsc::channel(100);

    let mut state = self.state.lock();
    state.clients.insert(
      metadata.id.clone(),
      Client {
        metadata,
        sender: tx,
      },
    );

    let stream = ReceiverStream::new(rx);

    Ok(Response::new(stream))
  }

  async fn report_log(
    &self,
    request: Request<astro_run_protocol::WorkflowLog>,
  ) -> Result<Response<astro_run_server::ReportLogResponse>, Status> {
    let inner = request.into_inner();

    let id = inner.step_id.clone();
    let state = self.state.lock();
    let client = state
      .running
      .get(&id)
      .ok_or_else(|| Status::not_found(format!("No running job with id {}", id)))?;

    match WorkflowLogType::from(inner.log_type) {
      WorkflowLogType::Log => client.sender.log(inner.message),
      WorkflowLogType::Error => client.sender.error(inner.message),
    }

    Ok(Response::new(astro_run_server::ReportLogResponse {}))
  }

  async fn report_run_completed(
    &self,
    request: Request<astro_run_server::ReportRunCompletedRequest>,
  ) -> Result<Response<astro_run_server::ReportRunCompletedResponse>, Status> {
    let inner = request.into_inner();

    let id = inner.id.clone();
    let running = self.state.lock().running.clone();
    let client = running
      .get(&id)
      .clone()
      .ok_or_else(|| Status::not_found(format!("No running job with id {}", id)))?;

    let result = inner
      .result
      .ok_or_else(|| Status::invalid_argument("No result provided"))?;

    client.sender.end(
      result
        .result
        .ok_or_else(|| Status::invalid_argument("No result provided in result"))?
        .into(),
    );

    let removed = self.state.lock().running.remove(&id);

    if let Some(removed) = removed {// is closed
      if let Err(err) = removed.completed_token.send(()).await {
        log::error!("Failed to send completed token: {}", err);
      }
    }

    Ok(Response::new(
      astro_run_server::ReportRunCompletedResponse {},
    ))
  }
}

impl Runner for AstroRunServer {
  fn run(&self, ctx: Context) -> RunResponse {
    let (sender, receiver) = stream();
    let id = ctx.command.id.to_string();

    let clients = self.state.lock().clients.clone();

    let runners = clients
      .values()
      .map(|c| c.metadata.clone())
      .collect::<Vec<_>>();

    let runner = match self.scheduler.schedule(&runners, &ctx) {
      Some(runner) => runner,
      None => {
        sender.error("No runner available");
        sender.end(astro_run::RunResult::Failed { exit_code: 1 });

        return Ok(receiver);
      }
    };

    let client = clients
      .get(&runner.id)
      .ok_or_else(|| Error::internal_runtime_error("No client found for runner"))?
      .clone();

    let event = astro_run_server::Event::try_from(ctx.clone())
      .map_err(|err| Error::internal_runtime_error(err.to_string()))?;

    if let Err(err) = client.sender.try_send(Ok(event)) {
      log::error!("Failed to send event to client: {}", err);
    }

    let (completed_sender, mut completed_receiver) = mpsc::channel(1);

    self.state.lock().running.insert(
      id.clone(),
      RunningClient {
        sender,
        completed_token: completed_sender.clone(),
      },
    );

    tokio::task::spawn(async move {
      tokio::select! {
        _ = completed_receiver.recv() => {
          log::trace!("Completed token received");
        }
        signal = ctx.signal.recv() => {
          log::trace!("Signal received {:?}", ctx.signal);
          let event = astro_run_server::Event::new_signal_event(
            ctx.id,
            signal
          );

          if let Err(err) = client.sender.send(Ok(event)).await {
            log::error!("Failed to send event to client: {}", err);
          }
        }
      }
    });

    Ok(receiver)
  }

  fn on_step_completed(&self, result: astro_run::StepRunResult) {
    self.send_event_to_clients(result);
  }

  fn on_job_completed(&self, result: astro_run::JobRunResult) {
    self.send_event_to_clients(result);
  }

  fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) {
    self.send_event_to_clients(result);
  }

  fn on_run_step(&self, step: astro_run::Step) {
    self.send_event_to_clients(step);
  }

  fn on_run_job(&self, job: astro_run::Job) {
    self.send_event_to_clients(job);
  }

  fn on_run_workflow(&self, workflow: astro_run::Workflow) {
    self.send_event_to_clients(workflow);
  }

  fn on_log(&self, log: astro_run::WorkflowLog) {
    self.send_event_to_clients(log);
  }

  fn on_state_change(&self, event: astro_run::WorkflowStateEvent) {
    self.send_event_to_clients(event);
  }
}

impl AstroRunServer {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(SharedState {
        running: HashMap::new(),
        clients: HashMap::new(),
      })),
      scheduler: Arc::new(Box::new(DefaultScheduler::new())),
    }
  }

  pub fn with_scheduler<T>(scheduler: T) -> Self
  where
    T: Scheduler + 'static,
  {
    Self {
      state: Arc::new(Mutex::new(SharedState {
        running: HashMap::new(),
        clients: HashMap::new(),
      })),
      scheduler: Arc::new(Box::new(scheduler)),
    }
  }

  pub async fn serve(self, url: impl Into<&str>) -> astro_run::Result<()> {
    Server::builder()
      .add_service(AstroRunServiceServer::new(self))
      .serve(url.into().parse().map_err(|err| {
        astro_run::Error::internal_runtime_error(format!("Failed to parse url: {}", err))
      })?)
      .await
      .map_err(|err| {
        astro_run::Error::internal_runtime_error(format!("Failed to start server: {}", err))
      })?;

    Ok(())
  }

  fn send_event_to_clients<T>(&self, event: T)
  where
    astro_run_server::Event: TryFrom<T>,
  {
    let event = match astro_run_server::Event::try_from(event) {
      Ok(event) => event,
      Err(_) => {
        log::error!("Failed to convert event to astro_run_protocol::astro_run_server::Event");
        return;
      }
    };
    let clients = self.state.lock().clients.clone();

    for client in clients.values() {
      if let Err(err) = client.sender.try_send(Ok(event.clone())) {
        log::error!("Failed to send event to client: {}", err);
      }
    }
  }
}

use crate::pb::{self, astro_service_server::AstroService};
use astro_run::{stream, Context, Error, RunResponse, Runner, StreamSender, WorkflowLogType};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone)]
struct AstroProtoState {
  running: HashMap<String, StreamSender>,
  clients: HashMap<String, mpsc::Sender<Result<pb::Event, Status>>>,
}

#[derive(Clone)]
pub struct AstroProtoServer {
  state: Arc<Mutex<AstroProtoState>>,
}

#[tonic::async_trait]
impl AstroService for AstroProtoServer {
  type SubscribeEventsStream = ReceiverStream<Result<pb::Event, Status>>;

  async fn subscribe_events(
    &self,
    request: Request<pb::SubscribeEventsRequest>,
  ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
    let req = request.into_inner();

    let (tx, rx) = mpsc::channel(100);

    let mut state = self.state.lock();

    // Print the pointer to the state to make sure we're using the same state
    state.clients.insert(req.id.clone(), tx);

    let stream = ReceiverStream::new(rx);

    Ok(Response::new(stream))
  }

  async fn report_log(
    &self,
    request: Request<pb::ReportLogRequest>,
  ) -> Result<Response<pb::ReportLogResponse>, Status> {
    let inner = request.into_inner();

    let id = inner.run_id.clone();
    let state = self.state.lock();
    let sender = state
      .running
      .get(&id)
      .ok_or_else(|| Status::not_found(format!("No running job with id {}", id)))?;

    match WorkflowLogType::from(inner.log_type) {
      WorkflowLogType::Log => sender.log(inner.log),
      WorkflowLogType::Error => sender.error(inner.log),
    }

    Ok(Response::new(pb::ReportLogResponse {}))
  }

  async fn report_run_completed(
    &self,
    request: Request<pb::ReportRunCompletedRequest>,
  ) -> Result<Response<pb::ReportRunCompletedResponse>, Status> {
    let inner = request.into_inner();

    let id = inner.run_id.clone();
    let state = self.state.lock();
    let sender = state
      .running
      .get(&id)
      .ok_or_else(|| Status::not_found(format!("No running job with id {}", id)))?;

    let result = inner
      .result
      .ok_or_else(|| Status::invalid_argument("No result provided"))?;

    sender.end(result.into());

    Ok(Response::new(pb::ReportRunCompletedResponse {}))
  }
}

impl Runner for AstroProtoServer {
  fn run(&self, ctx: Context) -> RunResponse {
    let (sender, receiver) = stream();
    let id = ctx.command.id.to_string();
    let mut state = self.state.lock();

    state.running.insert(id.clone(), sender);

    let clients = state.clients.clone();

    // TODO: This is a hack to get the first client. We need to figure out how to get the client
    let key = clients
      .keys()
      .next()
      .ok_or_else(|| Error::internal_runtime_error("No clients connected".to_string()))?
      .clone();
    if let Some(client) = clients.get(&key) {
      let event =
        pb::Event::try_from(ctx).map_err(|err| Error::internal_runtime_error(err.to_string()))?;

      if let Err(err) = client.try_send(Ok(event)) {
        log::error!("Failed to send event to client: {}", err);
      }
    }

    Ok(receiver)
  }

  fn on_job_completed(&self, result: astro_run::JobRunResult) {
    println!("Job completed: {:?}", result);
    let event = match pb::Event::try_from(result) {
      Ok(event) => event,
      Err(err) => {
        log::error!("Failed to convert JobRunResult to Event: {}", err);
        return;
      }
    };

    let state = self.state.lock();
    println!("Sending event to {} clients", state.clients.len());
    for client in state.clients.values() {
      if let Err(err) = client.try_send(Ok(event.clone())) {
        println!("Failed to send event to client: {}", err);
        log::error!("Failed to send event to client: {}", err);
      }
    }
  }

  fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) {
    println!("Workflow completed {:?}", result);
    let event = match pb::Event::try_from(result) {
      Ok(event) => event,
      Err(err) => {
        log::error!("Failed to convert JobRunResult to Event: {}", err);
        return;
      }
    };
    let state = self.state.lock();

    println!("Sending event to {} clients", state.clients.len());
    for client in state.clients.values() {
      if let Err(err) = client.try_send(Ok(event.clone())) {
        log::error!("Failed to send event to client: {}", err);
      }
    }
  }
}

impl AstroProtoServer {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(AstroProtoState {
        running: HashMap::new(),
        clients: HashMap::new(),
      })),
    }
  }

  pub async fn serve(self, url: impl Into<&str>) -> astro_run::Result<()> {
    Server::builder()
      .add_service(pb::astro_service_server::AstroServiceServer::new(self))
      .serve(url.into().parse().unwrap())
      .await
      .map_err(|err| {
        astro_run::Error::internal_runtime_error(format!("Failed to start server: {}", err))
      })?;

    Ok(())
  }
}

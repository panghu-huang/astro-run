use astro_run::{Context, Error, Result};
use astro_run_protocol::{
  astro_run_remote_runner::{run_response, ConnectRequest, Event},
  tonic::{self, Request},
};
use astro_run_scheduler::RunnerMetadata;
use astro_run_scheduler::Scheduler;
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

type GRPCClient = astro_run_protocol::AstroRunRemoteRunnerClient<tonic::transport::Channel>;

#[derive(Clone)]
struct Client {
  id: String,
  metadata: RunnerMetadata,
  client: GRPCClient,
}

#[derive(Clone)]
pub struct AstroRunRemoteRunnerClient {
  clients: Arc<Mutex<HashMap<String, Client>>>,
  scheduler: Arc<Box<dyn Scheduler>>,
  event_sender: broadcast::Sender<Event>,
}

impl astro_run::Runner for AstroRunRemoteRunnerClient {
  fn run(&self, context: Context) -> astro_run::RunResponse {
    let (sender, receiver) = astro_run::stream();

    let clients = self.clients.lock().clone();
    let runners: Vec<RunnerMetadata> = clients
      .iter()
      .map(|(_, client)| client.metadata.clone())
      .collect();

    let runner = match self.scheduler.schedule(&runners, &context) {
      Some(runner) => runner,
      None => {
        sender.error("No runner available");
        sender.end(astro_run::RunResult::Failed { exit_code: 1 });
        return Ok(receiver);
      }
    };

    let client = clients.get(&runner.id).unwrap().clone();

    tokio::task::spawn(async move {
      let result = Self::run(sender.clone(), client, context).await;
      if let Err(err) = result {
        log::error!("Failed to run: {}", err);
      }
      if !sender.is_ended() {
        sender.error("Failed to run");
        sender.end(astro_run::RunResult::Failed { exit_code: 1 });
      }
    });

    Ok(receiver)
  }

  fn on_log(&self, log: astro_run::WorkflowLog) {
    match Event::new_log(log) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_step_completed(&self, result: astro_run::StepRunResult) {
    match Event::new_step_completed(result) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_job_completed(&self, result: astro_run::JobRunResult) {
    match Event::new_job_completed(result) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) {
    match Event::new_workflow_completed(result) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_state_change(&self, event: astro_run::WorkflowStateEvent) {
    match Event::new_state_change(event) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_run_step(&self, step: astro_run::Step) {
    match Event::new_run_step(step) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_run_job(&self, job: astro_run::Job) {
    match Event::new_run_job(job) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }

  fn on_run_workflow(&self, workflow: astro_run::Workflow) {
    match Event::new_run_workflow(workflow) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => log::error!("Failed to create event: {}", err),
    }
  }
}

impl AstroRunRemoteRunnerClient {
  pub fn builder() -> AstroRunRemoteRunnerClientBuilder {
    AstroRunRemoteRunnerClientBuilder::new()
  }

  pub async fn start<T>(&mut self, urls: Vec<T>) -> Result<()>
  where
    T: Into<String>,
  {
    let mut receiver = self.event_sender.subscribe();

    for url in urls {
      let url = url.into();
      match Self::connect(url.clone()).await {
        Ok(client) => {
          let mut clients = self.clients.lock();
          if clients.contains_key(&client.id) {
            log::warn!("Runner already connected: {}", client.id);
            continue;
          }
          log::trace!("Connected to runner: {}", client.metadata.id);
          clients.insert(client.id.clone(), client);
        }
        Err(err) => {
          log::error!("Failed to connect {}: {}", url, err);
        }
      }
    }

    while let Ok(event) = receiver.recv().await {
      let clients = self.clients.lock().clone();
      for (_, mut client) in clients {
        let event = Request::new(event.clone());

        if let Err(err) = client.client.send_event(event).await {
          log::error!("Failed to send event: {}", err);
        }
      }
    }

    Ok(())
  }

  async fn connect(url: String) -> Result<Client> {
    let mut client = GRPCClient::connect(url)
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to connect: {}", e)))?;

    let res = client
      .get_runner_metadata(Request::new(ConnectRequest {}))
      .await
      .map_err(|e| {
        Error::internal_runtime_error(format!("Failed to get runner metadata: {}", e))
      })?;

    let metadata = res.into_inner();

    log::trace!("Runner metadata: {:?}", metadata);

    if metadata.version != crate::VERSION {
      return Err(Error::internal_runtime_error(format!(
        "Incompatible version: {}",
        metadata.version
      )));
    }

    Ok(Client {
      id: metadata.id.clone(),
      metadata: metadata
        .try_into()
        .map_err(|e| Error::internal_runtime_error(format!("Failed to parse metadata: {}", e)))?,
      client,
    })
  }

  async fn run(sender: astro_run::StreamSender, client: Client, context: Context) -> Result<()> {
    let context = context.try_into()?;
    let mut client = client;
    let response = client
      .client
      .run(Request::new(context))
      .await
      .map_err(|e| {
        let error = format!("Failed to run: {}", e.to_string());
        sender.error(error.clone());
        sender.end(astro_run::RunResult::Failed { exit_code: 1 });
        Error::internal_runtime_error(error)
      })?;

    let mut stream = response.into_inner();
    while let Some(response) = stream.next().await {
      match response {
        Ok(response) => {
          if let Some(payload) = response.payload {
            match payload {
              run_response::Payload::Log(log) => {
                let log: astro_run::WorkflowLog = log.try_into().map_err(|e| {
                  Error::internal_runtime_error(format!("Failed to parse log: {}", e))
                })?;

                if log.is_error() {
                  sender.error(log.message);
                } else {
                  sender.log(log.message);
                }
              }
              run_response::Payload::Result(result) => {
                let result: astro_run::RunResult = result
                  .result
                  .ok_or(Error::internal_runtime_error(
                    "Missing result in response".to_string(),
                  ))?
                  .try_into()
                  .map_err(|e| {
                    Error::internal_runtime_error(format!("Failed to parse result: {}", e))
                  })?;

                sender.end(result);
              }
            }
          }
        }
        Err(e) => {
          let error = format!("Failed to run: {}", e.to_string());
          sender.error(error.clone());
          sender.end(astro_run::RunResult::Failed { exit_code: 1 });
          return Err(Error::internal_runtime_error(error));
        }
      }
    }

    Ok(()) as Result<()>
  }
}

pub struct AstroRunRemoteRunnerClientBuilder {
  scheduler: Option<Box<dyn astro_run_scheduler::Scheduler>>,
}

impl AstroRunRemoteRunnerClientBuilder {
  pub fn new() -> Self {
    AstroRunRemoteRunnerClientBuilder { scheduler: None }
  }

  pub fn scheduler<T>(mut self, scheduler: T) -> Self
  where
    T: astro_run_scheduler::Scheduler + 'static,
  {
    self.scheduler = Some(Box::new(scheduler));
    self
  }

  pub fn build(self) -> Result<AstroRunRemoteRunnerClient> {
    let scheduler = self
      .scheduler
      .unwrap_or_else(|| Box::new(astro_run_scheduler::DefaultScheduler::new()));

    let (event_sender, _) = broadcast::channel(30);

    Ok(AstroRunRemoteRunnerClient {
      scheduler: Arc::new(scheduler),
      event_sender,
      clients: Arc::new(Mutex::new(HashMap::new())),
    })
  }
}

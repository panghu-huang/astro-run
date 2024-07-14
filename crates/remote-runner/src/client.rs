use astro_run::{Context, Error, HookBeforeRunStepResult, HookNoopResult, Result};
use astro_run_protocol::remote_runner::RemoteRunnerClient;
use astro_run_protocol::tonic;
use astro_run_protocol::RunnerMetadata;
use astro_run_protocol::{RunEvent, RunResponse};
use astro_run_scheduler::Scheduler;
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

type GRPCClient = RemoteRunnerClient<tonic::transport::Channel>;

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
  event_sender: broadcast::Sender<RunEvent>,
}

#[astro_run::async_trait]
impl astro_run::Runner for AstroRunRemoteRunnerClient {
  async fn run(&self, context: Context) -> astro_run::RunResponse {
    let (sender, receiver) = astro_run::stream();

    let clients = self.clients.lock().clone();
    let runners: Vec<RunnerMetadata> = clients
      .values()
      .map(|client| client.metadata.clone())
      .collect();

    let runner = match self.scheduler.schedule(&runners, &context).await {
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

  async fn on_log(&self, log: astro_run::WorkflowLog) -> HookNoopResult {
    let event = RunEvent::StepLog(log);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_step_completed(&self, result: astro_run::StepRunResult) -> HookNoopResult {
    let event = RunEvent::StepCompleted(result);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_job_completed(&self, result: astro_run::JobRunResult) -> HookNoopResult {
    let event = RunEvent::JobCompleted(result);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) -> HookNoopResult {
    let event = RunEvent::WorkflowCompleted(result);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_state_change(&self, event: astro_run::WorkflowStateEvent) -> HookNoopResult {
    let event = RunEvent::StateChange(event);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_run_step(&self, event: astro_run::RunStepEvent) -> HookNoopResult {
    let event = RunEvent::RunStep(event);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_run_job(&self, event: astro_run::RunJobEvent) -> HookNoopResult {
    let event = RunEvent::RunJob(event);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_run_workflow(&self, event: astro_run::RunWorkflowEvent) -> HookNoopResult {
    let event = RunEvent::RunWorkflow(event);

    if let Err(err) = self.event_sender.send(event) {
      log::error!("Failed to send event: {}", err);
    }

    Ok(())
  }

  async fn on_before_run_step(&self, step: astro_run::Step) -> HookBeforeRunStepResult {
    let mut clients = self.clients.lock().clone();

    let mut command = step.into();

    for client in clients.values_mut() {
      match client.client.call_before_run_step_hook(command).await {
        Ok(response) => {
          command = response.into_inner();
        }
        Err(err) => {
          log::error!("Failed to call before run step hook: {}", err);
          return Err(astro_run::Error::error(err.to_string()));
        }
      };
    }

    Ok(command.into())
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
        if let Err(err) = client.client.send_event(event.clone()).await {
          log::error!("Failed to send event: {}", err);
        }
      }
    }

    #[cfg(not(tarpaulin_include))]
    Ok(())
  }

  async fn connect(url: String) -> Result<Client> {
    let mut client = GRPCClient::connect(url)
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to connect: {}", e)))?;

    let res = client
      .get_runner_metadata(astro_run_protocol::Empty {})
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
      metadata,
      client,
    })
  }

  async fn run(sender: astro_run::StreamSender, client: Client, context: Context) -> Result<()> {
    let mut client = client;

    let response = client.client.run(context.clone()).await.map_err(|e| {
      let error = format!("Failed to run: {}", e);
      log::error!("{}", error);

      sender.error(error.clone());
      sender.end(astro_run::RunResult::Failed { exit_code: 1 });

      Error::internal_runtime_error(error)
    })?;

    let mut stream = response.into_inner();

    loop {
      tokio::select! {
        response = stream.next() => {
          let Some(response) = response else {
            break;
          };

          match response {
            Ok(response) => {
                match response {
                  RunResponse::Log { step_id: _, log } => {
                    if log.is_error() {
                      sender.error(log.message);
                    } else {
                      sender.log(log.message);
                    }
                  }
                  RunResponse::Result { step_id: _, result } => {
                    sender.end(result);
                  }
                }
            }
            Err(e) => {
              let error = format!("Failed to run: {}", e);

              sender.error(error.clone());
              sender.end(astro_run::RunResult::Failed { exit_code: 1 });

              return Err(Error::internal_runtime_error(error));
            }
          }
        }
        signal = context.signal.recv() => {
          let signal_event = astro_run_protocol::SignalEvent {
            step_id: context.id.clone(),
            signal,
          };
          let event = RunEvent::Signal(signal_event);

          client
            .client
            .send_event(event)
            .await
            .map_err(Error::internal_runtime_error)?;
        }
      }
    }

    Ok(()) as Result<()>
  }
}

#[derive(Default)]
pub struct AstroRunRemoteRunnerClientBuilder {
  scheduler: Option<Box<dyn astro_run_scheduler::Scheduler>>,
}

impl AstroRunRemoteRunnerClientBuilder {
  pub fn new() -> Self {
    Self::default()
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

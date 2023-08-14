use astro_run::{Context, Error, Result};
use astro_run_protocol::{
  astro_run_remote_runner::{run_response, Event},
  tonic,
};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

type Client = astro_run_protocol::AstroRunRemoteRunnerClient<tonic::transport::Channel>;

// TODO: Add runner version / token to request header
#[derive(Clone)]
pub struct AstroRunRemoteRunnerClient {
  client: Client,
  event_sender: broadcast::Sender<Event>,
}

impl astro_run::Runner for AstroRunRemoteRunnerClient {
  fn run(&self, context: Context) -> astro_run::RunResponse {
    let (sender, receiver) = astro_run::stream();

    let client = self.client.clone();
    tokio::task::spawn(async move {
      let result = Self::run(sender.clone(), client, context).await;
      if let Err(err) = result {
        log::error!("Failed to run: {}", err);
        if !sender.is_ended() {
          sender.error(err.to_string());
          sender.end(astro_run::RunResult::Failed { exit_code: 1 });
        }
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
      Err(err) => {
        log::error!("Failed to create event: {}", err);
      }
    }
  }

  fn on_job_completed(&self, result: astro_run::JobRunResult) {
    match Event::new_job_completed(result) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => {
        log::error!("Failed to create event: {}", err);
      }
    }
  }

  fn on_workflow_completed(&self, result: astro_run::WorkflowRunResult) {
    match Event::new_workflow_completed(result) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => {
        log::error!("Failed to create event: {}", err);
      }
    }
  }

  fn on_state_change(&self, event: astro_run::WorkflowStateEvent) {
    match Event::new_state_change(event) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => {
        log::error!("Failed to create event: {}", err);
      }
    }
  }

  fn on_run_job(&self, job: astro_run::Job) {
    match Event::new_run_job(job) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => {
        log::error!("Failed to create event: {}", err);
      }
    }
  }

  fn on_run_workflow(&self, workflow: astro_run::Workflow) {
    match Event::new_run_workflow(workflow) {
      Ok(event) => {
        if let Err(err) = self.event_sender.send(event) {
          log::error!("Failed to send event: {}", err);
        }
      }
      Err(err) => {
        log::error!("Failed to create event: {}", err);
      }
    }
  }
}

impl AstroRunRemoteRunnerClient {
  pub fn builder() -> AstroRunRemoteRunnerClientBuilder {
    AstroRunRemoteRunnerClientBuilder::new()
  }

  pub async fn start(&self) -> Result<()> {
    let mut receiver = self.event_sender.subscribe();

    while let Ok(event) = receiver.recv().await {
      let mut client = self.client.clone();
      let event = tonic::Request::new(event);

      if let Err(err) = client.send_event(event).await {
        log::error!("Failed to send event: {}", err);
      }
    }

    Ok(())
  }

  async fn run(sender: astro_run::StreamSender, client: Client, context: Context) -> Result<()> {
    let context = context.try_into()?;
    let mut client = client;
    let response = client
      .run(tonic::Request::new(context))
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
  url: Option<String>,
}

impl AstroRunRemoteRunnerClientBuilder {
  pub fn new() -> Self {
    AstroRunRemoteRunnerClientBuilder { url: None }
  }

  pub fn url(mut self, url: impl Into<String>) -> Self {
    self.url = Some(url.into());
    self
  }

  pub async fn build(self) -> Result<AstroRunRemoteRunnerClient> {
    let url = self
      .url
      .ok_or_else(|| Error::internal_runtime_error("Missing url".to_string()))?;

    let client = Client::connect(url)
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to connect: {}", e)))?;

    let (event_sender, _) = broadcast::channel(30);

    Ok(AstroRunRemoteRunnerClient {
      client,
      event_sender,
    })
  }
}

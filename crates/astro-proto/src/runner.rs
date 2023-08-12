use crate::pb::{self, astro_service_client::AstroServiceClient, event::Payload as EventPayload};
use astro_run::{Context, Error, Result, Runner};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

enum Command {
  ReportLog(pb::ReportLogRequest),
  ReportRunCompleted(pb::ReportRunCompletedRequest),
}

pub struct AstroProtoRunner {
  id: String,
  client: AstroServiceClient<tonic::transport::Channel>,
  runner: Arc<Box<dyn Runner>>,
}

impl AstroProtoRunner {
  pub fn builder() -> AstroProtoRunnerBuilder {
    AstroProtoRunnerBuilder::new()
  }

  pub async fn start(&mut self) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Command>(100);

    let stream = self
      .client
      .subscribe_events(pb::SubscribeEventsRequest {
        id: self.id.clone(),
        token: None,
        // TODO: version
        version: "0.0.1".to_string(),
      })
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to subscribe events: {}", e)))?;

    let mut stream = stream.into_inner();

    loop {
      tokio::select! {
        event = stream.next() => {
          let event = match event {
            Some(Ok(event)) => event.payload.unwrap(),
            None => {
              break;
            }
            _ => {
              continue;
            }
          };
          println!("Received event: {:?}", event);

          match event {
            EventPayload::Run(ctx) => {
              self.run(tx.clone(), ctx.try_into()?);
            }
            EventPayload::JobCompletedEvent(result) => {
              self.runner.on_job_completed(result.try_into()?);
            }
            EventPayload::WorkflowCompletedEvent(result) => {
              self.runner.on_workflow_completed(result.try_into()?);
            }
            EventPayload::Error(error) => {
              log::error!("Received error event: {:?}", error);
            }
            _ => {}
          }
        },
        Some(command) = rx.recv() => {
          match command {
            Command::ReportLog(req) => {
              if let Err(err) = self
                .client
                .report_log(req)
                .await {
                  log::error!("Failed to report log: {}", err);
                }
            }
            Command::ReportRunCompleted(req) => {
              if let Err(err) = self
                .client
                .report_run_completed(req)
                .await {
                  log::error!("Failed to report run completed: {}", err)
                }
            }
          }
        }
      }
    }

    Ok(())
  }

  fn run(&self, tx: mpsc::Sender<Command>, ctx: Context) {
    let runner = self.runner.clone();

    tokio::task::spawn(async move {
      let run_id = ctx.command.id.to_string();
      let mut receiver = runner.run(ctx)?;
      while let Some(log) = receiver.next().await {
        let request = pb::ReportLogRequest {
          run_id: run_id.clone(),
          log: log.message,
          log_type: log.log_type.to_string(),
        };
        if let Err(err) = tx.send(Command::ReportLog(request)).await {
          log::error!("Send command error {:?}", err);
        }
      }

      let result = receiver.result().ok_or_else(|| {
        Error::internal_runtime_error("Failed to get result from runner".to_string())
      })?;

      let request = pb::ReportRunCompletedRequest {
        run_id: run_id.clone(),
        result: Some(result.into()),
      };

      if let Err(err) = tx.send(Command::ReportRunCompleted(request)).await {
        log::error!("Send command error {:?}", err);
      }

      Ok::<(), astro_run::Error>(())
    });
  }
}

pub struct AstroProtoRunnerBuilder {
  runner: Option<Box<dyn Runner>>,
  id: Option<String>,
  url: Option<String>,
}

impl AstroProtoRunnerBuilder {
  pub fn new() -> Self {
    AstroProtoRunnerBuilder {
      runner: None,
      id: None,
      url: None,
    }
  }

  pub fn runner<T>(mut self, runner: T) -> Self
  where
    T: Runner + 'static,
  {
    self.runner = Some(Box::new(runner));
    self
  }

  pub fn id(mut self, id: impl Into<String>) -> Self {
    self.id = Some(id.into());
    self
  }

  pub fn url(mut self, url: impl Into<String>) -> Self {
    self.url = Some(url.into());
    self
  }

  pub async fn build(self) -> Result<AstroProtoRunner> {
    let id = self
      .id
      .ok_or_else(|| Error::internal_runtime_error("Missing id".to_string()))?;
    let url = self
      .url
      .ok_or_else(|| Error::internal_runtime_error("Missing url".to_string()))?;
    let runner = self
      .runner
      .ok_or_else(|| Error::internal_runtime_error("Missing runner".to_string()))?;

    let client = AstroServiceClient::connect(url)
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to connect: {}", e)))?;

    Ok(AstroProtoRunner {
      id,
      client,
      runner: Arc::new(runner),
    })
  }
}

use crate::pb::{self, astro_service_client::AstroServiceClient, event::Payload as EventPayload};
use astro_run::{AstroRunPlugin, Context, Error, PluginManager, Result, Runner};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

enum Command {
  ReportLog(pb::WorkflowLog),
  ReportRunCompleted(pb::ReportRunCompletedRequest),
}

pub struct AstroRunRunner {
  id: String,
  client: AstroServiceClient<tonic::transport::Channel>,
  runner: Arc<Box<dyn Runner>>,
  plugins: PluginManager,
}

impl AstroRunRunner {
  pub fn builder() -> AstroRunRunnerBuilder {
    AstroRunRunnerBuilder::new()
  }

  pub async fn start(&mut self) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Command>(100);

    let stream = self
      .client
      .subscribe_events(pb::SubscribeEventsRequest {
        id: self.id.clone(),
        token: None,
        version: crate::VERSION.to_string(),
      })
      .await
      .map_err(|e| {
        Error::internal_runtime_error(format!("Failed to subscribe events: {}", e.to_string()))
      })?;

    let mut stream = stream.into_inner();

    loop {
      tokio::select! {
        event = stream.next() => {
          let event = match event {
            Some(Ok(pb::Event {
              payload: Some(payload),
              ..
             })) => payload,
            None => {
              break;
            }
            _ => {
              log::error!("Received invalid event {:?}", event);
              continue;
            }
          };

          match event {
            EventPayload::Run(ctx) => {
              self.run(tx.clone(), ctx.try_into()?);
            }
            EventPayload::JobCompletedEvent(result) => {
              let result: astro_run::JobRunResult = result.try_into()?;
              self.plugins.on_job_completed(result.clone());
              self.runner.on_job_completed(result);
            }
            EventPayload::WorkflowCompletedEvent(result) => {
              let result: astro_run::WorkflowRunResult = result.try_into()?;
              self.plugins.on_workflow_completed(result.clone());
              self.runner.on_workflow_completed(result);
            }
            EventPayload::RunWorkflowEvent(workflow) => {
              let workflow: astro_run::Workflow = workflow.try_into()?;
              self.plugins.on_run_workflow(workflow.clone());
              self.runner.on_run_workflow(workflow);
            }
            EventPayload::RunJobEvent(job) => {
              let job: astro_run::Job = job.try_into()?;
              self.plugins.on_run_job(job.clone());
              self.runner.on_run_job(job);
            }
            EventPayload::Error(error) => {
              log::error!("Received error event: {:?}", error);
            }
            EventPayload::LogEvent(log) => {
              let log: astro_run::WorkflowLog = log.try_into()?;
              self.plugins.on_log(log.clone());
              self.runner.on_log(log);
            }
            EventPayload::WorkflowStateEvent(event) => {
              let event: astro_run::WorkflowStateEvent = event.try_into()?;
              self.plugins.on_state_change(event.clone());
              self.runner.on_state_change(event);
            }
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
      let step_id = ctx.command.id.clone();
      let mut receiver = runner.run(ctx)?;
      while let Some(log) = receiver.next().await {
        let request = pb::WorkflowLog::try_from(astro_run::WorkflowLog {
          step_id: step_id.clone(),
          message: log.message,
          log_type: log.log_type,
          time: chrono::Utc::now(),
        })
        .unwrap();
        if let Err(err) = tx.send(Command::ReportLog(request)).await {
          log::error!("Send command error {:?}", err);
        }
      }

      let result = receiver.result().ok_or_else(|| {
        Error::internal_runtime_error("Failed to get result from runner".to_string())
      })?;

      let request = pb::ReportRunCompletedRequest {
        run_id: step_id.to_string(),
        result: Some(result.into()),
      };

      if let Err(err) = tx.send(Command::ReportRunCompleted(request)).await {
        log::error!("Send command error {:?}", err);
      }

      Ok::<(), astro_run::Error>(())
    });
  }

  pub fn register_plugin(&self, plugin: AstroRunPlugin) {
    self.plugins.register(plugin);
  }

  pub fn unregister_plugin(&self, plugin: &'static str) {
    self.plugins.unregister(plugin);
  }
}

pub struct AstroRunRunnerBuilder {
  runner: Option<Box<dyn Runner>>,
  id: Option<String>,
  url: Option<String>,
  plugins: PluginManager,
}

impl AstroRunRunnerBuilder {
  pub fn new() -> Self {
    AstroRunRunnerBuilder {
      runner: None,
      id: None,
      url: None,
      plugins: PluginManager::new(),
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

  pub fn plugin(self, plugin: AstroRunPlugin) -> Self {
    self.plugins.register(plugin);

    self
  }

  pub async fn build(self) -> Result<AstroRunRunner> {
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

    Ok(AstroRunRunner {
      id,
      client,
      runner: Arc::new(runner),
      plugins: self.plugins,
    })
  }
}

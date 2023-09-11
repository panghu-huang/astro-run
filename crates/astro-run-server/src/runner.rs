use astro_run::{AstroRunPlugin, Context, Error, PluginManager, Result, Runner};
use astro_run_protocol::{
  astro_run_server::{self, event::Payload as EventPayload},
  tonic, AstroRunServiceClient, RunnerMetadata, WorkflowLog,
};
use parking_lot::Mutex;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

enum Command {
  ReportLog(astro_run_protocol::WorkflowLog),
  ReportRunCompleted(astro_run_server::ReportRunCompletedRequest),
}

pub struct AstroRunRunner {
  id: String,
  max_runs: i32,
  support_docker: bool,
  support_host: bool,
  client: AstroRunServiceClient<tonic::transport::Channel>,
  runner: Arc<Box<dyn Runner>>,
  plugins: PluginManager,
  signals: Arc<Mutex<HashMap<String, astro_run::AstroRunSignal>>>,
}

impl AstroRunRunner {
  pub fn builder() -> AstroRunRunnerBuilder {
    AstroRunRunnerBuilder::new()
  }

  pub async fn start(&mut self) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Command>(100);

    let stream = self
      .client
      .subscribe_events(RunnerMetadata {
        id: self.id.clone(),
        version: crate::VERSION.to_string(),
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        max_runs: self.max_runs,
        support_docker: self.support_docker,
        support_host: self.support_host,
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
            Some(Ok(astro_run_server::Event {
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
            EventPayload::StepCompletedEvent(result) => {
              let result: astro_run::StepRunResult = result.try_into()?;
              self.plugins.on_step_completed(result.clone());
              self.runner.on_step_completed(result);
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
            EventPayload::RunWorkflowEvent(event) => {
              let event: astro_run::RunWorkflowEvent = event.try_into()?;
              self.plugins.on_run_workflow(event.clone());
              self.runner.on_run_workflow(event);
            }
            EventPayload::RunJobEvent(event) => {
              let event: astro_run::RunJobEvent = event.try_into()?;
              self.plugins.on_run_job(event.clone());
              self.runner.on_run_job(event);
            }
            EventPayload::RunStepEvent(event) => {
              let event: astro_run::RunStepEvent = event.try_into()?;
              self.plugins.on_run_step(event.clone());
              self.runner.on_run_step(event);
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
            EventPayload::SignalEvent(signal) => {
              log::trace!("Received signal: {:?}", signal);
              let astro_run_signal = self.signals.lock().get(&signal.id).cloned();

              if let Some(astro_run_signal) = astro_run_signal {
                match signal.action.as_str() {
                  "cancel" => {
                    astro_run_signal.cancel().ok();
                  }
                  "timeout" => {
                    astro_run_signal.timeout().ok();
                  }
                  _ => {}
                }
              } else {
                log::trace!("Signal {} is not found", signal.id);
              }
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

    self
      .signals
      .lock()
      .insert(ctx.id.clone(), ctx.signal.clone());

    tokio::task::spawn(async move {
      let step_id = ctx.command.id.clone();
      let mut receiver = runner.run(ctx)?;

      while let Some(log) = receiver.next().await {
        let request = WorkflowLog::try_from(astro_run::WorkflowLog {
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

      let request = astro_run_server::ReportRunCompletedRequest {
        id: step_id.to_string(),
        result: Some(astro_run_protocol::RunResult {
          result: Some(result.into()),
        }),
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
  max_runs: i32,
  support_docker: Option<bool>,
  support_host: bool,
  plugins: PluginManager,
}

impl AstroRunRunnerBuilder {
  pub fn new() -> Self {
    AstroRunRunnerBuilder {
      runner: None,
      id: None,
      url: None,
      max_runs: 10,
      support_docker: None,
      support_host: true,
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

    let client = AstroRunServiceClient::connect(url)
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to connect: {}", e)))?;

    let support_docker = self.support_docker.unwrap_or_else(|| {
      log::info!("Support docker is not set, Checking if docker is installed and running");

      // Check if docker is installed and running
      std::process::Command::new("docker")
        .arg("ps")
        .status()
        .map_or(false, |status| status.success())
    });

    Ok(AstroRunRunner {
      id,
      client,
      max_runs: self.max_runs,
      support_docker,
      support_host: self.support_host,
      runner: Arc::new(runner),
      plugins: self.plugins,
      signals: Arc::new(Mutex::new(HashMap::new())),
    })
  }
}

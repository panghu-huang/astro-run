use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, Error, JobRunResult, Payload, PluginNoopResult,
  Result, RunJobEvent, RunResult, RunStepEvent, RunWorkflowEvent, Runner, StepRunResult, Workflow,
  WorkflowLog, WorkflowRunResult, WorkflowState, WorkflowStateEvent,
};
use astro_run_remote_runner::{
  AstroRunRemoteRunnerClient, AstroRunRemoteRunnerServer, DefaultScheduler,
};
use parking_lot::Mutex;

struct TestRunner;

impl TestRunner {
  fn new() -> Self {
    TestRunner
  }
}

#[astro_run::async_trait]
impl Runner for TestRunner {
  async fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);
    tx.end(RunResult::Succeeded);

    Ok(rx)
  }

  async fn on_run_workflow(&self, _: RunWorkflowEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_run_job(&self, _: RunJobEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_run_step(&self, _: RunStepEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_log(&self, _: WorkflowLog) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_state_change(&self, _: WorkflowStateEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_step_completed(&self, _: StepRunResult) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_job_completed(&self, _: JobRunResult) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_workflow_completed(&self, _: WorkflowRunResult) -> PluginNoopResult {
    Err(Error::error("Error"))
  }
}

fn assert_logs_plugin(excepted_logs: Vec<&'static str>) -> AstroRunPlugin {
  let index = Mutex::new(0);

  AstroRunPlugin::builder("assert-logs-plugin")
    .on_log(move |log| {
      let mut i = index.lock();
      assert_eq!(log.message, excepted_logs[*i]);
      *i += 1;
      Ok(())
    })
    .build()
}

fn success_plugin() -> AstroRunPlugin {
  AstroRunPlugin::builder("error")
    .on_run_workflow(|_| Ok(()))
    .on_run_job(|_| Ok(()))
    .on_run_step(|_| Ok(()))
    .on_log(|_| Ok(()))
    .on_state_change(|_| Ok(()))
    .on_step_completed(|_| Ok(()))
    .on_job_completed(|_| Ok(()))
    .on_workflow_completed(|_| Ok(()))
    .on_resolve_dynamic_action(|_| Ok(None))
    .build()
}

fn error_plugin() -> AstroRunPlugin {
  AstroRunPlugin::builder("error")
    .on_run_workflow(|_| Err(Error::error("Error")))
    .on_run_job(|_| Err(Error::error("Error")))
    .on_run_step(|_| Err(Error::error("Error")))
    .on_log(|_| Err(Error::error("Error")))
    .on_state_change(|_| Err(Error::error("Error")))
    .on_step_completed(|_| Err(Error::error("Error")))
    .on_job_completed(|_| Err(Error::error("Error")))
    .on_workflow_completed(|_| Err(Error::error("Error")))
    .on_resolve_dynamic_action(|_| Err(Error::error("Error")))
    .build()
}

#[astro_run_test::test]
async fn test_run() -> Result<()> {
  let (oneshot_tx, rx) = tokio::sync::oneshot::channel();

  // Check if docker is installed and running
  let is_support_docker = std::process::Command::new("docker")
    .arg("ps")
    .status()
    .map_or(false, |status| status.success());

  let client_thread_handle = tokio::spawn(async move {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client_runner = AstroRunRemoteRunnerClient::builder()
      .scheduler(DefaultScheduler::new())
      .build()
      .unwrap();

    let handle = tokio::task::spawn({
      let mut client_runner = client_runner.clone();

      async move {
        rx.await.unwrap();

        client_runner
          .start(vec!["http://127.0.0.1:5338"])
          .await
          .unwrap();
      }
    });

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let astro_run = AstroRun::builder()
      .plugin(assert_logs_plugin(vec![
        "Hello World",
        if is_support_docker {
          "Hello World1"
        } else {
          "No runner available"
        },
      ]))
      .plugin(error_plugin())
      .plugin(success_plugin())
      .runner(client_runner)
      .build();

    let workflow = format!(
      r#"
    jobs:
      test:
        steps:
          - container: host/{}
            run: Hello World
          - run: Hello World1
      "#,
      std::env::consts::OS,
    );

    let workflow = Workflow::builder()
      .config(workflow)
      .build(&astro_run)
      .await
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let res = workflow.run(ctx).await;

    if is_support_docker {
      assert_eq!(res.state, WorkflowState::Succeeded);
    } else {
      assert_eq!(res.state, WorkflowState::Failed);
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let server_thread_handle = tokio::spawn(async move {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let runner = TestRunner::new();

    let runner_server = AstroRunRemoteRunnerServer::builder()
      .id("test-runner")
      .runner(runner)
      .max_runs(5)
      .plugin(
        AstroRunPlugin::builder("test-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
            Ok(())
          })
          .build(),
      )
      .plugin(assert_logs_plugin(vec![
        "Hello World",
        if is_support_docker {
          "Hello World1"
        } else {
          "No runner available"
        },
      ]))
      .build()
      .unwrap();

    oneshot_tx.send(()).unwrap();

    tokio::select! {
      _ = rx.recv() => {}
      _ = runner_server.serve("127.0.0.1:5338") => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

#[astro_run_test::test]
async fn no_available_runners() {
  let client_runner = AstroRunRemoteRunnerClient::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .plugin(assert_logs_plugin(vec!["No runner available"]))
    .runner(client_runner)
    .build();

  let workflow = r#"
    jobs:
      test:
        steps:
          - run: Hello World
      "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
}

#[astro_run_test::test]
async fn connect_to_invalid_url() {
  let client_runner = AstroRunRemoteRunnerClient::builder().build().unwrap();

  let mut cloned_client_runner = client_runner.clone();
  let handle = tokio::task::spawn(async move {
    cloned_client_runner
      .start(vec!["http://1.1.1.1:8888"])
      .await
      .unwrap();
  });

  let astro_run = AstroRun::builder()
    .plugin(assert_logs_plugin(vec!["No runner available"]))
    .runner(client_runner)
    .build();

  let workflow = r#"
    jobs:
      test:
        steps:
          - run: Hello World
      "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);

  handle.abort();
}

struct WorkflowPayload(String);

impl Payload for WorkflowPayload {
  fn try_from_string(payload: &String) -> astro_run::Result<Self>
  where
    Self: Sized,
  {
    Ok(WorkflowPayload(payload.clone()))
  }

  fn try_into_string(&self) -> astro_run::Result<String> {
    Ok(self.0.clone())
  }
}

fn assert_workflow_payload_plugin(expected: impl Into<String>) -> AstroRunPlugin {
  let expected = expected.into();

  AstroRunPlugin::builder("assert_workflow_payload_plugin")
    .on_run_workflow(move |event| {
      let payload: WorkflowPayload = event.payload.payload().unwrap();
      assert_eq!(payload.0, expected);
      Ok(())
    })
    .build()
}

#[astro_run_test::test]
async fn test_remote_runner_workflow_payload() -> Result<()> {
  let (oneshot_tx, rx) = tokio::sync::oneshot::channel();

  let client_thread_handle = tokio::spawn(async {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client_runner = AstroRunRemoteRunnerClient::builder()
      .scheduler(DefaultScheduler::new())
      .build()
      .unwrap();

    let handle = tokio::task::spawn({
      let mut client_runner = client_runner.clone();

      async move {
        rx.await.unwrap();

        client_runner
          .start(vec!["http://127.0.0.1:5338"])
          .await
          .unwrap();
      }
    });

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let astro_run = AstroRun::builder()
      .plugin(assert_logs_plugin(vec!["Hello World"]))
      .plugin(assert_workflow_payload_plugin("This is a payload"))
      .runner(client_runner)
      .build();

    let workflow = format!(
      r#"
    jobs:
      test:
        steps:
          - container: host/{}
            run: Hello World
      "#,
      std::env::consts::OS,
    );

    let workflow = Workflow::builder()
      .config(workflow)
      .payload(WorkflowPayload("This is a payload".to_string()))
      .build(&astro_run)
      .await
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Succeeded);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let server_thread_handle = tokio::spawn(async {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let runner = TestRunner::new();

    let runner_server = AstroRunRemoteRunnerServer::builder()
      .id("test-runner")
      .runner(runner)
      .max_runs(5)
      .plugin(assert_workflow_payload_plugin("This is a payload"))
      .plugin(
        AstroRunPlugin::builder("test-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
            Ok(())
          })
          .build(),
      )
      .build()
      .unwrap();

    oneshot_tx.send(()).unwrap();

    tokio::select! {
      _ = rx.recv() => {}
      _ = runner_server.serve("127.0.0.1:5338") => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

#[astro_run_test::test]
fn test_build_server_error() {
  let error = AstroRunRemoteRunnerServer::builder().build().err();

  assert_eq!(error.unwrap(), Error::init_error("Runner is not set"));

  let runner = TestRunner::new();
  let error = AstroRunRemoteRunnerServer::builder()
    .runner(runner)
    .build()
    .err();

  assert_eq!(error.unwrap(), Error::init_error("Id is not set"));
}

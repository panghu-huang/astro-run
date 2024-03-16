use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, Error, JobRunResult, HookNoopResult, Result,
  RunJobEvent, RunResult, RunStepEvent, RunWorkflowEvent, Runner, StepRunResult, Workflow,
  WorkflowLog, WorkflowRunResult, WorkflowState, WorkflowStateEvent,
};
use astro_run_server::{AstroRunRunner, AstroRunServer, DefaultScheduler};
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

  async fn on_run_workflow(&self, _: RunWorkflowEvent) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_run_job(&self, _: RunJobEvent) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_run_step(&self, _: RunStepEvent) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_log(&self, _: WorkflowLog) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_state_change(&self, _: WorkflowStateEvent) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_step_completed(&self, _: StepRunResult) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_job_completed(&self, _: JobRunResult) -> HookNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_workflow_completed(&self, _: WorkflowRunResult) -> HookNoopResult {
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
  let (tx, rx) = tokio::sync::oneshot::channel();
  let server_thread_handle = tokio::spawn(async {
    // Check if docker is installed and running
    let is_support_docker = std::process::Command::new("docker")
      .arg("ps")
      .status()
      .map_or(false, |status| status.success());

    let server = AstroRunServer::with_scheduler(DefaultScheduler::new());

    let cloned_server = server.clone();
    let handle = tokio::task::spawn(async move {
      tx.send(()).unwrap();
      cloned_server.serve("127.0.0.1:5338").await.unwrap();
    });

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
      .runner(server)
      .build();

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

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

    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

    handle.abort();
  });

  let client_thread_handle = tokio::spawn(async {
    // Wait for server to start and listen for connections
    rx.await.unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let runner = TestRunner::new();

    let mut astro_run_runner = AstroRunRunner::builder()
      .id("test-runner")
      .runner(runner)
      .max_runs(5)
      .support_host(true)
      .plugin(
        AstroRunPlugin::builder("abort-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();

            Ok(())
          })
          .build(),
      )
      .url("http://127.0.0.1:5338")
      .build()
      .await
      .unwrap();

    tokio::select! {
      _ = astro_run_runner.start() => {}
      _ = rx.recv() => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

#[astro_run_test::test]
async fn no_available_runners() {
  let server = AstroRunServer::with_scheduler(DefaultScheduler::new());

  let astro_run = AstroRun::builder()
    .plugin(assert_logs_plugin(vec!["No runner available"]))
    .runner(server)
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
async fn test_build_runner_error() {
  let error = AstroRunRunner::builder().build().await.err();

  assert_eq!(error.unwrap(), Error::init_error("Missing id"));

  let error = AstroRunRunner::builder()
    .id("test-runner")
    .build()
    .await
    .err();

  assert_eq!(error.unwrap(), Error::init_error("Missing url"));

  let error = AstroRunRunner::builder()
    .id("test-runner")
    .url("http://127.0.0.1:5338")
    .build()
    .await
    .err();

  assert_eq!(error.unwrap(), Error::init_error("Missing runner"));
}

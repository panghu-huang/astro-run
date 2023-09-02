use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, PluginBuilder, Result, RunResult, Runner, Workflow,
  WorkflowState,
};
use astro_run_server::{AstroRunRunner, AstroRunServer, DefaultScheduler};
use parking_lot::Mutex;

struct TestRunner;

impl TestRunner {
  fn new() -> Self {
    TestRunner
  }
}

impl Runner for TestRunner {
  fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);

    tx.end(RunResult::Succeeded);

    Ok(rx)
  }
}

fn assert_logs_plugin(excepted_logs: Vec<&'static str>) -> AstroRunPlugin {
  let index = Mutex::new(0);

  PluginBuilder::new("assert-logs-plugin")
    .on_log(move |log| {
      let mut i = index.lock();
      assert_eq!(log.message, excepted_logs[*i]);
      *i += 1;
    })
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
      cloned_server.serve("127.0.0.1:5001").await.unwrap();
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
        PluginBuilder::new("abort-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
          })
          .build(),
      )
      .url("http://127.0.0.1:5001")
      .build()
      .await
      .unwrap();

    astro_run_runner.register_plugin(AstroRunPlugin::builder("test").build());

    astro_run_runner.unregister_plugin("test");

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
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
}

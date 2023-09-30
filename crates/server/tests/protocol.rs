use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, EnvironmentVariable, JobRunResult, PluginBuilder,
  Result, RunResult, Runner, Workflow, WorkflowLog, WorkflowRunResult, WorkflowState,
  WorkflowStateEvent,
};
use astro_run_server::{AstroRunRunner, AstroRunServer};
use parking_lot::Mutex;
use std::ops::AddAssign;

struct TestRunner {
  expected_event_count: usize,
  current_event_count: Mutex<usize>,
}

impl TestRunner {
  fn new(expected_event_count: usize) -> Self {
    TestRunner {
      expected_event_count,
      current_event_count: Mutex::new(0),
    }
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

  fn on_run_workflow(&self, event: astro_run::RunWorkflowEvent) {
    self.current_event_count.lock().add_assign(1);
    assert_eq!(event.payload.name.unwrap(), "CI");
    assert_eq!(event.workflow_event.unwrap().sha, "123456789");
  }

  fn on_run_job(&self, event: astro_run::RunJobEvent) {
    self.current_event_count.lock().add_assign(1);

    assert_eq!(event.workflow_event.unwrap().sha, "123456789");

    let job = event.payload;

    assert_eq!(job.name.unwrap(), "Test Job");
    let step = job.steps[0].clone();
    assert_eq!(step.run, "Hello World");
    assert_eq!(step.continue_on_error, false);
    assert_eq!(step.timeout, std::time::Duration::from_secs(60 * 60));
    let container = step.container.unwrap();
    assert_eq!(container.name, "alpine");
    assert_eq!(container.volumes.unwrap()[0], "from:to");
    assert_eq!(container.security_opts.unwrap()[0], "seccomp=unconfined");
    assert_eq!(
      step.environments.get("STRING").unwrap().clone(),
      EnvironmentVariable::from("VALUE")
    );
    assert_eq!(
      step.environments.get("NUMBER").unwrap().clone(),
      EnvironmentVariable::from(1.0)
    );
    assert_eq!(
      step.environments.get("BOOLEAN").unwrap().clone(),
      EnvironmentVariable::from(true)
    );
    assert_eq!(step.secrets[0], "secret-name");
  }

  fn on_state_change(&self, _event: WorkflowStateEvent) {
    self.current_event_count.lock().add_assign(1);
  }

  fn on_job_completed(&self, result: JobRunResult) {
    self.current_event_count.lock().add_assign(1);
    assert_eq!(result.state, WorkflowState::Succeeded);
  }

  fn on_step_completed(&self, result: astro_run::StepRunResult) {
    self.current_event_count.lock().add_assign(1);
    assert_eq!(result.state, WorkflowState::Succeeded);
  }

  fn on_log(&self, log: WorkflowLog) {
    self.current_event_count.lock().add_assign(1);

    let index = log.step_id.step_number();
    if index == 0 {
      assert_eq!(log.message, "Hello World");
    } else if index == 1 {
      assert_eq!(log.message, "Hello World1");
    }
  }

  fn on_workflow_completed(&self, result: WorkflowRunResult) {
    self.current_event_count.lock().add_assign(1);
    assert_eq!(result.state, WorkflowState::Succeeded);

    assert_eq!(*self.current_event_count.lock(), self.expected_event_count);
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
async fn test_protocol() -> Result<()> {
  let (tx, rx) = tokio::sync::oneshot::channel();
  let server_thread_handle = tokio::spawn(async {
    let server = AstroRunServer::new();

    let cloned_server = server.clone();
    let handle = tokio::task::spawn(async move {
      tx.send(()).unwrap();

      cloned_server.serve("127.0.0.1:5338").await.unwrap();
    });

    let astro_run = AstroRun::builder()
      .plugin(assert_logs_plugin(vec!["Hello World", "Hello World1"]))
      .runner(server)
      .build();

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

    let workflow = r#"
    name: CI
    jobs:
      test:
        name: Test Job
        steps:
          - timeout: 60m
            continue-on-error: false
            container:
              name: alpine
              volumes:
                - from:to
              security-opts:
                - seccomp=unconfined
            environments:
              STRING: VALUE
              NUMBER: 1.0
              BOOLEAN: true
            secrets:
              - secret-name
            run: Hello World
          - run: Hello World1
      "#;

    let workflow = Workflow::builder()
      .config(workflow)
      .build(&astro_run)
      .unwrap();

    let ctx = astro_run
      .execution_context()
      .event(astro_run::WorkflowEvent {
        sha: "123456789".to_string(),
        ..Default::default()
      })
      .build();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Succeeded);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let client_thread_handle = tokio::spawn(async {
    // Wait for server to start and listen for connections
    rx.await.unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let runner = TestRunner::new(18);

    let mut astro_run_runner = AstroRunRunner::builder()
      .id("test-runner")
      .runner(runner)
      .max_runs(5)
      .support_docker(true)
      .support_host(true)
      .plugin(assert_logs_plugin(vec!["Hello World", "Hello World1"]))
      .plugin(
        PluginBuilder::new("abort-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
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
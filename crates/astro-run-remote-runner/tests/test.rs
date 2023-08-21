use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, EnvironmentVariable, Job, JobRunResult, PluginBuilder,
  Result, RunResult, Runner, Workflow, WorkflowLog, WorkflowRunResult, WorkflowState,
  WorkflowStateEvent,
};
use astro_run_remote_runner::{AstroRunRemoteRunnerClient, AstroRunRemoteRunnerServer};
use parking_lot::Mutex;

struct TestRunner {}

impl TestRunner {
  fn new() -> Self {
    TestRunner {}
  }
}

impl Runner for TestRunner {
  fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);
    tx.end(RunResult::Succeeded);

    Ok(rx)
  }

  fn on_run_workflow(&self, workflow: Workflow) {
    println!(
      "Running workflow: {}",
      workflow.name.unwrap_or("None".to_string())
    );
  }

  fn on_run_job(&self, job: Job) {
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

  fn on_state_change(&self, event: WorkflowStateEvent) {
    println!("State changed: {:?}", event);
  }

  fn on_job_completed(&self, result: JobRunResult) {
    println!("Job completed: {:?}", result);
  }

  fn on_log(&self, log: WorkflowLog) {
    println!("Log: {:?}", log);
  }

  fn on_workflow_completed(&self, result: WorkflowRunResult) {
    println!("Workflow completed {:?}", result);
  }
}

fn assert_logs_plugin(excepted_logs: Vec<String>) -> AstroRunPlugin {
  let index = Mutex::new(0);

  PluginBuilder::new("assert-logs-plugin")
    .on_log(move |log| {
      let mut i = index.lock();
      assert_eq!(log.message, excepted_logs[*i]);
      *i += 1;
    })
    .build()
}

#[tokio::test]
async fn test_run() -> Result<()> {
  let client_thread_handle = tokio::spawn(async {
    let client_runner = AstroRunRemoteRunnerClient::builder()
      .url("http://127.0.0.1:5002")
      .build()
      .await
      .unwrap();

    let cloned_client_runner = client_runner.clone();
    let handle = tokio::task::spawn(async move {
      cloned_client_runner.start().await.unwrap();
    });

    let astro_run = AstroRun::builder()
      .plugin(assert_logs_plugin(vec![
        "Hello World".to_string(),
        "Hello World1".to_string(),
      ]))
      .runner(client_runner)
      .plugin(
        AstroRunPlugin::builder("abort-thread")
          .on_workflow_completed(move |_| {
            // handle.abort();
            println!("Workflow completed");
          })
          .build(),
      )
      .build();

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let workflow = r#"
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
      .event(astro_run::WorkflowEvent::default())
      .config(workflow)
      .build(&astro_run)
      .unwrap();

    let ctx = astro_run.execution_context();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Succeeded);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let server_thread_handle = tokio::spawn(async {
    let runner = TestRunner::new();

    let runner_server = AstroRunRemoteRunnerServer::builder()
      .id("test-runner")
      .runner(runner)
      .build()
      .unwrap();

    runner_server.serve("127.0.0.1:5002").await.unwrap();
  });

  client_thread_handle.await.unwrap();
  server_thread_handle.abort();

  Ok(())
}

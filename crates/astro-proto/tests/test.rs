use astro_proto::{AstroProtoRunner, AstroProtoServer};
use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, Job, JobRunResult, PluginBuilder, Result, RunResult,
  Runner, Workflow, WorkflowLog, WorkflowRunResult, WorkflowState, WorkflowStateEvent,
};
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

    if let Some(container) = ctx.command.container {
      match container.name.as_str() {
        "throw-error" => return Err(astro_run::Error::internal_runtime_error(0)),
        "failed" => {
          tx.error(ctx.command.run);
          tx.end(RunResult::Failed { exit_code: 1 });
        }
        "cancel" => {
          tx.log(ctx.command.run);
          tx.end(RunResult::Cancelled);
        }
        _ => {
          tx.log(ctx.command.run);
          tx.end(RunResult::Succeeded);
        }
      }
    } else {
      tx.log(ctx.command.run);

      tx.end(RunResult::Succeeded);
    }

    Ok(rx)
  }

  fn on_run_workflow(&self, workflow: Workflow) {
    println!(
      "Running workflow: {}",
      workflow.name.unwrap_or("None".to_string())
    );
  }

  fn on_run_job(&self, job: Job) {
    println!("Running job: {}", job.name.unwrap_or("None".to_string()));
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

  PluginBuilder::new("test-plugin")
    .on_log(move |log| {
      let mut i = index.lock();
      assert_eq!(log.message, excepted_logs[*i]);
      *i += 1;
    })
    .build()
}

#[tokio::test]
async fn test_run() -> Result<()> {
  let server_thread_handle = tokio::spawn(async {
    let server = AstroProtoServer::new();
    let astro_run = AstroRun::builder()
      .plugin(assert_logs_plugin(vec!["Hello World".to_string()]))
      .runner(server.clone())
      .build();

    let handle = tokio::task::spawn(async move {
      server.serve("127.0.0.1:5001").await.unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let workflow = r#"
    jobs:
      test:
        name: Test Job
        steps:
          - timeout: 60m
            continue-on-error: false
            run: Hello World
      "#;

    let workflow = Workflow::builder()
      .event(astro_run::WorkflowEvent::default())
      .config(workflow)
      .build()
      .unwrap();

    let ctx = astro_run.execution_context();

    let res = workflow.run(ctx).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    handle.abort();

    assert_eq!(res.state, WorkflowState::Succeeded);
  });

  let client_thread_handle = tokio::spawn(async {
    let runner = TestRunner::new();

    let mut proto_runner = AstroProtoRunner::builder()
      .runner(runner)
      .url("http://127.0.0.1:5001")
      .id("test-runner")
      .build()
      .await
      .unwrap();

    proto_runner.start().await.unwrap();
  });

  server_thread_handle.await.unwrap();
  client_thread_handle.abort();

  Ok(())
}

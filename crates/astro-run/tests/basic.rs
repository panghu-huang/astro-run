use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, Job, JobRunResult, PluginBuilder, RunResult, Runner,
  Workflow, WorkflowLog, WorkflowRunResult, WorkflowState, WorkflowStateEvent,
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
async fn test_run() {
  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - timeout: 60m
        continue-on-error: false
        run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec!["Hello World".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
}

#[tokio::test]
async fn test_multiple_steps() {
  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - run: Hello World1
      - name: Test Step
        run: Hello World2
      - container:
          name: test
        run: Hello World3
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec![
      "Hello World1".to_string(),
      "Hello World2".to_string(),
      "Hello World3".to_string(),
    ]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 3);

  for step in &job_result.steps {
    assert_eq!(step.state, WorkflowState::Succeeded);
  }
}

#[tokio::test]
async fn test_throw_error() {
  let workflow = r#"
jobs:
  test:
    steps:
      - container: throw-error
        run: Hello World1
      - run: Hello World2
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec!["Hello World1".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 2);

  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[1].state, WorkflowState::Skipped);
}

#[tokio::test]
async fn test_depends_on() {
  let workflow = r#"
  jobs:
    test:
      name: Test Job
      steps:
        - run: Hello World1
    depends_test:
      name: Depends Test Job
      depends-on: [test]
      steps:
        - name: Test Step
          run: Hello World2   
    "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec![
      "Hello World1".to_string(),
      "Hello World2".to_string(),
    ]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  let job_result = res.jobs.get("depends_test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);
}

#[tokio::test]
async fn test_failed_step() {
  let workflow = r#"
jobs:
  test:
    steps:
      - container: failed
        run: Failed step
      - run: Skipped step
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec!["Failed step".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 2);
  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[1].state, WorkflowState::Skipped);
}

#[tokio::test]
async fn test_continue_on_error() {
  let workflow = r#"
jobs:
  test:
    steps:
      - container: failed
        run: Failed step
      - continue-on-error: true
        run: Hello World
      - run: Skipped step
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec![
      "Failed step".to_string(),
      "Hello World".to_string(),
    ]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 3);
  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[1].state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps[2].state, WorkflowState::Skipped);
}

#[tokio::test]
async fn test_cancel_step() {
  let workflow = r#"
jobs:
  test:
    steps:
      - container: cancel
        run: Cancel step
      - continue-on-error: true
        run: Skipped step
      - run: Skipped step
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(assert_logs_plugin(vec!["Cancel step".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps.len(), 3);
  assert_eq!(job_result.steps[0].state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps[1].state, WorkflowState::Skipped);
  assert_eq!(job_result.steps[2].state, WorkflowState::Skipped);
}

#[tokio::test]
async fn test_astro_plugins() {
  let workflow = r#"
name: Test Workflow
jobs:
  test:
    name: Test Job
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder().runner(TestRunner::new()).build();

  astro_run.register_plugin(
    AstroRunPlugin::builder("test")
      .on_log(|log| {
        assert_eq!(log.message, "Hello World");
      })
      .on_run_workflow(|workflow| {
        assert_eq!(workflow.name.unwrap(), "Test Workflow");
        assert_eq!(workflow.jobs.len(), 1);
        assert_eq!(workflow.id.inner(), "id");
      })
      .on_run_job(|job| {
        assert_eq!(job.id.job_key(), "test");
        assert_eq!(job.name.unwrap(), "Test Job");
      })
      .on_job_completed(|res| {
        assert_eq!(res.state, WorkflowState::Succeeded);
      })
      .on_workflow_completed(|res| {
        assert_eq!(res.state, WorkflowState::Succeeded);
      })
      .build(),
  );

  let workflow = Workflow::builder()
    .id("id")
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  workflow.run(ctx).await;
}

#[tokio::test]
async fn test_unregister_astro_plugins() {
  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder().runner(TestRunner::new()).build();

  astro_run.register_plugin(
    AstroRunPlugin::builder("test")
      .on_log(|_| panic!("Should not be called"))
      .build(),
  );

  astro_run.unregister_plugin("test");

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  workflow.run(ctx).await;
}

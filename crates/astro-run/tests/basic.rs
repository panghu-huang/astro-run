use astro_run::{stream, AstroRun, AstroRunPlugin, PluginBuilder, Runner, Workflow};
use astro_run_shared::{Context, RunResult, WorkflowState};
use parking_lot::Mutex;

struct TestRunner {}

impl TestRunner {
  fn new() -> Self {
    TestRunner {}
  }
}

impl Runner for TestRunner {
  fn run(&self, ctx: Context) -> astro_run_shared::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);

    tx.end(RunResult::Succeeded);

    Ok(rx)
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
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(Box::new(TestRunner::new()))
    .plugin(assert_logs_plugin(vec!["Hello World".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run_shared::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await.unwrap();

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  for step in &job_result.steps {
    assert_eq!(step.state, WorkflowState::Succeeded);
  }
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
      - run: Hello World3
  "#;

  let astro_run = AstroRun::builder()
    .runner(Box::new(TestRunner::new()))
    .plugin(assert_logs_plugin(vec![
      "Hello World1".to_string(),
      "Hello World2".to_string(),
      "Hello World3".to_string(),
    ]))
    .build();

  let workflow = Workflow::builder()
    .event(astro_run_shared::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await.unwrap();

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 3);

  for step in &job_result.steps {
    assert_eq!(step.state, WorkflowState::Succeeded);
  }
}

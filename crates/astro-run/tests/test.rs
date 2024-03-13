use astro_run::{
  stream, Action, ActionSteps, AstroRun, AstroRunPlugin, Context, Payload, Plugin,
  PluginNoopResult, RunResult, Runner, UserActionStep, UserCommandStep, UserStep, Workflow,
  WorkflowEvent, WorkflowLog, WorkflowState,
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
}

struct AssetLogsPlugin {
  excepted_logs: Vec<&'static str>,
  index: Mutex<usize>,
}

impl AssetLogsPlugin {
  fn new(excepted_logs: Vec<&'static str>) -> Self {
    AssetLogsPlugin {
      excepted_logs,
      index: Mutex::new(0),
    }
  }
}

#[astro_run::async_trait]
impl Plugin for AssetLogsPlugin {
  fn name(&self) -> &'static str {
    "test-plugin"
  }

  async fn on_log(&self, log: WorkflowLog) -> PluginNoopResult {
    let mut i = self.index.lock();
    assert_eq!(log.message, self.excepted_logs[*i]);
    *i += 1;

    Ok(())
  }
}

#[astro_run_test::test]
async fn test_run() {
  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(AssetLogsPlugin::new(vec!["Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_full_features() {
  let workflow = r#"
on: [push]

jobs:
  test:
    name: Test Job
    on:
      pull_request:
        branches: [master]
    steps:
      - name: Step
        continue-on-error: false
        container:
          name: test
          volumes:
            - from:to
          security_opts:
            - label
        environments:
          name: value
        secrets:
          - secret-key
        timeout: 60m
        run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(AssetLogsPlugin::new(vec!["Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
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
    .plugin(AssetLogsPlugin::new(vec![
      "Hello World1",
      "Hello World2",
      "Hello World3",
    ]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 3);

  for step in &job_result.steps {
    assert_eq!(step.state, WorkflowState::Succeeded);
  }
}

#[astro_run_test::test]
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
    .plugin(AssetLogsPlugin::new(vec!["Hello World1"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 2);

  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[1].state, WorkflowState::Skipped);
}

#[astro_run_test::test]
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
        - run: Hello World2
    depends_test2:
      name: Depends Test Job
      depends-on: [test, depends_test]
      steps:
        - run: Hello World3
    "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(AssetLogsPlugin::new(vec![
      "Hello World1",
      "Hello World2",
      "Hello World3",
    ]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  let job_result = res.jobs.get("depends_test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  let job_result = res.jobs.get("depends_test2").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);
}

#[astro_run_test::test]
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
    .plugin(AssetLogsPlugin::new(vec!["Failed step"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 2);
  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[1].state, WorkflowState::Skipped);
}

#[astro_run_test::test]
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
    .plugin(AssetLogsPlugin::new(vec!["Failed step", "Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 3);
  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[1].state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps[2].state, WorkflowState::Skipped);
}

#[astro_run_test::test]
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
    .plugin(AssetLogsPlugin::new(vec!["Cancel step"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps.len(), 3);
  assert_eq!(job_result.steps[0].state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps[1].state, WorkflowState::Skipped);
  assert_eq!(job_result.steps[2].state, WorkflowState::Skipped);
}

#[astro_run_test::test]
async fn test_actions() {
  let workflow = r#"
name: Test Workflow
jobs:
  test:
    name: Test Job
    steps:
      - uses: test
      - run: Hello World
  "#;

  struct TestAction;

  impl Action for TestAction {
    fn normalize(&self, _step: UserActionStep) -> astro_run::Result<ActionSteps> {
      Ok(ActionSteps {
        pre: None,
        run: UserStep::Command(UserCommandStep {
          name: Some("Test".to_string()),
          run: String::from("test"),
          ..Default::default()
        }),
        post: None,
      })
    }
  }

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(AssetLogsPlugin::new(vec!["test", "Hello World"]))
    .action("test", TestAction)
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  workflow.run(ctx).await;
}

#[astro_run_test::test]
async fn test_actions_and_plugins() {
  let workflow = r#"
name: Test Workflow
jobs:
  test:
    name: Test Job
    steps:
      - uses: test
      - run: Hello World
  "#;

  struct TestAction {}

  impl Action for TestAction {
    fn normalize(&self, _step: UserActionStep) -> astro_run::Result<ActionSteps> {
      Ok(ActionSteps {
        pre: Some(UserStep::Command(UserCommandStep {
          name: Some("Pre test".to_string()),
          run: String::from("pre test"),
          ..Default::default()
        })),
        run: UserStep::Command(UserCommandStep {
          name: Some("Test".to_string()),
          run: String::from("test"),
          ..Default::default()
        }),
        post: None,
      })
    }
  }

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .plugin(
      AstroRunPlugin::builder("test")
        .on_run_workflow(|event| {
          let workflow = event.payload;
          assert_eq!(workflow.name.unwrap(), "Test Workflow");
          assert_eq!(workflow.jobs.len(), 1);
          assert_eq!(workflow.id.inner(), "id");

          Ok(())
        })
        .on_run_job(|event| {
          let job = event.payload;
          assert_eq!(job.id.job_key(), "test");
          assert_eq!(job.name.unwrap(), "Test Job");

          let steps = job.steps.clone();
          assert_eq!(steps.len(), 3);

          let step = steps[0].clone();
          assert_eq!(step.name.unwrap(), "Pre test");
          assert_eq!(step.run, "pre test");

          let step = steps[1].clone();
          assert_eq!(step.name.unwrap(), "Test");
          assert_eq!(step.run, "test");

          let step = steps[2].clone();
          assert_eq!(step.run, "Hello World");

          Ok(())
        })
        .on_run_step(|event| {
          let step = event.payload;
          let index = step.id.step_number();
          match index {
            0 => {
              assert_eq!(step.name.unwrap(), "Pre test");
              assert_eq!(step.run, "pre test");

              Ok(())
            }
            1 => {
              assert_eq!(step.name.unwrap(), "Test");
              assert_eq!(step.run, "test");

              Ok(())
            }
            2 => {
              assert_eq!(step.run, "Hello World");

              Ok(())
            }
            _ => panic!("Should not be called"),
          }
        })
        .on_step_completed(|res| {
          assert_eq!(res.state, WorkflowState::Succeeded);

          Ok(())
        })
        .on_job_completed(|res| {
          assert_eq!(res.state, WorkflowState::Succeeded);

          Ok(())
        })
        .on_workflow_completed(|res| {
          assert_eq!(res.state, WorkflowState::Succeeded);

          Ok(())
        })
        .build(),
    )
    .plugin(AssetLogsPlugin::new(vec![
      "pre test",
      "test",
      "Hello World",
    ]))
    .action("test", TestAction {})
    .build();

  let workflow = Workflow::builder()
    .id("id")
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_empty_event() {
  let workflow = r#"
on: [push]

jobs:
  test:
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .github_personal_token("token")
    .runner(TestRunner::new())
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_invalid_event() -> astro_run::Result<()> {
  dotenv::dotenv().ok();

  let app_id = std::env::var("GH_APP_ID")
    .map_err(|err| astro_run::Error::internal_runtime_error(format!("GH_APP_ID: {}", err)))?;

  let private_key = std::env::var("GH_APP_PRIVATE_KEY").map_err(|err| {
    astro_run::Error::internal_runtime_error(format!("GH_APP_PRIVATE_KEY: {}", err))
  })?;

  let workflow = r#"
on: [push]

jobs:
  test:
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .github_app(app_id.parse().unwrap(), private_key)
    .runner(TestRunner::new())
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(WorkflowEvent {
      event: "push".to_string(),
      ..Default::default()
    })
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);

  let ctx = astro_run
    .execution_context()
    .event(WorkflowEvent {
      event: "pull_request".to_string(),
      ..Default::default()
    })
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);

  Ok(())
}

struct WorkflowPayload(String);

impl Payload for WorkflowPayload {
  fn try_from(payload: &String) -> astro_run::Result<Self>
  where
    Self: Sized,
  {
    Ok(WorkflowPayload(payload.clone()))
  }

  fn try_into(&self) -> astro_run::Result<String> {
    Ok(self.0.clone())
  }
}

#[astro_run_test::test]
async fn test_workflow_payload() {
  let yaml = r#"
jobs:
  test:
    steps:
      - name: Hello World
        run: Hello World
"#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner)
    .plugin(AssetLogsPlugin::new(vec!["Hello World"]))
    .plugin(
      AstroRunPlugin::builder("test-plugin")
        .on_run_workflow(|event| {
          let payload: WorkflowPayload = event.payload.payload().unwrap();

          assert_eq!(payload.0, "Test payload");

          Ok(())
        })
        .build(),
    )
    .build();

  let workflow_payload = WorkflowPayload("Test payload".to_string());

  let workflow = Workflow::builder()
    .config(yaml)
    .payload(workflow_payload)
    .build(&astro_run)
    .await
    .unwrap();

  let payload = workflow.payload::<WorkflowPayload>().unwrap();

  assert_eq!(payload.0, "Test payload");

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

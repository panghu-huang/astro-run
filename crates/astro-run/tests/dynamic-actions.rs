use astro_run::{
  stream, Action, ActionSteps, AstroRun, AstroRunPlugin, Context, Error, Result, RunResult, Runner,
  UserActionStep, UserCommandStep, UserStep, Workflow, WorkflowState,
};
use parking_lot::Mutex;
use serde::Deserialize;

struct TestRunner;

#[astro_run::async_trait]
impl Runner for TestRunner {
  async fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);
    tx.end(RunResult::Succeeded);

    Ok(rx)
  }
}

fn assert_logs_plugin(excepted_logs: Vec<&'static str>) -> AstroRunPlugin {
  let index = Mutex::new(0);

  AstroRunPlugin::builder("test-plugin")
    .on_log(move |log| {
      let mut i = index.lock();
      assert_eq!(log.message, excepted_logs[*i]);
      *i += 1;

      Ok(())
    })
    .build()
}

struct DynamicAction;

#[derive(Deserialize)]
struct DynamicActionConfig {
  name: String,
}

impl Action for DynamicAction {
  fn normalize(&self, step: UserActionStep) -> Result<ActionSteps> {
    let with: DynamicActionConfig = serde_yaml::from_value(step.with.unwrap()).unwrap();

    if with.name == "Error" {
      return Err(Error::error("Test Error"));
    }

    Ok(ActionSteps {
      pre: None,
      run: UserStep::Command(UserCommandStep {
        name: Some(with.name.clone()),
        run: with.name,
        ..Default::default()
      }),
      post: None,
    })
  }
}

fn dynamic_action_plugin() -> AstroRunPlugin {
  AstroRunPlugin::builder("dynamic-action")
    .on_resolve_dynamic_action(|step| {
      let with: DynamicActionConfig = serde_yaml::from_value(step.with.unwrap()).unwrap();

      if with.name == "Hello World" || with.name == "Error" {
        Ok(Some(Box::new(DynamicAction)))
      } else {
        Ok(None)
      }
    })
    .build()
}

#[astro_run_test::test]
async fn test_dynamic_action() {
  let yaml = r#"
jobs:
  test:
    steps:
      - uses: dynamic-action
        with:
          name: Hello World
"#;

  let plugin = dynamic_action_plugin();

  let astro_run = AstroRun::builder()
    .runner(TestRunner)
    .plugin(plugin)
    .plugin(assert_logs_plugin(vec!["Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(yaml)
    .build(&astro_run)
    .await
    .unwrap();

  let dynamic_step = workflow.jobs.get("test").unwrap().steps.first().unwrap();

  assert_eq!(dynamic_step.run, "Hello World");
  assert_eq!(dynamic_step.name, Some("Hello World".to_string()));

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_dynamic_action_not_found() {
  let yaml = r#"
jobs:
  test:
    steps:
      - uses: dynamic-action
        with:
          name: Not Found
"#;

  let plugin = dynamic_action_plugin();

  let astro_run = AstroRun::builder()
    .runner(TestRunner)
    .plugin(plugin)
    .plugin(assert_logs_plugin(vec!["Hello World"]))
    .build();

  let res = Workflow::builder().config(yaml).build(&astro_run).await;

  assert_eq!(
    res.unwrap_err(),
    Error::workflow_config_error("Action `dynamic-action` is not found")
  );
}

#[astro_run_test::test]
async fn test_dynamic_action_error() {
  let yaml = r#"
jobs:
  test:
    steps:
      - uses: dynamic-action
        with:
          name: Error
"#;

  let plugin = dynamic_action_plugin();

  let astro_run = AstroRun::builder()
    .runner(TestRunner)
    .plugin(plugin)
    .build();

  let res = Workflow::builder().config(yaml).build(&astro_run).await;

  assert_eq!(res.unwrap_err(), Error::error("Test Error"));
}

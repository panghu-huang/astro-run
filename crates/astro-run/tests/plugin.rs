use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, Error, HookBeforeRunStepResult, HookNoopResult,
  JobRunResult, Plugin, RunJobEvent, RunResult, RunStepEvent, RunWorkflowEvent, Runner, Step,
  StepRunResult, Workflow, WorkflowLog, WorkflowRunResult, WorkflowState, WorkflowStateEvent,
};
use parking_lot::Mutex;

struct TestRunner;

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

  async fn on_before_run_step(&self, step: Step) -> HookBeforeRunStepResult {
    let mut step = step;
    step.run = "Updated command".to_string();
    Ok(step)
  }
}

struct ErrorBeforeRunStepPlugin;

#[astro_run::async_trait]
impl Plugin for ErrorBeforeRunStepPlugin {
  fn name(&self) -> &'static str {
    "error-before-run-step-plugin"
  }

  async fn on_before_run_step(&self, _step: Step) -> HookBeforeRunStepResult {
    Err(Error::error("Error"))
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
async fn test_plugin() {
  let yaml = r#"
jobs:
  test:
    steps:
      - run: Will be updated
"#;

  let plugin = error_plugin();

  let astro_run = AstroRun::builder()
    .runner(TestRunner)
    .plugin(plugin)
    .plugin(ErrorBeforeRunStepPlugin)
    .plugin(assert_logs_plugin(vec!["Updated command"]))
    .build();

  let workflow = Workflow::builder()
    .config(yaml)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

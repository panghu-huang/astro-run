use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, Error, JobRunResult, PluginNoopResult, RunJobEvent,
  RunResult, RunStepEvent, RunWorkflowEvent, Runner, StepRunResult, Workflow, WorkflowLog,
  WorkflowRunResult, WorkflowState, WorkflowStateEvent,
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

  async fn on_run_workflow(&self, _: RunWorkflowEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_run_job(&self, _: RunJobEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_run_step(&self, _: RunStepEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_log(&self, _: WorkflowLog) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_state_change(&self, _: WorkflowStateEvent) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_step_completed(&self, _: StepRunResult) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_job_completed(&self, _: JobRunResult) -> PluginNoopResult {
    Err(Error::error("Error"))
  }

  async fn on_workflow_completed(&self, _: WorkflowRunResult) -> PluginNoopResult {
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
async fn test_error_plugin() {
  let yaml = r#"
jobs:
  test:
    steps:
      - run: Hello World
"#;

  let plugin = error_plugin();

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

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

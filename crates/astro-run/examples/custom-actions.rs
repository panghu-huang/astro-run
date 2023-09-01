use astro_run::{
  stream, Action, ActionSteps, AstroRun, Context, Result, RunResult, UserActionStep,
  UserCommandStep, UserStep, Workflow,
};
use serde::{Deserialize, Serialize};

struct Runner {}

impl Runner {
  fn new() -> Self {
    Runner {}
  }
}

impl astro_run::Runner for Runner {
  fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    // Send running log
    tx.log(ctx.command.run);

    // Send success log
    tx.end(RunResult::Succeeded);

    Ok(rx)
  }
}

struct CacheAction {}

#[derive(Serialize, Deserialize)]
struct CacheOptions {
  path: String,
  key: String,
}

impl Action for CacheAction {
  fn normalize(&self, step: UserActionStep) -> Result<ActionSteps> {
    let options: CacheOptions = serde_yaml::from_value(step.with.unwrap()).unwrap();
    Ok(ActionSteps {
      pre: None,
      run: UserStep::Command(UserCommandStep {
        name: Some("Restore cache".to_string()),
        run: format!("restore cache {} {}", options.path, options.key),
        ..Default::default()
      }),
      post: Some(UserStep::Command(UserCommandStep {
        name: Some("Save cache".to_string()),
        run: format!("save cache {} {}", options.path, options.key),
        ..Default::default()
      })),
    })
  }
}

#[tokio::main]
async fn main() {
  // Create astro run
  let astro_run = AstroRun::builder()
    .runner(Runner::new())
    .action("caches", CacheAction {})
    .build();

  // Workflow
  let workflow = r#"
jobs:
  job:
    name: Job
    steps:
      - uses: caches
        with:
          path: /tmp
          key: test
      - run: Hello World
  "#;

  // Create workflow
  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  // Create a new execution context
  let ctx = astro_run.execution_context().build();

  // Run workflow
  let _res = workflow.run(ctx).await;
}

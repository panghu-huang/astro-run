use astro_run::{stream, AstroRun, Context, RunResult, Workflow};

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

#[tokio::main]
async fn main() {
  // Create astro run
  let astro_run = AstroRun::builder().runner(Runner::new()).build();

  // Workflow
  let workflow = r#"
jobs:
  job:
    name: Job
    steps:
      - timeout: 60m
        continue-on-error: false
        run: Hello World
  "#;

  // Create workflow
  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  // Create a new execution context
  let ctx = astro_run.execution_context();

  // Run workflow
  let _res = workflow.run(ctx).await;
}

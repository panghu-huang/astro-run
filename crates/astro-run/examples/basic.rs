use astro_run::{stream, AstroRun, RunResult, Workflow};

struct Runner;

impl Runner {
  fn new() -> Self {
    Runner
  }
}

#[astro_run::async_trait]
impl astro_run::Runner for Runner {
  async fn run(&self, ctx: astro_run::Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tokio::task::spawn(async move {
      // Send running log
      tx.log(ctx.command.run);

      // Send success log
      tx.end(RunResult::Succeeded);
    });

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

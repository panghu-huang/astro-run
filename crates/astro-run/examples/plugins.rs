use astro_run::{stream, AstroRun, AstroRunPlugin, Context, RunResult, Workflow};

struct Runner {}

impl Runner {
  fn new() -> Self {
    Runner {}
  }
}

#[astro_run::async_trait]
impl astro_run::Runner for Runner {
  async fn run(&self, ctx: Context) -> astro_run::RunResponse {
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
  let plugin = AstroRunPlugin::builder("plugin-name")
    .on_run_workflow(|event| println!("{:?}", event))
    .on_run_job(|job| {
      println!("{:?}", job);
    })
    .on_run_step(|step| {
      println!("{:?}", step);
    })
    .on_log(|log| {
      println!("{:?}", log);
    })
    .on_state_change(|event| {
      println!("{:?}", event);
    })
    .on_step_completed(|result| {
      println!("{:?}", result);
    })
    .on_job_completed(|result| {
      println!("{:?}", result);
    })
    .on_workflow_completed(|result| {
      println!("{:?}", result);
    })
    .build();

  // Create astro run
  let astro_run = AstroRun::builder()
    .runner(Runner::new())
    .plugin(plugin)
    .build();

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
    .await
    .unwrap();

  // Create a new execution context
  let ctx = astro_run.execution_context().build();

  // Run workflow
  let _res = workflow.run(ctx).await;
}

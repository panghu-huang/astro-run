use astro_run::{stream, Context, Result, RunResult};
use astro_run_server::AstroRunRunner;

struct Runner {}

impl Runner {
  fn new() -> Self {
    Runner {}
  }
}

impl astro_run::Runner for Runner {
  fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);
    tx.end(RunResult::Succeeded);

    Ok(rx)
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let runner = Runner::new();

  let mut astro_run_runner = AstroRunRunner::builder()
    .runner(runner)
    .url("http://127.0.0.1:5001")
    .id("test-runner")
    .build()
    .await
    .unwrap();

  astro_run_runner.start().await.unwrap();

  Ok(())
}

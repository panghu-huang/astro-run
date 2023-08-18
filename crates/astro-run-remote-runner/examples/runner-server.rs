use astro_run::{stream, Context, Result, RunResult};
use astro_run_remote_runner::AstroRunRemoteRunnerServer;

// Simulated implementation of a Runner
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

  let runner_server = AstroRunRemoteRunnerServer::builder()
    .runner(runner)
    .build()
    .unwrap();

  runner_server.serve("127.0.0.1:5002").await.unwrap();

  Ok(())
}

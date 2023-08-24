# Astro Run Remote Runner

`astro-run-remote-runner` is an extension of [astro-run](https://github.com/panghu-huang/astro-run) that allows runners to act as remote services and lets `astro-run` act as a client to connect via gRPC requests. The remote runner streams runtime logs, events, etc. to the client using streams.

## Example

Add `astro-run` and `astro-run-remote-runner` as dependencies in your `Cargo.toml`:

```toml
[dependencies]
astro-run = "0.1"
astro-run-remote-runner = "0.1"
```

### Remote Runner Server

```rust
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
```

### Astro-Run Client

```rust
use astro_run::{AstroRun, Result, Workflow};
use astro_run_remote_runner::AstroRunRemoteRunnerClient;

#[tokio::main]
async fn main() -> Result<()> {
  let client_runner = AstroRunRemoteRunnerClient::builder().build().unwrap();

  let mut cloned_client_runner = client_runner.clone();
  let handle = tokio::task::spawn(async move {
    // Run the client runner in background
    cloned_client_runner
      .start(vec!["http://127.0.0.1:5002"])
      .await
      .unwrap();
  });

  let astro_run = AstroRun::builder().runner(client_runner).build();

  let workflow = r#"
    jobs:
      job-id:
        steps:
          - run: Hello World
      "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  // Create a new execution context
  let ctx = astro_run.execution_context();

  // Run workflow
  let _res = workflow.run(ctx).await;

  // Wait for the client runner to finish
  handle.await.unwrap();

  Ok(())
}
```

In the above example, you can replace the runner with a specific implementation from [astro-runner](../astro-runner).
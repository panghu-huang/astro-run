# Astro Run Server

`astro-run-server` is an extension of [astro-run](https://github.com/panghu-huang/astro-run) that allows it to be served as a service and accept runners via gRPC requests.

`astro-run-server` forwards run requests, plugin events, logs, etc. from `astro-run` to runners using gRPC streams, enabling remote invocation and multi-runner scheduling.

## Example

Add `astro-run` and `astro-run-server` as dependencies in your `Cargo.toml`:

```toml
[dependencies]
astro-run = "0.1"
astro-run-server = "0.1"
```

### Astro Run Server

```rust
use astro_run::{AstroRun, Result, Workflow};
use astro_run_server::AstroRunServer;

#[tokio::main]
async fn main() -> Result<()> {
  let server = AstroRunServer::new();

  // Start server in background
  let handle = tokio::spawn({
    let server = server.clone();

    async move {
      server.serve("127.0.0.1:5338").await.unwrap();
    }
  });

  let astro_run = AstroRun::builder().runner(server).build();

  let workflow = r#"
    jobs:
      test:
        name: Test Job
        steps:
          - timeout: 60m
            continue-on-error: false
            run: Hello World
      "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  // Run workflow
  let _res = workflow.run(ctx).await;

  // Wait for server to stop
  handle.await.unwrap();

  Ok(())
}
```

### Runner

```rust
use astro_run::{stream, Context, Result, RunResult};
use astro_run_server::AstroRunRunner;

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
    .url("http://127.0.0.1:5338")
    .id("test-runner")
    .build()
    .await
    .unwrap();

  astro_run_runner.start().await.unwrap();

  Ok(())
}
```

In the above example, you can replace the runner with a specific implementation from [astro-runner](../runner).

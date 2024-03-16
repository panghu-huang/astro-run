# Astro run

Astro Run is a highly extensible runner that can execute any workflow.

![astro-run](https://img.shields.io/crates/v/astro-run.svg)
![CI](https://github.com/panghu-huang/astro-run/actions/workflows/test.yaml/badge.svg)
[![codecov](https://codecov.io/gh/panghu-huang/astro-run/branch/main/graph/badge.svg?token=B9P3T5C97U)](https://codecov.io/gh/panghu-huang/astro-run)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- [Workflow runtime for Docker](https://github.com/panghu-huang/astro-run/blob/main/crates/runner)
- Support for [gRPC server](https://github.com/panghu-huang/astro-run/blob/main/crates/server) to coordinate multiple runner clients
- Support for [connecting to remote runners](https://github.com/panghu-huang/astro-run/blob/main/crates/remote-runner)

## Example

### Dependencies

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
astro-run = "0.1"
```

### Code Example

```rust
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
    .await
    .unwrap();

  // Create a new execution context
  let ctx = astro_run.execution_context().build();

  // Run workflow
  let _res = workflow.run(ctx).await;
}
```

Astro Run only defines the interface for Runners. Users can implement their own desired Runners, e.g., [Docker runner](https://github.com/panghu-huang/astro-run/tree/main/crates/runner).

## More Examples

- [Workflow runtime for Docker](https://github.com/panghu-huang/astro-run/blob/main/crates/runner/examples/basic.rs)
- [Astro Run Plugins](https://github.com/panghu-huang/astro-run/blob/main/crates/astro-run/examples/plugins.rs)
- [Astro run gRPC Server](https://github.com/panghu-huang/astro-run/blob/main/crates/server/examples/server.rs)
- [gRPC Runner Client](https://github.com/panghu-huang/astro-run/blob/main/crates/server/examples/client.rs)
- [Remote Runner](https://github.com/panghu-huang/astro-run/blob/main/crates/remote-runner/examples/runner-server.rs)

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the project.

## License

This project is licensed under the MIT License.

# Astro run

Astro Run is a highly extensible runner that can execute any workflow.

![astro-run](https://img.shields.io/crates/v/astro-run.svg)
[![codecov](https://codecov.io/gh/panghu-huang/astro-run/branch/main/graph/badge.svg?token=B9P3T5C97U)](https://codecov.io/gh/panghu-huang/astro-run)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

* Workflow runtime for Docker
* Support for [gRPC server](https://github.com/panghu-huang/astro-run/blob/main/crates/astro-run-server/examples/server.rs) to coordinate multiple runner clients
* Support for connecting to remote runners [WIP]

## Example

### Dependencies

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
astro-run = "0.1"
```

### Code Example

```rust
use astro_run::{stream, AstroRun, Context, RunResult, Workflow};

// Custom Runner
struct Runner {}

impl Runner {
    fn new() -> Self {
        Runner {}
    }
}

impl astro_run::Runner for Runner {
    fn run(&self, ctx: Context) -> astro_run::RunResponse {
        let (tx, rx) = stream();

        // Send runtime logs
        tx.log(ctx.command.run);

        // Send run result
        tx.end(RunResult::Succeeded);

        Ok(rx)
    }
}

#[tokio::main]
async fn main() {
    // Create Astro Run
    let astro_run = AstroRun::builder().runner(Runner::new()).build();

    // Workflow Configuration
    let workflow = r#"
jobs:
  job:
    name: Job
    steps:
      - timeout: 60m
        continue-on-error: false
        run: Hello World
  "#;

    // Create Workflow
    let workflow = Workflow::builder()
        .event(astro_run::WorkflowEvent::default())
        .config(workflow)
        .build()
        .unwrap();

    // Create a new execution context
    let ctx = astro_run.execution_context();

    // Run the workflow
    let _res = workflow.run(ctx).await;
}
```

Astro Run only defines the interface for Runners. Users can implement their own desired Runners, e.g., [Docker runner](https://github.com/panghu-huang/astro-run/tree/main/crates/runner).

## More Examples

* Astro Run Plugins: TODO
* [Astro run gRPC Server](https://github.com/panghu-huang/astro-run/blob/main/crates/astro-run-server/examples/server.rs)
* [gRPC Runner Client](https://github.com/panghu-huang/astro-run/blob/main/crates/astro-run-server/examples/client.rs)
* gRPC Server Plugins: TODO
* gRPC Runner Client Plugins: TODO
* Remote Runner: TODO

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the project.

## License

This project is licensed under the MIT License.
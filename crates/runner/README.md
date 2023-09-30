# Astro Runner

`astro-run-runner` is a specific runner implementation within [astro-run](https://github.com/panghu-huang/astro-run). It supports running the smallest execution unit of workflow steps in both local and Dockerfile environments.

It can also be used in conjunction with other components of the `astro-run` ecosystem, such as [astro-run-server](../server) and [astro-run-remote-runner](../remote-runner).

## Example

Add `astro-run` and `astro-runner` as dependencies in your `Cargo.toml`:

```toml
[dependencies]
astro-run = "0.1"
astro-runner = "0.1"
```

### Code Example

```rust
use astro_run::{AstroRun, Workflow};
use astro_runner::AstroRunner;

#[tokio::main]
#[ignore]
async fn main() {
  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - timeout: 60m
        continue-on-error: false
        run: echo "Hello World" >> test.txt
      - run: |
          content=$(cat test.txt)
          echo Content is $content
          echo "Cache" >> /home/work/caches/test.txt
  "#;
  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder().runner(runner).build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let _res = workflow.run(ctx).await;
}
```

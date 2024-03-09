use astro_run::{AstroRun, Workflow, WorkflowState};
use astro_runner::AstroRunner;

#[tokio::main]
async fn main() {
  let workflow = r#"
jobs:
  test:
    steps:
      - container: node:alpine
        environments:
          NAME: Node
        run: |
          #!/usr/local/bin/node

          console.log(`Hello ${process.env.NAME}`);
  "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder().runner(runner).build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

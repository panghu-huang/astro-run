use astro_run::{AstroRun, Workflow};
use astro_runner::DockerRunner;

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
  let runner = DockerRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder().runner(runner).build();

  let workflow = Workflow::builder().config(workflow).build().unwrap();

  let ctx = astro_run.execution_context();

  let _res = workflow.run(ctx).await;
}

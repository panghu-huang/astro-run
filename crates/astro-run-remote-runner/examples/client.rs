use astro_run::{AstroRun, Result, Workflow};
use astro_run_remote_runner::AstroRunRemoteRunnerClient;

#[tokio::main]
async fn main() -> Result<()> {
  let client_runner = AstroRunRemoteRunnerClient::builder()
    .url("http://127.0.0.1:5002")
    .build()
    .await
    .unwrap();

  let cloned_client_runner = client_runner.clone();
  tokio::task::spawn(async move {
    // Run the client runner in background
    cloned_client_runner.start().await.unwrap();
  });

  let astro_run = AstroRun::builder().runner(client_runner).build();

  let workflow = r#"
    jobs:
      test:
        name: Test Job
        steps:
          - run: Hello World1
      "#;

  let workflow = Workflow::builder().config(workflow).build().unwrap();

  // Create a new execution context
  let ctx = astro_run.execution_context();

  // Run workflow
  let _res = workflow.run(ctx).await;

  Ok(())
}

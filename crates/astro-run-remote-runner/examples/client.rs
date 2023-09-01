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
  let ctx = astro_run.execution_context().build();

  // Run workflow
  let _res = workflow.run(ctx).await;

  // Wait for the client runner to finish
  handle.await.unwrap();

  Ok(())
}

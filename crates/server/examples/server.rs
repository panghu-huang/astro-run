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

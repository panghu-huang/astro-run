use astro_run::{AstroRun, Result, Workflow};
use astro_run_server::AstroRunServer;

#[tokio::main]
async fn main() -> Result<()> {
  let server = AstroRunServer::new();

  let cloned_server = server.clone();

  // Start server in background
  let handle = tokio::spawn(async move {
    println!("Starting server");
    cloned_server.serve("127.0.0.1:5001").await.unwrap();
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

  let workflow = Workflow::builder().config(workflow).build().unwrap();

  let ctx = astro_run.execution_context();

  // Run workflow
  let _res = workflow.run(ctx).await;

  // Wait for server to stop
  handle.await.unwrap();

  Ok(())
}

use astro_run::{AstroRun, Result, Workflow};
use astro_run_server::AstroRunServer;

#[tokio::main]
async fn main() -> Result<()> {
  let server = AstroRunServer::new();

  let cloned_server = server.clone();

  // Start server in background
  let handle = tokio::spawn(async move {
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

  let workflow = Workflow::builder()
    .event(astro_run::WorkflowEvent::default())
    .config(workflow)
    .build()
    .unwrap();

  let ctx = astro_run.execution_context();

  // Run workflow
  let _res = workflow.run(ctx).await;

  handle.await.unwrap();

  Ok(())
}

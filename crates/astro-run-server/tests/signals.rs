use astro_run::{
  stream, AstroRun, Context, PluginBuilder, Result, RunResult, Workflow, WorkflowState,
};
use astro_run_server::{AstroRunRunner, AstroRunServer};
use std::time::Duration;

struct TimeoutRunner {
  delay: Duration,
}

impl astro_run::Runner for TimeoutRunner {
  fn run(&self, config: Context) -> astro_run::RunResponse {
    let (sender, receiver) = stream();
    let delay = self.delay;

    tokio::task::spawn(async move {
      tokio::select! {
        _ = tokio::time::sleep(delay) => {
          log::trace!("Sleep completed");
          sender.end(RunResult::Succeeded);
        }
        signal = config.signal.recv() => {
          log::trace!("Received signal {:?}", signal);
          sender.end(RunResult::Failed { exit_code: 123 });
        }
      }
    });

    Ok(receiver)
  }
}

#[astro_run_test::test]
async fn test_signal() -> Result<()> {
  let (tx, rx) = tokio::sync::oneshot::channel();

  let server_thread_handle = tokio::spawn(async {
    let server = AstroRunServer::new();

    let cloned_server = server.clone();
    let handle = tokio::task::spawn(async move {
      tx.send(()).unwrap();
      cloned_server.serve("127.0.0.1:5001").await.unwrap();
    });

    let astro_run = AstroRun::builder().runner(server).build();

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let workflow = format!(
      r#"
    jobs:
      test:
        steps:
          - container: host/{}
            run: Hello World
            timeout: 2s
      "#,
      std::env::consts::OS,
    );

    let workflow = Workflow::builder()
      .config(workflow)
      .build(&astro_run)
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Succeeded);

    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

    handle.abort();
  });

  let client_thread_handle = tokio::spawn(async {
    // Wait for server to start and listen for connections
    rx.await.unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let mut astro_run_runner = AstroRunRunner::builder()
      .id("test-runner")
      .runner(TimeoutRunner {
        delay: Duration::from_secs(1),
      })
      .plugin(
        PluginBuilder::new("abort-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
          })
          .build(),
      )
      .url("http://127.0.0.1:5001")
      .build()
      .await
      .unwrap();

    tokio::select! {
      _ = astro_run_runner.start() => {}
      _ = rx.recv() => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

#[astro_run_test::test]
async fn test_timeout() -> Result<()> {
  let (tx, rx) = tokio::sync::oneshot::channel();

  let server_thread_handle = tokio::spawn(async {
    let server = AstroRunServer::new();

    let cloned_server = server.clone();
    let handle = tokio::task::spawn(async move {
      tx.send(()).unwrap();

      cloned_server.serve("127.0.0.1:5001").await.unwrap();
    });

    let astro_run = AstroRun::builder().runner(server).build();

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let workflow = format!(
      r#"
    jobs:
      test:
        steps:
          - container: host/{}
            run: Hello World
            timeout: 1s
      "#,
      std::env::consts::OS,
    );

    let workflow = Workflow::builder()
      .config(workflow)
      .build(&astro_run)
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Failed);

    assert_eq!(
      res.jobs.get("test").unwrap().steps[0].exit_code.unwrap(),
      123
    );

    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;

    handle.abort();
  });

  let client_thread_handle = tokio::spawn(async {
    // Wait for server to start and listen for connections
    rx.await.unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let mut astro_run_runner = AstroRunRunner::builder()
      .id("test-runner")
      .runner(TimeoutRunner {
        delay: Duration::from_secs(2),
      })
      .plugin(
        PluginBuilder::new("abort-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
          })
          .build(),
      )
      .url("http://127.0.0.1:5001")
      .build()
      .await
      .unwrap();

    tokio::select! {
      _ = astro_run_runner.start() => {}
      _ = rx.recv() => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

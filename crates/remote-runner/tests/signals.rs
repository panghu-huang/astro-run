use astro_run::{
  stream, AstroRun, Context, PluginBuilder, Result, RunResult, Signal, Workflow, WorkflowState,
};
use astro_run_remote_runner::{AstroRunRemoteRunnerClient, AstroRunRemoteRunnerServer};
use std::time::Duration;

struct TimeoutRunner {
  delay: Duration,
}

#[astro_run::async_trait]
impl astro_run::Runner for TimeoutRunner {
  async fn run(&self, config: Context) -> astro_run::RunResponse {
    let (sender, receiver) = stream();
    let delay = self.delay;

    tokio::task::spawn(async move {
      tokio::select! {
        _ = tokio::time::sleep(delay) => {
          sender.end(RunResult::Succeeded);
        }
        signal = config.signal.recv() => {
          log::error!("Received signal {:?}", signal);
          match signal {
            Signal::Cancel => {
              sender.end(RunResult::Cancelled);
            }
            Signal::Timeout => {
              sender.end(RunResult::Failed { exit_code: 123 });
            }
          }
        }
      }
    });

    Ok(receiver)
  }
}

#[astro_run_test::test]
async fn test_signal() -> Result<()> {
  let (oneshot_tx, rx) = tokio::sync::oneshot::channel();
  let client_thread_handle = tokio::spawn(async {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client_runner = AstroRunRemoteRunnerClient::builder().build().unwrap();

    let mut cloned_client_runner = client_runner.clone();
    let handle = tokio::task::spawn(async move {
      rx.await.unwrap();

      cloned_client_runner
        .start(vec!["http://127.0.0.1:5338"])
        .await
        .unwrap();
    });

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let astro_run = AstroRun::builder().runner(client_runner).build();

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
      .await
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Succeeded);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let server_thread_handle = tokio::spawn(async {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let runner_server = AstroRunRemoteRunnerServer::builder()
      .id("test-runner")
      .runner(TimeoutRunner {
        delay: Duration::from_secs(1),
      })
      .max_runs(5)
      .plugin(
        PluginBuilder::new("test-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
            Ok(())
          })
          .build(),
      )
      .build()
      .unwrap();

    oneshot_tx.send(()).unwrap();

    tokio::select! {
      _ = rx.recv() => {}
      _ = runner_server.serve("127.0.0.1:5338") => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

#[astro_run_test::test]
async fn test_timeout() -> Result<()> {
  let (oneshot_tx, rx) = tokio::sync::oneshot::channel();
  let client_thread_handle = tokio::spawn(async {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client_runner = AstroRunRemoteRunnerClient::builder().build().unwrap();

    let mut cloned_client_runner = client_runner.clone();
    let handle = tokio::task::spawn(async move {
      rx.await.unwrap();

      cloned_client_runner
        .start(vec!["http://127.0.0.1:5338"])
        .await
        .unwrap();
    });

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let astro_run = AstroRun::builder().runner(client_runner).build();

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
      .await
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Failed);

    assert_eq!(
      res.jobs.get("test").unwrap().steps[0].exit_code.unwrap(),
      123
    );

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let server_thread_handle = tokio::spawn(async {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let runner_server = AstroRunRemoteRunnerServer::builder()
      .id("test-runner")
      .runner(TimeoutRunner {
        delay: Duration::from_secs(5),
      })
      .max_runs(5)
      .plugin(
        PluginBuilder::new("test-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
            Ok(())
          })
          .build(),
      )
      .build()
      .unwrap();

    oneshot_tx.send(()).unwrap();

    tokio::select! {
      _ = rx.recv() => {}
      _ = runner_server.serve("127.0.0.1:5338") => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

#[astro_run_test::test]
async fn test_cancel() -> Result<()> {
  let (oneshot_tx, rx) = tokio::sync::oneshot::channel();
  let client_thread_handle = tokio::spawn(async {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let client_runner = AstroRunRemoteRunnerClient::builder().build().unwrap();

    let mut cloned_client_runner = client_runner.clone();
    let handle = tokio::task::spawn(async move {
      rx.await.unwrap();

      cloned_client_runner
        .start(vec!["http://127.0.0.1:5338"])
        .await
        .unwrap();
    });

    // Wait for server to start and listen for connections
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let astro_run = AstroRun::builder().runner(client_runner).build();

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
      .await
      .unwrap();

    let ctx = astro_run.execution_context().build();

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::task::spawn({
      let astro_run = astro_run.clone();
      let job_id = workflow.jobs.get("test").unwrap().id.clone();
      async move {
        tx.send(()).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        astro_run.cancel_job(&job_id).unwrap();
      }
    });

    // Wait for task to start
    rx.await.unwrap();

    let res = workflow.run(ctx).await;

    assert_eq!(res.state, WorkflowState::Cancelled);

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    handle.abort();
  });

  let server_thread_handle = tokio::spawn(async {
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let runner_server = AstroRunRemoteRunnerServer::builder()
      .id("test-runner")
      .runner(TimeoutRunner {
        delay: Duration::from_secs(60),
      })
      .max_runs(5)
      .plugin(
        PluginBuilder::new("test-plugin")
          .on_workflow_completed(move |_| {
            tx.try_send(()).unwrap();
            Ok(())
          })
          .build(),
      )
      .build()
      .unwrap();

    oneshot_tx.send(()).unwrap();

    tokio::select! {
      _ = rx.recv() => {}
      _ = runner_server.serve("127.0.0.1:5338") => {}
    }
  });

  tokio::try_join!(server_thread_handle, client_thread_handle).unwrap();

  Ok(())
}

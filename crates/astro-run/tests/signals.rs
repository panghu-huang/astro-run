use astro_run::{stream, AstroRun, Context, Error, RunResult, Signal, Workflow, WorkflowState};
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
          match signal {
            Signal::Timeout => {
              sender.failed(123);
            }
            Signal::Cancel => {
              sender.cancelled();
            }
          }
        }
      }
    });

    Ok(receiver)
  }
}

#[astro_run_test::test]
async fn test_signal() {
  let workflow = r#"
jobs:
  test:
    steps:
      - run: Hello World
        timeout: 2s
  "#;

  let astro_run = AstroRun::builder()
    .runner(TimeoutRunner {
      delay: Duration::from_secs(1),
    })
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_timeout() {
  let workflow = r#"
jobs:
  test:
    steps:
      - run: Hello World
        timeout: 1s
  "#;

  let astro_run = AstroRun::builder()
    .runner(TimeoutRunner {
      delay: Duration::from_secs(2),
    })
    .build();

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
}

#[astro_run_test::test]
async fn test_cancel() {
  let workflow = r#"
jobs:
  test:
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TimeoutRunner {
      delay: Duration::from_secs(60),
    })
    .build();

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
      tokio::time::sleep(Duration::from_secs(1)).await;
      astro_run.cancel_job(&job_id).unwrap();

      // Cancel again
      let err = astro_run.cancel_job(&job_id).unwrap_err();

      assert_eq!(Error::error("Signal has been cancelled or timeout."), err);
    }
  });

  // Wait for task to start
  rx.await.unwrap();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
}

#[astro_run_test::test]
async fn test_ctx_cancel() {
  let workflow = r#"
jobs:
  test:
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TimeoutRunner {
      delay: Duration::from_secs(60),
    })
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let (tx, rx) = tokio::sync::oneshot::channel();

  tokio::task::spawn({
    let ctx = ctx.clone();
    let job_id = workflow.jobs.get("test").unwrap().id.clone();
    async move {
      tx.send(()).unwrap();
      tokio::time::sleep(Duration::from_secs(1)).await;
      ctx.cancel_job(&job_id).unwrap();
    }
  });

  // Wait for task to start
  rx.await.unwrap();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
}

#[astro_run_test::test]
async fn test_cancel_not_started() {
  let workflow = r#"
jobs:
  test:
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TimeoutRunner {
      delay: Duration::from_secs(60),
    })
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let job_id = &workflow.jobs.get("test").unwrap().id;

  let err = ctx.cancel_job(job_id).unwrap_err();

  assert_eq!(
    Error::error(format!("Job {} not found", job_id.to_string())),
    err
  );
}

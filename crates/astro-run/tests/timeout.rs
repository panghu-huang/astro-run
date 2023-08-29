use astro_run::{stream, AstroRun, Context, RunResult, Workflow, WorkflowState};
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
          sender.end(RunResult::Succeeded);
        }
        _ = config.signal.recv() => {
          sender.end(RunResult::Failed { exit_code: 1 });
        }
      }
    });

    Ok(receiver)
  }
}

#[astro_run_test::test]
async fn test_timeout_success() {
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
    .unwrap();

  let ctx = astro_run.execution_context();

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
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
}

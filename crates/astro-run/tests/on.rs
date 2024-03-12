use astro_run::{
  stream, AstroRun, AstroRunPlugin, Context, PluginBuilder, RunResult, Runner, Workflow,
  WorkflowEvent, WorkflowState,
};
use parking_lot::Mutex;

struct TestRunner;

impl TestRunner {
  fn new() -> Self {
    TestRunner
  }
}

#[astro_run::async_trait]
impl Runner for TestRunner {
  async fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    tx.log(ctx.command.run);
    tx.end(RunResult::Succeeded);

    Ok(rx)
  }
}

fn assert_logs_plugin(excepted_logs: Vec<&'static str>) -> AstroRunPlugin {
  let index = Mutex::new(0);

  PluginBuilder::new("test-plugin")
    .on_log(move |log| {
      let mut i = index.lock();
      println!("{}: {}", *i, log.message);
      assert_eq!(log.message, excepted_logs[*i]);
      *i += 1;
    })
    .build()
}

fn get_push_event() -> WorkflowEvent {
  WorkflowEvent {
    // https://github.com/panghu-huang/astro-run/commit/2cef9002f2d23840c0820b9df2e549ab767e71ef
    // Changed files:
    //  - .cargo/config.toml
    //  - .github/workflows/test.yml
    //  - Cargo.toml
    //  - crates/astro-run-remote-runner/tests/protocol.rs
    //  - crates/astro-run-remote-runner/tests/test.rs
    //  - crates/astro-run-server/tests/protocol.rs
    //  - crates/astro-run-server/tests/test.rs
    //  - crates/astro-runner/src/lib.rs
    sha: "2cef9002f2d23840c0820b9df2e549ab767e71ef".to_string(),
    ..Default::default()
  }
}

fn get_pull_request_event() -> WorkflowEvent {
  WorkflowEvent {
    // https://github.com/panghu-huang/astro-run/pull/1
    // Changed files:
    //  - .cargo/config.toml
    //  - .github/workflows/test.yml
    //  - Cargo.toml
    //  - crates/astro-run-remote-runner/tests/protocol.rs
    //  - crates/astro-run-remote-runner/tests/test.rs
    //  - crates/astro-run-server/tests/protocol.rs
    //  - crates/astro-run-server/tests/test.rs
    //  - crates/astro-runner/src/lib.rs
    sha: "783d54071d9fc6c3141d2c11ccdc4a13ef58fcb2".to_string(),
    pr_number: Some(1),
    event: "pull_request".to_string(),
    ..Default::default()
  }
}

#[astro_run_test::test]
async fn test_workflow_skipped() {
  dotenv::dotenv().ok();

  let workflow = r#"
on:
  push:
    branches:
      - master
jobs:
  test:
    steps:
      - run: Hello World
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    // Use a empty token to request public repository
    .github_personal_token(std::env::var("PERSONAL_ACCESS_TOKEN").unwrap())
    .plugin(assert_logs_plugin(vec![]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(get_push_event())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Skipped);
  assert_eq!(res.jobs.len(), 0);
}

#[astro_run_test::test]
async fn test_push_event() {
  dotenv::dotenv().ok();

  let workflow = r#"
on:
  push:
    branches:
      - main
jobs:
  test:
    steps:
      - run: Skipped
        on:
          push:
            branches:
              - not-main
          pull_request:
            paths:
              - crates/astro-run-server/**/*.rs
      - run: Hello World
      - run: Skipped
        on:
          pull_request:
            paths:
              - crates/astro-run-server/**/*.rs
  skip:
    on:
      push:
        paths:
          - crates/astro-run/**/*.rs
    steps:
      - run: Skipped
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    // Use a empty token to request public repository
    .github_personal_token(std::env::var("PERSONAL_ACCESS_TOKEN").unwrap())
    .plugin(assert_logs_plugin(vec!["Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(get_push_event())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  assert_eq!(res.jobs.len(), 2);

  let job = res.jobs.get("test").unwrap();
  assert_eq!(job.state, WorkflowState::Succeeded);

  assert_eq!(job.steps.len(), 3);

  let step = job.steps.get(0).unwrap();
  assert_eq!(step.state, WorkflowState::Skipped);

  let step = job.steps.get(1).unwrap();
  assert_eq!(step.state, WorkflowState::Succeeded);

  let step = job.steps.get(2).unwrap();
  assert_eq!(step.state, WorkflowState::Skipped);

  let job = res.jobs.get("skip").unwrap();
  assert_eq!(job.state, WorkflowState::Skipped);
  assert_eq!(job.steps.len(), 0);
}

#[astro_run_test::test]
async fn test_pull_request_event() {
  dotenv::dotenv().ok();

  let workflow = r#"
on:
  pull_request:
    branches:
      - main
jobs:
  test:
    steps:
      - run: Skipped
        on:
          push:
            branches:
              - main
          pull_request:
            paths:
              - crates/astro-run/**/*.rs
      - run: Hello World
      - run: Skipped
        on:
          push:
            branches:
              - main
  skip:
    on:
      push:
        paths:
          - crates/astro-run-server/**/*.rs
    steps:
      - run: Skipped
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    // Use a empty token to request public repository
    .github_personal_token(std::env::var("PERSONAL_ACCESS_TOKEN").unwrap())
    .plugin(assert_logs_plugin(vec!["Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(get_pull_request_event())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  assert_eq!(res.jobs.len(), 2);

  let job = res.jobs.get("test").unwrap();
  assert_eq!(job.state, WorkflowState::Succeeded);

  assert_eq!(job.steps.len(), 3);

  let step = job.steps.get(0).unwrap();
  assert_eq!(step.state, WorkflowState::Skipped);

  let step = job.steps.get(1).unwrap();
  assert_eq!(step.state, WorkflowState::Succeeded);

  let step = job.steps.get(2).unwrap();
  assert_eq!(step.state, WorkflowState::Skipped);

  let job = res.jobs.get("skip").unwrap();
  assert_eq!(job.state, WorkflowState::Skipped);
  assert_eq!(job.steps.len(), 0);
}

#[astro_run_test::test]
async fn test_only_pull_request_event() {
  dotenv::dotenv().ok();

  let workflow = r#"
on:
  pull_request:
    branches:
      - main

jobs:
  test:
    steps:
      - run: Skipped
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .github_personal_token(std::env::var("PERSONAL_ACCESS_TOKEN").unwrap())
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(get_push_event())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Skipped);
}

#[astro_run_test::test]
async fn test_only_push_event() {
  dotenv::dotenv().ok();

  let workflow = r#"
on:
  push:
    branches:
      - main

jobs:
  test:
    steps:
      - run: Skipped
  "#;

  let astro_run = AstroRun::builder()
    .runner(TestRunner::new())
    .github_personal_token(std::env::var("PERSONAL_ACCESS_TOKEN").unwrap())
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(get_pull_request_event())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Skipped);
}

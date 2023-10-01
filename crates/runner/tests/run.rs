use astro_run::{AstroRun, AstroRunPlugin, Workflow, WorkflowState};
use astro_runner::{AstroRunner, Command};
use parking_lot::Mutex;
use std::sync::Arc;

fn assert_logs_plugin(excepted_logs: Vec<String>) -> AstroRunPlugin {
  let logs = Arc::new(Mutex::new(vec![]));

  let cloned_logs = logs.clone();
  AstroRunPlugin::builder("test-plugin")
    .on_log(move |log| {
      logs.lock().push(log.message);
    })
    .on_workflow_completed(move |_| {
      let logs = cloned_logs.lock();
      log::debug!("Logs: {:?}", logs);
      assert_eq!(logs.len(), excepted_logs.len());
      for (i, log) in logs.iter().enumerate() {
        assert_eq!(log, &excepted_logs[i]);
      }
    })
    .build()
}

#[astro_run_test::test(docker)]
async fn test_docker() {
  // Pull the image before running the test
  log::debug!("Pulling ubuntu image");
  Command::new("docker pull ubuntu").exec().await.unwrap();

  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - container:
          name: ubuntu
          security-opts: 
            - seccomp=unconfined
        continue-on-error: false
        run: echo "Hello World" >> test.txt
        timeout: 60m
      - run: |
          content=$(cat test.txt)
          echo Content is $content
  "#;
  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec![
      "Content is Hello World".to_string()
    ]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 2);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps[1].state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_host() {
  let workflow = format!(
    r#"
  jobs:
    test:
      name: Test Job
      steps:
        - container: host/{}
          run: echo "Hello world"
          environments:
            TEST: Value

    "#,
    std::env::consts::OS
  );
  #[allow(deprecated)]
  let working_dir = std::env::home_dir()
    .map(|home| home.join("astro-run"))
    .unwrap();
  let runner = AstroRunner::builder()
    .working_directory(working_dir)
    .build()
    .unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec!["Hello world".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_before_run() {
  struct TestPlugin;

  impl astro_runner::Plugin for TestPlugin {
    fn name(&self) -> &'static str {
      "test-plugin"
    }

    fn on_before_run(
      &self,
      mut ctx: astro_run::Context,
    ) -> Result<astro_run::Context, Box<dyn std::error::Error>> {
      ctx.command.run = "echo \"My custom run\"".to_string();

      Ok(ctx)
    }

    fn on_after_run(&self, ctx: astro_run::Context) {
      assert_eq!(ctx.command.run, "echo \"My custom run\"");
    }
  }

  let workflow = format!(
    r#"
  jobs:
    test:
      name: Test Job
      steps:
        - container: host/{}
          run: echo "Hello world"
          environments:
            TEST: Value

    "#,
    std::env::consts::OS
  );

  #[allow(deprecated)]
  let working_dir = std::env::home_dir()
    .map(|home| home.join("astro-run"))
    .unwrap();

  let runner = AstroRunner::builder()
    .working_directory(working_dir)
    .plugin(TestPlugin)
    .build()
    .unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec!["My custom run".to_string()]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
}

#[astro_run_test::test]
async fn test_before_run_error() {
  struct TestPlugin;

  impl astro_runner::Plugin for TestPlugin {
    fn name(&self) -> &'static str {
      "test-plugin"
    }

    fn on_before_run(
      &self,
      _ctx: astro_run::Context,
    ) -> Result<astro_run::Context, Box<dyn std::error::Error>> {
      Err("Error".into())
    }
  }

  let workflow = format!(
    r#"
  jobs:
    test:
      name: Test Job
      steps:
        - container: host/{}
          run: echo "Hello world"
          environments:
            TEST: Value

    "#,
    std::env::consts::OS
  );

  #[allow(deprecated)]
  let working_dir = std::env::home_dir()
    .map(|home| home.join("astro-run"))
    .unwrap();

  let runner = AstroRunner::builder()
    .working_directory(working_dir)
    .plugin(TestPlugin)
    .build()
    .unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
}

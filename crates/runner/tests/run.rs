use astro_run::{AstroRun, AstroRunPlugin, Workflow, WorkflowState};
use astro_runner::{AstroRunner, Command};
use parking_lot::Mutex;
use std::{fs, io::Write, sync::Arc};

fn assert_logs_plugin(excepted_logs: Vec<&'static str>) -> AstroRunPlugin {
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
  Command::new("docker pull ubuntu").exec().await.unwrap();

  fs::create_dir_all("/tmp/astro-run").unwrap();
  let mut file = fs::File::create("/tmp/astro-run/test.txt").unwrap();

  file.write_all(b"Hello World").unwrap();
  file.flush().unwrap();

  drop(file);

  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - container:
          name: ubuntu
          security-opts: 
            - seccomp=unconfined
          volumes:
            - /tmp/astro-run/test.txt:/tmp/test.txt
        continue-on-error: false
        environments:
          TEST: Value
        run: |
          cat /tmp/test.txt
          echo "Hello World $TEST" >> test.txt
        timeout: 60m
      - run: |
          content=$(cat test.txt)
          echo Content is $content
  "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec![
      "Hello World",
      "Content is Hello World Value",
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

  fs::remove_dir_all("/tmp/astro-run").unwrap();
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
          run: echo "Hello world $TEST"
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
    .plugin(assert_logs_plugin(vec!["Hello world Value"]))
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

  let runner = AstroRunner::builder().plugin(TestPlugin).build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec!["My custom run"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::WorkflowEvent::default())
    .build();

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

  let runner = AstroRunner::builder().build().unwrap();

  runner.register_plugin(TestPlugin);

  let astro_run = AstroRun::builder().runner(runner.clone()).build();

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

  runner.unregister_plugin("test-plugin");
}

#[astro_run_test::test]
async fn test_docker_cancel() {
  let workflow = r#"
  jobs:
    test:
      steps:
        - run: |
            sleep 10s
            echo "Hello world"
    "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec![]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::WorkflowEvent::default())
    .build();

  tokio::task::spawn({
    let astro_run = astro_run.clone();
    let job_id = workflow.jobs.get("test").unwrap().id.clone();
    async move {
      tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
      astro_run.cancel(&job_id).unwrap();
    }
  });

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Cancelled);
}

#[astro_run_test::test]
async fn test_docker_timeout() {
  let workflow = r#"
  jobs:
    test:
      steps:
        - run: |
            sleep 10s
            echo "Hello world"
          timeout: 2s
    "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec![]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::WorkflowEvent::default())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[0].exit_code.unwrap(), 123);
}

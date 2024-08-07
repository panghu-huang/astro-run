use astro_run::{AstroRun, AstroRunPlugin, HookNoopResult, Workflow, WorkflowState};
use astro_runner::{AstroRunner, Command};
use parking_lot::Mutex;
use std::{fs, io::Write, sync::Arc};

fn assert_logs_plugin(excepted_logs: Vec<&'static str>) -> AstroRunPlugin {
  let logs = Arc::new(Mutex::new(vec![]));

  let cloned_logs = logs.clone();
  AstroRunPlugin::builder("test-plugin")
    .on_log(move |log| {
      logs.lock().push(log.message);
      Ok(())
    })
    .on_workflow_completed(move |_| {
      let logs = cloned_logs.lock();
      log::debug!("Logs: {:?}", logs);
      assert_eq!(logs.len(), excepted_logs.len());
      for (i, log) in logs.iter().enumerate() {
        assert_eq!(log, &excepted_logs[i]);
      }
      Ok(())
    })
    .build()
}

#[astro_run_test::test(docker)]
async fn test_docker() {
  // Pull the image before running the test
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
        environments:
          TEST: Value
        run: |
          echo "Hello World $TEST" >> test.txt
        timeout: 60m
      - run: |
          content=$(cat test.txt)
          echo Content is $content
  "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec!["Content is Hello World Value"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
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

#[astro_run_test::test(docker)]
async fn test_docker_volume() {
  // Pull the image before running the test
  Command::new("docker pull ubuntu:22.04")
    .exec()
    .await
    .unwrap();

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
          name: ubuntu:22.04
          volumes:
            - /tmp/astro-run/test.txt:/home/runner/work/test.txt
        run: |
          ls /home/runner/work
          content=$(cat /home/runner/work/test.txt)
          echo $content
  "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec!["test.txt", "Hello World"]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run.execution_context().build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);

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
    .await
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

  #[astro_run::async_trait]
  impl astro_runner::Plugin for TestPlugin {
    fn name(&self) -> &'static str {
      "test-plugin"
    }

    async fn on_before_run(
      &self,
      mut ctx: astro_run::Context,
    ) -> astro_run::Result<astro_run::Context> {
      ctx.command.run = "echo \"My custom run\"".to_string();

      Ok(ctx)
    }

    async fn on_after_run(&self, ctx: astro_run::Context) -> HookNoopResult {
      assert_eq!(ctx.command.run, "echo \"My custom run\"");

      Ok(())
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
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::TriggerEvent::default())
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

  #[astro_run::async_trait]
  impl astro_runner::Plugin for TestPlugin {
    fn name(&self) -> &'static str {
      "test-plugin"
    }

    async fn on_before_run(
      &self,
      _ctx: astro_run::Context,
    ) -> astro_run::Result<astro_run::Context> {
      Err(astro_run::Error::error("Error"))
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

  let astro_run = AstroRun::builder().runner(runner.clone()).build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
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
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::TriggerEvent::default())
    .build();

  tokio::task::spawn({
    let astro_run = astro_run.clone();
    let job_id = workflow.jobs.get("test").unwrap().id.clone();
    async move {
      tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
      astro_run.cancel_job(&job_id).unwrap();
    }
  });

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Cancelled);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[astro_run_test::test]
async fn test_host_cancel() {
  let sleep = if cfg!(target_os = "linux") {
    "sleep 30s"
  } else {
    "sleep 30"
  };

  let workflow = format!(
    r#"
  jobs:
    test:
      name: Test Job
      steps:
        - container: host/{}
          run: |
            {}
            echo "Hello world"
    "#,
    std::env::consts::OS,
    sleep
  );

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec![]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::TriggerEvent::default())
    .build();

  tokio::task::spawn({
    let astro_run = astro_run.clone();
    let job_id = workflow.jobs.get("test").unwrap().id.clone();
    async move {
      tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
      astro_run.cancel_job(&job_id).unwrap();
    }
  });

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Cancelled);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Cancelled);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Cancelled);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[astro_run_test::test]
async fn test_host_timeout() {
  let sleep = if cfg!(target_os = "linux") {
    "sleep 30s"
  } else {
    "sleep 30"
  };

  let workflow = format!(
    r#"
  jobs:
    test:
      name: Test Job
      steps:
        - container: host/{}
          timeout: 3s
          run: |
            {}
            echo "Hello world"
    "#,
    std::env::consts::OS,
    sleep
  );

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec![]))
    .build();

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::TriggerEvent::default())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[0].exit_code.unwrap(), 123);
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
    .await
    .unwrap();

  let ctx = astro_run
    .execution_context()
    .event(astro_run::TriggerEvent::default())
    .build();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Failed);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Failed);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Failed);
  assert_eq!(job_result.steps[0].exit_code.unwrap(), 123);
}

#[astro_run_test::test]
async fn test_node_entrypoint() {
  // Pull the image before running the test
  Command::new("docker pull node:alpine")
    .exec()
    .await
    .unwrap();

  let workflow = r#"
jobs:
  test:
    steps:
      - container: node:alpine
        environments:
          NAME: Node
        run: |
          #!/usr/local/bin/node

          console.log(`Hello ${process.env.NAME}`);
  "#;

  let runner = AstroRunner::builder().build().unwrap();

  let astro_run = AstroRun::builder()
    .runner(runner)
    .plugin(assert_logs_plugin(vec!["Hello Node"]))
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

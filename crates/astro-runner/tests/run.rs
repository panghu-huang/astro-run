use astro_run::{AstroRun, AstroRunPlugin, Workflow, WorkflowEvent, WorkflowState};
use astro_runner::AstroRunner;
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
  let workflow = r#"
jobs:
  test:
    name: Test Job
    steps:
      - timeout: 60m
        continue-on-error: false
        run: echo "Hello World" >> test.txt
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
    .event(WorkflowEvent::default())
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context();

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
    .event(WorkflowEvent::default())
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context();

  let res = workflow.run(ctx).await;

  assert_eq!(res.state, WorkflowState::Succeeded);
  let job_result = res.jobs.get("test").unwrap();
  assert_eq!(job_result.state, WorkflowState::Succeeded);
  assert_eq!(job_result.steps.len(), 1);

  assert_eq!(job_result.steps[0].state, WorkflowState::Succeeded);
}

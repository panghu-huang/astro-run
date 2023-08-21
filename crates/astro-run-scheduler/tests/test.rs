use astro_run::{stream, AstroRun, Context, RunResult, Runner, Workflow};
use astro_run_scheduler::*;

struct RunnerController<T>
where
  T: Scheduler,
{
  scheduler: T,
  runners: Vec<RunnerMetadata>,
  expected_runners: Vec<Option<&'static str>>,
}

impl<T> Runner for RunnerController<T>
where
  T: Scheduler,
{
  fn run(&self, ctx: Context) -> astro_run::RunResponse {
    let (tx, rx) = stream();

    let runner = self.scheduler.schedule(&self.runners, &ctx);
    let index = ctx.command.id.step_number();
    let expected_runner = self.expected_runners[index];

    if let Some(runner) = runner {
      assert_eq!(runner.id, expected_runner.unwrap());
      tx.end(RunResult::Succeeded);
    } else {
      assert!(expected_runner.is_none());
      tx.end(RunResult::Failed { exit_code: 1 });
    }

    Ok(rx)
  }

  fn on_job_completed(&self, result: astro_run::JobRunResult) {
    let steps = result.steps.len();
    assert_eq!(steps, self.expected_runners.len());
    self.scheduler.on_job_completed(result);
  }

  fn on_step_completed(&self, result: astro_run::StepRunResult) {
    self.scheduler.on_step_completed(result);
  }
}

fn assert_default_scheduler_state(scheduler: &DefaultScheduler) {
  let state = scheduler.state.lock().clone();

  assert_eq!(state.runs_count.len(), 0);
  assert_eq!(state.step_runners.len(), 0);
  assert_eq!(state.job_runners.len(), 0);
}

#[tokio::test]
async fn test_default_schedule() {
  let scheduler = DefaultScheduler::new();

  let runners = vec![
    RunnerMetadata {
      id: "linux-runner".to_string(),
      os: "linux".to_string(),
      arch: "x64".to_string(),
      support_docker: true,
      support_host: false,
      ..Default::default()
    },
    RunnerMetadata {
      id: "windows-runner".to_string(),
      os: "windows".to_string(),
      arch: "x64".to_string(),
      support_docker: true,
      support_host: true,
      ..Default::default()
    },
  ];

  let controller = RunnerController {
    scheduler: scheduler.clone(),
    runners,
    expected_runners: vec![
      Some("linux-runner"),
      Some("windows-runner"),
      Some("linux-runner"),
    ],
  };

  let astro_run = AstroRun::builder().runner(controller).build();

  let workflow = r#"
  jobs:
    test:
      name: Test Job
      steps:
        - run: Hello World
        - container: host/windows
          run: Hello World
        - run: Hello World
    "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context();

  workflow.run(ctx).await;

  assert_default_scheduler_state(&scheduler);
}

#[tokio::test]
async fn test_default_schedule1() {
  let scheduler = DefaultScheduler::new();

  let runners = vec![
    RunnerMetadata {
      id: "linux-runner".to_string(),
      os: "linux".to_string(),
      arch: "x64".to_string(),
      support_docker: false,
      support_host: true,
      ..Default::default()
    },
    RunnerMetadata {
      id: "windows-runner".to_string(),
      os: "windows".to_string(),
      arch: "x64".to_string(),
      support_docker: true,
      support_host: true,
      ..Default::default()
    },
  ];

  let controller = RunnerController {
    scheduler: scheduler.clone(),
    runners,
    expected_runners: vec![
      Some("windows-runner"),
      Some("windows-runner"),
      Some("linux-runner"),
    ],
  };

  let astro_run = AstroRun::builder().runner(controller).build();

  let workflow = r#"
  jobs:
    test:
      name: Test Job
      steps:
        - run: Hello World
        - container: host/windows
          run: Hello World
        - container: host/linux-x64
          run: Hello World
    "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context();

  workflow.run(ctx).await;

  assert_default_scheduler_state(&scheduler);
}

#[tokio::test]
async fn test_default_schedule_none() {
  let scheduler = DefaultScheduler::new();

  let runners = vec![
    RunnerMetadata {
      id: "linux-runner".to_string(),
      os: "linux".to_string(),
      arch: "x64".to_string(),
      support_docker: false,
      support_host: true,
      ..Default::default()
    },
    RunnerMetadata {
      id: "windows-runner".to_string(),
      os: "windows".to_string(),
      arch: "x64".to_string(),
      support_docker: false,
      support_host: true,
      ..Default::default()
    },
  ];

  let controller = RunnerController {
    scheduler: scheduler.clone(),
    runners,
    expected_runners: vec![None, None, Some("linux-runner")],
  };

  let astro_run = AstroRun::builder().runner(controller).build();

  let workflow = r#"
  jobs:
    test:
      name: Test Job
      steps:
        - run: Hello World
        - container: host/windows-x86
          run: Hello World
        - container: host/linux
          run: Hello World
    "#;

  let workflow = Workflow::builder()
    .config(workflow)
    .build(&astro_run)
    .unwrap();

  let ctx = astro_run.execution_context();

  workflow.run(ctx).await;

  assert_default_scheduler_state(&scheduler);
}

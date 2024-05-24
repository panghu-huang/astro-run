use crate::{RunnerMetadata, Scheduler};
use astro_run::{Context, JobId, JobRunResult, StepId, StepRunResult};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct SchedulerState {
  /// Runner ID -> Job runs count
  pub runs_count: HashMap<String, i32>,
  /// Job ID -> Runner ID
  pub job_runners: HashMap<JobId, String>,
  /// Step ID -> Runner ID
  pub step_runners: HashMap<StepId, String>,
}

#[derive(Clone)]
pub struct DefaultScheduler {
  pub state: Arc<Mutex<SchedulerState>>,
}

impl DefaultScheduler {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for DefaultScheduler {
  fn default() -> Self {
    Self {
      state: Arc::new(Mutex::new(SchedulerState {
        runs_count: HashMap::new(),
        job_runners: HashMap::new(),
        step_runners: HashMap::new(),
      })),
    }
  }
}

#[astro_run::async_trait]
impl Scheduler for DefaultScheduler {
  async fn schedule<'a, 'b: 'a>(
    &'b self,
    runners: &'a [RunnerMetadata],
    ctx: &Context,
  ) -> Option<&'a RunnerMetadata> {
    log::trace!("Scheduling runners: {:?}", runners);
    let mut runner: Option<&'a RunnerMetadata> = None;

    let job_id = ctx.command.id.job_id();
    let container_name = ctx.command.container.clone().map(|c| c.name);
    let is_runs_on_host = container_name
      .clone()
      .map(|c| c.starts_with("host/"))
      .unwrap_or(false);

    log::trace!("Is runs on host: {}", is_runs_on_host);

    let last_used_id = self.state.lock().job_runners.get(&job_id).cloned();

    if let Some(last_used_id) = last_used_id {
      runner = runners.iter().find(|r| {
        if r.id == last_used_id {
          if is_runs_on_host {
            let container_name = container_name.clone().unwrap();
            return r.support_host && container_name == format!("host/{}", r.os)
              || container_name == format!("host/{}-{}", r.os, r.arch);
          }

          return true;
        }

        false
      });

      log::trace!("Last used runner: {:?}", runner);
    }

    if runner.is_none() {
      runner = self.pick_runner(runners, container_name);
      log::trace!("Picked runner: {:?}", runner);
    }

    if let Some(runner) = &runner {
      let mut state = self.state.lock();
      state
        .step_runners
        .insert(ctx.command.id.clone(), runner.id.clone());
      // Update runs count
      let runs_count = state.runs_count.entry(runner.id.clone()).or_insert(0);
      *runs_count += 1;

      if !is_runs_on_host {
        // Update job runner
        state.job_runners.insert(job_id, runner.id.clone());
      }

      log::trace!("Runs count: {:?}", state.runs_count);
    }

    runner
  }

  fn on_step_completed(&self, result: StepRunResult) {
    let mut state = self.state.lock();
    let step_id = result.id;
    let runner_id = state.step_runners.get(&step_id).cloned();

    if let Some(runner_id) = runner_id {
      let runs_count = state
        .runs_count
        .entry(runner_id.clone())
        .and_modify(|c| *c -= 1)
        .or_insert(0);

      if *runs_count <= 0 {
        state.runs_count.remove(&runner_id);
      }
    }

    state.step_runners.remove(&step_id);
  }

  fn on_job_completed(&self, result: JobRunResult) {
    let mut state = self.state.lock();
    let job_id = result.id;

    state.job_runners.remove(&job_id);
  }
}

impl DefaultScheduler {
  fn pick_runner<'a, 'b: 'a>(
    &'b self,
    runners: &'a [RunnerMetadata],
    container: Option<String>,
  ) -> Option<&'a RunnerMetadata> {
    let is_runs_on_host = container
      .clone()
      .map(|c| c.starts_with("host/"))
      .unwrap_or(false);

    if is_runs_on_host {
      self.pick_host_runner(runners, container.unwrap())
    } else {
      self.pick_docker_runner(runners)
    }
  }

  fn pick_docker_runner<'a, 'b: 'a>(
    &'b self,
    runners: &'a [RunnerMetadata],
  ) -> Option<&'a RunnerMetadata> {
    let runs_count = self.state.lock().runs_count.clone();
    let min_runs = runners
      .iter()
      .filter(|r| r.support_docker)
      .min_by_key(|r| runs_count.get(&r.id).unwrap_or(&0));

    min_runs
  }

  fn pick_host_runner<'a, 'b: 'a>(
    &'b self,
    runners: &'a [RunnerMetadata],
    container: String,
  ) -> Option<&'a RunnerMetadata> {
    runners.iter().filter(|r| r.support_host).find(|r| {
      container == format!("host/{}", r.os) || container == format!("host/{}-{}", r.os, r.arch)
    })
  }
}

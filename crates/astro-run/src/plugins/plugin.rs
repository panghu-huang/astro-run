use crate::{
  HookBeforeRunStepResult, HookNoopResult, HookResolveActionResult, JobRunResult, Plugin,
  RunJobEvent, RunStepEvent, RunWorkflowEvent, Step, StepRunResult, UserActionStep, WorkflowLog,
  WorkflowRunResult, WorkflowStateEvent,
};

type OnStateChange = dyn Fn(WorkflowStateEvent) -> HookNoopResult + Send + Sync;
type OnLog = dyn Fn(WorkflowLog) -> HookNoopResult + Send + Sync;
type OnRunWorkflow = dyn Fn(RunWorkflowEvent) -> HookNoopResult + Send + Sync;
type OnRunJob = dyn Fn(RunJobEvent) -> HookNoopResult + Send + Sync;
type OnRunStep = dyn Fn(RunStepEvent) -> HookNoopResult + Send + Sync;
type OnWorkflowComplete = dyn Fn(WorkflowRunResult) -> HookNoopResult + Send + Sync;
type OnJobComplete = dyn Fn(JobRunResult) -> HookNoopResult + Send + Sync;
type OnStepComplete = dyn Fn(StepRunResult) -> HookNoopResult + Send + Sync;
type OnResolveDynamicAction = dyn Fn(UserActionStep) -> HookResolveActionResult + Send + Sync;
type OnBeforeRunStep = dyn Fn(Step) -> HookBeforeRunStepResult + Send + Sync;

pub struct PluginBuilder {
  name: &'static str,
  on_resolve_dynamic_action: Option<Box<OnResolveDynamicAction>>,
  on_run_workflow: Option<Box<OnRunWorkflow>>,
  on_run_job: Option<Box<OnRunJob>>,
  on_before_run_step: Option<Box<OnBeforeRunStep>>,
  on_run_step: Option<Box<OnRunStep>>,
  on_state_change: Option<Box<OnStateChange>>,
  on_log: Option<Box<OnLog>>,
  on_step_completed: Option<Box<OnStepComplete>>,
  on_job_completed: Option<Box<OnJobComplete>>,
  on_workflow_completed: Option<Box<OnWorkflowComplete>>,
}

impl PluginBuilder {
  fn new(name: &'static str) -> Self {
    PluginBuilder {
      name,
      on_resolve_dynamic_action: None,
      on_state_change: None,
      on_log: None,
      on_run_workflow: None,
      on_run_job: None,
      on_before_run_step: None,
      on_run_step: None,
      on_step_completed: None,
      on_job_completed: None,
      on_workflow_completed: None,
    }
  }

  pub fn on_state_change<T>(mut self, on_state_change: T) -> Self
  where
    T: Fn(WorkflowStateEvent) -> HookNoopResult + 'static + Send + Sync,
  {
    self.on_state_change = Some(Box::new(on_state_change));
    self
  }

  pub fn on_log<T>(mut self, on_log: T) -> Self
  where
    T: Fn(WorkflowLog) -> HookNoopResult + 'static + Send + Sync,
  {
    self.on_log = Some(Box::new(on_log));
    self
  }

  pub fn on_run_workflow<T>(mut self, on_run_workflow: T) -> Self
  where
    T: Fn(RunWorkflowEvent) -> HookNoopResult + 'static + Send + Sync,
  {
    self.on_run_workflow = Some(Box::new(on_run_workflow));
    self
  }

  pub fn on_run_job<T>(mut self, on_run_job: T) -> Self
  where
    T: Fn(RunJobEvent) -> HookNoopResult + 'static + Send + Sync,
  {
    self.on_run_job = Some(Box::new(on_run_job));
    self
  }

  pub fn on_run_step<T>(mut self, on_run_step: T) -> Self
  where
    T: Fn(RunStepEvent) -> HookNoopResult + 'static + Send + Sync,
  {
    self.on_run_step = Some(Box::new(on_run_step));
    self
  }

  pub fn on_workflow_completed<T>(mut self, on_workflow_completed: T) -> Self
  where
    T: Fn(WorkflowRunResult) -> HookNoopResult + Send + Sync + 'static,
  {
    self.on_workflow_completed = Some(Box::new(on_workflow_completed));

    self
  }

  pub fn on_job_completed<T>(mut self, on_job_completed: T) -> Self
  where
    T: Fn(JobRunResult) -> HookNoopResult + Send + Sync + 'static,
  {
    self.on_job_completed = Some(Box::new(on_job_completed));

    self
  }

  pub fn on_step_completed<T>(mut self, on_step_completed: T) -> Self
  where
    T: Fn(StepRunResult) -> HookNoopResult + Send + Sync + 'static,
  {
    self.on_step_completed = Some(Box::new(on_step_completed));

    self
  }

  pub fn on_resolve_dynamic_action<T>(mut self, on_resolve_dynamic_action: T) -> Self
  where
    T: Fn(UserActionStep) -> HookResolveActionResult + Send + Sync + 'static,
  {
    self.on_resolve_dynamic_action = Some(Box::new(on_resolve_dynamic_action));

    self
  }

  pub fn on_before_run_step<T>(mut self, on_before_run_step: T) -> Self
  where
    T: Fn(Step) -> HookBeforeRunStepResult + Send + Sync + 'static,
  {
    self.on_before_run_step = Some(Box::new(on_before_run_step));

    self
  }

  pub fn build(self) -> AstroRunPlugin {
    AstroRunPlugin {
      name: self.name,
      on_state_change: self.on_state_change,
      on_log: self.on_log,
      on_run_workflow: self.on_run_workflow,
      on_run_job: self.on_run_job,
      on_run_step: self.on_run_step,
      on_workflow_completed: self.on_workflow_completed,
      on_job_completed: self.on_job_completed,
      on_step_completed: self.on_step_completed,
      on_resolve_dynamic_action: self.on_resolve_dynamic_action,
      on_before_run_step: self.on_before_run_step,
    }
  }
}

/// `AstroRunPlugin` enables rapid definition of a synchronous astro-run plugin
/// without the need to declare a new struct to implement the `Plugin` trait.
pub struct AstroRunPlugin {
  name: &'static str,
  on_resolve_dynamic_action: Option<Box<OnResolveDynamicAction>>,
  on_run_workflow: Option<Box<OnRunWorkflow>>,
  on_run_job: Option<Box<OnRunJob>>,
  on_before_run_step: Option<Box<OnBeforeRunStep>>,
  on_run_step: Option<Box<OnRunStep>>,
  on_state_change: Option<Box<OnStateChange>>,
  on_log: Option<Box<OnLog>>,
  on_step_completed: Option<Box<OnStepComplete>>,
  on_job_completed: Option<Box<OnJobComplete>>,
  on_workflow_completed: Option<Box<OnWorkflowComplete>>,
}

impl AstroRunPlugin {
  pub fn builder(name: &'static str) -> PluginBuilder {
    PluginBuilder::new(name)
  }
}

#[async_trait::async_trait]
impl Plugin for AstroRunPlugin {
  fn name(&self) -> &'static str {
    self.name
  }

  async fn on_state_change(&self, event: WorkflowStateEvent) -> HookNoopResult {
    if let Some(on_state_change) = &self.on_state_change {
      on_state_change(event)?;
    }

    Ok(())
  }

  async fn on_log(&self, log: WorkflowLog) -> HookNoopResult {
    if let Some(on_log) = &self.on_log {
      on_log(log)?;
    }

    Ok(())
  }

  async fn on_run_workflow(&self, event: RunWorkflowEvent) -> HookNoopResult {
    if let Some(on_run_workflow) = &self.on_run_workflow {
      on_run_workflow(event)?;
    }

    Ok(())
  }

  async fn on_run_job(&self, event: RunJobEvent) -> HookNoopResult {
    if let Some(on_run_job) = &self.on_run_job {
      on_run_job(event)?;
    }

    Ok(())
  }

  async fn on_run_step(&self, event: RunStepEvent) -> HookNoopResult {
    if let Some(on_run_step) = &self.on_run_step {
      on_run_step(event)?;
    }

    Ok(())
  }

  async fn on_workflow_completed(&self, result: WorkflowRunResult) -> HookNoopResult {
    if let Some(on_workflow_completed) = &self.on_workflow_completed {
      on_workflow_completed(result)?;
    }

    Ok(())
  }

  async fn on_job_completed(&self, result: JobRunResult) -> HookNoopResult {
    if let Some(on_job_completed) = &self.on_job_completed {
      on_job_completed(result)?;
    }

    Ok(())
  }

  async fn on_step_completed(&self, result: StepRunResult) -> HookNoopResult {
    if let Some(on_step_completed) = &self.on_step_completed {
      on_step_completed(result)?;
    }

    Ok(())
  }

  async fn on_before_run_step(&self, step: Step) -> HookBeforeRunStepResult {
    let mut step = step;

    if let Some(on_before_run_step) = &self.on_before_run_step {
      match on_before_run_step(step.clone()) {
        Ok(new_step) => {
          step = new_step;
        }
        Err(err) => return Err(err),
      }
    }

    Ok(step)
  }

  async fn on_resolve_dynamic_action(&self, step: UserActionStep) -> HookResolveActionResult {
    if let Some(on_resolve_dynamic_action) = &self.on_resolve_dynamic_action {
      return on_resolve_dynamic_action(step);
    }

    Ok(None)
  }
}

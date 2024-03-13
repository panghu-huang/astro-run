use crate::{
  Job, JobRunResult, Plugin, PluginNoopResult, PluginResolveActionResult, Step, StepRunResult,
  UserActionStep, Workflow, WorkflowEvent, WorkflowLog, WorkflowRunResult, WorkflowStateEvent,
};

#[derive(Clone, Debug)]
pub struct RunEvent<T> {
  pub payload: T,
  pub workflow_event: Option<WorkflowEvent>,
}

pub type RunWorkflowEvent = RunEvent<Workflow>;

pub type RunJobEvent = RunEvent<Job>;

pub type RunStepEvent = RunEvent<Step>;

type OnStateChange = dyn Fn(WorkflowStateEvent) -> PluginNoopResult + Send + Sync;
type OnLog = dyn Fn(WorkflowLog) -> PluginNoopResult + Send + Sync;
type OnRunWorkflow = dyn Fn(RunWorkflowEvent) -> PluginNoopResult + Send + Sync;
type OnRunJob = dyn Fn(RunJobEvent) -> PluginNoopResult + Send + Sync;
type OnRunStep = dyn Fn(RunStepEvent) -> PluginNoopResult + Send + Sync;
type OnWorkflowComplete = dyn Fn(WorkflowRunResult) -> PluginNoopResult + Send + Sync;
type OnJobComplete = dyn Fn(JobRunResult) -> PluginNoopResult + Send + Sync;
type OnStepComplete = dyn Fn(StepRunResult) -> PluginNoopResult + Send + Sync;
type OnResolveDynamicAction = dyn Fn(UserActionStep) -> PluginResolveActionResult + Send + Sync;

pub struct PluginBuilder {
  name: &'static str,
  on_state_change: Option<Box<OnStateChange>>,
  on_log: Option<Box<OnLog>>,
  on_run_workflow: Option<Box<OnRunWorkflow>>,
  on_run_job: Option<Box<OnRunJob>>,
  on_run_step: Option<Box<OnRunStep>>,
  on_workflow_completed: Option<Box<OnWorkflowComplete>>,
  on_job_completed: Option<Box<OnJobComplete>>,
  on_step_completed: Option<Box<OnStepComplete>>,
  on_resolve_dynamic_action: Option<Box<OnResolveDynamicAction>>,
}

impl PluginBuilder {
  fn new(name: &'static str) -> Self {
    PluginBuilder {
      name,
      on_state_change: None,
      on_log: None,
      on_run_workflow: None,
      on_run_job: None,
      on_run_step: None,
      on_workflow_completed: None,
      on_job_completed: None,
      on_step_completed: None,
      on_resolve_dynamic_action: None,
    }
  }

  pub fn on_state_change<T>(mut self, on_state_change: T) -> Self
  where
    T: Fn(WorkflowStateEvent) -> PluginNoopResult + 'static + Send + Sync,
  {
    self.on_state_change = Some(Box::new(on_state_change));
    self
  }

  pub fn on_log<T>(mut self, on_log: T) -> Self
  where
    T: Fn(WorkflowLog) -> PluginNoopResult + 'static + Send + Sync,
  {
    self.on_log = Some(Box::new(on_log));
    self
  }

  pub fn on_run_workflow<T>(mut self, on_run_workflow: T) -> Self
  where
    T: Fn(RunWorkflowEvent) -> PluginNoopResult + 'static + Send + Sync,
  {
    self.on_run_workflow = Some(Box::new(on_run_workflow));
    self
  }

  pub fn on_run_job<T>(mut self, on_run_job: T) -> Self
  where
    T: Fn(RunJobEvent) -> PluginNoopResult + 'static + Send + Sync,
  {
    self.on_run_job = Some(Box::new(on_run_job));
    self
  }

  pub fn on_run_step<T>(mut self, on_run_step: T) -> Self
  where
    T: Fn(RunStepEvent) -> PluginNoopResult + 'static + Send + Sync,
  {
    self.on_run_step = Some(Box::new(on_run_step));
    self
  }

  pub fn on_workflow_completed<T>(mut self, on_workflow_completed: T) -> Self
  where
    T: Fn(WorkflowRunResult) -> PluginNoopResult + Send + Sync + 'static,
  {
    self.on_workflow_completed = Some(Box::new(on_workflow_completed));

    self
  }

  pub fn on_job_completed<T>(mut self, on_job_completed: T) -> Self
  where
    T: Fn(JobRunResult) -> PluginNoopResult + Send + Sync + 'static,
  {
    self.on_job_completed = Some(Box::new(on_job_completed));

    self
  }

  pub fn on_step_completed<T>(mut self, on_step_completed: T) -> Self
  where
    T: Fn(StepRunResult) -> PluginNoopResult + Send + Sync + 'static,
  {
    self.on_step_completed = Some(Box::new(on_step_completed));

    self
  }

  pub fn on_resolve_dynamic_action<T>(mut self, on_resolve_dynamic_action: T) -> Self
  where
    T: Fn(UserActionStep) -> PluginResolveActionResult + Send + Sync + 'static,
  {
    self.on_resolve_dynamic_action = Some(Box::new(on_resolve_dynamic_action));

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
    }
  }
}

/// `AstroRunPlugin` enables rapid definition of a synchronous astro-run plugin
/// without the need to declare a new struct to implement the `Plugin` trait.
pub struct AstroRunPlugin {
  name: &'static str,
  on_state_change: Option<Box<OnStateChange>>,
  on_log: Option<Box<OnLog>>,
  on_run_workflow: Option<Box<OnRunWorkflow>>,
  on_run_job: Option<Box<OnRunJob>>,
  on_run_step: Option<Box<OnRunStep>>,
  on_workflow_completed: Option<Box<OnWorkflowComplete>>,
  on_job_completed: Option<Box<OnJobComplete>>,
  on_step_completed: Option<Box<OnStepComplete>>,
  on_resolve_dynamic_action: Option<Box<OnResolveDynamicAction>>,
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

  async fn on_state_change(&self, event: WorkflowStateEvent) -> PluginNoopResult {
    if let Some(on_state_change) = &self.on_state_change {
      on_state_change(event)?;
    }

    Ok(())
  }

  async fn on_log(&self, log: WorkflowLog) -> PluginNoopResult {
    if let Some(on_log) = &self.on_log {
      on_log(log)?;
    }

    Ok(())
  }

  async fn on_run_workflow(&self, event: RunWorkflowEvent) -> PluginNoopResult {
    if let Some(on_run_workflow) = &self.on_run_workflow {
      on_run_workflow(event)?;
    }

    Ok(())
  }

  async fn on_run_job(&self, event: RunJobEvent) -> PluginNoopResult {
    if let Some(on_run_job) = &self.on_run_job {
      on_run_job(event)?;
    }

    Ok(())
  }

  async fn on_run_step(&self, event: RunStepEvent) -> PluginNoopResult {
    if let Some(on_run_step) = &self.on_run_step {
      on_run_step(event)?;
    }

    Ok(())
  }

  async fn on_workflow_completed(&self, result: WorkflowRunResult) -> PluginNoopResult {
    if let Some(on_workflow_completed) = &self.on_workflow_completed {
      on_workflow_completed(result)?;
    }

    Ok(())
  }

  async fn on_job_completed(&self, result: JobRunResult) -> PluginNoopResult {
    if let Some(on_job_completed) = &self.on_job_completed {
      on_job_completed(result)?;
    }

    Ok(())
  }

  async fn on_step_completed(&self, result: StepRunResult) -> PluginNoopResult {
    if let Some(on_step_completed) = &self.on_step_completed {
      on_step_completed(result)?;
    }

    Ok(())
  }

  async fn on_resolve_dynamic_action(&self, step: UserActionStep) -> PluginResolveActionResult {
    if let Some(on_resolve_dynamic_action) = &self.on_resolve_dynamic_action {
      return on_resolve_dynamic_action(step);
    }

    Ok(None)
  }
}

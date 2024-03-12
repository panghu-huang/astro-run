use crate::{
  Action, Job, JobRunResult, Step, StepRunResult, UserActionStep, Workflow, WorkflowEvent,
  WorkflowLog, WorkflowRunResult, WorkflowStateEvent,
};

#[derive(Clone, Debug)]
pub struct RunEvent<T> {
  pub payload: T,
  pub workflow_event: Option<WorkflowEvent>,
}

// #[derive(Clone, Debug)]
// pub struct CompletedEvent<T, R> {
//   pub payload: T,
//   pub workflow_event: Option<WorkflowEvent>,
//   pub result: R,
// }

pub type RunWorkflowEvent = RunEvent<Workflow>;

pub type RunJobEvent = RunEvent<Job>;

pub type RunStepEvent = RunEvent<Step>;

type OnStateChange = dyn Fn(WorkflowStateEvent) -> () + Send + Sync;
type OnLog = dyn Fn(WorkflowLog) -> () + Send + Sync;
type OnRunWorkflow = dyn Fn(RunWorkflowEvent) -> () + Send + Sync;
type OnRunJob = dyn Fn(RunJobEvent) -> () + Send + Sync;
type OnRunStep = dyn Fn(RunStepEvent) -> () + Send + Sync;
type OnWorkflowComplete = dyn Fn(WorkflowRunResult) -> () + Send + Sync;
type OnJobComplete = dyn Fn(JobRunResult) -> () + Send + Sync;
type OnStepComplete = dyn Fn(StepRunResult) -> () + Send + Sync;
type OnResolveDynamicAction = dyn Fn(UserActionStep) -> Option<Box<dyn Action>> + Send + Sync;

pub trait Plugin: Send + Sync {
  fn name(&self) -> &'static str;
  fn on_state_change(&self, _event: WorkflowStateEvent) {}
  fn on_log(&self, _log: WorkflowLog) {}
  fn on_run_workflow(&self, _event: RunWorkflowEvent) {}
  fn on_run_job(&self, _event: RunJobEvent) {}
  fn on_run_step(&self, _event: RunStepEvent) {}
  fn on_workflow_completed(&self, _result: WorkflowRunResult) {}
  fn on_job_completed(&self, _result: JobRunResult) {}
  fn on_step_completed(&self, _result: StepRunResult) {}
  fn on_resolve_dynamic_action(&self, _step: UserActionStep) -> Option<Box<dyn Action>> {
    None
  }
}

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
  pub fn new(name: &'static str) -> Self {
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
    T: Fn(WorkflowStateEvent) -> () + 'static + Send + Sync,
  {
    self.on_state_change = Some(Box::new(on_state_change));
    self
  }

  pub fn on_log<T>(mut self, on_log: T) -> Self
  where
    T: Fn(WorkflowLog) -> () + 'static + Send + Sync,
  {
    self.on_log = Some(Box::new(on_log));
    self
  }

  pub fn on_run_workflow<T>(mut self, on_run_workflow: T) -> Self
  where
    T: Fn(RunWorkflowEvent) -> () + 'static + Send + Sync,
  {
    self.on_run_workflow = Some(Box::new(on_run_workflow));
    self
  }

  pub fn on_run_job<T>(mut self, on_run_job: T) -> Self
  where
    T: Fn(RunJobEvent) -> () + 'static + Send + Sync,
  {
    self.on_run_job = Some(Box::new(on_run_job));
    self
  }

  pub fn on_run_step<T>(mut self, on_run_step: T) -> Self
  where
    T: Fn(RunStepEvent) -> () + 'static + Send + Sync,
  {
    self.on_run_step = Some(Box::new(on_run_step));
    self
  }

  pub fn on_workflow_completed<T>(mut self, on_workflow_completed: T) -> Self
  where
    T: Fn(WorkflowRunResult) -> () + 'static + Send + Sync,
  {
    self.on_workflow_completed = Some(Box::new(on_workflow_completed));
    self
  }

  pub fn on_job_completed<T>(mut self, on_job_completed: T) -> Self
  where
    T: Fn(JobRunResult) -> () + 'static + Send + Sync,
  {
    self.on_job_completed = Some(Box::new(on_job_completed));
    self
  }

  pub fn on_step_completed<T>(mut self, on_step_completed: T) -> Self
  where
    T: Fn(StepRunResult) -> () + 'static + Send + Sync,
  {
    self.on_step_completed = Some(Box::new(on_step_completed));
    self
  }

  pub fn on_resolve_dynamic_action<T>(mut self, on_resolve_dynamic_action: T) -> Self
  where
    T: Fn(UserActionStep) -> Option<Box<dyn Action>> + 'static + Send + Sync,
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

impl Plugin for AstroRunPlugin {
  fn name(&self) -> &'static str {
    self.name
  }

  fn on_state_change(&self, event: WorkflowStateEvent) {
    if let Some(on_state_change) = &self.on_state_change {
      on_state_change(event);
    }
  }

  fn on_log(&self, log: WorkflowLog) {
    if let Some(on_log) = &self.on_log {
      on_log(log);
    }
  }

  fn on_run_workflow(&self, event: RunWorkflowEvent) {
    if let Some(on_run_workflow) = &self.on_run_workflow {
      on_run_workflow(event);
    }
  }

  fn on_run_job(&self, event: RunJobEvent) {
    if let Some(on_run_job) = &self.on_run_job {
      on_run_job(event);
    }
  }

  fn on_run_step(&self, event: RunStepEvent) -> () {
    if let Some(on_run_step) = &self.on_run_step {
      on_run_step(event);
    }
  }

  fn on_workflow_completed(&self, result: WorkflowRunResult) {
    if let Some(on_workflow_completed) = &self.on_workflow_completed {
      on_workflow_completed(result);
    }
  }

  fn on_job_completed(&self, result: JobRunResult) {
    if let Some(on_job_completed) = &self.on_job_completed {
      on_job_completed(result);
    }
  }

  fn on_step_completed(&self, result: StepRunResult) {
    if let Some(on_step_completed) = &self.on_step_completed {
      on_step_completed(result);
    }
  }

  fn on_resolve_dynamic_action(&self, step: UserActionStep) -> Option<Box<dyn Action>> {
    if let Some(on_resolve_dynamic_action) = &self.on_resolve_dynamic_action {
      return on_resolve_dynamic_action(step);
    }

    None
  }
}

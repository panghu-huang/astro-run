use crate::{
  Job, JobRunResult, Step, StepRunResult, Workflow, WorkflowLog, WorkflowRunResult,
  WorkflowStateEvent,
};
use parking_lot::Mutex;
use std::sync::Arc;

type OnStateChange = dyn Fn(WorkflowStateEvent) -> () + Send + Sync;
type OnLog = dyn Fn(WorkflowLog) -> () + Send + Sync;
type OnRunWorkflow = dyn Fn(Workflow) -> () + Send + Sync;
type OnRunJob = dyn Fn(Job) -> () + Send + Sync;
type OnRunStep = dyn Fn(Step) -> () + Send + Sync;
type OnWorkflowComplete = dyn Fn(WorkflowRunResult) -> () + Send + Sync;
type OnJobComplete = dyn Fn(JobRunResult) -> () + Send + Sync;
type OnStepComplete = dyn Fn(StepRunResult) -> () + Send + Sync;

pub trait Plugin: Send {
  fn on_state_change(&self, event: WorkflowStateEvent) -> ();
  fn on_log(&self, log: WorkflowLog) -> ();
  fn on_run_workflow(&self, workflow: Workflow) -> ();
  fn on_run_job(&self, job: Job) -> ();
  fn on_run_step(&self, step: Step) -> ();
  fn on_workflow_completed(&self, result: WorkflowRunResult) -> ();
  fn on_job_completed(&self, result: JobRunResult) -> ();
  fn on_step_completed(&self, result: StepRunResult) -> ();
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
    T: Fn(Workflow) -> () + 'static + Send + Sync,
  {
    self.on_run_workflow = Some(Box::new(on_run_workflow));
    self
  }

  pub fn on_run_job<T>(mut self, on_run_job: T) -> Self
  where
    T: Fn(Job) -> () + 'static + Send + Sync,
  {
    self.on_run_job = Some(Box::new(on_run_job));
    self
  }

  pub fn on_run_step<T>(mut self, on_run_step: T) -> Self
  where
    T: Fn(Step) -> () + 'static + Send + Sync,
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
}

impl AstroRunPlugin {
  pub fn builder(name: &'static str) -> PluginBuilder {
    PluginBuilder::new(name)
  }
}

impl Plugin for AstroRunPlugin {
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

  fn on_run_workflow(&self, workflow: Workflow) {
    if let Some(on_run_workflow) = &self.on_run_workflow {
      on_run_workflow(workflow);
    }
  }

  fn on_run_job(&self, job: Job) {
    if let Some(on_run_job) = &self.on_run_job {
      on_run_job(job);
    }
  }

  fn on_run_step(&self, step: Step) -> () {
    if let Some(on_run_step) = &self.on_run_step {
      on_run_step(step);
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
}

#[derive(Clone)]
pub struct PluginManager {
  pub(crate) plugins: Arc<Mutex<Vec<AstroRunPlugin>>>,
}

impl PluginManager {
  pub fn new() -> Self {
    PluginManager {
      plugins: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn size(&self) -> usize {
    self.plugins.lock().len()
  }

  pub fn register(&self, plugin: AstroRunPlugin) {
    self.plugins.lock().push(plugin);
  }

  pub fn unregister(&self, name: &'static str) {
    self.plugins.lock().retain(|plugin| plugin.name != name);
  }

  pub fn on_state_change(&self, event: WorkflowStateEvent) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_state_change(event.clone());
    }
  }

  pub fn on_log(&self, log: WorkflowLog) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_log(log.clone());
    }
  }

  pub fn on_run_workflow(&self, workflow: Workflow) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_run_workflow(workflow.clone());
    }
  }

  pub fn on_run_job(&self, job: Job) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_run_job(job.clone());
    }
  }

  pub fn on_run_step(&self, step: Step) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_run_step(step.clone());
    }
  }

  pub fn on_workflow_completed(&self, result: WorkflowRunResult) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_workflow_completed(result.clone());
    }
  }

  pub fn on_job_completed(&self, result: JobRunResult) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_job_completed(result.clone());
    }
  }

  pub fn on_step_completed(&self, result: StepRunResult) {
    let plugins = self.plugins.lock();
    for plugin in plugins.iter() {
      plugin.on_step_completed(result.clone());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{WorkflowId, WorkflowState, WorkflowStateEvent};

  #[test]
  fn plugin_manager_register() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test").build();

    plugin_manager.register(plugin);

    assert_eq!(plugin_manager.size(), 1);
  }

  #[test]
  fn plugin_manager_unregister() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test").build();

    plugin_manager.register(plugin);
    plugin_manager.unregister("test");

    assert_eq!(plugin_manager.size(), 0);
  }

  #[test]
  fn plugin_manager_on_state_change() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test")
      .on_state_change(|event| {
        if let WorkflowStateEvent::WorkflowStateUpdated { id, state } = event {
          assert_eq!(id, WorkflowId::new("test"));
          assert_eq!(state, WorkflowState::Cancelled);
        } else {
          panic!("Unexpected event type");
        }
      })
      .build();

    plugin_manager.register(plugin);
    plugin_manager.on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
      id: WorkflowId::new("test"),
      state: WorkflowState::Cancelled,
    });
  }

  #[test]
  fn plugin_manager_on_log() {
    let plugin_manager = PluginManager::new();
    let plugin = PluginBuilder::new("test")
      .on_log(|log| {
        assert_eq!(log.message, "test");
      })
      .build();

    plugin_manager.register(plugin);
    plugin_manager.on_log(WorkflowLog {
      message: "test".to_string(),
      ..Default::default()
    });
  }
}

use crate::{
  Actions, AstroRunPlugin, AstroRunSignal, Error, Job, JobId, JobRunResult, PluginManager, Result,
  Workflow, WorkflowRunResult, WorkflowStateEvent,
};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
struct SharedState {
  plugins: PluginManager,
  actions: Actions,
  signals: HashMap<JobId, AstroRunSignal>,
}

impl SharedState {
  pub fn new() -> Self {
    SharedState {
      plugins: PluginManager::new(),
      actions: Actions::new(),
      signals: HashMap::new(),
    }
  }
}

#[derive(Clone)]
pub struct AstroRunSharedState(Arc<Mutex<SharedState>>);

impl AstroRunSharedState {
  pub fn new() -> Self {
    AstroRunSharedState(Arc::new(Mutex::new(SharedState::new())))
  }

  pub fn register_plugin(&self, plugin: AstroRunPlugin) {
    self.0.lock().plugins.register(plugin);
  }

  pub fn unregister_plugin(&self, plugin_name: &'static str) {
    self.0.lock().plugins.unregister(plugin_name);
  }

  pub fn plugins(&self) -> PluginManager {
    self.0.lock().plugins.clone()
  }

  pub fn register_action<T>(&self, name: impl Into<String>, action: T)
  where
    T: crate::actions::Action + 'static,
  {
    self.0.lock().actions.register(name, action);
  }

  pub fn unregister_action(&self, name: &str) {
    self.0.lock().actions.unregister(name);
  }

  pub fn actions(&self) -> Actions {
    self.0.lock().actions.clone()
  }

  pub fn on_state_change(&self, event: WorkflowStateEvent) {
    self.0.lock().plugins.on_state_change(event);
  }

  pub fn on_run_workflow(&self, workflow: Workflow) {
    self.0.lock().plugins.on_run_workflow(workflow);
  }

  pub fn on_run_job(&self, job: Job) {
    self.0.lock().plugins.on_run_job(job);
  }

  pub fn on_workflow_completed(&self, result: WorkflowRunResult) {
    self.0.lock().plugins.on_workflow_completed(result);
  }

  pub fn on_job_completed(&self, result: JobRunResult) {
    self.0.lock().plugins.on_job_completed(result);
  }

  pub fn add_signal(&self, job_id: JobId, signal: AstroRunSignal) {
    self.0.lock().signals.insert(job_id, signal);
  }

  pub fn get_signal(&self, job_id: &JobId) -> Option<AstroRunSignal> {
    self.0.lock().signals.get(job_id).cloned()
  }

  pub fn remove_signal(&self, job_id: &JobId) {
    self.0.lock().signals.remove(job_id);
  }

  pub fn cancel(&self, job_id: &JobId) -> Result<()> {
    let signal = self
      .get_signal(job_id)
      .ok_or_else(|| Error::error(format!("Job {} not found", job_id.to_string())))?;

    signal.cancel()?;

    Ok(())
  }
}

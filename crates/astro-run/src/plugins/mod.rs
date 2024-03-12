mod plugin;

use crate::{
  Action, JobRunResult, StepRunResult, UserActionStep, WorkflowLog, WorkflowRunResult,
  WorkflowStateEvent,
};
pub use plugin::*;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
  fn name(&self) -> &'static str;
  fn on_state_change(&self, _event: WorkflowStateEvent) {}
  fn on_log(&self, _log: WorkflowLog) {}
  fn on_run_workflow(&self, _event: RunWorkflowEvent) {}
  fn on_run_job(&self, _event: RunJobEvent) {}
  fn on_run_step(&self, _event: RunStepEvent) {}
  async fn on_workflow_completed(&self, _result: WorkflowRunResult) {}
  async fn on_job_completed(&self, _result: JobRunResult) {}
  async fn on_step_completed(&self, _result: StepRunResult) {}
  async fn on_resolve_dynamic_action(&self, _step: UserActionStep) -> Option<Box<dyn Action>> {
    None
  }
}

pub type SharedPluginDriver = Arc<PluginDriver>;

pub struct PluginDriver {
  pub(crate) plugins: Vec<Box<dyn Plugin>>,
}

impl PluginDriver {
  pub fn new(plugins: Vec<Box<dyn Plugin>>) -> Self {
    PluginDriver { plugins }
  }

  pub fn on_state_change(&self, event: WorkflowStateEvent) {
    for plugin in &self.plugins {
      plugin.on_state_change(event.clone());
    }
  }

  pub fn on_log(&self, log: WorkflowLog) {
    for plugin in &self.plugins {
      plugin.on_log(log.clone());
    }
  }

  pub fn on_run_workflow(&self, event: RunWorkflowEvent) {
    for plugin in &self.plugins {
      plugin.on_run_workflow(event.clone());
    }
  }

  pub fn on_run_job(&self, event: RunJobEvent) {
    for plugin in &self.plugins {
      plugin.on_run_job(event.clone());
    }
  }

  pub fn on_run_step(&self, event: RunStepEvent) {
    for plugin in &self.plugins {
      plugin.on_run_step(event.clone());
    }
  }

  pub async fn on_workflow_completed(&self, result: WorkflowRunResult) {
    for plugin in &self.plugins {
      plugin.on_workflow_completed(result.clone()).await;
    }
  }

  pub async fn on_job_completed(&self, result: JobRunResult) {
    for plugin in &self.plugins {
      plugin.on_job_completed(result.clone()).await;
    }
  }

  pub async fn on_step_completed(&self, result: StepRunResult) {
    for plugin in &self.plugins {
      plugin.on_step_completed(result.clone()).await;
    }
  }

  pub async fn on_resolve_dynamic_action(&self, step: UserActionStep) -> Option<Box<dyn Action>> {
    for plugin in &self.plugins {
      if let Some(action) = plugin.on_resolve_dynamic_action(step.clone()).await {
        return Some(action);
      }
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{WorkflowId, WorkflowState, WorkflowStateEvent};

  #[test]
  fn plugin_manager_on_state_change() {
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

    let plugin_driver = PluginDriver::new(vec![Box::new(plugin)]);

    plugin_driver.on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
      id: WorkflowId::new("test"),
      state: WorkflowState::Cancelled,
    });
  }

  #[test]
  fn plugin_manager_on_log() {
    let plugin = PluginBuilder::new("test")
      .on_log(|log| {
        assert_eq!(log.message, "test");
      })
      .build();

    let plugin_driver = PluginDriver::new(vec![Box::new(plugin)]);

    plugin_driver.on_log(WorkflowLog {
      message: "test".to_string(),
      ..Default::default()
    });
  }

  #[tokio::test]
  async fn test_plugin_trait() {
    struct TestPlugin;

    impl Plugin for TestPlugin {
      fn name(&self) -> &'static str {
        "test"
      }
    }

    let plugin_driver = PluginDriver::new(vec![Box::new(TestPlugin)]);

    plugin_driver.on_log(WorkflowLog {
      message: "test".to_string(),
      ..Default::default()
    });

    let action = plugin_driver
      .on_resolve_dynamic_action(UserActionStep {
        name: Some("test".to_string()),
        ..Default::default()
      })
      .await;

    assert_eq!(action.is_none(), true);
  }
}

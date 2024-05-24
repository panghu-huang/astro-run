use crate::{
  Action, JobRunResult, Plugin, RunJobEvent, RunStepEvent, RunWorkflowEvent, Step, StepRunResult,
  UserActionStep, WorkflowLog, WorkflowRunResult, WorkflowStateEvent,
};
use std::sync::Arc;

pub type SharedPluginDriver = Arc<PluginDriver>;

pub struct PluginDriver {
  pub(crate) plugins: Vec<Box<dyn Plugin>>,
}

impl PluginDriver {
  pub fn new(plugins: Vec<Box<dyn Plugin>>) -> Self {
    PluginDriver { plugins }
  }

  pub async fn on_state_change(&self, event: WorkflowStateEvent) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_state_change(event.clone()).await {
        log::error!(
          "Plugin {} failed to handle state change: {}",
          plugin.name(),
          err
        );
      }
    }
  }

  pub async fn on_log(&self, log: WorkflowLog) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_log(log.clone()).await {
        log::error!("Plugin {} failed to handle log: {}", plugin.name(), err);
      }
    }
  }

  pub async fn on_run_workflow(&self, event: RunWorkflowEvent) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_run_workflow(event.clone()).await {
        log::error!(
          "Plugin {} failed to handle run workflow: {}",
          plugin.name(),
          err
        );
      }
    }
  }

  pub async fn on_run_job(&self, event: RunJobEvent) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_run_job(event.clone()).await {
        log::error!("Plugin {} failed to handle run job: {}", plugin.name(), err);
      }
    }
  }

  pub async fn on_run_step(&self, event: RunStepEvent) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_run_step(event.clone()).await {
        log::error!(
          "Plugin {} failed to handle run step: {}",
          plugin.name(),
          err
        );
      }
    }
  }

  pub async fn on_workflow_completed(&self, result: WorkflowRunResult) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_workflow_completed(result.clone()).await {
        log::error!(
          "Plugin {} failed to handle workflow completed: {}",
          plugin.name(),
          err
        );
      }
    }
  }

  pub async fn on_job_completed(&self, result: JobRunResult) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_job_completed(result.clone()).await {
        log::error!(
          "Plugin {} failed to handle job completed: {}",
          plugin.name(),
          err
        );
      }
    }
  }

  pub async fn on_step_completed(&self, result: StepRunResult) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_step_completed(result.clone()).await {
        log::error!(
          "Plugin {} failed to handle step completed: {}",
          plugin.name(),
          err
        );
      }
    }
  }

  pub async fn on_before_run_step(&self, step: Step) -> Step {
    let mut step = step;
    for plugin in &self.plugins {
      match plugin.on_before_run_step(step.clone()).await {
        Ok(new_step) => step = new_step,
        Err(err) => {
          log::error!(
            "Plugin {} failed to handle before run step: {}",
            plugin.name(),
            err
          );
        }
      }
    }

    step
  }

  pub async fn on_resolve_dynamic_action(&self, step: UserActionStep) -> Option<Box<dyn Action>> {
    for plugin in &self.plugins {
      match plugin.on_resolve_dynamic_action(step.clone()).await {
        Ok(Some(action)) => return Some(action),
        Ok(None) => {}
        Err(err) => {
          log::error!(
            "Plugin {} failed to handle resolve dynamic action: {}",
            plugin.name(),
            err
          );
        }
      }
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{AstroRunPlugin, Error, WorkflowId, WorkflowState, WorkflowStateEvent};

  #[astro_run_test::test]
  async fn plugin_driver_on_state_change() {
    let plugin = AstroRunPlugin::builder("test")
      .on_state_change(|event| {
        if let WorkflowStateEvent::WorkflowStateUpdated { id, state } = event {
          assert_eq!(id, WorkflowId::new("test"));
          assert_eq!(state, WorkflowState::Cancelled);

          Ok(())
        } else {
          panic!("Unexpected event type");
        }
      })
      .build();

    let error_plugin = AstroRunPlugin::builder("error")
      .on_state_change(|_| Err(Error::error("test")))
      .build();

    let plugin_driver = PluginDriver::new(vec![Box::new(plugin), Box::new(error_plugin)]);

    plugin_driver
      .on_state_change(WorkflowStateEvent::WorkflowStateUpdated {
        id: WorkflowId::new("test"),
        state: WorkflowState::Cancelled,
      })
      .await;
  }

  #[astro_run_test::test]
  async fn plugin_driver_on_before_run_step() {
    let plugin = AstroRunPlugin::builder("test")
      .on_before_run_step(|step| {
        let mut step = step;
        step.run = "Updated".to_string();

        Ok(step)
      })
      .build();

    let update_name_plugin = AstroRunPlugin::builder("update_name")
      .on_before_run_step(|step| {
        let mut step = step;
        step.name = Some("Updated".to_string());

        Ok(step)
      })
      .build();

    let error_plugin = AstroRunPlugin::builder("error")
      .on_before_run_step(|_| Err(Error::error("test")))
      .build();

    let plugin_driver = PluginDriver::new(vec![
      Box::new(plugin),
      Box::new(error_plugin),
      Box::new(update_name_plugin),
    ]);

    plugin_driver
      .on_before_run_step(Step {
        ..Default::default()
      })
      .await;
  }

  #[astro_run_test::test]
  async fn plugin_driver_on_log() {
    let plugin = AstroRunPlugin::builder("test")
      .on_log(|log| {
        assert_eq!(log.message, "test");

        Ok(())
      })
      .build();

    let error_plugin = AstroRunPlugin::builder("error")
      .on_log(|_| Err(Error::error("test")))
      .build();

    let plugin_driver = PluginDriver::new(vec![Box::new(plugin), Box::new(error_plugin)]);

    plugin_driver
      .on_log(WorkflowLog {
        message: "test".to_string(),
        ..Default::default()
      })
      .await;
  }

  #[astro_run_test::test]
  async fn test_plugin_trait() {
    struct TestPlugin;

    impl Plugin for TestPlugin {
      fn name(&self) -> &'static str {
        "test"
      }
    }

    struct ErrorPlugin;

    #[async_trait::async_trait]
    impl Plugin for ErrorPlugin {
      fn name(&self) -> &'static str {
        "error"
      }

      async fn on_resolve_dynamic_action(
        &self,
        _: UserActionStep,
      ) -> Result<Option<Box<dyn Action>>, Error> {
        Err(Error::error("test"))
      }
    }

    let plugin_driver = PluginDriver::new(vec![Box::new(TestPlugin)]);

    plugin_driver
      .on_log(WorkflowLog {
        message: "test".to_string(),
        ..Default::default()
      })
      .await;

    let action = plugin_driver
      .on_resolve_dynamic_action(UserActionStep {
        name: Some("test".to_string()),
        ..Default::default()
      })
      .await;

    assert!(action.is_none());
  }
}

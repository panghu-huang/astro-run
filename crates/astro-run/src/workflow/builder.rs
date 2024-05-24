use super::{parser::WorkflowParser, Workflow};
use crate::{AstroRun, Error, Id, Payload, Result, UserWorkflow};

pub struct WorkflowBuilder {
  id: Option<Id>,
  config: Option<String>,
  payload: Option<Box<dyn Payload>>,
}

impl Default for WorkflowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowBuilder {
  pub fn new() -> Self {
    Self {
      id: None,
      config: None,
      payload: None,
    }
  }

  pub fn id(mut self, id: impl Into<Id>) -> Self {
    self.id = Some(id.into());
    self
  }

  pub fn config(mut self, config: impl Into<String>) -> Self {
    self.config = Some(config.into());
    self
  }

  pub fn payload<T>(mut self, payload: T) -> Self
  where
    T: Payload + 'static,
  {
    self.payload = Some(Box::new(payload));

    self
  }

  pub async fn build(self, astro_run: &AstroRun) -> Result<Workflow> {
    let config = self
      .config
      .ok_or(Error::init_error("Workflow config is required".to_string()))?;

    let payload = match self.payload {
      Some(payload) => {
        let payload = payload.as_ref().try_into_string()?;

        Some(payload)
      }
      None => None,
    };

    let user_workflow = UserWorkflow::try_from(config)?;
    let id = self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let parser = WorkflowParser {
      id,
      user_workflow,
      astro_run,
      payload,
    };

    parser.parse().await
  }
}

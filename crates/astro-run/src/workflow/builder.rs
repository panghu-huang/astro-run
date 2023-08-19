use super::{parser::WorkflowParser, Workflow};
use crate::{AstroRun, Error, Id, Result, UserWorkflow, WorkflowEvent};

pub struct WorkflowBuilder {
  id: Option<Id>,
  config: Option<String>,
  event: Option<WorkflowEvent>,
}

impl WorkflowBuilder {
  pub fn new() -> Self {
    Self {
      id: None,
      config: None,
      event: None,
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

  pub fn event(mut self, event: WorkflowEvent) -> Self {
    self.event = Some(event);
    self
  }

  pub fn build(self, astro_run: &AstroRun) -> Result<Workflow> {
    let config = self
      .config
      .ok_or(Error::init_error("Workflow config is required".to_string()))?;

    let user_workflow = UserWorkflow::try_from(config)?;
    let id = self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let parser = WorkflowParser {
      id,
      event: self.event,
      user_workflow,
      actions: astro_run.shared_state.actions(),
    };

    parser.parse()
  }
}

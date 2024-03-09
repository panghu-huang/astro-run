use super::{parser::WorkflowParser, Workflow};
use crate::{AstroRun, Error, Id, Payload, Result, UserWorkflow};

pub struct WorkflowBuilder {
  id: Option<Id>,
  config: Option<String>,
  payload: Option<Box<dyn Payload>>,
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

  pub fn build(self, astro_run: &AstroRun) -> Result<Workflow> {
    let config = self
      .config
      .ok_or(Error::init_error("Workflow config is required".to_string()))?;

    let payload = match self.payload {
      Some(payload) => {
        let payload = payload.as_ref().try_into()?;

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

    parser.parse()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{async_trait, AstroRun, Context, Error, Result, RunResponse, Runner};

  struct TestRunner;

  #[async_trait]
  impl Runner for TestRunner {
    async fn run(&self, _ctx: Context) -> RunResponse {
      unreachable!("TestRunner should not be called")
    }
  }

  #[test]
  fn test_workflow_payload_to_string_error() {
    struct WorkflowPayload;

    impl crate::Payload for WorkflowPayload {
      fn try_into(&self) -> Result<String> {
        Err(Error::workflow_config_error("Payload error"))
      }

      fn try_from(_payload: &String) -> Result<Self> {
        unimplemented!()
      }
    }

    let workflow = r#"
      jobs:
        test:
          steps:
            - run: echo "Hello World"
      "#;

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let workflow = Workflow::builder()
      .config(workflow)
      .payload(WorkflowPayload)
      .build(&astro_run);

    assert_eq!(
      workflow.unwrap_err(),
      Error::workflow_config_error("Payload error")
    );
  }

  #[test]
  fn test_workflow_payload_not_set() {
    #[derive(Debug)]
    struct WorkflowPayload;

    impl crate::Payload for WorkflowPayload {
      fn try_into(&self) -> Result<String> {
        unimplemented!()
      }

      fn try_from(_payload: &String) -> Result<Self> {
        unimplemented!()
      }
    }

    let workflow = r#"
      jobs:
        test:
          steps:
            - run: echo "Hello World"
      "#;

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let workflow = Workflow::builder()
      .config(workflow)
      .build(&astro_run)
      .unwrap();

    let result = workflow.payload::<WorkflowPayload>();

    assert_eq!(
      result.unwrap_err(),
      Error::error("Payload is not set for this workflow")
    );
  }
}

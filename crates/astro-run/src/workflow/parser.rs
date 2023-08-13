use super::{job::Job, Step, Workflow};
use crate::{
  Error, Id, JobId, Result, StepId, UserCommandStep, UserStep, UserWorkflow, WorkflowEvent,
  WorkflowEventPayload, WorkflowId,
};
use std::collections::HashMap;

pub struct WorkflowParser {
  pub id: Id,
  pub user_workflow: UserWorkflow,
  pub event: WorkflowEvent,
}

impl WorkflowParser {
  pub fn parse(self) -> Result<Workflow> {
    let id = self.id;
    let event = self.event.payload()?;
    let user_workflow = self.user_workflow;

    let mut jobs = HashMap::new();
    for (key, job) in user_workflow.jobs {
      let mut steps = Vec::new();
      let job_container = job.container;
      let job_working_dirs = job.working_dirs.unwrap_or_default();
      for (idx, step) in job.steps.iter().enumerate() {
        if let UserStep::Command(UserCommandStep {
          name,
          container,
          run,
          continue_on_error,
          environments,
          timeout,
          ..
        }) = step.clone()
        {
          let timeout = timeout.unwrap_or("60m".to_string());
          let timeout = humantime::parse_duration(&timeout).map_err(|err| {
            log::error!("Invalid timeout format: {}", err);
            Error::workflow_config_error(
              "Invalid timeout format. The format should like `60m` or `1h`.",
            )
          })?;

          steps.push(Step {
            id: StepId::new(id.clone(), key.clone(), idx),
            name,
            container: container.or(job_container.clone()).map(|c| c.normalize()),
            run,
            continue_on_error: continue_on_error.unwrap_or(false),
            environments: environments.unwrap_or_default(),
            // TODO: support volumes and secrets
            secrets: vec![],
            timeout,
          });
        } else {
          return Err(Error::unsupported_feature("Only command step is supported"));
        }
      }

      jobs.insert(
        key.clone(),
        Job {
          id: JobId::new(id.clone(), key.clone()),
          name: job.name,
          steps,
          on: job.on,
          depends_on: job.depends_on.unwrap_or_default(),
          working_directories: job_working_dirs,
        },
      );
    }

    Ok(Workflow {
      id: WorkflowId::new(id),
      event,
      name: user_workflow.name,
      on: user_workflow.on,
      jobs,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::EnvironmentVariable;

  #[test]
  fn test_parse() {
    let yaml = r#"
name: Test Workflow
on: 
  push:
    branches:
      - master

jobs:
  test-job:
    name: Test Job
    working-directories:
    - /home/runner/work
    steps:
      - name: Test Step
        continue-on-error: true
        timeout: 10m
        environments:
          TEST_ENV: test
        run: echo "Hello World"
  "#;

    let user_workflow: UserWorkflow = serde_yaml::from_str(yaml).unwrap();
    let event = WorkflowEvent::default();

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow,
      event,
    };

    let workflow = parser.parse().unwrap();

    assert_eq!(workflow.id, WorkflowId::new("test-id"));
    assert_eq!(workflow.name.unwrap(), "Test Workflow");
    assert_eq!(workflow.jobs.len(), 1);

    let job = workflow.jobs.get("test-job").unwrap();
    assert_eq!(job.name.clone().unwrap(), "Test Job");
    assert_eq!(job.steps.len(), 1);

    let step = job.steps.get(0).unwrap();
    assert_eq!(step.name.clone().unwrap(), "Test Step");
    assert_eq!(step.continue_on_error, true);
    assert_eq!(step.timeout, std::time::Duration::from_secs(600));
    assert_eq!(step.environments.len(), 1);
    assert_eq!(
      step.environments.get("TEST_ENV").unwrap(),
      &EnvironmentVariable::String("test".to_string())
    );
    assert_eq!(step.run, "echo \"Hello World\"");
  }

  #[test]
  fn test_invalid_time_format() {
    let yaml = r#"
jobs:
  test:
    name: Test Job
    steps:
      - timeout: 1ss
        run: Hello World
  "#;

    let user_workflow: UserWorkflow = serde_yaml::from_str(yaml).unwrap();
    let event = WorkflowEvent::default();

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow,
      event,
    };

    let workflow = parser.parse();

    let excepted_error =
      Error::workflow_config_error("Invalid timeout format. The format should like `60m` or `1h`.");

    assert_eq!(workflow.unwrap_err(), excepted_error);
  }
}

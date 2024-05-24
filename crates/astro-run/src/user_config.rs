use crate::{Condition, EnvironmentVariables, Error, Id, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContainerOptions {
  pub name: String,
  pub volumes: Option<Vec<String>>,
  #[serde(rename = "security-opts")]
  pub security_opts: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Container {
  Options(ContainerOptions),
  Name(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UserCommandStep {
  pub name: Option<String>,
  pub container: Option<Container>,
  pub run: String,
  pub on: Option<Condition>,
  #[serde(rename = "continue-on-error")]
  pub continue_on_error: Option<bool>,
  pub environments: Option<EnvironmentVariables>,
  pub secrets: Option<Vec<String>>,
  pub timeout: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UserActionStep {
  pub name: Option<String>,
  pub uses: String,
  pub with: Option<serde_yaml::Value>,
  pub on: Option<Condition>,
  #[serde(rename = "continue-on-error")]
  pub continue_on_error: Option<bool>,
  pub environments: Option<EnvironmentVariables>,
  pub secrets: Option<Vec<String>>,
  pub timeout: Option<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UserStep {
  Command(UserCommandStep),
  Action(UserActionStep),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserJob {
  pub name: Option<String>,
  pub container: Option<Container>,
  /// Working directory for all steps in this job
  #[serde(rename = "working-directories")]
  pub working_dirs: Option<Vec<String>>,
  pub steps: Vec<UserStep>,
  pub on: Option<Condition>,
  #[serde(rename = "depends-on")]
  pub depends_on: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserWorkflow {
  pub name: Option<String>,
  pub on: Option<Condition>,
  pub jobs: HashMap<Id, UserJob>,
}

impl UserWorkflow {
  fn validate(workflow: &UserWorkflow) -> Result<()> {
    if workflow.jobs.is_empty() {
      return Err(Error::workflow_config_error(
        "Workflow must have at least one job",
      ));
    }

    let mut is_all_jobs_has_dependencies = true;
    // Validate dependencies key in jobs
    for (job_name, job) in &workflow.jobs {
      if let Some(depends_on) = &job.depends_on {
        if !depends_on.is_empty() {
          for depend_job_key in depends_on {
            if !workflow.jobs.contains_key(depend_job_key) {
              return Err(Error::workflow_config_error(format!(
                "Job {} depends on job {}, but job {} is not defined",
                job_name, depend_job_key, depend_job_key
              )));
            }
          }
        } else {
          is_all_jobs_has_dependencies = false;
        }
      } else {
        is_all_jobs_has_dependencies = false;
      }

      if job.steps.is_empty() {
        return Err(Error::workflow_config_error(format!(
          "Job `{}` must have at least one step",
          job_name
        )));
      }
    }

    if is_all_jobs_has_dependencies {
      return Err(Error::workflow_config_error(
        "Cannot have all jobs has dependencies",
      ));
    }

    Ok(())
  }
}

impl Container {
  pub fn name(&self) -> &str {
    match self {
      Self::Options(docker) => &docker.name,
      Self::Name(name) => name,
    }
  }

  pub fn normalize(&self) -> ContainerOptions {
    match self {
      Self::Options(docker) => docker.clone(),
      Self::Name(name) => ContainerOptions {
        name: name.clone(),
        security_opts: None,
        volumes: None,
      },
    }
  }
}

impl TryFrom<&str> for UserWorkflow {
  type Error = Error;

  fn try_from(value: &str) -> Result<Self> {
    let workflow = serde_yaml::from_str(value)
      .map_err(|e| Error::workflow_config_error(format!("Failed to parse workflow: {}", e)))?;

    Self::validate(&workflow)?;

    Ok(workflow)
  }
}

impl TryFrom<String> for UserWorkflow {
  type Error = Error;

  fn try_from(value: String) -> Result<Self> {
    Self::try_from(value.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{ConditionConfig, EnvironmentVariable, PullRequestCondition, PushCondition};

  #[test]
  fn test_parse() {
    let yaml = r#"
name: Test Workflow

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
          number: 1
          boolean: true
        run: echo "Hello World"
      - name: Action step
        uses: cache
"#;

    let workflow = UserWorkflow::try_from(yaml).unwrap();

    assert_eq!(workflow.name, Some("Test Workflow".to_string()));

    let job = workflow.jobs.get("test-job").unwrap();
    assert_eq!(job.name, Some("Test Job".to_string()));
    // assert_eq!(job.working_dir, Some("/home/runner/work".to_string()));

    let step = job.steps.get(0).unwrap();

    if let UserStep::Command(command_step) = step {
      let UserCommandStep {
        name,
        environments,
        run,
        continue_on_error,
        timeout,
        ..
      } = command_step;
      assert_eq!(name.as_ref().unwrap(), "Test Step");
      // assert_eq!(working_dir.as_ref().unwrap(), "/home/runner/work");
      assert_eq!(timeout.as_ref().unwrap(), "10m");
      assert_eq!(continue_on_error, &Some(true));

      let environments = environments.clone().unwrap();
      assert_eq!(
        environments.get("TEST_ENV").unwrap(),
        &EnvironmentVariable::String("test".to_string())
      );
      assert_eq!(
        environments.get("number").unwrap(),
        &EnvironmentVariable::Number(1.0)
      );
      assert_eq!(
        environments.get("boolean").unwrap(),
        &EnvironmentVariable::Boolean(true)
      );

      assert_eq!(run, "echo \"Hello World\"");
    } else {
      panic!("Step should be command step");
    }
  }

  #[test]
  fn test_empty_jobs() {
    let yaml = r#"jobs:"#;

    let res = UserWorkflow::try_from(yaml);

    assert_eq!(
      res.unwrap_err(),
      Error::workflow_config_error("Workflow must have at least one job")
    );
  }

  #[test]
  fn test_job_depend_not_exist() {
    let yaml = r#"
jobs:
  job1:
    depends-on: [job2]
    steps:
      - run: echo "Hello World"
"#;

    let res = UserWorkflow::try_from(yaml);
    assert_eq!(
      res.unwrap_err(),
      Error::workflow_config_error("Job job1 depends on job job2, but job job2 is not defined")
    );
  }

  #[test]
  fn test_empty_depend() {
    let yaml = r#"
    jobs:
      job1:
        depends-on: []
        steps:
          - run: echo "Hello World"
      job2:
        depends-on: [job1]
        steps:
          - run: echo "Hello World"
    "#;

    UserWorkflow::try_from(yaml).unwrap();
  }

  #[test]
  fn test_job_dependencies() {
    let yaml = r#"
jobs:
  job1:
    depends-on: [job2]
    steps:
      - run: echo "Hello World"
  job2:
    depends-on: [job1]
    steps:
      - run: echo "Hello World"
"#;

    let res = UserWorkflow::try_from(yaml);
    assert_eq!(
      res.unwrap_err(),
      Error::workflow_config_error("Cannot have all jobs has dependencies")
    );
  }

  #[test]
  fn test_empty_steps() {
    let yaml = r#"
jobs:
  job1:
    name: Test Job
    steps:
"#;

    let res = UserWorkflow::try_from(yaml);
    assert_eq!(
      res.unwrap_err(),
      Error::workflow_config_error("Job `job1` must have at least one step")
    );
  }

  #[test]
  fn test_container_name() {
    let yaml = r#"
jobs:
  job1:
    name: Test Job
    container: test
    steps:
      - run: echo "Hello World"
"#;

    let workflow = UserWorkflow::try_from(yaml).unwrap();
    let job = workflow.jobs.get("job1").unwrap();
    let container = job.container.as_ref().unwrap();
    assert_eq!(container.name(), "test");
  }

  #[test]
  fn test_container_options() {
    let yaml = r#"
jobs:
  job1:
    name: Test Job
    container: 
      name: test
      volumes:
        - /home/runner/work
      security-opts:
        - seccomp=unconfined
    steps:
      - run: echo "Hello World"
"#;

    let workflow = UserWorkflow::try_from(yaml).unwrap();
    let job = workflow.jobs.get("job1").unwrap();
    let container = job.container.as_ref().unwrap();
    assert_eq!(container.name(), "test");

    let normalized = container.normalize();
    assert_eq!(normalized.name, "test");
    assert_eq!(
      normalized.security_opts,
      Some(vec!["seccomp=unconfined".to_string()])
    );
    assert_eq!(
      normalized.volumes,
      Some(vec!["/home/runner/work".to_string()])
    );
  }

  #[test]
  fn test_events_condition() {
    let yaml = r#"
on:
  - push
  - pull_request
jobs:
  job:
    name: Test Job
    on:
      - push
      - pull_request
    steps:
      - run: echo "Hello World"
        on:
          - push
          - pull_request
"#;

    let workflow = UserWorkflow::try_from(yaml).unwrap();
    let on = Some(Condition::Event(vec![
      "push".to_string(),
      "pull_request".to_string(),
    ]));

    assert_eq!(&workflow.on, &on);

    let job = workflow.jobs.get("job").unwrap();
    assert_eq!(&job.on, &on);

    let step = job.steps.get(0).unwrap();
    if let UserStep::Command(command_step) = step {
      assert_eq!(&command_step.on, &on);
    } else {
      panic!("Step should be command step");
    }
  }

  #[test]
  fn test_config_condition() {
    let yaml = r#"
on:
  push:
    branches:
      - master
    paths:
      - "src/**"
jobs:
  job:
    name: Test Job
    on:
      push:
        paths:
          - "src/**"
    steps:
      - run: echo "Hello World"
        on:
          pull_request:
            branches:
              - master
"#;

    let workflow = UserWorkflow::try_from(yaml).unwrap();
    let on = Some(Condition::Config(ConditionConfig {
      push: Some(PushCondition {
        branches: Some(vec!["master".to_string()]),
        paths: Some(vec!["src/**".to_string()]),
      }),
      pull_request: None,
    }));

    assert_eq!(workflow.on, on);

    let on = Some(Condition::Config(ConditionConfig {
      push: Some(PushCondition {
        branches: None,
        paths: Some(vec!["src/**".to_string()]),
      }),
      pull_request: None,
    }));
    let job = workflow.jobs.get("job").unwrap();
    assert_eq!(job.on, on);

    let step = job.steps.get(0).unwrap();
    if let UserStep::Command(command_step) = step {
      assert_eq!(
        command_step.on,
        Some(Condition::Config(ConditionConfig {
          push: None,
          pull_request: Some(PullRequestCondition {
            branches: Some(vec!["master".to_string()]),
            paths: None,
          }),
        }))
      );
    } else {
      panic!("Step should be command step");
    }
  }
}

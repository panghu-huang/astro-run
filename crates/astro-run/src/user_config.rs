use astro_run_shared::{EnvironmentVariables, Error, Id, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserCommandStep {
  pub name: Option<String>,
  pub image: Option<String>,
  pub run: String,
  #[serde(rename = "continue-on-error")]
  pub continue_on_error: Option<bool>,
  pub environments: Option<EnvironmentVariables>,
  pub secrets: Option<Vec<String>>,
  pub volumes: Option<Vec<String>>,
  pub timeout: Option<String>,
  #[serde(rename = "security-opts")]
  pub security_opts: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserActionStep {
  pub name: Option<String>,
  pub uses: String,
  pub with: Option<serde_yaml::Value>,
  #[serde(rename = "continue-on-error")]
  pub continue_on_error: Option<bool>,
  pub environments: Option<EnvironmentVariables>,
  pub secrets: Option<Vec<String>>,
  pub volumes: Option<Vec<String>>,
  pub timeout: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UserStep {
  Command(UserCommandStep),
  Action(UserActionStep),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserJob {
  pub name: Option<String>,
  pub image: Option<String>,
  pub on: Option<WorkflowTriggerEvents>,
  /// Working directory for all steps in this job
  #[serde(rename = "working-directories")]
  pub working_dirs: Option<Vec<String>>,
  pub steps: Vec<UserStep>,
  #[serde(rename = "depends-on")]
  pub depends_on: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserWorkflow {
  pub name: Option<String>,
  pub on: Option<WorkflowTriggerEvents>,
  pub jobs: HashMap<Id, UserJob>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkflowPushEvent {
  pub branches: Option<Vec<String>>,
  pub tags: Option<Vec<String>>,
  pub paths: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkflowPullRequestEvent {
  pub types: Option<Vec<String>>,
  pub branches: Option<Vec<String>>,
  pub paths: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkflowLabelEvent {
  pub types: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct WorkflowTriggerEvents {
  pub push: Option<WorkflowPushEvent>,
  pub pull_request: Option<WorkflowPullRequestEvent>,
  pub label: Option<WorkflowLabelEvent>,
}

impl UserWorkflow {
  pub fn from_str(str: &str) -> Result<Self> {
    let workflow = serde_yaml::from_str(str)
      .map_err(|e| Error::workflow_config_error(format!("Failed to parse workflow: {}", e)))?;

    Self::validate(&workflow)?;

    Ok(workflow)
  }

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

#[cfg(test)]
mod tests {
  use super::*;
  use astro_run_shared::EnvironmentVariable;

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
          number: 1
          boolean: true
        run: echo "Hello World"
      - name: Action step
        uses: cache
"#;

    let workflow = UserWorkflow::from_str(yaml).unwrap();

    assert_eq!(workflow.name, Some("Test Workflow".to_string()));

    assert_eq!(
      workflow.on,
      Some(WorkflowTriggerEvents {
        push: Some(WorkflowPushEvent {
          branches: Some(vec!["master".to_string()]),
          tags: None,
          paths: None,
        }),
        pull_request: None,
        label: None,
      })
    );

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

    let res = UserWorkflow::from_str(yaml);

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

    let res = UserWorkflow::from_str(yaml);
    assert_eq!(
      res.unwrap_err(),
      Error::workflow_config_error("Job job1 depends on job job2, but job job2 is not defined")
    );
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

    let res = UserWorkflow::from_str(yaml);
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

    let res = UserWorkflow::from_str(yaml);
    assert_eq!(
      res.unwrap_err(),
      Error::workflow_config_error("Job `job1` must have at least one step")
    );
  }
}

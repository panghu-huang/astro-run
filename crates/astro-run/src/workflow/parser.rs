use super::{job::Job, Step, Workflow};
use crate::{
  ActionSteps, Actions, AstroRun, Error, Id, JobId, PluginManager, Result, StepId, UserActionStep,
  UserCommandStep, UserStep, UserWorkflow, WorkflowId,
};
use std::collections::HashMap;

pub struct WorkflowParser<'a> {
  pub id: Id,
  pub user_workflow: UserWorkflow,
  pub payload: Option<String>,
  pub astro_run: &'a AstroRun,
}

impl<'a> WorkflowParser<'a> {
  fn try_normalize_action(
    &self,
    plugins: &PluginManager,
    actions: &Actions,
    user_action_step: UserActionStep,
  ) -> crate::Result<ActionSteps> {
    let action_steps = match actions.try_normalize(user_action_step.clone())? {
      Some(steps) => {
        log::trace!("Action `{}` is found and normalized", user_action_step.uses);
        steps
      }
      None => {
        let action = plugins.on_resolve_dynamic_action(user_action_step.clone());

        match action {
          Some(action) => action.normalize(user_action_step)?,
          None => {
            return Err(Error::workflow_config_error(&format!(
              "Action `{}` is not found",
              user_action_step.uses
            )));
          }
        }
      }
    };

    Ok(action_steps)
  }

  fn try_normalize_user_steps(
    &self,
    plugins: &PluginManager,
    actions: &Actions,
    user_steps: Vec<UserStep>,
  ) -> crate::Result<Vec<UserStep>> {
    let mut pre_steps = vec![];
    let mut steps = vec![];
    let mut post_steps = vec![];

    for step in user_steps {
      if let UserStep::Action(user_action_step) = &step {
        let action_steps = self.try_normalize_action(plugins, actions, user_action_step.clone())?;

        if let Some(pre) = action_steps.pre {
          pre_steps.push(pre);
        }

        if let Some(post) = action_steps.post {
          post_steps.insert(0, post)
        }

        steps.push(action_steps.run);
        continue;
      }

      steps.push(step.clone());
    }

    let steps: Vec<UserStep> = vec![]
      .into_iter()
      .chain(pre_steps.into_iter())
      .chain(steps.into_iter())
      .chain(post_steps.into_iter())
      .collect();

    Ok(steps)
  }

  pub fn parse(self) -> Result<Workflow> {
    let id = self.id.clone();
    let user_workflow = self.user_workflow.clone();
    let actions = self.astro_run.actions();
    let plugins = self.astro_run.plugins();

    let mut jobs = HashMap::new();

    for (key, job) in user_workflow.jobs {
      let mut steps = Vec::new();
      let job_container = job.container;
      let job_working_dirs = job.working_dirs.unwrap_or_default();

      let job_steps = self.try_normalize_user_steps(&plugins, &actions, job.steps)?;

      for (idx, step) in job_steps.iter().enumerate() {
        if let UserStep::Command(UserCommandStep {
          name,
          container,
          run,
          continue_on_error,
          environments,
          timeout,
          secrets,
          on,
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
            secrets: secrets.unwrap_or_default(),
            timeout,
            on,
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
          on: job.on,
          steps,
          depends_on: job.depends_on.unwrap_or_default(),
          working_directories: job_working_dirs,
        },
      );
    }

    Ok(Workflow {
      id: WorkflowId::new(id),
      // event: self.event,
      name: user_workflow.name,
      on: user_workflow.on,
      jobs,
      payload: self.payload,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    async_trait, Action, ActionSteps, AstroRun, Context, EnvironmentVariable, Result, RunResponse,
    Runner, UserActionStep,
  };
  use serde::{Deserialize, Serialize};

  struct TestRunner;

  #[async_trait]
  impl Runner for TestRunner {
    async fn run(&self, _ctx: Context) -> RunResponse {
      unreachable!("TestRunner should not be called")
    }
  }

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

  test-job2:
    container: alpine:latest
    steps:
      - run: echo "Hello World2"
      - run: echo "Hello World3"
  "#;

    let user_workflow: UserWorkflow = serde_yaml::from_str(yaml).unwrap();

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow,
      astro_run: &astro_run,
      payload: None,
    };

    let workflow = parser.parse().unwrap();

    assert_eq!(workflow.id, WorkflowId::new("test-id"));
    assert_eq!(workflow.name.unwrap(), "Test Workflow");
    assert_eq!(workflow.jobs.len(), 2);

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

    let job = workflow.jobs.get("test-job2").unwrap();
    assert_eq!(job.steps.len(), 2);

    let step = job.steps.get(0).unwrap();
    assert_eq!(step.run, "echo \"Hello World2\"");

    let step = job.steps.get(1).unwrap();
    assert_eq!(step.run, "echo \"Hello World3\"");
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
    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow,
      astro_run: &astro_run,
      payload: None,
    };

    let workflow = parser.parse();

    let excepted_error =
      Error::workflow_config_error("Invalid timeout format. The format should like `60m` or `1h`.");

    assert_eq!(workflow.unwrap_err(), excepted_error);
  }

  #[test]
  fn test_custom_action() {
    let workflow = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - uses: caches
        with:
          path: /tmp
          key: test
      - run: Hello World
  "#;

    struct CacheAction;

    #[derive(Serialize, Deserialize)]
    struct CacheOptions {
      path: String,
      key: String,
    }

    impl Action for CacheAction {
      fn normalize(&self, step: UserActionStep) -> Result<ActionSteps> {
        let options: CacheOptions = serde_yaml::from_value(step.with.unwrap()).unwrap();
        Ok(ActionSteps {
          pre: Some(UserStep::Command(UserCommandStep {
            name: Some("Pre cache".to_string()),
            run: format!("pre cache {} {}", options.path, options.key),
            ..Default::default()
          })),
          run: UserStep::Command(UserCommandStep {
            name: Some("Restore cache".to_string()),
            run: format!("restore cache {} {}", options.path, options.key),
            ..Default::default()
          }),
          post: Some(UserStep::Command(UserCommandStep {
            name: Some("Save cache".to_string()),
            run: format!("save cache {} {}", options.path, options.key),
            ..Default::default()
          })),
        })
      }
    }

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    astro_run.register_action("caches", CacheAction {});

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow: serde_yaml::from_str(workflow).unwrap(),
      astro_run: &astro_run,
      payload: None,
    };

    let workflow = parser.parse().unwrap();

    let steps = workflow.jobs.get("test").unwrap().steps.clone();

    assert_eq!(steps.len(), 4);

    let step = steps.get(0).unwrap();
    assert_eq!(step.name, Some("Pre cache".to_string()));
    assert_eq!(step.run, "pre cache /tmp test".to_string());

    let step = steps.get(1).unwrap();
    assert_eq!(step.name, Some("Restore cache".to_string()));
    assert_eq!(step.run, "restore cache /tmp test".to_string());

    let step = steps.get(2).unwrap();
    assert_eq!(step.name, None);
    assert_eq!(step.run, "Hello World".to_string());

    let step = steps.get(3).unwrap();
    assert_eq!(step.name, Some("Save cache".to_string()));
    assert_eq!(step.run, "save cache /tmp test".to_string());
  }

  #[test]
  fn unsupported_nested_actions() {
    let workflow = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - uses: nested
  "#;

    struct NestedAction;

    impl Action for NestedAction {
      fn normalize(&self, _step: UserActionStep) -> Result<ActionSteps> {
        Ok(ActionSteps {
          pre: None,
          run: UserStep::Action(UserActionStep {
            ..Default::default()
          }),
          post: None,
        })
      }
    }

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    astro_run.register_action("nested", NestedAction);

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow: serde_yaml::from_str(workflow).unwrap(),
      astro_run: &astro_run,
      payload: None,
    };

    let error = parser.parse().unwrap_err();

    assert_eq!(
      error,
      Error::unsupported_feature("Only command step is supported")
    );
  }

  #[test]
  fn test_not_defined_action() {
    let workflow = r#"
      jobs:
        test:
          steps:
            - uses: not_defined
      "#;

    let astro_run = AstroRun::builder().runner(TestRunner).build();

    let parser = WorkflowParser {
      id: "test-id".to_string(),
      user_workflow: serde_yaml::from_str(workflow).unwrap(),
      astro_run: &astro_run,
      payload: None,
    };

    let error = parser.parse().unwrap_err();

    assert_eq!(
      error,
      Error::workflow_config_error("Action `not_defined` is not found")
    );
  }
}

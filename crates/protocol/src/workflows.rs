use super::*;

impl TryInto<astro_run::Step> for Command {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::Step, Self::Error> {
    let command: astro_run::Command = self.try_into()?;

    Ok(astro_run::Step {
      id: command.id,
      name: command.name,
      run: command.run,
      timeout: command.timeout,
      container: command.container,
      continue_on_error: command.continue_on_error,
      environments: command.environments,
      secrets: command.secrets,
      on: None,
    })
  }
}

impl TryFrom<astro_run::Step> for Command {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Step) -> Result<Self, Self::Error> {
    let command: astro_run::Command = value.into();
    let command = Command::try_from(command)?;

    Ok(Command {
      id: command.id,
      name: command.name,
      run: command.run,
      timeout: command.timeout,
      container: command.container,
      continue_on_error: command.continue_on_error,
      environments: command.environments,
      secrets: command.secrets,
    })
  }
}

impl TryInto<astro_run::Job> for Job {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::Job, Self::Error> {
    Ok(astro_run::Job {
      id: astro_run::JobId::try_from(self.id.as_str())?,
      name: self.name,
      on: None,
      steps: self
        .steps
        .into_iter()
        .map(|s| s.try_into())
        .collect::<Result<_, _>>()?,
      depends_on: self.depends_on,
      working_directories: self.working_directories,
    })
  }
}

impl TryFrom<astro_run::Job> for Job {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Job) -> Result<Self, Self::Error> {
    Ok(Job {
      id: value.id.to_string(),
      name: value.name,
      steps: value
        .steps
        .into_iter()
        .map(|s| s.try_into())
        .collect::<Result<_, _>>()?,
      depends_on: value.depends_on,
      working_directories: value.working_directories,
    })
  }
}

impl From<astro_run::WorkflowEvent> for WorkflowEvent {
  fn from(value: astro_run::WorkflowEvent) -> Self {
    Self {
      event: value.event,
      repo_owner: value.repo_owner,
      repo_name: value.repo_name,
      pr_number: value.pr_number,
      sha: value.sha,
      ref_name: value.ref_name,
      branch: value.branch,
    }
  }
}

impl From<WorkflowEvent> for astro_run::WorkflowEvent {
  fn from(event: WorkflowEvent) -> Self {
    astro_run::WorkflowEvent {
      event: event.event,
      repo_owner: event.repo_owner,
      repo_name: event.repo_name,
      pr_number: event.pr_number,
      sha: event.sha,
      ref_name: event.ref_name,
      branch: event.branch,
    }
  }
}

impl TryInto<astro_run::Workflow> for Workflow {
  type Error = astro_run::Error;

  fn try_into(self) -> Result<astro_run::Workflow, Self::Error> {
    Ok(astro_run::Workflow {
      id: astro_run::WorkflowId::try_from(self.id.as_str())?,
      name: self.name,
      jobs: self
        .jobs
        .into_iter()
        .map(|(id, job)| Ok::<(String, astro_run::Job), Self::Error>((id, job.try_into()?)))
        .collect::<Result<_, _>>()?,
      on: None,
      payload: self.payload,
    })
  }
}

impl TryFrom<astro_run::Workflow> for Workflow {
  type Error = astro_run::Error;

  fn try_from(value: astro_run::Workflow) -> Result<Self, Self::Error> {
    Ok(Workflow {
      id: value.id.to_string(),
      name: value.name,
      jobs: value
        .jobs
        .into_iter()
        .map(|(id, job)| Ok::<(String, Job), Self::Error>((id, job.try_into()?)))
        .collect::<Result<_, _>>()?,
      payload: value.payload,
    })
  }
}

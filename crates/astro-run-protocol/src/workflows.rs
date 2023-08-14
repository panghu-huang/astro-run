use super::*;

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

impl From<astro_run::WorkflowAPIEvent> for WorkflowEvent {
  fn from(value: astro_run::WorkflowAPIEvent) -> Self {
    Self {
      event: value.event,
      repo_owner: value.repo_owner,
      repo_name: value.repo_name,
      pr_number: value.pr_number,
      sha: value.sha,
      ref_name: value.ref_name,
    }
  }
}

impl Into<astro_run::WorkflowAPIEvent> for WorkflowEvent {
  fn into(self) -> astro_run::WorkflowAPIEvent {
    astro_run::WorkflowAPIEvent {
      event: self.event,
      repo_owner: self.repo_owner,
      repo_name: self.repo_name,
      pr_number: self.pr_number,
      sha: self.sha,
      ref_name: self.ref_name,
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
      event: self
        .event
        .map(|e| e.into())
        .ok_or(astro_run::Error::internal_runtime_error(
          "Workflow event is missing",
        ))?,
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
      event: Some(value.event.into()),
    })
  }
}

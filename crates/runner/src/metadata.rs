use astro_run::{Error, Result, StepId};
use std::path::PathBuf;

pub trait PathBufTryToString {
  fn to_string(&self) -> Result<String>;
}

impl PathBufTryToString for PathBuf {
  fn to_string(&self) -> Result<String> {
    self
      .to_str()
      .map(|s| s.to_string())
      .ok_or_else(|| Error::internal_runtime_error("PathBuf to string error"))
  }
}

#[derive(Clone)]
pub struct Metadata {
  /// Step working directory
  pub step_host_working_directory: PathBuf,
  /// Job data directory
  pub job_data_directory: PathBuf,
  /// Workflow cache directory
  pub cache_directory: PathBuf,
  /// Entrypoint path
  pub entrypoint_path: PathBuf,
  /// Docker name
  pub docker_name: String,
  /// Working directory on docker container
  pub docker_working_directory: String,
}

impl Metadata {
  pub fn builder() -> MetadataBuilder {
    MetadataBuilder::new()
  }
}

pub struct MetadataBuilder {
  pub runner_working_directory: Option<PathBuf>,
  pub repository: Option<(String, String)>,
  pub step_id: Option<StepId>,
}

impl MetadataBuilder {
  pub fn new() -> Self {
    Self {
      runner_working_directory: None,
      repository: None,
      step_id: None,
    }
  }

  pub fn runner_working_directory(mut self, runner_working_directory: PathBuf) -> Self {
    self.runner_working_directory = Some(runner_working_directory);
    self
  }

  pub fn repository(mut self, owner: String, name: String) -> Self {
    self.repository = Some((owner, name));
    self
  }

  pub fn step_id(mut self, step_id: StepId) -> Self {
    self.step_id = Some(step_id);
    self
  }

  pub fn build(self) -> Metadata {
    let runner_working_directory = self.runner_working_directory.unwrap();
    let step_id = self.step_id.unwrap();

    let mut repo_working_directory = runner_working_directory;

    if let Some((owner, name)) = self.repository {
      repo_working_directory = repo_working_directory.join(owner).join(name);
    }

    let cache_directory = repo_working_directory.join("caches");

    let workflow_id = step_id.workflow_id().inner();
    let job_key = step_id.job_key();
    let step_number = step_id.step_number();

    let job_data_directory = repo_working_directory
      .join(&workflow_id)
      .join(&job_key)
      .join("data");

    // Step working directory
    let step_host_working_directory = repo_working_directory
      .join(&workflow_id)
      .join(&job_key)
      .join(step_number.to_string());

    let entrypoint_path = step_host_working_directory.join("entrypoint");
    let docker_name = format!("{}-{}-{}", workflow_id, job_key, step_number);
    let docker_working_directory = String::from("/home/runner/work");

    Metadata {
      docker_name,
      step_host_working_directory,
      job_data_directory,
      cache_directory,
      docker_working_directory,
      entrypoint_path,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_directories_builder() {
    let directories = MetadataBuilder::new()
      .runner_working_directory(PathBuf::from("/home/runner/work"))
      .repository("panghu-huang".to_string(), "astro-run".to_string())
      .step_id(StepId::new(
        "workflow-id".to_string(),
        "job-key".to_string(),
        1,
      ))
      .build();

    assert_eq!(
      directories.step_host_working_directory,
      PathBuf::from("/home/runner/work/panghu-huang/astro-run/workflow-id/job-key/1")
    );
    assert_eq!(
      directories.job_data_directory,
      PathBuf::from("/home/runner/work/panghu-huang/astro-run/workflow-id/job-key/data")
    );
    assert_eq!(
      directories.cache_directory,
      PathBuf::from("/home/runner/work/panghu-huang/astro-run/caches")
    );
    assert_eq!(
      directories.entrypoint_path,
      PathBuf::from("/home/runner/work/panghu-huang/astro-run/workflow-id/job-key/1/entrypoint")
    );
    assert_eq!(directories.docker_name, "workflow-id-job-key-1");
    assert_eq!(directories.docker_working_directory, "/home/runner/work");
  }

  #[test]
  fn test_to_string() {
    let path = PathBuf::from("/home/runner/work");
    let path_string = path.to_string().unwrap();
    assert_eq!(path_string, "/home/runner/work");
  }
}

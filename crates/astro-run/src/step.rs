use crate::{docker::Docker, utils, ExecutionContext};
use astro_run_shared::{Command, EnvironmentVariables, Id, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepSecret {
  /// The name of the secret
  pub key: String,
  /// The name of the environment variable to set
  pub env: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepVolume {
  /// The name of the volume
  pub key: String,
  /// The path to mount the volume to
  pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Step {
  pub id: (Id, Id, usize),
  pub name: Option<String>,
  pub image: String,
  pub run: String,
  pub continue_on_error: bool,
  pub working_directory: String,
  pub environments: EnvironmentVariables,
  pub secrets: Vec<StepSecret>,
  pub volumes: Vec<StepVolume>,
  pub timeout: Duration,
  pub security_opts: Option<Vec<String>>,
}

impl Step {
  /**
   * Run the step
   * Step shared the same execution context as the job
   */
  pub async fn run(&self, ctx: &ExecutionContext) -> Result<()> {
    // Runner working directory
    let (working_directory, entrypoint_path, job_data_directory, workflow_cache_directory) =
      self.get_working_paths(ctx);
    let (workflow_id, job_key, step_number) = self.id.clone();

    utils::create_executable_file(&entrypoint_path, self.run.clone()).await?;

    // Generate docker command
    let command = self.into_command(
      entrypoint_path,
      job_data_directory,
      workflow_cache_directory,
    )?;

    // Run the command
    ctx.run(workflow_id, job_key, step_number, command).await?;

    // Kill the container on step run error
    // Command::new("docker")
    //   .arg("kill")
    //   .arg(docker_name)
    //   .exec()
    //   .await?;

    // Clean up working directory
    utils::cleanup_working_directory(&working_directory).await?;

    Ok(())
  }

  fn get_working_paths(&self, ctx: &ExecutionContext) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    // Runner working directory
    let working_directory = ctx.workflow_shared.working_directory.clone();
    let workflow_cache_directory = ctx.workflow_shared.cache_directory.clone();

    let (workflow_id, job_key, step_number) = self.id.clone();
    // Job data directory
    let job_data_directory = working_directory
      .join(&workflow_id)
      .join(&job_key)
      .join("data");

    // Step working directory
    let working_directory = working_directory
      .join(&workflow_id)
      .join(&job_key)
      .join(step_number.to_string());

    let entrypoint_path = working_directory.join("entrypoint.sh");

    (
      working_directory,
      entrypoint_path,
      job_data_directory,
      workflow_cache_directory,
    )
  }

  fn into_command(
    &self,
    entrypoint_path: PathBuf,
    job_data_directory: PathBuf,
    workflow_cache_directory: PathBuf,
  ) -> Result<Command> {
    let (workflow_id, job_key, step_number) = self.id.clone();

    let docker_name = format!("{}-{}-{}", workflow_id, job_key, step_number);

    let docker = Docker::new(&docker_name)
      .image(self.image.clone())
      .working_dir(self.working_directory.clone())
      .volume(
        entrypoint_path.to_string_lossy(),
        "/home/work/runner/entrypoint.sh",
      )
      // Working directory, such as /home/work/runner/{repo}
      .volume(
        job_data_directory.to_string_lossy(),
        self.working_directory.clone(),
      )
      .volume(
        "/home/work/cache",
        workflow_cache_directory.to_string_lossy(),
      )
      .entrypoint("/home/work/runner/entrypoint.sh")
      .auto_remove(true);

    docker.try_into()
  }
}

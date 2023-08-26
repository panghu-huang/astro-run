use crate::{
  command::Command,
  docker::Docker,
  executors::Executor,
  metadata::{Metadata, PathBufTryToString},
  utils,
};
use astro_run::{Context, Result, StreamSender, WorkflowEvent};
use std::path::PathBuf;
use tokio::fs;

pub struct DockerExecutor {
  pub working_directory: PathBuf,
}

#[async_trait::async_trait]
impl Executor for DockerExecutor {
  /**
   * Run the step
   * Step shared the same execution context as the job
   */
  async fn execute(
    &self,
    ctx: Context,
    sender: StreamSender,
    event: Option<WorkflowEvent>,
  ) -> Result<()> {
    // Runner working directory
    let mut builder = Metadata::builder()
      .runner_working_directory(self.working_directory.clone())
      .step_id(ctx.command.id.clone());

    if let Some(event) = event {
      builder = builder.repository(event.repo_owner, event.repo_name);
    }

    let metadata = builder.build();

    // Create step working directory
    fs::create_dir_all(&metadata.step_host_working_directory).await?;
    utils::create_executable_file(&metadata.entrypoint_path, ctx.command.run).await?;

    let image = ctx
      .command
      .container
      .clone()
      .map(|c| c.name)
      .unwrap_or("ubuntu".to_string());

    // Generate docker command
    let mut command = self.into_command(image, metadata.clone())?;

    // Run the command
    if let Err(err) = command.run(sender).await {
      log::error!("Step run error: {}", err);
      // Kill the container on step run error
      Command::new(format!("docker kill {}", metadata.docker_name))
        .exec()
        .await?;
    }

    // Clean up working directory
    fs::remove_dir_all(&metadata.step_host_working_directory).await?;

    log::info!("Step run finished");
    Ok(())
  }
}

impl DockerExecutor {
  fn into_command(&self, image: String, metadata: Metadata) -> Result<Command> {
    let docker = Docker::new(image)
      .name(&metadata.docker_name)
      .working_dir(metadata.docker_working_directory.clone())
      .volume(
        metadata.entrypoint_path.to_string()?,
        "/home/work/runner/entrypoint.sh",
      )
      // Working directory, such as /home/work/{repo}
      .volume(
        metadata.job_data_directory.to_string()?,
        metadata.docker_working_directory,
      )
      .volume(metadata.cache_directory.to_string()?, "/home/work/caches")
      .entrypoint("/home/work/runner/entrypoint.sh")
      .auto_remove(true);

    Ok(docker.into())
  }
}

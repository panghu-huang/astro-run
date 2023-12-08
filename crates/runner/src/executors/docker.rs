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

    // Generate docker command
    let mut command = Self::into_command(ctx.clone(), metadata.clone())?;

    let is_completed = ctx.signal.is_cancelled() || ctx.signal.is_timeout();

    if !is_completed {
      // Create step working directory
      fs::create_dir_all(&metadata.step_host_working_directory).await?;
      utils::create_executable_file(&metadata.entrypoint_path, &ctx.command.run).await?;

      // Run the command
      tokio::select! {
        Err(err) = command.run(sender.clone()) => {
          log::error!("Step run error: {}", err);
        }
        signal = ctx.signal.recv() => {
          log::trace!("Killing running docker: {}", metadata.docker_name);
          // Kill the container on step run error
          Command::new(format!("docker kill {}", metadata.docker_name))
            .exec()
            .await
            .ok();

          log::info!("Step received signal: {:?}", signal);
          if let astro_run::Signal::Cancel = signal {
            sender.cancelled();
          } else {
            sender.timeout();
          }
        }
      }

      // Clean up working directory
      fs::remove_dir_all(&metadata.step_host_working_directory).await?;
      log::trace!("Step run finished");
    } else {
      log::trace!("Step is already completed");
    }

    Ok(())
  }
}

impl DockerExecutor {
  fn into_command(ctx: Context, metadata: Metadata) -> Result<Command> {
    let image = ctx
      .command
      .container
      .clone()
      .map(|c| c.name)
      .unwrap_or("ubuntu".to_string());

    let mut docker = Docker::new(image)
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

    for (key, env) in ctx.command.environments {
      docker = docker.environment(key, env.to_string());
    }

    if let Some(Some(volumes)) = ctx.command.container.as_ref().map(|c| c.volumes.clone()) {
      for volume in volumes {
        if let [host_path, container_path] = volume.split(':').collect::<Vec<&str>>()[..] {
          docker = docker.volume(host_path, container_path);
        }
      }
    }

    if let Some(Some(security_opts)) = ctx.command.container.map(|c| c.security_opts) {
      for security_opt in security_opts {
        docker = docker.security_opt(security_opt);
      }
    }

    Ok(docker.into())
  }
}

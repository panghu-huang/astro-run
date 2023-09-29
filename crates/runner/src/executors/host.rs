use crate::{command::Command, executors::Executor, metadata::Metadata};
use astro_run::{Context, Result, StreamSender, WorkflowEvent};
use std::path::PathBuf;
use tokio::fs;

pub struct HostExecutor {
  pub working_directory: PathBuf,
}

#[async_trait::async_trait]
impl Executor for HostExecutor {
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
    fs::create_dir_all(&metadata.job_data_directory).await?;
    // utils::create_executable_file(&metadata.entrypoint_path, ctx.command.run).await?;

    // Generate docker command
    let mut command = self.into_command(ctx, metadata.clone())?;

    // Run the command
    if let Err(err) = command.run(sender).await {
      log::error!("Step run error: {}", err);
    }

    // Clean up working directory
    // fs::remove_dir_all(&metadata.step_host_working_directory).await?;

    log::trace!("Step run finished");

    Ok(())
  }
}

impl HostExecutor {
  fn into_command(&self, ctx: Context, metadata: Metadata) -> Result<Command> {
    let original_command = ctx.command.clone();
    let mut command = Command::new(original_command.run);

    command.dir(&metadata.job_data_directory);

    for (key, env) in original_command.environments {
      command.env(key, env.to_string());
    }

    Ok(command)
  }
}

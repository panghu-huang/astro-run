use crate::{command::Command, executors::Executor, metadata::Metadata};
use astro_run::{Context, Result, StreamSender, WorkflowEvent};
use std::path::PathBuf;
use tokio::fs;

pub struct HostExecutor {
  pub working_directory: PathBuf,
}

#[astro_run::async_trait]
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

    // Generate docker command
    let mut command = Self::into_command(&ctx, &metadata);

    let is_completed = ctx.signal.is_cancelled() || ctx.signal.is_timeout();

    if !is_completed {
      // Create step working directory
      fs::create_dir_all(&metadata.job_data_directory).await?;

      tokio::select! {
        // Run the command
        Err(err) = command.run(sender.clone()) => {
          log::error!("Step run error: {}", err);
        }
        signal = ctx.signal.recv() => {
          // TODO: cancel the command
          log::trace!("Step run received signal: {:?}", signal);
          if let astro_run::Signal::Cancel = signal {
            sender.cancelled();
          } else {
            sender.timeout();
          }
        }
      }

      // Clean up working directory
      fs::remove_dir_all(&metadata.job_data_directory).await?;

      log::trace!("Step run finished");
    } else {
      log::trace!("Step run has been completed before it started");
    }

    Ok(())
  }
}

impl HostExecutor {
  fn into_command(ctx: &Context, metadata: &Metadata) -> Command {
    let mut command = Command::new(ctx.command.run.clone());

    command.dir(&metadata.job_data_directory);

    for (key, env) in &ctx.command.environments {
      command.env(key, env.to_string());
    }

    command
  }
}

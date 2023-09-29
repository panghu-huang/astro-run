mod docker;
mod host;

use astro_run::{Context, Result, StreamSender, WorkflowEvent};
pub use docker::DockerExecutor;
pub use host::HostExecutor;

#[async_trait::async_trait]
pub trait Executor: Send + Sync {
  async fn execute(
    &self,
    ctx: Context,
    sender: StreamSender,
    event: Option<WorkflowEvent>,
  ) -> Result<()>;
}

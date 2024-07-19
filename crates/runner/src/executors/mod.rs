mod docker;
mod host;

use astro_run::{Context, Result, StreamSender, TriggerEvent};
pub use docker::DockerExecutor;
pub use host::HostExecutor;

#[astro_run::async_trait]
pub trait Executor: Send + Sync {
  async fn execute(
    &self,
    ctx: Context,
    sender: StreamSender,
    event: Option<TriggerEvent>,
  ) -> Result<()>;
}

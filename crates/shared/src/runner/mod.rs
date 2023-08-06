mod response;
mod stream;

pub use self::{
  response::{ExitStatus, Log, StreamResponse},
  stream::{stream, StreamReceiver, StreamSender},
};
use crate::Command;
pub use tokio_stream::{Stream, StreamExt};

pub type LogStream = crate::Result<StreamReceiver>;

pub trait Runner {
  fn run(&self, command: Command) -> LogStream;
}

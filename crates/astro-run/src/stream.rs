use super::{Log, RunResult};
use parking_lot::Mutex;
use std::{sync::Arc, task::Waker};
use tokio_stream::Stream;

struct SharedState {
  logs: Vec<Log>,
  result: Option<RunResult>,
  waker: Option<Waker>,
}

pub struct StreamReceiver {
  current_index: Mutex<usize>,
  state: Arc<Mutex<SharedState>>,
}

impl StreamReceiver {
  fn new(state: Arc<Mutex<SharedState>>) -> Self {
    Self {
      current_index: Mutex::new(0),
      state,
    }
  }

  pub fn result(&self) -> Option<RunResult> {
    self.state.lock().result.clone()
  }
}

impl Stream for StreamReceiver {
  type Item = Log;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let mut state = self.state.lock();
    state.waker = Some(cx.waker().clone());

    let logs = state.logs.clone();
    let total = logs.len();
    let current_index = self.current_index.lock().clone();

    if current_index < total {
      let log = logs[current_index].clone();
      *self.current_index.lock() += 1;

      cx.waker().wake_by_ref();

      return std::task::Poll::Ready(Some(log));
    }

    if state.result.is_some() {
      return std::task::Poll::Ready(None);
    }

    std::task::Poll::Pending
  }
}

#[derive(Clone)]
pub struct StreamSender {
  state: Arc<Mutex<SharedState>>,
}

impl StreamSender {
  fn new(state: Arc<Mutex<SharedState>>) -> Self {
    Self { state }
  }

  pub fn log(&self, message: impl Into<String>) {
    let mut state = self.state.lock();
    state.logs.push(Log::log(message.into()));

    if let Some(waker) = state.waker.take() {
      waker.wake();
    }
  }

  pub fn error(&self, message: impl Into<String>) {
    let mut state = self.state.lock();
    state.logs.push(Log::error(message.into()));

    if let Some(waker) = state.waker.take() {
      waker.wake();
    }
  }

  pub fn succeeded(&self) {
    self.end(RunResult::Succeeded)
  }

  pub fn cancelled(&self) {
    self.end(RunResult::Cancelled)
  }

  pub fn failed(&self, exit_code: i32) {
    self.end(RunResult::Failed { exit_code })
  }

  pub fn timeout(&self) {
    // TODO: use a different exit code
    self.end(RunResult::Failed { exit_code: 123 })
  }

  pub fn end(&self, result: RunResult) {
    let mut state = self.state.lock();
    state.result = Some(result);

    if let Some(waker) = state.waker.take() {
      waker.wake();
    }
  }

  pub fn is_ended(&self) -> bool {
    self.state.lock().result.is_some()
  }
}

pub fn stream() -> (StreamSender, StreamReceiver) {
  let state = Arc::new(Mutex::new(SharedState {
    logs: Vec::new(),
    waker: None,
    result: None,
  }));

  let sender = StreamSender::new(state.clone());
  let receiver = StreamReceiver::new(state);

  (sender, receiver)
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokio_stream::StreamExt;

  #[tokio::test]
  async fn test_stream() {
    let (sender, mut receiver) = stream();

    sender.log("test");
    sender.error("error");
    sender.succeeded();

    let mut logs = Vec::new();
    while let Some(log) = receiver.next().await {
      logs.push(log);
    }

    assert_eq!(logs, vec![Log::log("test"), Log::error("error"),]);
    assert_eq!(receiver.result().unwrap(), RunResult::Succeeded);
  }
}

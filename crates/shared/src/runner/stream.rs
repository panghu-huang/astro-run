use super::response::{Log, StreamResponse};
use parking_lot::Mutex;
use std::{process::ExitStatus, sync::Arc, task::Waker};

use tokio_stream::Stream;

struct SharedState {
  responses: Vec<StreamResponse>,
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
}

impl Stream for StreamReceiver {
  type Item = StreamResponse;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let mut state = self.state.lock();
    state.waker = Some(cx.waker().clone());

    let responses = state.responses.clone();
    let total = responses.len();
    let current_index = self.current_index.lock().clone();

    if current_index < total {
      let response = responses[current_index].clone();
      *self.current_index.lock() += 1;

      cx.waker().wake_by_ref();

      return std::task::Poll::Ready(Some(response));
    }

    if current_index == total && total > 0 {
      return std::task::Poll::Ready(None);
    }

    std::task::Poll::Pending
  }
}

pub struct StreamSender {
  state: Arc<Mutex<SharedState>>,
}

impl StreamSender {
  fn new(state: Arc<Mutex<SharedState>>) -> Self {
    Self { state }
  }

  pub fn receiver(&self) -> StreamReceiver {
    StreamReceiver {
      current_index: Mutex::new(0),
      state: self.state.clone(),
    }
  }

  pub fn log(&self, log: Log) {
    let mut state = self.state.lock();
    state.responses.push(StreamResponse::Log(log));

    if let Some(waker) = state.waker.take() {
      waker.wake();
    }
  }

  pub fn end(&self, exit_status: ExitStatus) {
    let mut state = self.state.lock();
    state.responses.push(StreamResponse::End(exit_status));

    if let Some(waker) = state.waker.take() {
      waker.wake();
    }
  }
}

pub fn stream() -> (StreamSender, StreamReceiver) {
  let state = Arc::new(Mutex::new(SharedState {
    responses: Vec::new(),
    waker: None,
  }));

  let sender = StreamSender::new(state.clone());
  let receiver = StreamReceiver::new(state);

  (sender, receiver)
}

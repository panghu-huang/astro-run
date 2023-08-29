use parking_lot::Mutex;
use std::{
  future::Future,
  sync::Arc,
  task::{Context, Poll, Waker},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
  Cancel,
  Timeout,
}

#[derive(Debug)]
struct SignalState {
  signal: Option<Signal>,
  waker: Option<Waker>,
}

pub struct Receiver<'a> {
  signal: &'a AstroRunSignal,
}

#[derive(Clone, Debug)]
pub struct AstroRunSignal {
  state: Arc<Mutex<SignalState>>,
}

impl AstroRunSignal {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(SignalState {
        signal: None,
        waker: None,
      })),
    }
  }

  pub fn recv(&self) -> Receiver {
    let receiver = Receiver { signal: self };

    receiver
  }

  pub fn cancel(&self) {
    let mut state = self.state.lock();
    // assert!(state.signal.is_none(), "Signal can only be set once.");

    state.signal = Some(Signal::Cancel);

    state.waker.take().map(|waker| waker.wake());
  }

  pub fn timeout(&self) {
    let mut state = self.state.lock();
    // assert!(state.signal.is_none(), "Signal can only be set once.");

    state.signal = Some(Signal::Timeout);

    state.waker.take().map(|waker| waker.wake());
  }

  pub fn is_cancelled(&self) -> bool {
    self.state.lock().signal == Some(Signal::Cancel)
  }

  pub fn is_timeout(&self) -> bool {
    self.state.lock().signal == Some(Signal::Timeout)
  }
}

impl<'a> Future for Receiver<'a> {
  type Output = Signal;

  fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.signal.state.lock();

    if let Some(signal) = state.signal {
      Poll::Ready(signal)
    } else {
      state.waker = Some(cx.waker().clone());
      Poll::Pending
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // #[test]
  // #[should_panic(expected = "Signal can only be set once.")]
  // fn test_set_signal_twice() {
  //   let signal = AstroRunSignal::new();
  //   assert_eq!(signal.is_cancelled(), false);
  //   assert_eq!(signal.is_timeout(), false);

  //   signal.cancel();
  //   assert_eq!(signal.is_cancelled(), true);
  //   assert_eq!(signal.is_timeout(), false);

  //   signal.timeout();
  // }

  #[astro_run_test::test]
  async fn test_wait_for_cancel_signal() {
    let signal = AstroRunSignal::new();
    assert_eq!(signal.is_cancelled(), false);
    assert_eq!(signal.is_timeout(), false);

    let receiver = signal.recv();

    let cloned_signal = signal.clone();

    tokio::spawn(async move {
      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      cloned_signal.cancel();
    });

    assert_eq!(receiver.await, Signal::Cancel);
    assert_eq!(signal.is_cancelled(), true);
    assert_eq!(signal.is_timeout(), false);
  }

  #[astro_run_test::test]
  async fn test_wait_for_timeout_signal() {
    let signal = AstroRunSignal::new();
    assert_eq!(signal.is_cancelled(), false);
    assert_eq!(signal.is_timeout(), false);

    let receiver = signal.recv();

    let cloned_signal = signal.clone();

    tokio::spawn(async move {
      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
      cloned_signal.timeout();
    });

    assert_eq!(receiver.await, Signal::Timeout);
    assert_eq!(signal.is_cancelled(), false);
    assert_eq!(signal.is_timeout(), true);
  }
}

use crate::{Error, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
  future::Future,
  sync::Arc,
  task::{Context, Poll, Waker},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
  Cancel,
  Timeout,
}

#[derive(Debug)]
struct SignalState {
  is_notified: bool,
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
    Self::default()
  }

  pub fn recv(&self) -> Receiver {
    Receiver { signal: self }
  }

  pub fn cancel(&self) -> Result<()> {
    let mut state = self.state.lock();
    if state.signal.is_some() {
      return Err(Error::error("Signal has been cancelled or timeout."));
    }

    state.signal = Some(Signal::Cancel);

    if let Some(waker) = state.waker.take() {
      waker.wake()
    }

    Ok(())
  }

  pub fn timeout(&self) -> Result<()> {
    let mut state = self.state.lock();

    if state.signal.is_some() {
      return Err(Error::error("Signal has been cancelled or timeout."));
    }

    state.signal = Some(Signal::Timeout);

    if let Some(waker) = state.waker.take() {
      waker.wake()
    }

    Ok(())
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

    if state.is_notified {
      return Poll::Pending;
    }

    if let Some(signal) = state.signal {
      state.is_notified = true;

      Poll::Ready(signal)
    } else {
      state.waker = Some(cx.waker().clone());
      Poll::Pending
    }
  }
}

impl std::fmt::Display for Signal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Signal::Cancel => write!(f, "cancel"),
      Signal::Timeout => write!(f, "timeout"),
    }
  }
}

impl From<&str> for Signal {
  fn from(s: &str) -> Self {
    match s {
      "cancel" => Signal::Cancel,
      "timeout" => Signal::Timeout,
      _ => panic!("Invalid signal: {}", s),
    }
  }
}

impl Default for AstroRunSignal {
  fn default() -> Self {
    Self {
      state: Arc::new(Mutex::new(SignalState {
        signal: None,
        waker: None,
        is_notified: false,
      })),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::pin::Pin;

  #[test]
  fn test_set_signal_twice() {
    let signal = AstroRunSignal::new();
    assert_eq!(signal.is_cancelled(), false);
    assert_eq!(signal.is_timeout(), false);

    signal.cancel().unwrap();
    assert_eq!(signal.is_cancelled(), true);
    assert_eq!(signal.is_timeout(), false);

    let err = signal.timeout().unwrap_err();

    assert_eq!(err, Error::error("Signal has been cancelled or timeout."));

    let err = signal.cancel().unwrap_err();
    assert_eq!(err, Error::error("Signal has been cancelled or timeout."));
  }

  #[astro_run_test::test]
  async fn test_wait_for_cancel_signal() {
    let signal = AstroRunSignal::new();
    assert_eq!(signal.is_cancelled(), false);
    assert_eq!(signal.is_timeout(), false);

    let receiver = signal.recv();

    tokio::spawn({
      let signal = signal.clone();
      async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        signal.cancel().unwrap();
      }
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

    tokio::spawn({
      let signal = signal.clone();
      async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        signal.timeout().unwrap();
      }
    });

    assert_eq!(receiver.await, Signal::Timeout);
    assert_eq!(signal.is_cancelled(), false);
    assert_eq!(signal.is_timeout(), true);
  }

  #[astro_run_test::test]
  fn to_string() {
    assert_eq!(Signal::Cancel.to_string(), "cancel".to_string());
    assert_eq!(Signal::Timeout.to_string(), "timeout".to_string());
  }

  #[astro_run_test::test]
  fn from_str() {
    assert_eq!(Signal::from("cancel"), Signal::Cancel);
    assert_eq!(Signal::from("timeout"), Signal::Timeout);
  }

  #[astro_run_test::test]
  async fn test_wait_signal_twice() {
    std::future::poll_fn(|cx| {
      let signal = AstroRunSignal::new();
      assert_eq!(signal.is_cancelled(), false);
      assert_eq!(signal.is_timeout(), false);

      signal.cancel().unwrap();

      let receiver = &mut signal.recv();
      let mut receiver = Pin::new(receiver);
      let res = receiver.as_mut().poll(cx);

      assert_eq!(res, Poll::Ready(Signal::Cancel));

      assert_eq!(signal.is_cancelled(), true);
      assert_eq!(signal.is_timeout(), false);

      let res = receiver.poll(cx);

      assert_eq!(res, Poll::Pending);
      Poll::Ready(())
    })
    .await;
  }

  #[test]
  #[should_panic(expected = "Invalid signal: invalid")]
  fn test_invalid_signal() {
    let _ = Signal::from("invalid");
  }
}

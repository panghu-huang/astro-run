use astro_run_shared::{Error, Log, Result};
use parking_lot::Mutex;
use std::{
  path::PathBuf,
  process::{ExitStatus, Stdio},
  sync::Arc,
  task::Waker,
};
use tokio::{
  io::{AsyncBufReadExt, BufReader},
  process::Command as Cmd,
};
use tokio_stream::Stream;

struct State {
  logs: Vec<Log>,
  exit_status: Option<ExitStatus>,
  waker: Option<Waker>,
}

pub struct Receiver {
  current_index: Mutex<usize>,
  state: Arc<Mutex<State>>,
}

impl Stream for Receiver {
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

    if state.exit_status.is_some() {
      return std::task::Poll::Ready(None);
    }

    std::task::Poll::Pending
  }
}

impl Receiver {
  pub fn exit_status(&self) -> Option<ExitStatus> {
    self.state.lock().exit_status
  }
}

pub struct Command {
  command: Cmd,
}

impl Command {
  pub fn new(cmd: impl Into<String>) -> Self {
    if cfg!(target_os = "windows") {
      Command::powershell(cmd)
    } else {
      Command::sh(cmd)
    }
  }

  pub fn powershell(cmd: impl Into<String>) -> Self {
    let cmd: String = cmd.into();
    let mut command = Cmd::new("powershell.exe");

    command
      .arg("-NoProfile")
      .arg("-NonInteractive")
      .arg("-Command")
      .arg(cmd);

    Command { command }
  }

  pub fn sh(cmd: impl Into<String>) -> Self {
    let cmd: String = cmd.into();
    let mut command = Cmd::new("sh");

    command.arg("-c").arg(cmd);

    Command { command }
  }

  pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
    self.command.env(key.into(), value.into());

    self
  }

  pub fn dir(&mut self, dir: &PathBuf) -> &mut Self {
    self.command.current_dir(dir);

    self
  }

  pub fn arg<S>(&mut self, arg: S) -> &mut Self
  where
    S: AsRef<std::ffi::OsStr>,
  {
    self.command.arg(arg);

    self
  }

  pub async fn exec(&mut self) -> Result<String> {
    let output = self.command.output().await.map_err(|err| {
      Error::internal_runtime_error(format!("Failed to spawn child process: {}", err))
    })?;

    if output.status.success() {
      let stdout = String::from_utf8(output.stdout)
        .map_err(|err| Error::internal_runtime_error(format!("Failed to parse stdout: {}", err)))?;
      return Ok(stdout.trim().to_string());
    }

    let stderr = String::from_utf8(output.stderr)
      .map_err(|err| Error::internal_runtime_error(format!("Failed to parse stderr: {}", err)))?;

    Err(Error::internal_runtime_error(stderr))
  }

  pub fn run(&mut self) -> Result<Receiver> {
    let mut child = self
      .command
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|err| {
        Error::internal_runtime_error(format!("Failed to spawn child process: {}", err))
      })?;

    let state = Arc::new(Mutex::new(State {
      logs: vec![],
      exit_status: None,
      waker: None,
    }));

    let receiver = Receiver {
      current_index: Mutex::new(0),
      state: state.clone(),
    };

    let _res = tokio::task::spawn(async move {
      let out = child.stdout.take().ok_or(Error::internal_runtime_error(
        "Failed to get stdout from child process".to_string(),
      ))?;
      let err = child.stderr.take().ok_or(Error::internal_runtime_error(
        "Failed to get stderr from child process".to_string(),
      ))?;

      let out = BufReader::new(out);
      let err = BufReader::new(err);

      let mut lines = out.lines();
      let mut errors = err.lines();

      // Add a log to the state
      let add_log = |log: Log| {
        let mut state = state.lock();
        state.logs.push(log);

        if let Some(waker) = state.waker.take() {
          waker.wake();
        }
      };

      loop {
        tokio::select! {
          line = lines.next_line() => {
            match line {
              Ok(Some(line)) => {
                let log = Log::log(line);
                add_log(log);
              }
              Ok(None) => {
                  break;
              }
              Err(err) => {
                add_log(Log::error(err.to_string()));
                break;
              }
            }
          }
          error = errors.next_line() => {
            match error {
              Ok(Some(error)) => {
                add_log(Log::error(error));
              }
              Ok(None) => {
                break;
              }
              Err(err) => {
                add_log(Log::error(err.to_string()));
                break;
              }
            }
          }
        }
      }

      let status = child.wait().await.map_err(|err| {
        Error::internal_runtime_error(format!("Failed to wait for child process: {}", err))
      })?;

      let mut state = state.lock();
      state.exit_status = Some(status);
      if let Some(waker) = state.waker.take() {
        waker.wake();
      }

      Ok(()) as Result<()>
    });

    Ok(receiver)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokio_stream::StreamExt;

  #[tokio::test]
  async fn test_command() {
    let mut cmd = Command::new("echo hello");
    let mut receiver = cmd.run().unwrap();

    let mut logs = vec![];

    while let Some(log) = receiver.next().await {
      logs.push(log);
    }

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "hello");

    let exit_status = receiver.exit_status();

    assert!(exit_status.unwrap().success());
  }
}

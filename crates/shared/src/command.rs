use crate::{stream, Error, Log, Result, StreamReceiver};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Stdio};
use tokio::{
  io::{AsyncBufReadExt, BufReader},
  process::Command as Cmd,
};

/// A command to be executed by the runner.
///
/// And also can be send via Network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
  command: String,
  current_dir: Option<PathBuf>,
  envs: Vec<(String, String)>,
  args: Vec<String>,
}

impl Command {
  pub fn new(cmd: impl Into<String>) -> Self {
    Self {
      command: cmd.into(),
      current_dir: None,
      envs: vec![],
      args: vec![],
    }
  }

  pub fn env(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
    self.envs.push((key.into(), value.into()));

    self
  }

  pub fn dir(&mut self, dir: &PathBuf) -> &mut Self {
    self.current_dir = Some(dir.clone());

    self
  }

  pub fn arg<S>(&mut self, arg: S) -> &mut Self
  where
    S: Into<String>,
  {
    self.args.push(arg.into());

    self
  }

  pub async fn exec(&mut self) -> Result<String> {
    let mut command = self.build_command();
    let output = command.output().await.map_err(|err| {
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

  pub fn run(&mut self) -> Result<StreamReceiver> {
    let mut command = self.build_command();
    let mut child = command
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|err| {
        Error::internal_runtime_error(format!("Failed to spawn child process: {}", err))
      })?;

    let (sender, receiver) = stream();

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

      loop {
        tokio::select! {
          line = lines.next_line() => {
            match line {
              Ok(Some(line)) => {
                let log = Log::log(line);
                sender.log(log);
              }
              Ok(None) => {
                  break;
              }
              Err(err) => {
                sender.log(Log::error(err.to_string()));
                break;
              }
            }
          }
          error = errors.next_line() => {
            match error {
              Ok(Some(error)) => {
                sender.log(Log::error(error));
              }
              Ok(None) => {
                break;
              }
              Err(err) => {
                sender.log(Log::error(err.to_string()));
                break;
              }
            }
          }
        }
      }

      let status = child.wait().await.map_err(|err| {
        Error::internal_runtime_error(format!("Failed to wait for child process: {}", err))
      })?;

      sender.end(status);

      Ok(()) as Result<()>
    });

    Ok(receiver)
  }

  fn build_command(&self) -> Cmd {
    let mut command;

    if cfg!(target_os = "windows") {
      command = Cmd::new("powershell.exe");

      command
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(self.command.clone());
    } else {
      command = Cmd::new("sh");

      command.arg("-c").arg(self.command.clone());
    }

    if let Some(dir) = &self.current_dir {
      command.current_dir(dir);
    }

    for (key, value) in &self.envs {
      command.env(key, value);
    }

    for arg in &self.args {
      command.arg(arg);
    }

    command
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::StreamResponse;
  use tokio_stream::StreamExt;

  #[tokio::test]
  async fn test_command() {
    let mut cmd = Command::new("echo hello");
    let mut receiver = cmd.run().unwrap();

    let mut logs = vec![];

    while let Some(res) = receiver.next().await {
      match res {
        StreamResponse::Log(log) => {
          logs.push(log);
        }
        StreamResponse::End(exit_status) => {
          assert!(exit_status.success());
          break;
        }
      }
    }

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "hello");
  }

  #[tokio::test]
  async fn test_command_with_env() {
    let command = if cfg!(target_os = "windows") {
      "echo $env:HELLO"
    } else {
      "echo $HELLO"
    };
    let mut cmd = Command::new(command);
    cmd.env("HELLO", "world");
    let mut receiver = cmd.run().unwrap();

    let mut logs = vec![];

    while let Some(res) = receiver.next().await {
      match res {
        StreamResponse::Log(log) => {
          logs.push(log);
        }
        StreamResponse::End(exit_status) => {
          assert!(exit_status.success());
          break;
        }
      }
    }

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "world");
  }

  #[tokio::test]
  async fn test_exec_command() {
    let mut cmd = Command::new("echo hello");
    let stdout = cmd.exec().await.unwrap();

    assert_eq!(stdout, "hello");
  }
}

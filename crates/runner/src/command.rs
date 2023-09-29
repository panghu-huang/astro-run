use astro_run::{Error, Result, RunResult, StreamSender};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Stdio};
use tokio::{
  io::{AsyncBufReadExt, BufReader},
  process::Command as Cmd,
};

/// A command to be executed by the runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
  pub command: String,
  pub current_dir: Option<PathBuf>,
  pub envs: Vec<(String, String)>,
}

impl Command {
  pub fn new(cmd: impl Into<String>) -> Self {
    Self {
      command: cmd.into(),
      current_dir: None,
      envs: vec![],
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

  pub async fn run(&mut self, sender: StreamSender) -> Result<()> {
    let mut command = self.build_command();
    let mut child = command
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|err| {
        Error::internal_runtime_error(format!("Failed to spawn child process: {}", err))
      })?;

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
              sender.log(line);
            }
            Ok(None) => {
                break;
            }
            Err(err) => {
              sender.error(err.to_string());
              break;
            }
          }
        }
        error = errors.next_line() => {
          match error {
            Ok(Some(error)) => {
              sender.error(error);
            }
            Ok(None) => {
              break;
            }
            Err(err) => {
              sender.error(err.to_string());
              break;
            }
          }
        }
      }
    }

    let status = child.wait().await.map_err(|err| {
      Error::internal_runtime_error(format!("Failed to wait for child process: {}", err))
    })?;

    let res = status
      .code()
      .map(|code| {
        if code == 0 {
          RunResult::Succeeded
        } else {
          RunResult::Failed { exit_code: code }
        }
      })
      .unwrap_or_else(|| RunResult::Failed { exit_code: 1 });

    sender.end(res);

    Ok(())
  }

  fn build_command(&self) -> Cmd {
    let mut command;

    #[cfg(target_os = "windows")]
    {
      command = Cmd::new("powershell.exe");

      command
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(self.command.clone());
    }
    #[cfg(not(target_os = "windows"))]
    {
      command = Cmd::new("sh");

      command.arg("-c").arg(self.command.clone());
    }

    if let Some(dir) = &self.current_dir {
      command.current_dir(dir);
    }

    for (key, value) in &self.envs {
      command.env(key, value);
    }

    command
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use astro_run::stream;
  use tokio_stream::StreamExt;

  #[tokio::test]
  async fn test_command() {
    let mut cmd = Command::new("echo hello");
    let (sender, mut receiver) = stream();

    let mut logs = vec![];

    tokio::join!(
      async {
        while let Some(log) = receiver.next().await {
          logs.push(log);
        }
      },
      async {
        cmd.run(sender).await.unwrap();
      }
    );

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "hello");

    assert_eq!(receiver.result().unwrap(), RunResult::Succeeded);
  }

  #[tokio::test]
  async fn test_command_with_env() {
    let command = if cfg!(target_os = "windows") {
      "echo $env:HELLO"
    } else {
      "echo ${HELLO}"
    };
    let mut cmd = Command::new(command);
    cmd.env("HELLO", "world");
    let (sender, mut receiver) = stream();

    let mut logs = vec![];

    tokio::join!(
      async {
        while let Some(log) = receiver.next().await {
          logs.push(log);
        }
      },
      async {
        cmd.run(sender).await.unwrap();
      }
    );

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "world");
  }

  #[tokio::test]
  async fn test_exec_command() {
    let mut cmd = Command::new("echo hello");
    let stdout = cmd.exec().await.unwrap();

    assert_eq!(stdout, "hello");
  }

  #[astro_run_test::test]
  async fn test_stderr_command() {
    let mut cmd = Command::new("cd /not/exist");

    let (sender, mut receiver) = stream();
    let mut logs = vec![];

    tokio::join!(
      async {
        while let Some(log) = receiver.next().await {
          logs.push(log);
        }
      },
      async {
        cmd.run(sender).await.unwrap();
      }
    );

    assert_eq!(logs.len(), 1);
    assert!(logs[0].is_error());
    assert!(matches!(
      receiver.result().unwrap(),
      RunResult::Failed { .. }
    ));
  }

  #[astro_run_test::test]
  async fn test_exec_stderr_command() {
    let mut cmd = Command::new("cd /not/exist");

    let res = cmd.exec().await;

    assert!(res.is_err());
  }
}

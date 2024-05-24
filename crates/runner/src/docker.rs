use crate::command::Command;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Docker {
  pub image: String,
  pub name: Option<String>,
  pub environments: HashMap<String, String>,
  pub working_dir: Option<String>,
  pub entrypoint: Option<String>,
  pub volumes: Vec<String>,
  pub auto_remove: bool,
  pub security_opts: Vec<String>,
}

impl Docker {
  pub fn new(image: impl Into<String>) -> Self {
    Self {
      image: image.into(),
      name: None,
      environments: HashMap::new(),
      working_dir: None,
      entrypoint: None,
      volumes: Vec::new(),
      security_opts: Vec::new(),
      auto_remove: true,
    }
  }

  pub fn name(mut self, name: impl Into<String>) -> Self {
    self.name = Some(name.into());
    self
  }

  pub fn auto_remove(mut self, auto_remove: bool) -> Self {
    self.auto_remove = auto_remove;
    self
  }

  pub fn environment(mut self, key: String, value: String) -> Self {
    self.environments.insert(key, value);
    self
  }

  pub fn working_dir(mut self, working_dir: impl Into<String>) -> Self {
    self.working_dir = Some(working_dir.into());
    self
  }

  pub fn entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
    self.entrypoint = Some(entrypoint.into());
    self
  }

  pub fn security_opt(mut self, security_opt: String) -> Self {
    self.security_opts.push(security_opt);
    self
  }

  pub fn volume(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
    self
      .volumes
      .push(format!("\"{}:{}\"", from.into(), to.into()));
    self
  }

  fn generate_docker_command(&self) -> String {
    let mut docker_command: Vec<String> = ["docker", "run", "--tty"]
      .iter()
      .map(|item| item.to_string())
      .collect();

    if self.auto_remove {
      docker_command.push("--rm".to_string());
    }

    for security_opt in &self.security_opts {
      docker_command.push("--security-opt".to_string());
      docker_command.push(security_opt.to_string());
    }

    for volume in &self.volumes {
      docker_command.push("-v".to_string());
      docker_command.push(volume.to_string());
    }

    for (key, value) in &self.environments {
      docker_command.push("-e".to_string());
      docker_command.push(format!("{}=\"{}\"", key, value));
    }

    if let Some(working_dir) = &self.working_dir {
      docker_command.push("-w".to_string());
      docker_command.push(working_dir.to_string());
    }

    if let Some(entrypoint) = &self.entrypoint {
      docker_command.push("--entrypoint".to_string());
      docker_command.push(entrypoint.to_string());
    }

    if let Some(name) = &self.name {
      docker_command.push("--name".to_string());
      docker_command.push(name.to_string());
    }

    docker_command.push(self.image.clone());

    docker_command.join(" ")
  }
}

impl From<Docker> for Command {
  fn from(docker: Docker) -> Self {
    let command = docker.generate_docker_command();

    Command::new(command)
  }
}

#[cfg(test)]
mod tests {
  use super::Docker;

  #[test]
  fn test_generate_docker_command() {
    let common = Docker::new("ubuntu")
      .name("test")
      .environment("key".to_string(), "value".to_string())
      .working_dir("/home/runner/work".to_string())
      .entrypoint("entrypoint".to_string())
      .volume("/app".to_string(), "/home/runner/work".to_string())
      .security_opt("seccomp=unconfined".to_string())
      .generate_docker_command();

    assert_eq!(
      common,
      "docker run --tty --rm --security-opt seccomp=unconfined -v \"/app:/home/runner/work\" -e key=\"value\" -w /home/runner/work --entrypoint entrypoint --name test ubuntu"
    );
  }
}

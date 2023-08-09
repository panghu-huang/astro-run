// use octocrate::GithubError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Failed to parse user config: {0}")]
  WorkflowConfigError(String),

  #[error("Error while running workflow: {0}")]
  InternalRuntimeError(String),

  #[error("Failed with exit code: {0:?}")]
  Failed(usize),

  #[error("IO error: {message}")]
  IOError {
    source: std::io::Error,
    message: String,
  },

  #[error("Github API error: {message}")]
  GithubError {
    // source: GithubError,
    message: String,
  },

  #[error("Failed to initialize workflow: {0}")]
  InitError(String),

  #[error("Unsupported feature: {0}")]
  UnsupportedFeature(String),
}

impl Error {
  pub fn workflow_config_error<T: ToString>(message: T) -> Self {
    Self::WorkflowConfigError(message.to_string())
  }

  pub fn internal_runtime_error<T: ToString>(message: T) -> Self {
    Self::InternalRuntimeError(message.to_string())
  }

  pub fn io_error<T: ToString>(source: std::io::Error, message: T) -> Self {
    Self::IOError {
      source,
      message: message.to_string(),
    }
  }

  pub fn github_error<T: ToString>(message: T) -> Self {
    Self::GithubError {
      // source,
      message: message.to_string(),
    }
  }

  pub fn failed(exit_code: usize) -> Self {
    Self::Failed(exit_code)
  }

  pub fn unsupported_feature<T: ToString>(message: T) -> Self {
    Self::UnsupportedFeature(message.to_string())
  }

  pub fn init_error<T: ToString>(message: T) -> Self {
    Self::InitError(message.to_string())
  }
}

// implement Eq and PartialEq for Error so that we can compare errors in tests
impl PartialEq for Error {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::WorkflowConfigError(a), Self::WorkflowConfigError(b)) => a == b,
      (Self::InternalRuntimeError(a), Self::InternalRuntimeError(b)) => a == b,
      (Self::Failed(a), Self::Failed(b)) => a == b,
      (Self::IOError { message: a, .. }, Self::IOError { message: b, .. }) => a == b,
      (Self::GithubError { message: a, .. }, Self::GithubError { message: b, .. }) => a == b,
      (Self::UnsupportedFeature(a), Self::UnsupportedFeature(b)) => a == b,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_eq() {
    assert_eq!(
      Error::workflow_config_error("hello"),
      Error::workflow_config_error("hello")
    );
    assert_eq!(
      Error::internal_runtime_error("hello"),
      Error::internal_runtime_error("hello")
    );
    assert_eq!(
      Error::io_error(
        std::io::Error::new(std::io::ErrorKind::Other, "hello"),
        "hello"
      ),
      Error::io_error(
        std::io::Error::new(std::io::ErrorKind::Other, "hello"),
        "hello"
      )
    );
    assert_eq!(Error::github_error("hello"), Error::github_error("hello"));
    assert_eq!(
      Error::unsupported_feature("hello"),
      Error::unsupported_feature("hello")
    );
  }
}

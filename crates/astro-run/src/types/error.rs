#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Failed to parse user config: {0}")]
  WorkflowConfigError(String),

  #[error("Error while running workflow: {0}")]
  InternalRuntimeError(String),

  #[error("Failed with exit code: {0:?}")]
  Failed(usize),

  #[error("Error: {0}")]
  Error(String),

  #[error("IO error: {0}")]
  IOError(#[from] std::io::Error),

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

  pub fn io_error(source: std::io::Error) -> Self {
    Self::IOError(source)
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

  pub fn error<T: ToString>(message: T) -> Self {
    Self::Error(message.to_string())
  }
}

// implement Eq and PartialEq for Error so that we can compare errors in tests
impl PartialEq for Error {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::WorkflowConfigError(a), Self::WorkflowConfigError(b)) => a == b,
      (Self::InternalRuntimeError(a), Self::InternalRuntimeError(b)) => a == b,
      (Self::Failed(a), Self::Failed(b)) => a == b,
      (Self::IOError(a), Self::IOError(b)) => a.kind() == b.kind(),
      (Self::Error(a), Self::Error(b)) => a == b,
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
      Error::io_error(std::io::Error::new(std::io::ErrorKind::Other, "hello"),),
      Error::io_error(std::io::Error::new(std::io::ErrorKind::Other, "hello"))
    );
    assert_eq!(
      Error::unsupported_feature("hello"),
      Error::unsupported_feature("hello")
    );
    assert_eq!(Error::error("hello"), Error::error("hello"));
    assert_eq!(Error::failed(1), Error::failed(1));
  }

  #[test]
  fn test_ne() {
    assert_ne!(
      Error::workflow_config_error("hello"),
      Error::workflow_config_error("world")
    );
    assert_ne!(
      Error::internal_runtime_error("hello"),
      Error::internal_runtime_error("world")
    );
    assert_ne!(
      Error::io_error(std::io::Error::new(std::io::ErrorKind::Other, "hello"),),
      Error::io_error(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "world"
      ),)
    );
    assert_ne!(
      Error::unsupported_feature("hello"),
      Error::unsupported_feature("world")
    );
    assert_ne!(Error::error("hello"), Error::error("world"));
    assert_ne!(Error::failed(1), Error::failed(2));
    assert_ne!(Error::failed(1), Error::internal_runtime_error("hello"));
  }
}

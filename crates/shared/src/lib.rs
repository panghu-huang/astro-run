#[allow(dead_code)]
mod command;
mod error;
mod runner;
mod types;

pub use command::*;
pub use error::Error;
pub use runner::*;
pub use types::*;

pub type Result<T> = std::result::Result<T, Error>;

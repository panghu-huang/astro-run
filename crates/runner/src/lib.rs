#![allow(dead_code)]
mod command;

use astro_run_shared::{stream, Command, LogStream, Runner};

pub struct LocalRunner {}

impl Runner for LocalRunner {
  fn run(&self, _command: Command) -> LogStream {
    let (_, receiver) = stream();

    Ok(receiver)
  }
}

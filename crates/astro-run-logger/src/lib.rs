use colored::Colorize;
use log::Level;
use std::{env, str::FromStr, sync::OnceLock};

#[derive(Clone)]
pub struct Logger;

impl log::Log for Logger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= Level::Debug
  }

  fn log(&self, record: &log::Record) {
    if !self.enabled(record.metadata()) {
      return;
    }

    let time = chrono::Local::now()
      .format("%Y-%m-%d %H:%M:%S")
      .to_string()
      .magenta();

    let level = match record.level() {
      Level::Error => "ERROR".red(),
      Level::Warn => "WARN".yellow(),
      Level::Info => "INFO".green(),
      Level::Debug => "DEBUG".green(),
      Level::Trace => "TRACE".green(),
    };

    let prefix = match (record.module_path(), record.line()) {
      (Some(module_path), Some(line)) => format!("{}:{}", module_path, line).cyan(),
      (Some(module_path), None) => module_path.cyan(),
      _ => "".cyan(),
    };

    let log = format!("{} {} {} {}", time, prefix, level, record.args());
    println!("{}", log);
  }

  fn flush(&self) {}
}

static LOGGER: OnceLock<()> = OnceLock::new();

pub fn init_logger() {
  if LOGGER.get().is_some() {
    return;
  }

  LOGGER.get_or_init(|| {
    let level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let level = log::Level::from_str(&level).unwrap_or(log::Level::Info);

    log::set_logger(&Logger).unwrap();
    log::set_max_level(level.to_level_filter());
  });
}

pub fn init_logger_with_level(level: log::Level) {
  if LOGGER.get().is_some() {
    return;
  }

  LOGGER.get_or_init(|| {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(level.to_level_filter());
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test() {
    init_logger();
    log::info!("Hello, world!");
    log::warn!("Hello, world!");
    log::error!("Hello, world!");
    log::debug!("Hello, world!");
    log::trace!("Hello, world!");
  }

  #[test]
  fn test_with_env() {
    env::set_var("RUST_LOG", "debug");
    init_logger();
    log::info!("Hello, world!");
    log::warn!("Hello, world!");
    log::error!("Hello, world!");
    log::debug!("Hello, world!");
    log::trace!("Hello, world!");
    env::remove_var("RUST_LOG");
  }

  #[test]
  fn test_with_level_debug() {
    init_logger_with_level(log::Level::Trace);
    log::info!("Hello, world!");
    log::warn!("Hello, world!");
    log::error!("Hello, world!");
    log::debug!("Hello, world!");
    log::trace!("Hello, world!");
  }
}

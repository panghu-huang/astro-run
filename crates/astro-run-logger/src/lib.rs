use colored::Colorize;
use log::Level;
use std::sync::OnceLock;

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

    let prefix = match (record.file(), record.line()) {
      (Some(file), Some(line)) => format!("{}:{} ", file, line).cyan(),
      _ => String::new().black(),
    };

    let log = format!("{}{} {} {}", prefix, time, level, record.args());
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
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::Level::Trace.to_level_filter());
  });
}

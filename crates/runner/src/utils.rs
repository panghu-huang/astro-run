use astro_run::Result;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

pub async fn create_executable_file(file_path: &PathBuf, content: String) -> Result<()> {
  let mut file;
  #[cfg(unix)]
  {
    file = fs::OpenOptions::new()
      .create(true)
      .write(true)
      .mode(0o777)
      .open(file_path)
      .await?;
  }
  #[cfg(not(unix))]
  {
    file = fs::File::create(file_path).await?;
  }

  file.write(b"#!/bin/sh\n").await?;
  file.write_all(content.as_bytes()).await?;

  // Fix Text file busy
  drop(file);

  Ok(())
}

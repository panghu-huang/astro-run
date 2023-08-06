use astro_run_shared::{Error, Result};
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

pub async fn cleanup_working_directory(working_directory: &PathBuf) -> Result<()> {
  fs::remove_dir_all(&working_directory)
    .await
    .map_err(|err| {
      Error::io_error(
        err,
        format!(
          "Failed to remove working directory: {:?}",
          working_directory
        ),
      )
    })?;

  Ok(())
}

pub async fn create_executable_file(file_path: &PathBuf, content: String) -> Result<()> {
  let mut file;
  #[cfg(unix)]
  {
    file = fs::OpenOptions::new()
      .create(true)
      .write(true)
      .mode(0o777)
      .open(file_path)
      .await
      .map_err(|err| Error::io_error(err, "Failed to create entrypoint file"))?;
  }
  #[cfg(not(unix))]
  {
    file = fs::File::create(file_path)
      .await
      .map_err(|err| Error::io_error(err, "Failed to create entrypoint file"))?;
  }

  file
    .write(b"#!/bin/sh\n")
    .await
    .map_err(|err| Error::io_error(err, "Failed to write entrypoint file"))?;
  file
    .write_all(content.as_bytes())
    .await
    .map_err(|err| Error::io_error(err, "Failed to write entrypoint file"))?;

  // Fix Text file busy
  drop(file);

  Ok(())
}

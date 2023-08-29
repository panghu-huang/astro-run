#[astro_run_test::test(docker)]
fn docker_required() -> Result<(), ()> {
  log::info!("Hello, world!");
  log::warn!("Hello, world!");
  log::error!("Hello, world!");
  log::debug!("Hello, world!");
  log::trace!("Hello, world!");

  Ok(())
}

#[astro_run_test::test]
async fn test() -> Result<(), ()> {
  log::info!("Hello, world!");
  log::warn!("Hello, world!");
  log::error!("Hello, world!");
  log::debug!("Hello, world!");
  log::trace!("Hello, world!");

  Ok(())
}

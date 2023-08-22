#[astro_run_test::test]
fn basic_test2() -> Result<(), ()> {
  log::info!("Hello, world !");
  log::warn!("Hello, world !");
  log::error!("Hello, world !");
  log::debug!("Hello, world !");
  log::trace!("Hello, world !");

  Ok(())
}

#[astro_run_test::test]
async fn basic_test3() -> Result<(), ()> {
  log::info!("Hello, world !");
  log::warn!("Hello, world !");
  log::error!("Hello, world !");
  log::debug!("Hello, world !");
  log::trace!("Hello, world !");

  Ok(())
}

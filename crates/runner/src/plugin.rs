use astro_run::{Context, HookNoopResult, Result};
use std::sync::Arc;

#[astro_run::async_trait]
pub trait Plugin: Send + Sync {
  fn name(&self) -> &'static str;
  async fn on_before_run(&self, ctx: Context) -> Result<Context> {
    Ok(ctx)
  }
  async fn on_after_run(&self, _ctx: Context) -> HookNoopResult {
    Ok(())
  }
}

pub type SharedPluginDriver = Arc<PluginDriver>;

pub struct PluginDriver {
  pub(crate) plugins: Vec<Box<dyn Plugin>>,
}

impl PluginDriver {
  pub fn new(plugins: Vec<Box<dyn Plugin>>) -> Self {
    PluginDriver { plugins }
  }

  pub async fn on_before_run(&self, ctx: Context) -> Context
  where
    Self: Send + Sync,
  {
    let mut ctx = ctx;

    for plugin in &self.plugins {
      if let Ok(updated_ctx) = plugin.on_before_run(ctx.clone()).await {
        ctx = updated_ctx;
      } else {
        log::error!("Plugin {} on_before_run error", plugin.name());
      }
    }

    ctx
  }

  pub async fn on_after_run(&self, ctx: Context) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_after_run(ctx.clone()).await {
        log::error!("Plugin {} on_after_run error: {}", plugin.name(), err);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use astro_run::StepId;

  use super::*;

  #[astro_run_test::test]
  async fn test_plugin_driver() {
    struct TestPlugin;

    #[astro_run::async_trait]
    impl Plugin for TestPlugin {
      fn name(&self) -> &'static str {
        "test"
      }

      async fn on_before_run(&self, ctx: Context) -> Result<Context> {
        let mut ctx = ctx;

        ctx.id = StepId::try_from("abc/1/1").unwrap();

        Ok(ctx)
      }

      async fn on_after_run(&self, _ctx: Context) -> HookNoopResult {
        Ok(())
      }
    }

    struct ErrorBeforeRunPlugin;

    #[astro_run::async_trait]
    impl Plugin for ErrorBeforeRunPlugin {
      fn name(&self) -> &'static str {
        "error-before-run"
      }

      async fn on_before_run(&self, _ctx: Context) -> Result<Context> {
        Err(astro_run::Error::error("Error"))
      }
    }

    struct ErrorAfterRunPlugin;

    #[astro_run::async_trait]
    impl Plugin for ErrorAfterRunPlugin {
      fn name(&self) -> &'static str {
        "error-before-run"
      }

      async fn on_after_run(&self, _ctx: Context) -> HookNoopResult {
        Err(astro_run::Error::error("Error"))
      }
    }

    let driver = PluginDriver::new(vec![
      Box::new(TestPlugin),
      Box::new(ErrorBeforeRunPlugin),
      Box::new(ErrorAfterRunPlugin),
    ]);

    let ctx = astro_run::Context {
      id: StepId::try_from("aaa/1/1").unwrap(),
      command: Default::default(),
      event: None,
      signal: astro_run::AstroRunSignal::new(),
      payload: None,
    };

    let ctx = driver.on_before_run(ctx).await;

    assert_eq!(ctx.id, StepId::try_from("abc/1/1").unwrap());

    driver.on_after_run(ctx).await;
  }
}

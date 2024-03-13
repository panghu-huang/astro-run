use astro_run::{Context, PluginNoopResult};
use std::sync::Arc;

#[astro_run::async_trait]
pub trait Plugin: Send + Sync {
  fn name(&self) -> &'static str;
  async fn on_before_run(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error>> {
    Ok(ctx)
  }
  async fn on_after_run(&self, _ctx: Context) -> PluginNoopResult {
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

  pub async fn on_before_run(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error>>
  where
    Self: Send + Sync,
  {
    let mut ctx = ctx;

    for plugin in &self.plugins {
      ctx = plugin.on_before_run(ctx).await?;
    }

    Ok(ctx)
  }

  pub async fn on_after_run(&self, ctx: Context) {
    for plugin in &self.plugins {
      if let Err(err) = plugin.on_after_run(ctx.clone()).await {
        log::error!("Plugin {} on_after_run error: {}", plugin.name(), err);
      }
    }
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[astro_run_test::test]
//   async fn test_plugin_manager() {

//     struct TestPlugin {
//       name: &'static str,
//     }

//     impl TestPlugin {
//       fn new(name: &'static str) -> Self {
//         TestPlugin { name }
//       }
//     }

//     impl Plugin for TestPlugin {
//       fn name(&self) -> &'static str {
//         self.name
//       }

//       fn on_before_run(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error>> {
//         Ok(ctx)
//       }

//       fn on_after_run(&self, _ctx: Context) {}
//     }

//     plugin_manager.register(TestPlugin::new("test1"));
//     plugin_manager.register(TestPlugin::new("test2"));

//     assert_eq!(plugin_manager.size(), 2);

//     plugin_manager.unregister("test1");

//     assert_eq!(plugin_manager.size(), 1);

//     plugin_manager.unregister("test2");

//     assert_eq!(plugin_manager.size(), 0);
//   }
// }

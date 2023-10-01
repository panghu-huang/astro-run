use astro_run::Context;
use parking_lot::Mutex;
use std::sync::Arc;

pub trait Plugin: Send + Sync {
  fn name(&self) -> &'static str;
  fn on_before_run(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error>> {
    Ok(ctx)
  }
  fn on_after_run(&self, _ctx: Context) {}
}

#[derive(Clone)]
pub struct PluginManager {
  pub(crate) plugins: Arc<Mutex<Vec<Box<dyn Plugin>>>>,
}

impl PluginManager {
  pub fn new() -> Self {
    PluginManager {
      plugins: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn size(&self) -> usize {
    self.plugins.lock().len()
  }

  pub fn register<P: Plugin + 'static>(&self, plugin: P) {
    let mut plugins = self.plugins.lock();

    plugins.retain(|p| p.name() != plugin.name());

    plugins.push(Box::new(plugin));
  }

  pub fn unregister(&self, name: &'static str) {
    self.plugins.lock().retain(|plugin| plugin.name() != name);
  }

  pub fn on_before_run(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error>>
  where
    Self: Send + Sync,
  {
    let plugins = self.plugins.lock();

    let mut ctx = ctx;

    for plugin in plugins.iter() {
      ctx = plugin.on_before_run(ctx)?;
    }

    Ok(ctx)
  }

  pub fn on_after_run(&self, ctx: Context) {
    let plugins = self.plugins.lock();

    for plugin in plugins.iter() {
      plugin.on_after_run(ctx.clone());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[astro_run_test::test]
  async fn test_plugin_manager() {
    let plugin_manager = PluginManager::new();

    struct TestPlugin {
      name: &'static str,
    }

    impl TestPlugin {
      fn new(name: &'static str) -> Self {
        TestPlugin { name }
      }
    }

    impl Plugin for TestPlugin {
      fn name(&self) -> &'static str {
        self.name
      }

      fn on_before_run(&self, ctx: Context) -> Result<Context, Box<dyn std::error::Error>> {
        Ok(ctx)
      }

      fn on_after_run(&self, _ctx: Context) {}
    }

    plugin_manager.register(TestPlugin::new("test1"));
    plugin_manager.register(TestPlugin::new("test2"));

    assert_eq!(plugin_manager.size(), 2);

    plugin_manager.unregister("test1");

    assert_eq!(plugin_manager.size(), 1);

    plugin_manager.unregister("test2");

    assert_eq!(plugin_manager.size(), 0);
  }
}

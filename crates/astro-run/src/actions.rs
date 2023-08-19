use crate::{Error, Result, UserActionStep, UserStep};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActionSteps {
  pub pre: Option<UserStep>,
  pub run: UserStep,
  pub post: Option<UserStep>,
}

pub trait Action
where
  Self: Send + Sync,
{
  fn normalize(&self, step: UserActionStep) -> Result<ActionSteps>;
}

#[derive(Clone)]
pub struct Actions {
  actions: Arc<Mutex<HashMap<String, Box<dyn Action>>>>,
}

impl Actions {
  pub fn new() -> Self {
    let actions: HashMap<String, Box<dyn Action>> = HashMap::new();

    Self {
      actions: Arc::new(Mutex::new(actions)),
    }
  }

  pub fn register<T>(&self, name: impl Into<String>, action: T)
  where
    T: Action + 'static,
  {
    self.actions.lock().insert(name.into(), Box::new(action));
  }

  pub fn unregister(&self, name: &str) {
    self.actions.lock().remove(name);
  }

  pub fn normalize(&self, step: UserActionStep) -> Result<ActionSteps> {
    let actions = self.actions.lock();
    let action = actions.get(&step.uses).ok_or_else(|| {
      Error::workflow_config_error(format!("Action `{}` is not found", step.uses))
    })?;

    action.normalize(step)
  }

  pub fn size(&self) -> usize {
    self.actions.lock().len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::UserCommandStep;

  #[test]
  fn test_normalize_step_actions() -> Result<()> {
    struct CacheAction {}

    impl Action for CacheAction {
      fn normalize(&self, _step: UserActionStep) -> Result<ActionSteps> {
        Ok(ActionSteps {
          pre: None,
          run: UserStep::Command(UserCommandStep {
            name: Some("Restore cache".to_string()),
            run: "restore cache".to_string(),
            ..Default::default()
          }),
          post: Some(UserStep::Command(UserCommandStep {
            name: Some("Save cache".to_string()),
            run: "save cache".to_string(),
            ..Default::default()
          })),
        })
      }
    }

    let actions = Actions::new();

    actions.register("caches", CacheAction {});

    let test_step = UserActionStep {
      uses: "caches".to_string(),
      ..Default::default()
    };

    let steps = actions.normalize(test_step)?;

    assert!(steps.pre.is_none());

    if let UserStep::Command(step) = steps.run {
      assert_eq!(step.name, Some("Restore cache".to_string()));
      assert_eq!(step.run, "restore cache".to_string());
    } else {
      panic!("Should be command step");
    }

    if let Some(UserStep::Command(step)) = steps.post {
      assert_eq!(step.name, Some("Save cache".to_string()));
      assert_eq!(step.run, "save cache".to_string());
    } else {
      panic!("Should be command step");
    }

    Ok(())
  }

  #[test]
  fn test_not_exists_action() -> Result<()> {
    let actions = Actions::new();

    let step = UserActionStep {
      uses: "not-exists-action".to_string(),
      ..Default::default()
    };

    let result = actions.normalize(step);

    let err = result.unwrap_err();

    assert_eq!(
      err,
      Error::workflow_config_error("Action `not-exists-action` is not found")
    );

    Ok(())
  }
}

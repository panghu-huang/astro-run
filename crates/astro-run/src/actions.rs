use crate::{Result, UserActionStep, UserStep};
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

pub type SharedActionDriver = Arc<ActionDriver>;

#[derive(Clone)]
pub struct ActionDriver {
  actions: Arc<HashMap<String, Box<dyn Action>>>,
}

impl ActionDriver {
  pub fn new(actions: HashMap<String, Box<dyn Action>>) -> Self {
    Self {
      actions: Arc::new(actions),
    }
  }

  pub fn try_normalize(&self, step: UserActionStep) -> Result<Option<ActionSteps>> {
    if let Some(action) = &self.actions.get(&step.uses) {
      let normalized = action.normalize(step)?;

      Ok(Some(normalized))
    } else {
      Ok(None)
    }
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

    let mut actions = HashMap::new();

    actions.insert(
      "caches".to_string(),
      Box::new(CacheAction {}) as Box<dyn Action>,
    );

    let actions = ActionDriver::new(actions);

    let test_step = UserActionStep {
      uses: "caches".to_string(),
      ..Default::default()
    };

    let steps = actions.try_normalize(test_step)?.unwrap();

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
    let actions = HashMap::new();

    let actions = ActionDriver::new(actions);

    let step = UserActionStep {
      uses: "not-exists-action".to_string(),
      ..Default::default()
    };

    let result = actions.try_normalize(step).unwrap();

    assert!(result.is_none(),);

    Ok(())
  }
}

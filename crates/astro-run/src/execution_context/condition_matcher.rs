use crate::{Condition, ConditionPayload, Error, GithubAuthorization, Result, WorkflowEvent};
use octocrate::{GithubAPI, GithubApp, GithubPersonalAccessToken};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct ConditionMatcher {
  pub github_auth: Option<GithubAuthorization>,
  pub event: Option<WorkflowEvent>,
  pub payload: Arc<Mutex<Option<ConditionPayload>>>,
}

impl ConditionMatcher {
  pub fn new(event: Option<WorkflowEvent>, github_auth: Option<GithubAuthorization>) -> Self {
    Self {
      github_auth,
      event,
      payload: Arc::new(Mutex::new(None)),
    }
  }

  pub async fn is_match(&self, condition: &Condition) -> bool {
    if self.github_auth.is_none() {
      log::trace!("Github authorization is not provided");
      return true;
    }

    if self.event.is_none() {
      log::trace!("Event is not provided");
      return true;
    }
    log::trace!("Matching condition {:#?}", condition);

    if let Some(payload) = self.payload.lock().as_ref() {
      return condition.is_match(payload);
    }

    match self.condition_payload().await {
      Ok(payload) => {
        let mut payload_lock = self.payload.lock();
        *payload_lock = Some(payload.clone());

        return condition.is_match(&payload);
      }
      Err(err) => {
        log::trace!("Failed to get condition payload: {}", err);
        return true;
      }
    }
  }

  async fn condition_payload(&self) -> Result<ConditionPayload> {
    let WorkflowEvent { event, branch, .. } = self.event.as_ref().unwrap();

    let files = self.get_changed_files().await?;
    let payload = ConditionPayload {
      event: event.clone(),
      branch: branch.clone(),
      paths: files,
    };

    log::trace!("Condition payload: {:#?}", payload);

    Ok(payload)
  }

  async fn get_changed_files(&self) -> Result<Vec<String>> {
    let WorkflowEvent { event, .. } = self.event.as_ref().unwrap();

    match event.as_str() {
      "push" => self.get_push_changed_files().await,
      "pull_request" => self.get_pull_request_changed_files().await,
      _ => Err(Error::unsupported_feature(format!(
        "Event {} is not supported",
        event
      ))),
    }
  }

  async fn get_push_changed_files(&self) -> Result<Vec<String>> {
    let WorkflowEvent {
      repo_owner,
      repo_name,
      sha,
      ..
    } = self.event.as_ref().unwrap();
    let github_api = self.get_github_api(repo_owner, repo_name).await?;

    let commit = github_api
      .commits
      .get_commit(repo_owner, repo_name, sha)
      .send()
      .await
      .map_err(|e| Error::internal_runtime_error(format!("Failed to get commit: {}", e)))?;

    let files: Vec<String> = commit
      .files
      .map(|files| files.iter().map(|f| f.filename.clone()).collect())
      .unwrap_or(vec![]);

    Ok(files)
  }

  async fn get_pull_request_changed_files(&self) -> Result<Vec<String>> {
    let WorkflowEvent {
      repo_owner,
      repo_name,
      pr_number,
      ..
    } = self.event.as_ref().unwrap();
    let github_api = self.get_github_api(repo_owner, repo_name).await?;

    let pull_request_files = github_api
      .pulls
      .list_pull_request_files(
        repo_owner,
        repo_name,
        pr_number.ok_or(Error::workflow_config_error("pr_number is not provided"))?,
      )
      .send()
      .await
      .map_err(|e| {
        Error::internal_runtime_error(format!("Failed to get pull request files: {}", e))
      })?;

    Ok(pull_request_files.into_iter().map(|f| f.filename).collect())
  }

  async fn get_github_api(
    &self,
    repo_owner: &String,
    repo_name: &String,
  ) -> crate::Result<GithubAPI> {
    let github_auth = self.github_auth.as_ref().unwrap();
    let github_api = match &github_auth {
      GithubAuthorization::PersonalAccessToken(token) => {
        let access_token = GithubPersonalAccessToken::new(token);

        GithubAPI::with_token(access_token)
      }
      GithubAuthorization::GithubApp {
        app_id,
        private_key,
      } => {
        let github_app = GithubApp::builder()
          .app_id(app_id.to_string())
          .private_key(private_key)
          .build()
          .unwrap();

        let installation = github_app
          .get_repository_installation(repo_owner, repo_name)
          .await
          .map_err(|err| {
            Error::internal_runtime_error(format!(
              "Failed to get installation for repository: {}",
              err
            ))
          })?;

        let github_api = github_app.get_api(installation.id).await.map_err(|err| {
          Error::internal_runtime_error(format!("Failed to get github api: {}", err))
        })?;

        github_api
      }
    };

    Ok(github_api)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[astro_run_test::test]
  async fn test_unsupported_event() {
    let matcher = ConditionMatcher::new(
      Some(WorkflowEvent {
        event: "unsupported".to_string(),
        ..Default::default()
      }),
      None,
    );

    let err = matcher.get_changed_files().await.unwrap_err();

    assert_eq!(
      err.to_string(),
      "Unsupported feature: Event unsupported is not supported"
    );
  }

  #[astro_run_test::test]
  async fn invalid_github_app_id() {
    dotenv::dotenv().ok();

    let private_key = std::env::var("GH_APP_PRIVATE_KEY")
      .map_err(|err| crate::Error::internal_runtime_error(format!("GH_APP_PRIVATE_KEY: {}", err)))
      .unwrap();

    let matcher = ConditionMatcher::new(
      Some(WorkflowEvent {
        event: "pull_request".to_string(),
        ..Default::default()
      }),
      Some(GithubAuthorization::GithubApp {
        app_id: 0,
        private_key,
      }),
    );

    let res = matcher
      .get_github_api(&"panghu-huang".to_string(), &"astro-run".to_string())
      .await;

    assert!(res.is_err());
  }

  #[astro_run_test::test]
  async fn test_pr_number_not_provided() {
    dotenv::dotenv().ok();

    let matcher = ConditionMatcher::new(
      Some(WorkflowEvent {
        event: "pull_request".to_string(),
        ..Default::default()
      }),
      Some(GithubAuthorization::PersonalAccessToken(
        std::env::var("PERSONAL_ACCESS_TOKEN").unwrap(),
      )),
    );

    let err = matcher.get_changed_files().await.unwrap_err();

    assert_eq!(
      err,
      Error::workflow_config_error("pr_number is not provided")
    );
  }

  #[astro_run_test::test]
  async fn test_invalid_pr_number() {
    dotenv::dotenv().ok();

    let matcher = ConditionMatcher::new(
      Some(WorkflowEvent {
        event: "pull_request".to_string(),
        pr_number: Some(0),
        ..Default::default()
      }),
      Some(GithubAuthorization::PersonalAccessToken(
        std::env::var("PERSONAL_ACCESS_TOKEN").unwrap(),
      )),
    );

    let err = matcher.get_changed_files().await.unwrap_err();

    assert!(err
      .to_string()
      .starts_with("Error while running workflow: Failed to get pull request files: "));
  }
}

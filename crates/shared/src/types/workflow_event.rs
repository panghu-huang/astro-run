use octocrate::{GithubWebhookPullRequestEvent, GithubWebhookPushEvent};
use serde::{Deserialize, Serialize};

pub trait WorkflowEventPayload {
  fn payload(self) -> crate::Result<WorkflowAPIEvent>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowAPIEvent {
  pub repo_owner: String,
  pub repo_name: String,
  pub pr_number: Option<u64>,
  pub sha: String,
  /// refs/heads/master / refs/tags/v1.0.0 / refs/pull/1/merge
  pub ref_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowEvent {
  Push(GithubWebhookPushEvent),
  PullRequest(GithubWebhookPullRequestEvent),
  /// Trigger by API
  API(WorkflowAPIEvent),
}

impl WorkflowEventPayload for WorkflowAPIEvent {
  fn payload(self) -> crate::Result<WorkflowAPIEvent> {
    Ok(self)
  }
}

impl WorkflowEventPayload for GithubWebhookPushEvent {
  fn payload(self) -> crate::Result<WorkflowAPIEvent> {
    let api_event = WorkflowAPIEvent {
      repo_owner: self.repository.owner.login,
      repo_name: self.repository.name,
      ref_name: self.ref_name,
      sha: self.after,
      pr_number: None,
    };

    Ok(api_event)
  }
}

impl WorkflowEventPayload for GithubWebhookPullRequestEvent {
  fn payload(self) -> crate::Result<WorkflowAPIEvent> {
    let api_event = WorkflowAPIEvent {
      repo_owner: self.repository.owner.login,
      repo_name: self.repository.name,
      ref_name: self.pull_request.base.ref_name,
      sha: self.pull_request.head.sha,
      pr_number: Some(self.pull_request.number),
    };

    Ok(api_event)
  }
}

impl WorkflowEventPayload for WorkflowEvent {
  fn payload(self) -> crate::Result<WorkflowAPIEvent> {
    match self {
      WorkflowEvent::API(api_event) => api_event.payload(),
      WorkflowEvent::Push(push_event) => push_event.payload(),
      WorkflowEvent::PullRequest(pull_request_event) => pull_request_event.payload(),
    }
  }
}

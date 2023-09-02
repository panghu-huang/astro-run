#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnvironmentVariable {
    #[prost(oneof = "environment_variable::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<environment_variable::Value>,
}
/// Nested message and enum types in `EnvironmentVariable`.
pub mod environment_variable {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        String(::prost::alloc::string::String),
        #[prost(float, tag = "2")]
        Number(f32),
        #[prost(bool, tag = "3")]
        Boolean(bool),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Command {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "3")]
    pub container: ::core::option::Option<Container>,
    #[prost(string, tag = "4")]
    pub run: ::prost::alloc::string::String,
    #[prost(bool, tag = "5")]
    pub continue_on_error: bool,
    #[prost(map = "string, message", tag = "6")]
    pub environments: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        EnvironmentVariable,
    >,
    #[prost(string, repeated, tag = "7")]
    pub secrets: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(uint64, tag = "8")]
    pub timeout: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Job {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "3")]
    pub steps: ::prost::alloc::vec::Vec<Command>,
    #[prost(string, repeated, tag = "4")]
    pub depends_on: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub working_directories: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkflowEvent {
    #[prost(string, tag = "1")]
    pub event: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub repo_owner: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub repo_name: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "4")]
    pub pr_number: ::core::option::Option<u64>,
    #[prost(string, tag = "5")]
    pub sha: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub ref_name: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub branch: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Workflow {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(map = "string, message", tag = "3")]
    pub jobs: ::std::collections::HashMap<::prost::alloc::string::String, Job>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StepRunResult {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(enumeration = "WorkflowState", tag = "2")]
    pub state: i32,
    #[prost(int32, optional, tag = "3")]
    pub exit_code: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "4")]
    pub started_at: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "5")]
    pub completed_at: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobRunResult {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(enumeration = "WorkflowState", tag = "2")]
    pub state: i32,
    #[prost(message, optional, tag = "3")]
    pub started_at: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub completed_at: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, repeated, tag = "5")]
    pub steps: ::prost::alloc::vec::Vec<StepRunResult>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkflowRunResult {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(enumeration = "WorkflowState", tag = "2")]
    pub state: i32,
    #[prost(message, optional, tag = "3")]
    pub started_at: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "4")]
    pub completed_at: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(map = "string, message", tag = "5")]
    pub jobs: ::std::collections::HashMap<::prost::alloc::string::String, JobRunResult>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkflowLog {
    #[prost(string, tag = "1")]
    pub step_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub log_type: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub message: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<::prost_types::Timestamp>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkflowStateEvent {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    #[prost(enumeration = "WorkflowState", tag = "3")]
    pub state: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Container {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub volumes: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "3")]
    pub security_opts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Signal {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub action: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Context {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub command: ::core::option::Option<Command>,
    #[prost(message, optional, tag = "3")]
    pub event: ::core::option::Option<WorkflowEvent>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RunResult {
    #[prost(oneof = "run_result::Result", tags = "2, 3, 4")]
    pub result: ::core::option::Option<run_result::Result>,
}
/// Nested message and enum types in `RunResult`.
pub mod run_result {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Result {
        #[prost(message, tag = "2")]
        Succeeded(()),
        #[prost(int32, tag = "3")]
        Failed(i32),
        #[prost(message, tag = "4")]
        Cancelled(()),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RunnerMetadata {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub os: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub arch: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub version: ::prost::alloc::string::String,
    #[prost(int32, tag = "5")]
    pub max_runs: i32,
    #[prost(bool, tag = "6")]
    pub support_docker: bool,
    #[prost(bool, tag = "7")]
    pub support_host: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum WorkflowState {
    Pending = 0,
    Queued = 1,
    InProgress = 2,
    Succeeded = 3,
    Failed = 4,
    Cancelled = 5,
    Skipped = 6,
}
impl WorkflowState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            WorkflowState::Pending => "Pending",
            WorkflowState::Queued => "Queued",
            WorkflowState::InProgress => "InProgress",
            WorkflowState::Succeeded => "Succeeded",
            WorkflowState::Failed => "Failed",
            WorkflowState::Cancelled => "Cancelled",
            WorkflowState::Skipped => "Skipped",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Pending" => Some(Self::Pending),
            "Queued" => Some(Self::Queued),
            "InProgress" => Some(Self::InProgress),
            "Succeeded" => Some(Self::Succeeded),
            "Failed" => Some(Self::Failed),
            "Cancelled" => Some(Self::Cancelled),
            "Skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

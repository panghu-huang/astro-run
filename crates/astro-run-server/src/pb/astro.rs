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
    #[prost(uint64, tag = "7")]
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
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Workflow {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "3")]
    pub event: ::core::option::Option<WorkflowEvent>,
    #[prost(map = "string, message", tag = "4")]
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
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Context {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub command: ::core::option::Option<Command>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub event_name: ::prost::alloc::string::String,
    #[prost(oneof = "event::Payload", tags = "2, 3, 4, 5, 6, 7, 8, 9")]
    pub payload: ::core::option::Option<event::Payload>,
}
/// Nested message and enum types in `Event`.
pub mod event {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "2")]
        Run(super::Context),
        #[prost(message, tag = "3")]
        RunWorkflowEvent(super::Workflow),
        #[prost(message, tag = "4")]
        RunJobEvent(super::Job),
        #[prost(message, tag = "5")]
        WorkflowCompletedEvent(super::WorkflowRunResult),
        #[prost(message, tag = "6")]
        JobCompletedEvent(super::JobRunResult),
        #[prost(message, tag = "7")]
        WorkflowStateEvent(super::WorkflowStateEvent),
        #[prost(message, tag = "8")]
        LogEvent(super::WorkflowLog),
        #[prost(string, tag = "9")]
        Error(::prost::alloc::string::String),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeEventsRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "3")]
    pub token: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportLogResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportRunCompletedRequest {
    #[prost(string, tag = "1")]
    pub run_id: ::prost::alloc::string::String,
    #[prost(oneof = "report_run_completed_request::Result", tags = "2, 3, 4")]
    pub result: ::core::option::Option<report_run_completed_request::Result>,
}
/// Nested message and enum types in `ReportRunCompletedRequest`.
pub mod report_run_completed_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Result {
        #[prost(int32, tag = "2")]
        Succeeded(i32),
        #[prost(int32, tag = "3")]
        Failed(i32),
        #[prost(int32, tag = "4")]
        Cancelled(i32),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportRunCompletedResponse {}
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
/// Generated client implementations.
pub mod astro_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct AstroServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AstroServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> AstroServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> AstroServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            AstroServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn subscribe_events(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribeEventsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::Event>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/astro.AstroService/SubscribeEvents",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("astro.AstroService", "SubscribeEvents"));
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn report_log(
            &mut self,
            request: impl tonic::IntoRequest<super::WorkflowLog>,
        ) -> std::result::Result<
            tonic::Response<super::ReportLogResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/astro.AstroService/ReportLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("astro.AstroService", "ReportLog"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn report_run_completed(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportRunCompletedRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReportRunCompletedResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/astro.AstroService/ReportRunCompleted",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("astro.AstroService", "ReportRunCompleted"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod astro_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AstroServiceServer.
    #[async_trait]
    pub trait AstroService: Send + Sync + 'static {
        /// Server streaming response type for the SubscribeEvents method.
        type SubscribeEventsStream: futures_core::Stream<
                Item = std::result::Result<super::Event, tonic::Status>,
            >
            + Send
            + 'static;
        async fn subscribe_events(
            &self,
            request: tonic::Request<super::SubscribeEventsRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::SubscribeEventsStream>,
            tonic::Status,
        >;
        async fn report_log(
            &self,
            request: tonic::Request<super::WorkflowLog>,
        ) -> std::result::Result<
            tonic::Response<super::ReportLogResponse>,
            tonic::Status,
        >;
        async fn report_run_completed(
            &self,
            request: tonic::Request<super::ReportRunCompletedRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReportRunCompletedResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct AstroServiceServer<T: AstroService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: AstroService> AstroServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AstroServiceServer<T>
    where
        T: AstroService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/astro.AstroService/SubscribeEvents" => {
                    #[allow(non_camel_case_types)]
                    struct SubscribeEventsSvc<T: AstroService>(pub Arc<T>);
                    impl<
                        T: AstroService,
                    > tonic::server::ServerStreamingService<
                        super::SubscribeEventsRequest,
                    > for SubscribeEventsSvc<T> {
                        type Response = super::Event;
                        type ResponseStream = T::SubscribeEventsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SubscribeEventsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).subscribe_events(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubscribeEventsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/astro.AstroService/ReportLog" => {
                    #[allow(non_camel_case_types)]
                    struct ReportLogSvc<T: AstroService>(pub Arc<T>);
                    impl<T: AstroService> tonic::server::UnaryService<super::WorkflowLog>
                    for ReportLogSvc<T> {
                        type Response = super::ReportLogResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WorkflowLog>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).report_log(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReportLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/astro.AstroService/ReportRunCompleted" => {
                    #[allow(non_camel_case_types)]
                    struct ReportRunCompletedSvc<T: AstroService>(pub Arc<T>);
                    impl<
                        T: AstroService,
                    > tonic::server::UnaryService<super::ReportRunCompletedRequest>
                    for ReportRunCompletedSvc<T> {
                        type Response = super::ReportRunCompletedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReportRunCompletedRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).report_run_completed(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReportRunCompletedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: AstroService> Clone for AstroServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: AstroService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: AstroService> tonic::server::NamedService for AstroServiceServer<T> {
        const NAME: &'static str = "astro.AstroService";
    }
}

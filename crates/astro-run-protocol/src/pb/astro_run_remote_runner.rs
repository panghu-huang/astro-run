#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub event_name: ::prost::alloc::string::String,
    #[prost(oneof = "event::Payload", tags = "2, 3, 4, 5, 6, 7, 8, 9, 10")]
    pub payload: ::core::option::Option<event::Payload>,
}
/// Nested message and enum types in `Event`.
pub mod event {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "2")]
        RunWorkflowEvent(super::super::astro_run::Workflow),
        #[prost(message, tag = "3")]
        RunJobEvent(super::super::astro_run::Job),
        #[prost(message, tag = "4")]
        RunStepEvent(super::super::astro_run::Command),
        #[prost(message, tag = "5")]
        WorkflowCompletedEvent(super::super::astro_run::WorkflowRunResult),
        #[prost(message, tag = "6")]
        JobCompletedEvent(super::super::astro_run::JobRunResult),
        #[prost(message, tag = "7")]
        StepCompletedEvent(super::super::astro_run::StepRunResult),
        #[prost(message, tag = "8")]
        WorkflowStateEvent(super::super::astro_run::WorkflowStateEvent),
        #[prost(message, tag = "9")]
        LogEvent(super::super::astro_run::WorkflowLog),
        #[prost(message, tag = "10")]
        SignalEvent(super::super::astro_run::Signal),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RunResponse {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(oneof = "run_response::Payload", tags = "2, 3")]
    pub payload: ::core::option::Option<run_response::Payload>,
}
/// Nested message and enum types in `RunResponse`.
pub mod run_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "2")]
        Result(super::super::astro_run::RunResult),
        #[prost(message, tag = "3")]
        Log(super::super::astro_run::WorkflowLog),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendEventResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectRequest {}
/// Generated client implementations.
pub mod astro_run_remote_runner_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct AstroRunRemoteRunnerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl AstroRunRemoteRunnerClient<tonic::transport::Channel> {
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
    impl<T> AstroRunRemoteRunnerClient<T>
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
        ) -> AstroRunRemoteRunnerClient<InterceptedService<T, F>>
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
            AstroRunRemoteRunnerClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn get_runner_metadata(
            &mut self,
            request: impl tonic::IntoRequest<super::ConnectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::astro_run::RunnerMetadata>,
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
                "/astro_run_remote_runner.AstroRunRemoteRunner/GetRunnerMetadata",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "astro_run_remote_runner.AstroRunRemoteRunner",
                        "GetRunnerMetadata",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn run(
            &mut self,
            request: impl tonic::IntoRequest<super::super::astro_run::Context>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::RunResponse>>,
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
                "/astro_run_remote_runner.AstroRunRemoteRunner/Run",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "astro_run_remote_runner.AstroRunRemoteRunner",
                        "Run",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn send_event(
            &mut self,
            request: impl tonic::IntoRequest<super::Event>,
        ) -> std::result::Result<
            tonic::Response<super::SendEventResponse>,
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
                "/astro_run_remote_runner.AstroRunRemoteRunner/SendEvent",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "astro_run_remote_runner.AstroRunRemoteRunner",
                        "SendEvent",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod astro_run_remote_runner_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AstroRunRemoteRunnerServer.
    #[async_trait]
    pub trait AstroRunRemoteRunner: Send + Sync + 'static {
        async fn get_runner_metadata(
            &self,
            request: tonic::Request<super::ConnectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::super::astro_run::RunnerMetadata>,
            tonic::Status,
        >;
        /// Server streaming response type for the Run method.
        type RunStream: futures_core::Stream<
                Item = std::result::Result<super::RunResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn run(
            &self,
            request: tonic::Request<super::super::astro_run::Context>,
        ) -> std::result::Result<tonic::Response<Self::RunStream>, tonic::Status>;
        async fn send_event(
            &self,
            request: tonic::Request<super::Event>,
        ) -> std::result::Result<
            tonic::Response<super::SendEventResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct AstroRunRemoteRunnerServer<T: AstroRunRemoteRunner> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: AstroRunRemoteRunner> AstroRunRemoteRunnerServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for AstroRunRemoteRunnerServer<T>
    where
        T: AstroRunRemoteRunner,
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
                "/astro_run_remote_runner.AstroRunRemoteRunner/GetRunnerMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct GetRunnerMetadataSvc<T: AstroRunRemoteRunner>(pub Arc<T>);
                    impl<
                        T: AstroRunRemoteRunner,
                    > tonic::server::UnaryService<super::ConnectRequest>
                    for GetRunnerMetadataSvc<T> {
                        type Response = super::super::astro_run::RunnerMetadata;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ConnectRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).get_runner_metadata(request).await
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
                        let method = GetRunnerMetadataSvc(inner);
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
                "/astro_run_remote_runner.AstroRunRemoteRunner/Run" => {
                    #[allow(non_camel_case_types)]
                    struct RunSvc<T: AstroRunRemoteRunner>(pub Arc<T>);
                    impl<
                        T: AstroRunRemoteRunner,
                    > tonic::server::ServerStreamingService<
                        super::super::astro_run::Context,
                    > for RunSvc<T> {
                        type Response = super::RunResponse;
                        type ResponseStream = T::RunStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::astro_run::Context>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).run(request).await };
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
                        let method = RunSvc(inner);
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
                "/astro_run_remote_runner.AstroRunRemoteRunner/SendEvent" => {
                    #[allow(non_camel_case_types)]
                    struct SendEventSvc<T: AstroRunRemoteRunner>(pub Arc<T>);
                    impl<
                        T: AstroRunRemoteRunner,
                    > tonic::server::UnaryService<super::Event> for SendEventSvc<T> {
                        type Response = super::SendEventResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Event>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).send_event(request).await };
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
                        let method = SendEventSvc(inner);
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
    impl<T: AstroRunRemoteRunner> Clone for AstroRunRemoteRunnerServer<T> {
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
    impl<T: AstroRunRemoteRunner> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: AstroRunRemoteRunner> tonic::server::NamedService
    for AstroRunRemoteRunnerServer<T> {
        const NAME: &'static str = "astro_run_remote_runner.AstroRunRemoteRunner";
    }
}

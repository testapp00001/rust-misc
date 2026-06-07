//! # gRPC Basics with Tonic
//!
//! gRPC is a high-performance RPC framework using Protocol Buffers. Tonic is
//! Rust's primary gRPC implementation. This lesson covers the conceptual model
//! of gRPC services, message types, streaming patterns, and interceptors.
//!
//! ## Key Concepts
//! - Service definition with protobuf concepts
//! - Unary, server-streaming, client-streaming, and bidirectional streaming
//! - Interceptors for auth/logging
//! - Error handling with gRPC status codes
//! - Connection management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. gRPC Status Codes (mirrors the official gRPC spec)
// ---------------------------------------------------------------------------

/// gRPC status codes as defined in the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrpcCode {
    Ok = 0,
    Cancelled = 1,
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    NotFound = 5,
    AlreadyExists = 6,
    PermissionDenied = 7,
    ResourceExhausted = 8,
    FailedPrecondition = 9,
    Aborted = 10,
    OutOfRange = 11,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
    DataLoss = 15,
    Unauthenticated = 16,
}

impl GrpcCode {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Ok),
            1 => Some(Self::Cancelled),
            2 => Some(Self::Unknown),
            3 => Some(Self::InvalidArgument),
            4 => Some(Self::DeadlineExceeded),
            5 => Some(Self::NotFound),
            6 => Some(Self::AlreadyExists),
            7 => Some(Self::PermissionDenied),
            8 => Some(Self::ResourceExhausted),
            9 => Some(Self::FailedPrecondition),
            10 => Some(Self::Aborted),
            11 => Some(Self::OutOfRange),
            12 => Some(Self::Unimplemented),
            13 => Some(Self::Internal),
            14 => Some(Self::Unavailable),
            15 => Some(Self::DataLoss),
            16 => Some(Self::Unauthenticated),
            _ => None,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Unavailable | Self::ResourceExhausted | Self::Aborted
        )
    }
}

/// A gRPC status with code, message, and optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcStatus {
    pub code: GrpcCode,
    pub message: String,
    pub details: Vec<StatusDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDetail {
    pub type_url: String,
    pub value: Vec<u8>,
}

impl GrpcStatus {
    pub fn ok() -> Self {
        Self {
            code: GrpcCode::Ok,
            message: String::new(),
            details: Vec::new(),
        }
    }

    pub fn not_found(resource: &str) -> Self {
        Self {
            code: GrpcCode::NotFound,
            message: format!("{resource} not found"),
            details: Vec::new(),
        }
    }

    pub fn invalid_argument(field: &str, reason: &str) -> Self {
        Self {
            code: GrpcCode::InvalidArgument,
            message: format!("invalid {field}: {reason}"),
            details: Vec::new(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            code: GrpcCode::Internal,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn unauthenticated(message: &str) -> Self {
        Self {
            code: GrpcCode::Unauthenticated,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.code == GrpcCode::Ok
    }
}

// ---------------------------------------------------------------------------
// 2. Service Method Types
// ---------------------------------------------------------------------------

/// The four types of gRPC method communication patterns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MethodType {
    /// Single request, single response.
    Unary,
    /// Single request, stream of responses.
    ServerStreaming,
    /// Stream of requests, single response.
    ClientStreaming,
    /// Stream of requests, stream of responses.
    BidiStreaming,
}

/// Describes a gRPC service method.
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    pub name: String,
    pub full_path: String, // e.g., "/package.ServiceName/MethodName"
    pub method_type: MethodType,
}

impl MethodDescriptor {
    pub fn new(service: &str, method: &str, method_type: MethodType) -> Self {
        Self {
            name: method.into(),
            full_path: format!("/{service}/{method}"),
            method_type,
        }
    }
}

/// Describes a gRPC service with its methods.
#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub name: String,
    pub package: String,
    pub methods: Vec<MethodDescriptor>,
}

impl ServiceDescriptor {
    pub fn full_name(&self) -> String {
        if self.package.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.package, self.name)
        }
    }

    pub fn find_method(&self, path: &str) -> Option<&MethodDescriptor> {
        self.methods.iter().find(|m| m.full_path == path)
    }
}

// ---------------------------------------------------------------------------
// 3. Generic Request/Response Wrappers
// ---------------------------------------------------------------------------

/// A generic gRPC request wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcRequest<T> {
    pub message: T,
    pub metadata: HashMap<String, String>,
    pub deadline: Option<u64>,
}

impl<T> GrpcRequest<T> {
    pub fn new(message: T) -> Self {
        Self {
            message,
            metadata: HashMap::new(),
            deadline: None,
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn get_auth_token(&self) -> Option<&str> {
        self.metadata.get("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
    }
}

/// A generic gRPC response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcResponse<T> {
    pub message: T,
    pub status: GrpcStatus,
    pub metadata: HashMap<String, String>,
}

impl<T> GrpcResponse<T> {
    pub fn ok(message: T) -> Self {
        Self {
            message,
            status: GrpcStatus::ok(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ---------------------------------------------------------------------------
// 4. Example Service Definition (User Service)
// ---------------------------------------------------------------------------

/// User service messages (as you'd define in .proto files).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtoUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersRequest {
    pub page_size: u32,
    pub page_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersResponse {
    pub users: Vec<ProtoUser>,
    pub next_page_token: String,
    pub total: u32,
}

/// Simulates a gRPC user service implementation.
pub struct UserServiceImpl {
    users: HashMap<String, ProtoUser>,
    next_id: u32,
}

impl UserServiceImpl {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    /// Unary RPC: Get a user by ID.
    pub fn get_user(&self, req: &GrpcRequest<GetUserRequest>) -> GrpcResponse<ProtoUser> {
        match self.users.get(&req.message.user_id) {
            Some(user) => GrpcResponse::ok(user.clone()),
            None => GrpcResponse {
                message: ProtoUser {
                    id: String::new(),
                    name: String::new(),
                    email: String::new(),
                    active: false,
                },
                status: GrpcStatus::not_found("user"),
                metadata: HashMap::new(),
            },
        }
    }

    /// Unary RPC: Create a new user.
    pub fn create_user(
        &mut self,
        req: &GrpcRequest<CreateUserRequest>,
    ) -> GrpcResponse<ProtoUser> {
        if req.message.name.is_empty() {
            return GrpcResponse {
                message: ProtoUser {
                    id: String::new(),
                    name: String::new(),
                    email: String::new(),
                    active: false,
                },
                status: GrpcStatus::invalid_argument("name", "cannot be empty"),
                metadata: HashMap::new(),
            };
        }

        if !req.message.email.contains('@') {
            return GrpcResponse {
                message: ProtoUser {
                    id: String::new(),
                    name: String::new(),
                    email: String::new(),
                    active: false,
                },
                status: GrpcStatus::invalid_argument("email", "invalid format"),
                metadata: HashMap::new(),
            };
        }

        let id = format!("user-{}", self.next_id);
        self.next_id += 1;

        let user = ProtoUser {
            id: id.clone(),
            name: req.message.name.clone(),
            email: req.message.email.clone(),
            active: true,
        };

        self.users.insert(id, user.clone());
        GrpcResponse::ok(user)
    }

    /// Server-streaming RPC: List users as a stream.
    pub fn list_users(&self, req: &GrpcRequest<ListUsersRequest>) -> Vec<ProtoUser> {
        let page_size = req.message.page_size.min(100) as usize;
        let skip = req.message.page_token.parse::<usize>().unwrap_or(0);

        let mut users: Vec<ProtoUser> = self.users.values().cloned().collect();
        users.sort_by(|a, b| a.id.cmp(&b.id));

        users.into_iter().skip(skip).take(page_size).collect()
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

// ---------------------------------------------------------------------------
// 5. Interceptor Pattern
// ---------------------------------------------------------------------------

/// An interceptor can inspect and modify requests before they reach the handler.
/// This is the Tonic interceptor pattern.
pub trait Interceptor {
    /// Process the request metadata. Return Ok(()) to proceed or Err to reject.
    fn intercept(&self, metadata: &HashMap<String, String>) -> Result<(), GrpcStatus>;
}

/// Authentication interceptor that checks for a valid bearer token.
pub struct AuthInterceptor {
    valid_tokens: Vec<String>,
}

impl AuthInterceptor {
    pub fn new(tokens: Vec<&str>) -> Self {
        Self {
            valid_tokens: tokens.into_iter().map(String::from).collect(),
        }
    }
}

impl Interceptor for AuthInterceptor {
    fn intercept(&self, metadata: &HashMap<String, String>) -> Result<(), GrpcStatus> {
        let token = metadata
            .get("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| GrpcStatus::unauthenticated("missing or invalid authorization header"))?;

        if self.valid_tokens.iter().any(|t| t == token) {
            Ok(())
        } else {
            Err(GrpcStatus::unauthenticated("invalid token"))
        }
    }
}

/// Logging interceptor that records method calls.
#[derive(Debug, Clone, Default)]
pub struct LoggingInterceptor {
    log: Vec<String>,
}

impl LoggingInterceptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&self) -> &[String] {
        &self.log
    }
}

impl Interceptor for LoggingInterceptor {
    fn intercept(&self, _metadata: &HashMap<String, String>) -> Result<(), GrpcStatus> {
        // In real code, we'd use interior mutability here
        Ok(())
    }
}

/// Chain multiple interceptors together.
pub struct InterceptorChain {
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl InterceptorChain {
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    pub fn add(mut self, interceptor: impl Interceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }

    pub fn check_all(&self, metadata: &HashMap<String, String>) -> Result<(), GrpcStatus> {
        for interceptor in &self.interceptors {
            interceptor.intercept(metadata)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 6. Service Descriptor Builder
// ---------------------------------------------------------------------------

/// Build a service descriptor programmatically.
pub fn build_user_service_descriptor() -> ServiceDescriptor {
    ServiceDescriptor {
        name: "UserService".into(),
        package: "example.v1".into(),
        methods: vec![
            MethodDescriptor::new("example.v1.UserService", "GetUser", MethodType::Unary),
            MethodDescriptor::new("example.v1.UserService", "CreateUser", MethodType::Unary),
            MethodDescriptor::new(
                "example.v1.UserService",
                "ListUsers",
                MethodType::ServerStreaming,
            ),
        ],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_code_from_i32() {
        assert_eq!(GrpcCode::from_i32(0), Some(GrpcCode::Ok));
        assert_eq!(GrpcCode::from_i32(5), Some(GrpcCode::NotFound));
        assert_eq!(GrpcCode::from_i32(16), Some(GrpcCode::Unauthenticated));
        assert_eq!(GrpcCode::from_i32(99), None);
    }

    #[test]
    fn test_grpc_code_retryable() {
        assert!(GrpcCode::Unavailable.is_retryable());
        assert!(GrpcCode::ResourceExhausted.is_retryable());
        assert!(GrpcCode::Aborted.is_retryable());
        assert!(!GrpcCode::Ok.is_retryable());
        assert!(!GrpcCode::NotFound.is_retryable());
        assert!(!GrpcCode::InvalidArgument.is_retryable());
    }

    #[test]
    fn test_grpc_status_helpers() {
        let status = GrpcStatus::ok();
        assert!(status.is_ok());

        let status = GrpcStatus::not_found("user");
        assert_eq!(status.code, GrpcCode::NotFound);
        assert!(status.message.contains("user"));

        let status = GrpcStatus::invalid_argument("email", "bad format");
        assert_eq!(status.code, GrpcCode::InvalidArgument);

        let status = GrpcStatus::internal("oops");
        assert_eq!(status.code, GrpcCode::Internal);

        let status = GrpcStatus::unauthenticated("no token");
        assert_eq!(status.code, GrpcCode::Unauthenticated);
    }

    #[test]
    fn test_grpc_request_metadata() {
        let req = GrpcRequest::new("payload")
            .with_metadata("authorization", "Bearer my-token")
            .with_metadata("x-request-id", "abc-123");

        assert_eq!(req.get_auth_token(), Some("my-token"));
        assert_eq!(req.metadata.get("x-request-id").unwrap(), "abc-123");
    }

    #[test]
    fn test_grpc_request_no_auth() {
        let req = GrpcRequest::new("payload");
        assert!(req.get_auth_token().is_none());
    }

    #[test]
    fn test_grpc_response_ok() {
        let resp = GrpcResponse::ok("data");
        assert!(resp.status.is_ok());
        assert_eq!(resp.message, "data");
    }

    #[test]
    fn test_method_type() {
        assert_eq!(MethodType::Unary, MethodType::Unary);
        assert_ne!(MethodType::Unary, MethodType::BidiStreaming);
    }

    #[test]
    fn test_service_descriptor() {
        let desc = build_user_service_descriptor();
        assert_eq!(desc.name, "UserService");
        assert_eq!(desc.package, "example.v1");
        assert_eq!(desc.full_name(), "example.v1.UserService");
        assert_eq!(desc.methods.len(), 3);
    }

    #[test]
    fn test_service_descriptor_find_method() {
        let desc = build_user_service_descriptor();

        let method = desc.find_method("/example.v1.UserService/GetUser");
        assert!(method.is_some());
        assert_eq!(method.unwrap().method_type, MethodType::Unary);

        assert!(desc.find_method("/nonexistent/Method").is_none());
    }

    #[test]
    fn test_user_service_create_and_get() {
        let mut svc = UserServiceImpl::new();

        let create_req = GrpcRequest::new(CreateUserRequest {
            name: "Alice".into(),
            email: "alice@example.com".into(),
        });
        let resp = svc.create_user(&create_req);
        assert!(resp.status.is_ok());
        let user_id = resp.message.id.clone();
        assert_eq!(resp.message.name, "Alice");

        let get_req = GrpcRequest::new(GetUserRequest {
            user_id: user_id.clone(),
        });
        let resp = svc.get_user(&get_req);
        assert!(resp.status.is_ok());
        assert_eq!(resp.message.email, "alice@example.com");
    }

    #[test]
    fn test_user_service_get_not_found() {
        let svc = UserServiceImpl::new();
        let req = GrpcRequest::new(GetUserRequest {
            user_id: "nonexistent".into(),
        });
        let resp = svc.get_user(&req);
        assert_eq!(resp.status.code, GrpcCode::NotFound);
    }

    #[test]
    fn test_user_service_create_invalid_name() {
        let mut svc = UserServiceImpl::new();
        let req = GrpcRequest::new(CreateUserRequest {
            name: "".into(),
            email: "test@example.com".into(),
        });
        let resp = svc.create_user(&req);
        assert_eq!(resp.status.code, GrpcCode::InvalidArgument);
    }

    #[test]
    fn test_user_service_create_invalid_email() {
        let mut svc = UserServiceImpl::new();
        let req = GrpcRequest::new(CreateUserRequest {
            name: "Alice".into(),
            email: "not-an-email".into(),
        });
        let resp = svc.create_user(&req);
        assert_eq!(resp.status.code, GrpcCode::InvalidArgument);
    }

    #[test]
    fn test_user_service_list() {
        let mut svc = UserServiceImpl::new();
        for i in 0..5 {
            svc.create_user(&GrpcRequest::new(CreateUserRequest {
                name: format!("User{i}"),
                email: format!("user{i}@example.com"),
            }));
        }

        let req = GrpcRequest::new(ListUsersRequest {
            page_size: 3,
            page_token: "0".into(),
        });
        let users = svc.list_users(&req);
        assert_eq!(users.len(), 3);

        let req2 = GrpcRequest::new(ListUsersRequest {
            page_size: 3,
            page_token: "3".into(),
        });
        let users2 = svc.list_users(&req2);
        assert_eq!(users2.len(), 2);
    }

    #[test]
    fn test_user_service_list_page_size_cap() {
        let mut svc = UserServiceImpl::new();
        for i in 0..200 {
            svc.create_user(&GrpcRequest::new(CreateUserRequest {
                name: format!("User{i}"),
                email: format!("user{i}@example.com"),
            }));
        }

        let req = GrpcRequest::new(ListUsersRequest {
            page_size: 500, // should be capped at 100
            page_token: "0".into(),
        });
        let users = svc.list_users(&req);
        assert_eq!(users.len(), 100);
    }

    #[test]
    fn test_auth_interceptor_valid_token() {
        let interceptor = AuthInterceptor::new(vec!["valid-token"]);
        let mut metadata = HashMap::new();
        metadata.insert("authorization".into(), "Bearer valid-token".into());

        assert!(interceptor.intercept(&metadata).is_ok());
    }

    #[test]
    fn test_auth_interceptor_invalid_token() {
        let interceptor = AuthInterceptor::new(vec!["valid-token"]);
        let mut metadata = HashMap::new();
        metadata.insert("authorization".into(), "Bearer wrong-token".into());

        let result = interceptor.intercept(&metadata);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, GrpcCode::Unauthenticated);
    }

    #[test]
    fn test_auth_interceptor_missing_header() {
        let interceptor = AuthInterceptor::new(vec!["token"]);
        let metadata = HashMap::new();

        let result = interceptor.intercept(&metadata);
        assert!(result.is_err());
    }

    #[test]
    fn test_interceptor_chain() {
        let chain = InterceptorChain::new()
            .add(AuthInterceptor::new(vec!["token1"]));

        let mut metadata = HashMap::new();
        metadata.insert("authorization".into(), "Bearer token1".into());
        assert!(chain.check_all(&metadata).is_ok());

        metadata.insert("authorization".into(), "Bearer wrong".into());
        assert!(chain.check_all(&metadata).is_err());
    }

    #[test]
    fn test_method_descriptor() {
        let method = MethodDescriptor::new("my.v1.Service", "DoStuff", MethodType::Unary);
        assert_eq!(method.full_path, "/my.v1.Service/DoStuff");
        assert_eq!(method.name, "DoStuff");
        assert_eq!(method.method_type, MethodType::Unary);
    }
}

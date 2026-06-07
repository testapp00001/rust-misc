//! # Axum Fundamentals
//!
//! Axum is an ergonomic, modular web framework built on top of Tower and Hyper.
//! This lesson covers the core building blocks: routing, handlers, extractors,
//! responses, shared state, and error handling patterns.
//!
//! ## Key Concepts
//! - `Router` for defining URL-to-handler mappings
//! - Extractors (`Path`, `Query`, `Json`, `State`) for parsing request data
//! - `IntoResponse` for building typed responses
//! - Shared application state with `State`
//! - Nested routers and route groups
//! - Error handling patterns with `Result` and `IntoResponse`

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ---------------------------------------------------------------------------
// 1. Domain Models
// ---------------------------------------------------------------------------

/// A user entity as stored in our "database."
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

/// Payload for creating a new user.
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub role: Option<UserRole>,
}

/// Payload for updating an existing user.
#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
}

/// Query parameters for listing users.
#[derive(Debug, Deserialize, Default)]
pub struct UserQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub role: Option<UserRole>,
    pub search: Option<String>,
}

// ---------------------------------------------------------------------------
// 2. Shared Application State
// ---------------------------------------------------------------------------

/// Thread-safe in-memory user store.
/// In production, this would be backed by a database pool.
pub type UserStore = Arc<RwLock<HashMap<u64, User>>>;

/// Application state that gets injected into handlers via `State<T>`.
#[derive(Clone)]
pub struct AppState {
    pub users: UserStore,
    pub next_id: Arc<RwLock<u64>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Seed with sample data for testing/demos.
    pub fn with_seed_data() -> Self {
        let state = Self::new();
        let seed = vec![
            User {
                id: 1,
                name: "Alice".into(),
                email: "alice@example.com".into(),
                role: UserRole::Admin,
            },
            User {
                id: 2,
                name: "Bob".into(),
                email: "bob@example.com".into(),
                role: UserRole::Editor,
            },
            User {
                id: 3,
                name: "Charlie".into(),
                email: "charlie@example.com".into(),
                role: UserRole::Viewer,
            },
        ];
        {
            let mut store = state.users.write().unwrap();
            let mut next_id = state.next_id.write().unwrap();
            for user in seed {
                store.insert(user.id, user);
            }
            *next_id = 4;
        }
        state
    }
}

// ---------------------------------------------------------------------------
// 3. API Error Type
// ---------------------------------------------------------------------------

/// A structured error type that implements `IntoResponse` so handlers
/// can return `Result<T, ApiError>` directly.
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

// ---------------------------------------------------------------------------
// 4. Standard API Response Wrapper
// ---------------------------------------------------------------------------

/// Wraps successful responses in a consistent envelope.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { data, meta: None }
    }

    pub fn with_meta(data: T, meta: serde_json::Value) -> Self {
        Self {
            data,
            meta: Some(meta),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Handler Functions
// ---------------------------------------------------------------------------

/// List all users with optional filtering and pagination.
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserQuery>,
) -> Result<Json<ApiResponse<Vec<User>>>, ApiError> {
    let store = state.users.read().map_err(|e| {
        ApiError::Internal(format!("lock poisoned: {e}"))
    })?;

    let mut users: Vec<User> = store.values().cloned().collect();

    // Filter by role
    if let Some(ref role) = query.role {
        users.retain(|u| u.role == *role);
    }

    // Search by name or email
    if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        users.retain(|u| {
            u.name.to_lowercase().contains(&search_lower)
                || u.email.to_lowercase().contains(&search_lower)
        });
    }

    // Sort by ID
    users.sort_by_key(|u| u.id);

    // Pagination
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let total = users.len();
    let start = ((page - 1) * per_page) as usize;
    let paged: Vec<User> = users.into_iter().skip(start).take(per_page as usize).collect();

    let meta = serde_json::json!({
        "page": page,
        "per_page": per_page,
        "total": total,
    });

    Ok(Json(ApiResponse::with_meta(paged, meta)))
}

/// Get a single user by ID.
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<User>>, ApiError> {
    let store = state.users.read().map_err(|e| {
        ApiError::Internal(format!("lock poisoned: {e}"))
    })?;

    store
        .get(&id)
        .cloned()
        .map(|u| Json(ApiResponse::ok(u)))
        .ok_or_else(|| ApiError::NotFound(format!("user {id} not found")))
}

/// Create a new user.
pub async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), ApiError> {
    // Validate
    if input.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name cannot be empty".into()));
    }
    if !input.email.contains('@') {
        return Err(ApiError::BadRequest("invalid email format".into()));
    }

    let id = {
        let mut next = state.next_id.write().map_err(|e| {
            ApiError::Internal(format!("lock poisoned: {e}"))
        })?;
        let id = *next;
        *next += 1;
        id
    };

    let user = User {
        id,
        name: input.name,
        email: input.email,
        role: input.role.unwrap_or(UserRole::Viewer),
    };

    {
        let mut store = state.users.write().map_err(|e| {
            ApiError::Internal(format!("lock poisoned: {e}"))
        })?;

        // Check for duplicate email
        if store.values().any(|u| u.email == user.email) {
            return Err(ApiError::Conflict(format!(
                "email {} already exists",
                user.email
            )));
        }

        store.insert(id, user.clone());
    }

    Ok((StatusCode::CREATED, Json(ApiResponse::ok(user))))
}

/// Update an existing user.
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateUser>,
) -> Result<Json<ApiResponse<User>>, ApiError> {
    let mut store = state.users.write().map_err(|e| {
        ApiError::Internal(format!("lock poisoned: {e}"))
    })?;

    // Validate email uniqueness before mutating
    if let Some(ref email) = input.email {
        if !email.contains('@') {
            return Err(ApiError::BadRequest("invalid email format".into()));
        }
        let has_duplicate = store.values().any(|u| u.id != id && u.email == *email);
        if has_duplicate {
            return Err(ApiError::Conflict(format!(
                "email {email} already exists"
            )));
        }
    }

    let user = store.get_mut(&id).ok_or_else(|| {
        ApiError::NotFound(format!("user {id} not found"))
    })?;

    if let Some(name) = input.name {
        if name.trim().is_empty() {
            return Err(ApiError::BadRequest("name cannot be empty".into()));
        }
        user.name = name;
    }

    if let Some(email) = input.email {
        user.email = email;
    }

    if let Some(role) = input.role {
        user.role = role;
    }

    Ok(Json(ApiResponse::ok(user.clone())))
}

/// Delete a user.
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    let mut store = state.users.write().map_err(|e| {
        ApiError::Internal(format!("lock poisoned: {e}"))
    })?;

    store
        .remove(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or_else(|| ApiError::NotFound(format!("user {id} not found")))
}

/// Health check endpoint.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// ---------------------------------------------------------------------------
// 6. Router Construction
// ---------------------------------------------------------------------------

/// Build the full application router with all routes and middleware.
pub fn build_router(state: AppState) -> Router {
    let user_routes = Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user));

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/users", user_routes)
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`

    fn test_app() -> Router {
        build_router(AppState::with_seed_data())
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_users() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let users = json["data"].as_array().unwrap();
        assert_eq!(users.len(), 3);
    }

    #[tokio::test]
    async fn test_list_users_filter_by_role() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/users?role=admin")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let users = json["data"].as_array().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["name"], "Alice");
    }

    #[tokio::test]
    async fn test_list_users_pagination() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/users?page=1&per_page=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let users = json["data"].as_array().unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(json["meta"]["total"], 3);
    }

    #[tokio::test]
    async fn test_get_user_found() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/users/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"]["name"], "Alice");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/users/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_user() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/users")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "name": "Dave",
                            "email": "dave@example.com",
                            "role": "editor"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"]["name"], "Dave");
        assert_eq!(json["data"]["id"], 4);
    }

    #[tokio::test]
    async fn test_create_user_invalid_email() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/users")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "name": "Bad User",
                            "email": "not-an-email"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_user_empty_name() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/users")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "name": "",
                            "email": "ok@example.com"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_update_user() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/users/1")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "name": "Alice Updated"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"]["name"], "Alice Updated");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/users/3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let app = test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/users/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        let store = state.users.read().unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn test_app_state_with_seed_data() {
        let state = AppState::with_seed_data();
        let store = state.users.read().unwrap();
        assert_eq!(store.len(), 3);
        assert!(store.contains_key(&1));
        assert!(store.contains_key(&2));
        assert!(store.contains_key(&3));
    }

    #[test]
    fn test_api_error_not_found_status() {
        let err = ApiError::NotFound("thing missing".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_api_error_bad_request_status() {
        let err = ApiError::BadRequest("bad input".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_conflict_status() {
        let err = ApiError::Conflict("duplicate".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_api_error_internal_status() {
        let err = ApiError::Internal("oops".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_api_response_ok() {
        let resp = ApiResponse::ok("hello");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["data"], "hello");
        assert!(json.get("meta").is_none());
    }

    #[test]
    fn test_api_response_with_meta() {
        let meta = serde_json::json!({"total": 42});
        let resp = ApiResponse::with_meta(vec![1, 2, 3], meta);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 3);
        assert_eq!(json["meta"]["total"], 42);
    }

    #[test]
    fn test_user_role_serde() {
        let json = serde_json::to_string(&UserRole::Admin).unwrap();
        assert_eq!(json, "\"admin\"");
        let role: UserRole = serde_json::from_str("\"editor\"").unwrap();
        assert_eq!(role, UserRole::Editor);
    }
}

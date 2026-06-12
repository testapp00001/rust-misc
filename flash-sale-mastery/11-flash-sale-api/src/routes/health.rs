//! # Health Route Handler
//!
//! GET /health -- reports Redis connectivity, DB connectivity, and overall status.
//!
//! ## Exercise
//!
//! 1. Ping Redis and report connectivity.
//! 2. Check the circuit breaker state.
//! 3. Return a structured health response.
//! 4. Write tests for healthy and degraded states.

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub redis: ComponentHealth,
    pub circuit_breaker: String,
    pub uptime_seconds: u64,
}

/// Individual component health.
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub latency_ms: Option<u64>,
}

/// Application start time (set once at startup).
static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Initialize the start time (call once from main).
pub fn init_start_time() {
    START_TIME.get_or_init(std::time::Instant::now);
}

/// GET /health handler.
#[cfg(feature = "solution")]
pub async fn handle_health(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    // Check Redis connectivity.
    let (redis_status, redis_latency) = {
        let start = std::time::Instant::now();
        let result = state.redis_pool.get().await;
        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(mut conn) => {
                // Try a PING.
                let ping_result: Result<String, _> = deadpool_redis::redis::cmd("PING")
                    .query_async(&mut conn)
                    .await;

                match ping_result {
                    Ok(_) => ("healthy".to_string(), Some(latency)),
                    Err(_) => ("unhealthy".to_string(), Some(latency)),
                }
            }
            Err(_) => ("unavailable".to_string(), None),
        }
    };

    let cb_state = format!("{:?}", state.circuit_breaker.state("redis"));

    let overall = if redis_status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    let uptime = START_TIME
        .get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    Json(HealthResponse {
        status: overall.to_string(),
        redis: ComponentHealth {
            status: redis_status,
            latency_ms: redis_latency,
        },
        circuit_breaker: cb_state,
        uptime_seconds: uptime,
    })
}

#[cfg(not(feature = "solution"))]
pub async fn handle_health(
    State(_state): State<AppState>,
) -> Json<HealthResponse> {
    todo!("Implement health check: ping Redis, check circuit breaker, return status")
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = crate::tests::test_app().await;
        let Some(app) = app else {
            eprintln!("SKIP: Redis not available");
            return;
        };

        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

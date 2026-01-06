// File: translator-server/src/routes/health.rs

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use crate::state::SharedState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub environment: String,
    pub redis_connected: bool,
}

pub async fn health_check(State(state): State<SharedState>) -> impl IntoResponse {
    let ping: Result<String, _> = redis::cmd("PING")
        .query_async(&mut state.redis.clone())
        .await;

    let redis_connected = ping.is_ok();

    let response = HealthResponse {
        status: if redis_connected { "healthy" } else { "degraded" },
        version: env!("CARGO_PKG_VERSION"),
        environment: state.config.environment.clone(),
        redis_connected,
    };

    let status = if redis_connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}

pub fn router() -> Router<SharedState> {
    Router::new().route("/health", get(health_check))
}
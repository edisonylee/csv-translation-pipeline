// File: translator-server/src/routes/stats.rs

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use redis::AsyncCommands;
use serde::Serialize;
use crate::state::SharedState;

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub translation_count: u64,
    pub job_count: u64,
    pub redis_connected: bool,
}

pub async fn get_stats(State(state): State<SharedState>) -> impl IntoResponse {
    let translation_keys: Result<Vec<String>, _> = state
        .redis
        .clone()
        .keys::<_, Vec<String>>("translation:*")
        .await;

    let translation_count = translation_keys.map(|k| k.len() as u64).unwrap_or(0);

    let job_keys: Result<Vec<String>, _> = state
        .redis
        .clone()
        .keys::<_, Vec<String>>("job:*")
        .await;

    let job_count = job_keys.map(|k| k.len() as u64).unwrap_or(0);

    let ping: Result<String, _> = redis::cmd("PING")
        .query_async(&mut state.redis.clone())
        .await;

    let stats = CacheStats {
        translation_count,
        job_count,
        redis_connected: ping.is_ok(),
    };

    (StatusCode::OK, Json(stats))
}

pub async fn clear_cache(State(state): State<SharedState>) -> impl IntoResponse {
    let keys: Result<Vec<String>, _> = state
        .redis
        .clone()
        .keys::<_, Vec<String>>("translation:*")
        .await;

    if let Ok(keys) = keys {
        if !keys.is_empty() {
            let _: Result<(), _> = state.redis.clone().del::<_, ()>(keys).await;
        }
    }

    StatusCode::NO_CONTENT
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/stats/cache", axum::routing::delete(clear_cache))
}
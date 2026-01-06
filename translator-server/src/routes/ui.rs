// File: translator-server/src/routes/ui.rs

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use redis::AsyncCommands;
use crate::state::SharedState;
use crate::templates::pages;

pub async fn home() -> impl IntoResponse {
    Html(pages::home().into_string())
}

pub async fn stats_page(State(state): State<SharedState>) -> impl IntoResponse {
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

    Html(pages::stats(translation_count, job_count, ping.is_ok()).into_string())
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(home))
        .route("/ui/stats", get(stats_page))
}
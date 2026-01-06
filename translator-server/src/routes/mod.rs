// File: translator-server/src/routes/mod.rs

pub mod health;
pub mod jobs;
pub mod stats;
pub mod translate;
pub mod ui;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use crate::state::SharedState;

pub fn create_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(health::router())
        .merge(translate::router())
        .merge(jobs::router())
        .merge(stats::router())
        .merge(ui::router())
        .layer(cors)
        .with_state(state)
}
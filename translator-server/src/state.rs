// File: translator-server/src/state.rs

use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use translator::{provider::mock::MockProvider, JobManager};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub jobs: JobManager<MockProvider>,
    pub redis: MultiplexedConnection,
    pub config: Config,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(config.redis_url.as_str())?;
        let redis = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            jobs: JobManager::new(MockProvider),
            redis,
            config,
        })
    }
}

pub type SharedState = Arc<AppState>;
// File: translator-server/src/config.rs

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub redis_url: String,
    pub port: u16,
    pub cache_ttl_seconds: u64,
    pub environment: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(86400),
            environment: env::var("RENDER_SERVICE_NAME")
                .map(|_| "production".into())
                .unwrap_or_else(|_| "development".into()),
        }
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
// File: translator-server/src/job_storage.rs

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::state::SharedState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredJob {
    pub texts: Vec<String>,
    pub src_lang: String,
    pub targets: HashMap<String, String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translations: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn job_key(job_id: &str) -> String {
    format!("job:{}", job_id)
}

pub struct JobStorage;

impl JobStorage {
    pub async fn store_pending(
        state: &SharedState,
        job_id: &str,
        texts: Vec<String>,
        src_lang: String,
        targets: HashMap<String, String>,
    ) {
        let job = StoredJob {
            texts,
            src_lang,
            targets,
            status: "pending".into(),
            translations: None,
            error: None,
        };

        let key = job_key(job_id);
        let value = serde_json::to_string(&job).unwrap_or_default();
        let ttl = 86400u64; // 24 hours

        let result: Result<(), _> = state.redis.clone().set_ex(&key, &value, ttl).await;

        if let Err(e) = result {
            tracing::warn!(error = %e, job_id = %job_id, "Failed to store job");
        }
    }

    pub async fn mark_complete(
        state: &SharedState,
        job_id: &str,
        translations: HashMap<String, Vec<String>>,
    ) {
        let key = job_key(job_id);
        let existing: Result<Option<String>, _> = state.redis.clone().get(&key).await;

        if let Ok(Some(value)) = existing {
            if let Ok(mut job) = serde_json::from_str::<StoredJob>(&value) {
                job.status = "complete".into();
                job.translations = Some(translations);
                let new_value = serde_json::to_string(&job).unwrap_or_default();
                let _: Result<(), _> = state.redis.clone().set(&key, &new_value).await;
            }
        }
    }

    pub async fn mark_failed(state: &SharedState, job_id: &str, error: String) {
        let key = job_key(job_id);
        let existing: Result<Option<String>, _> = state.redis.clone().get(&key).await;

        if let Ok(Some(value)) = existing {
            if let Ok(mut job) = serde_json::from_str::<StoredJob>(&value) {
                job.status = "failed".into();
                job.error = Some(error);
                let new_value = serde_json::to_string(&job).unwrap_or_default();
                let _: Result<(), _> = state.redis.clone().set(&key, &new_value).await;
            }
        }
    }

    pub async fn get(state: &SharedState, job_id: &str) -> Option<StoredJob> {
        let key = job_key(job_id);
        let result: Result<Option<String>, _> = state.redis.clone().get(&key).await;

        match result {
            Ok(Some(value)) => serde_json::from_str(&value).ok(),
            _ => None,
        }
    }

    pub async fn delete(state: &SharedState, job_id: &str) {
        let key = job_key(job_id);
        let _: Result<(), _> = state.redis.clone().del(&key).await;
    }
}
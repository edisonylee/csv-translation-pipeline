// File: translator-server/src/routes/jobs.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::Serialize;
use translator::JobStatus;
use crate::state::SharedState;
use super::translate::TranslateApiError;

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum JobStatusResponse {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "complete")]
    Complete {
        translations: std::collections::HashMap<String, Vec<String>>,
    },
    #[serde(rename = "failed")]
    Failed { error: String },
}

impl From<JobStatus> for JobStatusResponse {
    fn from(status: JobStatus) -> Self {
        match status {
            JobStatus::Pending => Self::Pending,
            JobStatus::Complete { translations } => Self::Complete { translations },
            JobStatus::Failed { error } => Self::Failed { error },
        }
    }
}

pub async fn get_job_status(
    State(state): State<SharedState>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, TranslateApiError> {
    let status = state
        .jobs
        .get(&job_id)
        .ok_or_else(|| TranslateApiError::NotFound(format!("Job {} not found", job_id)))?;

    let response: JobStatusResponse = status.into();
    Ok(Json(response))
}

pub async fn delete_job(
    State(state): State<SharedState>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, TranslateApiError> {
    state
        .jobs
        .remove(&job_id)
        .ok_or_else(|| TranslateApiError::NotFound(format!("Job {} not found", job_id)))?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/jobs/:id", get(get_job_status))
        .route("/jobs/:id", delete(delete_job))
}
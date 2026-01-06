use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use translator::TranslationRequest;
use crate::state::SharedState;
use crate::job_storage::JobStorage;


#[derive(Debug, Deserialize)]
pub struct SubmitTranslationRequest {
    pub texts: Vec<String>,
    pub src_lang: String,
    pub targets: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitTranslationResponse {
    pub job_id: String,
    pub status_url: String,
}

pub async fn submit_translation(
    State(state): State<SharedState>,
    Json(payload): Json<SubmitTranslationRequest>,
) -> Result<impl IntoResponse, TranslateApiError> {
    if payload.texts.is_empty() {
        return Err(TranslateApiError::BadRequest(
            "texts array cannot be empty".into(),
        ));
    }

    if payload.targets.is_empty() {
        return Err(TranslateApiError::BadRequest(
            "targets cannot be empty".into(),
        ));
    }

    let targets: Vec<(String, String)> = payload
        .targets
        .iter()
        .map(|(col, lang)| (col.clone(), lang.clone()))
        .collect();

    let request = translator::TranslationRequest {
        texts: payload.texts.clone(),
        src_lang: payload.src_lang.clone(),
        targets,
    };

    let job_id = state.jobs.submit(request);

    // Store job metadata in Redis
    JobStorage::store_pending(
        &state,
        &job_id,
        payload.texts,
        payload.src_lang,
        payload.targets,
    )
    .await;

    let response = SubmitTranslationResponse {
        status_url: format!("/jobs/{}", job_id),
        job_id,
    };

    Ok((StatusCode::ACCEPTED, Json(response)))
}

#[derive(Debug)]
pub enum TranslateApiError {
    BadRequest(String),
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for TranslateApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

pub fn router() -> Router<SharedState> {
    Router::new().route("/translate", post(submit_translation))
}
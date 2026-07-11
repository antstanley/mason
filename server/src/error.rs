use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error, Clone)]
pub enum AppError {
    #[error("missing required parameter: {0}")]
    BadRequest(&'static str),
    #[error("actor not found: {0}")]
    ActorNotFound(String),
    #[error("upstream error: {0}")]
    Upstream(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::ActorNotFound(_) => (StatusCode::NOT_FOUND, "actor_not_found"),
            AppError::Upstream(_) => (StatusCode::BAD_GATEWAY, "upstream"),
        };
        (status, Json(json!({ "error": code, "message": self.to_string() }))).into_response()
    }
}

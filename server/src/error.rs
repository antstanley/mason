use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // constructed from M1 when real sources land
pub enum AppError {
    #[error("actor not found: {0}")]
    ActorNotFound(String),
    #[error("upstream error: {0}")]
    Upstream(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::ActorNotFound(_) => (StatusCode::NOT_FOUND, "actor_not_found"),
            AppError::Upstream(_) => (StatusCode::BAD_GATEWAY, "upstream"),
        };
        (status, Json(json!({ "error": code, "message": self.to_string() }))).into_response()
    }
}

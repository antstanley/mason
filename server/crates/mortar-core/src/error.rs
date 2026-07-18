use serde_json::{Value, json};

#[derive(Debug, thiserror::Error, Clone)]
pub enum AppError {
    #[error("missing required parameter: {0}")]
    BadRequest(&'static str),
    #[error("actor not found: {0}")]
    ActorNotFound(String),
    #[error("login required: {0}")]
    LoginRequired(String),
    #[error("upstream error: {0}")]
    Upstream(String),
}

impl AppError {
    /// (http status, machine code); consumed by the axum IntoResponse in
    /// mortar-server and the Response builder in the service worker.
    pub fn status_and_code(&self) -> (u16, &'static str) {
        match self {
            AppError::BadRequest(_) => (400, "bad_request"),
            AppError::ActorNotFound(_) => (404, "actor_not_found"),
            // the owner asked to be seen only by signed-in visitors; mason has
            // no sign-in, so the wall stays sealed
            AppError::LoginRequired(_) => (403, "login_required"),
            AppError::Upstream(_) => (502, "upstream"),
        }
    }

    pub fn body(&self) -> Value {
        let (_, code) = self.status_and_code();
        json!({ "error": code, "message": self.to_string() })
    }
}

use serde::{Deserialize, Serialize};

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

/// The one wire shape for a mortar error, shared by both build modes: the
/// axum IntoResponse in mortar-server serializes it as the response body, and
/// mortar-wasm throws it as a JSON string for the service worker to rebuild
/// into a Response. The web side (web/src/service-worker.ts, web/src/lib/api.ts)
/// parses exactly this shape; the fixture test below pins it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    /// Machine code ("actor_not_found", "login_required", ...); the web
    /// classifies errors on this, so codes are wire contract, not cosmetics.
    pub error: String,
    /// Human-readable detail; display only, never matched on.
    pub message: String,
    /// HTTP status. Carried in-band only over the wasm channel, where the
    /// throw has no HTTP layer of its own; in server mode the status rides on
    /// the response line and is omitted from the body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
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

    /// The server-mode envelope: status rides on the HTTP response, not in
    /// the body.
    pub fn envelope(&self) -> ErrorEnvelope {
        let (_, code) = self.status_and_code();
        ErrorEnvelope {
            error: code.to_string(),
            message: self.to_string(),
            status: None,
        }
    }

    /// The wasm-mode envelope: the same body with the status spliced in-band,
    /// so the service worker can build a Response with the right status.
    pub fn envelope_with_status(&self) -> ErrorEnvelope {
        let (status, _) = self.status_and_code();
        ErrorEnvelope {
            status: Some(status),
            ..self.envelope()
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn variants() -> [AppError; 4] {
        [
            AppError::BadRequest("actor"),
            AppError::ActorNotFound("nobody.example.com".into()),
            AppError::LoginRequired("sealed.example.com".into()),
            AppError::Upstream("appview timed out".into()),
        ]
    }

    /// The TS parse contract: web/src/service-worker.ts JSON.parses exactly
    /// these strings off the wasm throw, and web/src/lib/state/feed.svelte.ts
    /// classifies on the `error` code. Any diff here is a wire change; treat
    /// it as one.
    #[test]
    fn wasm_envelope_is_pinned_per_variant() {
        let expected = [
            r#"{"error":"bad_request","message":"missing required parameter: actor","status":400}"#,
            r#"{"error":"actor_not_found","message":"actor not found: nobody.example.com","status":404}"#,
            r#"{"error":"login_required","message":"login required: sealed.example.com","status":403}"#,
            r#"{"error":"upstream","message":"upstream error: appview timed out","status":502}"#,
        ];
        for (error, wire) in variants().iter().zip(expected) {
            let json = serde_json::to_string(&error.envelope_with_status()).expect("serializes");
            assert_eq!(json, wire);
        }
    }

    /// Server mode sends the same body without the in-band status: the HTTP
    /// response line carries it instead.
    #[test]
    fn server_envelope_is_pinned_per_variant() {
        let expected = [
            r#"{"error":"bad_request","message":"missing required parameter: actor"}"#,
            r#"{"error":"actor_not_found","message":"actor not found: nobody.example.com"}"#,
            r#"{"error":"login_required","message":"login required: sealed.example.com"}"#,
            r#"{"error":"upstream","message":"upstream error: appview timed out"}"#,
        ];
        for (error, wire) in variants().iter().zip(expected) {
            let json = serde_json::to_string(&error.envelope()).expect("serializes");
            assert_eq!(json, wire);
        }
    }

    /// The envelope survives its own round trip in both shapes, so a native
    /// mortar's error body (status absent) parses back into the same struct
    /// the wasm side throws (status in-band).
    #[test]
    fn envelope_round_trips() {
        for error in variants() {
            for envelope in [error.envelope(), error.envelope_with_status()] {
                let json = serde_json::to_string(&envelope).expect("serializes");
                let back: ErrorEnvelope = serde_json::from_str(&json).expect("parses");
                assert_eq!(back, envelope);
            }
        }
    }
}

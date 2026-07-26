use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use mortar_core::error::AppError;
use mortar_core::feed::{FeedIntent, FeedTarget, handle_feed};
use mortar_core::mode::Mode;
use mortar_core::model::FeedResponse;
use mortar_core::state::AppState;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FeedParams {
    pub actor: Option<String>,
    /// A feed generator reference: an AT-URI or a bsky.app feed link. One of
    /// this and `actor` names the wall, and `feed` wins when both are given;
    /// `FeedTarget::from_query` owns that rule for both fronts.
    pub feed: Option<String>,
    pub cursor: Option<String>,
    /// The wall variant: "glaze" for the image wall, absent for the full wall.
    pub mode: Option<String>,
    /// "preview" or "freeze" drive the warm-then-commit first screen; absent is
    /// a normal committed page. Server mode serves the same SPA, so it honours
    /// these exactly as the wasm front does.
    pub intent: Option<String>,
}

pub struct ErrorResponse(AppError);

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let (status, _) = self.0.status_and_code();
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        // the same ErrorEnvelope the wasm build throws, minus the in-band
        // status: here the response line carries it
        (status, Json(self.0.envelope())).into_response()
    }
}

pub async fn feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FeedParams>,
) -> Result<Json<FeedResponse>, ErrorResponse> {
    // the precedence between the two, and the missing-parameter error, are
    // mortar's rather than this route's: the wasm front answers the same query
    // string and nothing here can be tested from mortar-core
    let target = FeedTarget::from_query(params.actor.as_deref(), params.feed.as_deref())
        .map_err(ErrorResponse)?;
    let mode = Mode::from_query(params.mode.as_deref());
    let intent = FeedIntent::from_query(params.intent.as_deref());
    handle_feed(&state, target, params.cursor.as_deref(), mode, intent)
        .await
        .map(Json)
        .map_err(ErrorResponse)
}

mod feed;
mod health;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use mortar_core::state::AppState;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

/// Build the router. `allowed_origins` is the CORS allowlist for the SPA that
/// calls this unauthenticated, read-only API: only these origins may read the
/// feed cross-origin, only GET (the layer answers the OPTIONS preflight itself),
/// and never with credentials. An unparseable origin is dropped; an empty list
/// allows no cross-origin reader at all.
pub fn router(state: Arc<AppState>, allowed_origins: &[String]) -> Router {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    // server mode is called directly from the static SPA's origin - there is no
    // proxy anymore, so the SPA origin(s) are named explicitly rather than
    // waved through with a permissive wildcard.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET]);

    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/feed", get(feed::feed))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

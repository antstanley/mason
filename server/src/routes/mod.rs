mod feed;
mod health;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/feed", get(feed::feed))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

mod feed;
mod health;

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use mortar_core::state::AppState;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/feed", get(feed::feed))
        .layer(TraceLayer::new_for_http())
        // server mode is called directly from the static SPA's origin -
        // there is no proxy anymore
        .layer(CorsLayer::permissive())
        .with_state(state)
}

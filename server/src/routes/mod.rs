mod feed;
mod health;

use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/feed", get(feed::feed))
        .layer(TraceLayer::new_for_http())
}

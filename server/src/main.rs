mod algo;
mod cache;
mod config;
mod error;
mod fixtures;
mod http;
mod model;
mod routes;
mod sources;
mod state;

use std::sync::Arc;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "mortar=debug,info".into()),
        )
        .init();

    let config = config::Config::from_env();
    let port = config.port;
    let state = Arc::new(state::AppState::new(config));
    let app = routes::router(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind");
    tracing::info!("mortar mixing on port {port}");
    axum::serve(listener, app).await.expect("server error");
}

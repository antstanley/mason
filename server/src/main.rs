mod algo;
mod config;
mod error;
mod fixtures;
mod model;
mod routes;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "mortar=debug,info".into()),
        )
        .init();

    let config = config::Config::from_env();
    let app = routes::router();

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("failed to bind");
    tracing::info!("mortar mixing on port {}", config.port);
    axum::serve(listener, app).await.expect("server error");
}

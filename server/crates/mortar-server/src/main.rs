mod routes;

use std::sync::Arc;

use mortar_core::config::Config;
use mortar_core::state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mortar_server=debug,mortar_core=debug,info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8787);
    let config = Config {
        appview_base: env_or("APPVIEW_BASE", "https://public.api.bsky.app"),
        plc_base: env_or("PLC_BASE", "https://plc.directory"),
        steam_store_base: env_or("STEAM_STORE_BASE", "https://store.steampowered.com"),
        steam_enabled: true,
    };
    let state = Arc::new(AppState::new(config));
    let app = routes::router(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("failed to bind");
    tracing::info!("mortar mixing on port {port}");
    axum::serve(listener, app).await.expect("server error");
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

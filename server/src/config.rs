/// Runtime configuration. Every upstream base URL is overridable so tests can
/// point mortar at a wiremock server instead of the real network.
#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    #[allow(dead_code)] // consumed by sources from M1 onward
    pub appview_base: String,
    #[allow(dead_code)]
    pub plc_base: String,
    #[allow(dead_code)]
    pub steam_store_base: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env_or("PORT", 8787),
            appview_base: env_or("APPVIEW_BASE", "https://public.api.bsky.app".to_string()),
            plc_base: env_or("PLC_BASE", "https://plc.directory".to_string()),
            steam_store_base: env_or("STEAM_STORE_BASE", "https://store.steampowered.com".to_string()),
        }
    }
}

fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

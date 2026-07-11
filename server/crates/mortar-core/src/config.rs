/// Feed-engine configuration, shared by the native server and the wasm
/// service worker. Every upstream base URL is overridable so tests can point
/// mortar at a wiremock server instead of the real network.
#[derive(Clone, Debug)]
pub struct Config {
    pub appview_base: String,
    pub plc_base: String,
    pub steam_store_base: String,
    /// Steam's storefront API has no CORS headers, so the browser build
    /// disables it (or routes it through a proxy via `steam_store_base`).
    pub steam_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appview_base: "https://public.api.bsky.app".into(),
            plc_base: "https://plc.directory".into(),
            steam_store_base: "https://store.steampowered.com".into(),
            steam_enabled: true,
        }
    }
}

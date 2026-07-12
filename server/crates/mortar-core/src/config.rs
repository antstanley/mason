/// Feed-engine configuration, shared by the native server and the wasm
/// service worker. Every upstream base URL is overridable so tests can point
/// mortar at a wiremock server instead of the real network.
#[derive(Clone, Debug)]
pub struct Config {
    pub appview_base: String,
    pub plc_base: String,
    /// Streamplace: the live list, archived streams, and HLS playback for
    /// both. It serves permissive CORS, so the browser build talks to it
    /// directly, exactly as the server build does.
    pub streamplace_base: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appview_base: "https://public.api.bsky.app".into(),
            plc_base: "https://plc.directory".into(),
            streamplace_base: "https://stream.place".into(),
        }
    }
}

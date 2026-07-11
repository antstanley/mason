//! mortar compiled for the browser: a thin wasm-bindgen wrapper around
//! mortar-core, driven by the SvelteKit service worker. State lives in a
//! thread_local — the SW is single-threaded and may be killed at any idle
//! moment; the cursor's embedded seed makes rebuilds deterministic.

use std::cell::RefCell;
use std::sync::Arc;

use mortar_core::config::Config;
use mortar_core::feed::handle_feed;
use mortar_core::state::AppState;
use wasm_bindgen::prelude::*;

thread_local! {
    static STATE: RefCell<Option<Arc<AppState>>> = const { RefCell::new(None) };
}

fn state() -> Arc<AppState> {
    STATE.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            console_error_panic_hook::set_once();
            // Steam's storefront API has no CORS headers — disabled in the
            // browser build. Point steam_store_base at a proxy and flip
            // steam_enabled via init_config to bring trailers back.
            *slot = Some(Arc::new(AppState::new(Config {
                steam_enabled: false,
                ..Config::default()
            })));
        }
        Arc::clone(slot.as_ref().expect("state initialized"))
    })
}

/// Optional: override upstreams before first use (e.g. a Steam CORS proxy).
#[wasm_bindgen]
pub fn init_config(steam_proxy_base: Option<String>) {
    STATE.with(|cell| {
        let mut config = Config {
            steam_enabled: false,
            ..Config::default()
        };
        if let Some(base) = steam_proxy_base {
            config.steam_store_base = base;
            config.steam_enabled = true;
        }
        console_error_panic_hook::set_once();
        *cell.borrow_mut() = Some(Arc::new(AppState::new(config)));
    });
}

/// One feed page as a JSON string (FeedResponse). Errors throw a JSON string
/// `{"status": u16, "error": code, "message": ...}` so the service worker
/// can build a Response with the right status.
#[wasm_bindgen]
pub async fn feed_page(actor: String, cursor: Option<String>) -> Result<String, JsValue> {
    let state = state();
    match handle_feed(&state, &actor, cursor.as_deref()).await {
        Ok(response) => {
            serde_json::to_string(&response).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(error) => {
            let (status, _) = error.status_and_code();
            let mut body = error.body();
            body["status"] = serde_json::json!(status);
            Err(JsValue::from_str(&body.to_string()))
        }
    }
}

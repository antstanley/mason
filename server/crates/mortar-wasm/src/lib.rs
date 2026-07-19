//! mortar compiled for the browser: a thin wasm-bindgen wrapper around
//! mortar-core, driven by the SvelteKit service worker. State lives in a
//! thread_local; the SW is single-threaded and may be killed at any idle
//! moment; the cursor's embedded seed makes rebuilds deterministic.

use std::cell::RefCell;
use std::sync::Arc;

use mortar_core::config::Config;
use mortar_core::feed::{FeedIntent, handle_feed};
use mortar_core::mode::Mode;
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
            // Every upstream mason reads (the AppView, plc.directory, each
            // PDS, Streamplace) serves permissive CORS, so the browser build
            // needs no overrides and no proxy: it is the same engine talking
            // to the same network, from a different address.
            *slot = Some(Arc::new(AppState::new(Config::default())));
        }
        Arc::clone(slot.as_ref().expect("state initialized"))
    })
}

/// Snapshot of the warm caches as JSON; the service worker stores this in
/// IndexedDB after serving a page, so a reaped SW instance wakes up warm.
#[wasm_bindgen]
pub async fn export_caches() -> Result<String, JsValue> {
    let state = state();
    let bundle = mortar_core::persist::export(&state.caches).await;
    serde_json::to_string(&bundle).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Restore a previously exported bundle. Anything unparseable or stale is
/// silently discarded; it's only a cache.
#[wasm_bindgen]
pub async fn import_caches(json: String) {
    if let Ok(bundle) = serde_json::from_str(&json) {
        let state = state();
        mortar_core::persist::import(&state.caches, bundle).await;
    }
}

/// One feed page as a JSON string (FeedResponse). `mode` is the wall variant
/// ("glaze" for the image wall; anything else is the full wall). `intent` is
/// "preview" or "freeze" for the warm-then-commit first screen, absent for a
/// normal committed page. Errors throw a JSON string
/// `{"status": u16, "error": code, "message": ...}` so the service worker can
/// build a Response with the right status.
#[wasm_bindgen]
pub async fn feed_page(
    actor: String,
    cursor: Option<String>,
    mode: Option<String>,
    intent: Option<String>,
) -> Result<String, JsValue> {
    let state = state();
    let mode = Mode::from_query(mode.as_deref());
    let intent = FeedIntent::from_query(intent.as_deref());
    match handle_feed(&state, &actor, cursor.as_deref(), mode, intent).await {
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

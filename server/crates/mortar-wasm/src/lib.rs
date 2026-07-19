//! mortar compiled for the browser: a thin wasm-bindgen wrapper around
//! mortar-core, driven by the SvelteKit service worker. State lives in a
//! thread_local; the SW is single-threaded and may be killed at any idle
//! moment; the cursor's embedded seed makes rebuilds deterministic.

use std::cell::RefCell;
use std::sync::Arc;

use mortar_core::config::Config;
use mortar_core::error::ErrorEnvelope;
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

/// Every persistable cache name; the service worker iterates this on startup
/// to import whatever survived in IndexedDB.
#[wasm_bindgen]
pub fn cache_names() -> Vec<String> {
    mortar_core::persist::CACHE_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

/// Names of the caches written to since their last export; the service worker
/// persists exactly these, so a page that only warmed one cache only pays for
/// one.
#[wasm_bindgen]
pub fn dirty_cache_names() -> Vec<String> {
    let state = state();
    mortar_core::persist::dirty_cache_names(&state.caches)
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

/// One cache as JSON, dirty flag cleared; the service worker stores it under
/// a per-cache IndexedDB key, so a reaped SW instance wakes up warm.
#[wasm_bindgen]
pub async fn export_cache(name: String) -> Option<String> {
    let state = state();
    mortar_core::persist::export_cache(&state.caches, &name).await
}

/// Restore one previously exported cache. Anything unparseable or stale is
/// silently discarded; it's only a cache.
#[wasm_bindgen]
pub async fn import_cache(name: String, json: String) {
    let state = state();
    mortar_core::persist::import_cache(&state.caches, &name, &json).await;
}

/// Serialize an ErrorEnvelope into the JsValue this module throws. Everything
/// feed_page throws goes through here, so the service worker only ever has to
/// parse one shape: the envelope pinned in mortar-core's error.rs.
fn throw(envelope: ErrorEnvelope) -> JsValue {
    // an envelope is two strings and an int; serializing it cannot fail
    JsValue::from_str(&serde_json::to_string(&envelope).expect("envelope serializes"))
}

/// One feed page as a JSON string (FeedResponse). `mode` is the wall variant
/// ("glaze" for the image wall; anything else is the full wall). `intent` is
/// "preview" or "freeze" for the warm-then-commit first screen, absent for a
/// normal committed page. Errors throw the ErrorEnvelope JSON
/// `{"error": code, "message": ..., "status": u16}` so the service worker can
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
        Ok(response) => serde_json::to_string(&response).map_err(|e| {
            // even a serializer failure speaks the envelope, so the service
            // worker never sees a bare non-JSON message on this channel
            throw(ErrorEnvelope {
                error: "internal".to_string(),
                message: e.to_string(),
                status: Some(500),
            })
        }),
        Err(error) => Err(throw(error.envelope_with_status())),
    }
}

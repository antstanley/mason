//! The feed entrypoint shared by both fronts: the axum route and the wasm
//! service worker are thin wrappers around `handle_feed`.

use std::sync::Arc;

use crate::algo::cursor::{self, Cursor};
use crate::algo::snapshot;
use crate::error::AppError;
use crate::fixtures;
use crate::http::HttpError;
use crate::model::FeedResponse;
use crate::sources::bluesky;
use crate::state::AppState;

pub const PAGE_SIZE: usize = 24;

pub async fn handle_feed(
    state: &Arc<AppState>,
    actor: &str,
    cursor: Option<&str>,
) -> Result<FeedResponse, AppError> {
    let decoded = cursor.and_then(cursor::decode);

    // offline demo wall, kept from M0
    if actor == "demo" {
        return Ok(demo_page(decoded.map(|c| c.offset).unwrap_or(0)));
    }

    let did = resolve_actor(state, actor).await?;
    let (seed, offset) = match decoded {
        Some(c) => (c.seed, c.offset),
        None => (snapshot::fresh_seed(&did), 0),
    };

    let snap = snapshot::get_or_build(state, &did, seed).await?;
    let (items, has_more) = snapshot::get_page(state, &snap, offset, PAGE_SIZE).await;
    let next = has_more.then(|| {
        cursor::encode(&Cursor {
            snapshot: snap.id.clone(),
            seed,
            offset: offset + items.len(),
        })
    });
    Ok(FeedResponse {
        items,
        cursor: next,
    })
}

async fn resolve_actor(state: &Arc<AppState>, actor: &str) -> Result<String, AppError> {
    if actor.starts_with("did:") {
        return Ok(actor.to_string());
    }
    if let Some(did) = state.caches.did.get(&actor.to_string()).await {
        return Ok(did);
    }
    match bluesky::resolve_handle(&state.http, &state.config.appview_base, actor).await {
        Ok(did) => {
            state
                .caches
                .did
                .insert(actor.to_string(), did.clone())
                .await;
            Ok(did)
        }
        Err(HttpError::Status(400 | 404)) => Err(AppError::ActorNotFound(actor.to_string())),
        Err(e) => Err(AppError::Upstream(e.to_string())),
    }
}

fn demo_page(offset: usize) -> FeedResponse {
    let pool = fixtures::pool();
    let items: Vec<_> = pool.iter().skip(offset).take(PAGE_SIZE).cloned().collect();
    let next_offset = offset + items.len();
    let cursor = (next_offset < pool.len()).then(|| {
        cursor::encode(&Cursor {
            snapshot: "fixture".into(),
            seed: 0,
            offset: next_offset,
        })
    });
    FeedResponse { items, cursor }
}

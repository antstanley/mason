use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::algo::cursor::{self, Cursor};
use crate::algo::snapshot;
use crate::error::AppError;
use crate::fixtures;
use crate::http::HttpError;
use crate::model::FeedResponse;
use crate::sources::bluesky;
use crate::state::AppState;

const PAGE_SIZE: usize = 24;

#[derive(Deserialize)]
pub struct FeedParams {
    pub actor: Option<String>,
    pub cursor: Option<String>,
}

pub async fn feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FeedParams>,
) -> Result<Json<FeedResponse>, AppError> {
    let actor = params.actor.ok_or(AppError::BadRequest("actor"))?;
    let decoded = params.cursor.as_deref().and_then(cursor::decode);

    // offline demo wall, kept from M0
    if actor == "demo" {
        return Ok(Json(demo_page(decoded.map(|c| c.offset).unwrap_or(0))));
    }

    let did = resolve_actor(&state, &actor).await?;
    let (seed, offset) = match decoded {
        Some(c) => (c.seed, c.offset),
        None => (snapshot::fresh_seed(&did), 0),
    };

    let snap = snapshot::get_or_build(&state, &did, seed).await?;
    let (items, has_more) = snapshot::get_page(&snap, offset, PAGE_SIZE).await;
    let next = has_more.then(|| {
        cursor::encode(&Cursor {
            snapshot: snap.id.clone(),
            seed,
            offset: offset + items.len(),
        })
    });
    Ok(Json(FeedResponse {
        items,
        cursor: next,
    }))
}

async fn resolve_actor(state: &Arc<AppState>, actor: &str) -> Result<String, AppError> {
    if actor.starts_with("did:") {
        return Ok(actor.to_string());
    }
    let http = &state.http;
    let base = state.config.appview_base.clone();
    let handle = actor.to_string();
    state
        .caches
        .did
        .try_get_with(actor.to_string(), async move {
            bluesky::resolve_handle(http, &base, &handle).await
        })
        .await
        .map_err(|e: Arc<HttpError>| match e.as_ref() {
            HttpError::Status(400) | HttpError::Status(404) => {
                AppError::ActorNotFound(actor.to_string())
            }
            other => AppError::Upstream(other.to_string()),
        })
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

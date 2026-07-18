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

    // A wall is the owner's social graph on display; if they asked to be seen
    // only by signed-in visitors, a logged-out mason must not lay it. Their own
    // posts never reach the fill, so this is the one place their opt-out is
    // checked. Fail open on an upstream hiccup: the preference is best-effort,
    // and treating a flaky getProfile as a wall of nothing helps no one.
    if owner_wants_auth(state, &did).await {
        return Err(AppError::LoginRequired(actor.to_string()));
    }

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

/// Whether the wall owner opted out of logged-out visibility, cached for an
/// hour. A miss costs one getProfile; an upstream error is treated as "not
/// opted out" so a flaky AppView never seals a wall by accident.
async fn owner_wants_auth(state: &Arc<AppState>, did: &str) -> bool {
    if let Some(cached) = state.caches.profiles.get(&did.to_string()).await {
        return cached;
    }
    let opted_out =
        match bluesky::get_profile_optout(&state.http, &state.config.appview_base, did).await {
            Ok(opted_out) => opted_out,
            Err(e) => {
                tracing::debug!("profile opt-out check for {did} failed: {e}");
                false
            }
        };
    state
        .caches
        .profiles
        .insert(did.to_string(), opted_out)
        .await;
    opted_out
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A wall owner who opted out of logged-out visibility gets a login-required
    /// error, and no snapshot is built. A `did:` actor skips handle resolution,
    /// so the only upstream call this needs to mock is getProfile.
    #[tokio::test]
    async fn an_opted_out_owner_seals_their_wall() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:owner",
                "handle": "owner.test",
                "labels": [{"val": "!no-unauthenticated"}]
            })))
            .mount(&server)
            .await;

        let state = Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            ..Default::default()
        }));

        let err = handle_feed(&state, "did:plc:owner", None)
            .await
            .expect_err("an opted-out owner must not lay a wall");
        assert!(matches!(err, AppError::LoginRequired(_)));
        assert_eq!(err.status_and_code(), (403, "login_required"));
    }
}

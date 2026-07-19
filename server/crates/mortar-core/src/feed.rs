//! The feed entrypoint shared by both fronts: the axum route and the wasm
//! service worker are thin wrappers around `handle_feed`.

use std::sync::Arc;

use crate::algo::cursor::{self, Cursor};
use crate::algo::snapshot;
use crate::error::AppError;
use crate::fixtures;
use crate::http::HttpError;
use crate::mode::Mode;
use crate::model::{Brick, FeedResponse};
use crate::sources::bluesky;
use crate::state::AppState;

pub const PAGE_SIZE: usize = 24;

/// What a feed request is for.
///
/// The wasm front polls `Preview` while a wall warms — each poll lays a fresh,
/// non-committed first screen from the growing pool, which the client reflows —
/// then asks `Freeze` exactly once to commit that screen and begin paging. The
/// native server (and any client without the preview loop) asks `Normal`, which
/// waits for a good mix before committing the first page so it does not open on
/// nothing but posts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FeedIntent {
    #[default]
    Normal,
    Preview,
    Freeze,
}

impl FeedIntent {
    pub fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some("preview") => Self::Preview,
            Some("freeze") => Self::Freeze,
            _ => Self::Normal,
        }
    }
}

pub async fn handle_feed(
    state: &Arc<AppState>,
    actor: &str,
    cursor: Option<&str>,
    mode: Mode,
    intent: FeedIntent,
) -> Result<FeedResponse, AppError> {
    let decoded = cursor.and_then(cursor::decode);

    // offline demo wall, kept from M0. Its bricks are fixtures compiled into the
    // wasm, so there is nothing to warm: a preview reports itself already
    // settled, and the client freezes to the real page at once.
    if actor == "demo" {
        let offset = decoded.map(|c| c.offset).unwrap_or(0);
        let mut page = demo_page(offset, mode);
        if intent == FeedIntent::Preview {
            page.warming = Some(false);
            // a preview's cursor points at the CURRENT screen (not the next
            // page), so the freeze that follows commits from here. Demo warms
            // instantly, so the client freezes on the first poll either way.
            page.cursor = Some(cursor::encode(&Cursor {
                snapshot: "fixture".into(),
                seed: 0,
                offset,
            }));
        }
        return Ok(page);
    }

    // Resolving the actor and reading the owner's opt-out are the same call
    // now: getProfile carries both the DID and the label. A wall is the owner's
    // social graph on display; if they asked to be seen only by signed-in
    // visitors, a logged-out mason must not lay it. Their own posts never reach
    // the fill, so this is the one place their opt-out is checked.
    let did = resolve_and_gate(state, actor).await?;

    let (seed, offset) = match decoded {
        Some(c) => (c.seed, c.offset),
        None => (snapshot::fresh_seed(&did), 0),
    };

    // A preview never commits and never waits: it lays the current best first
    // screen from a clone of the pool and reports whether more is still on the
    // way. The cursor it hands back carries the same seed, so the next poll (and
    // the freeze) land on this very snapshot rather than rolling a new one.
    if intent == FeedIntent::Preview {
        let snap = snapshot::ensure_snapshot(state, &did, seed, mode).await;
        let (items, warming) = snapshot::preview_page(&snap, PAGE_SIZE).await;
        return Ok(FeedResponse {
            items,
            cursor: Some(cursor::encode(&Cursor {
                snapshot: snap.id.clone(),
                seed,
                offset: 0,
            })),
            warming: Some(warming),
        });
    }

    let snap = snapshot::get_or_build(state, &did, seed, mode).await;
    // Freeze commits the first screen immediately: the preview loop already gave
    // the reader the warming reflow, so re-paying the mix wait here is the exact
    // stall reflow exists to remove. Normal (server mode, no preview loop) still
    // waits, so its first page is a proper mix and not just the fast posts.
    let wait_for_mix = intent == FeedIntent::Normal;
    let (items, has_more) = snapshot::get_page(state, &snap, offset, PAGE_SIZE, wait_for_mix).await;
    let next = has_more.then(|| {
        cursor::encode(&Cursor {
            snapshot: snap.id.clone(),
            seed,
            // saturating: the offset came off an attacker-writable cursor
            offset: offset.saturating_add(items.len()),
        })
    });
    Ok(FeedResponse {
        items,
        cursor: next,
        warming: None,
    })
}

/// Resolve `actor` to a DID and, in the same breath, honour the owner's
/// logged-out opt-out. Returns the DID, or `LoginRequired` for a sealed wall.
///
/// One `getProfile` does what used to take a `resolveHandle` then a separate
/// `getProfile`: its response carries both the DID and the opt-out label, so a
/// cold handle load pays one AppView round trip on this path instead of two.
///
/// The fail direction depends on what is already known. When the DID is not yet
/// in hand (a cold handle), this call is load-bearing for resolution, so a
/// network error fails the wall closed (`Upstream`) exactly as handle
/// resolution always did. When the DID is already known (a `did:` actor or a
/// cached handle) and only the opt-out is outstanding, the preference is
/// best-effort: a flaky getProfile is treated as "not opted out" so it can
/// never seal a wall by accident.
async fn resolve_and_gate(state: &Arc<AppState>, actor: &str) -> Result<String, AppError> {
    let known_did = if actor.starts_with("did:") {
        Some(actor.to_string())
    } else {
        state.caches.did.get(&actor.to_string()).await
    };

    // DID already in hand: the only open question is the opt-out, and it fails
    // open. A cached negative answer needs no network at all.
    if let Some(did) = known_did {
        if let Some(opted_out) = state.caches.profiles.get(&did).await {
            return gate(actor, did, opted_out);
        }
        return match bluesky::get_profile(&state.http, &state.config.appview_base, &did).await {
            Ok(profile) => {
                state
                    .caches
                    .profiles
                    .insert(did.clone(), profile.opted_out)
                    .await;
                gate(actor, did, profile.opted_out)
            }
            Err(e) => {
                // best-effort: never let a flaky getProfile seal a known wall
                tracing::debug!("profile opt-out check for {did} failed: {e}");
                Ok(did)
            }
        };
    }

    // Cold handle: one getProfile resolves the DID and reads the opt-out. This
    // call is load-bearing, so its failure fails the wall closed.
    match bluesky::get_profile(&state.http, &state.config.appview_base, actor).await {
        Ok(profile) => {
            state
                .caches
                .did
                .insert(actor.to_string(), profile.did.clone())
                .await;
            state
                .caches
                .profiles
                .insert(profile.did.clone(), profile.opted_out)
                .await;
            gate(actor, profile.did, profile.opted_out)
        }
        Err(HttpError::Status(400 | 404)) => Err(AppError::ActorNotFound(actor.to_string())),
        Err(e) => Err(AppError::Upstream(e.to_string())),
    }
}

/// A sealed wall becomes an error; an open one hands back its DID.
fn gate(actor: &str, did: String, opted_out: bool) -> Result<String, AppError> {
    if opted_out {
        Err(AppError::LoginRequired(actor.to_string()))
    } else {
        Ok(did)
    }
}

fn demo_page(offset: usize, mode: Mode) -> FeedResponse {
    let pool = fixtures::pool();
    // the offline demo obeys the mode too: glaze narrows the fixture wall to its
    // image-bearing posts, so toggling it on `demo` shows the same shape of wall
    // a real actor would get.
    let pool: Vec<Brick> = match mode {
        Mode::Wall => pool,
        Mode::Glaze => pool.into_iter().filter(Brick::is_image_post).collect(),
    };
    let items: Vec<_> = pool.iter().skip(offset).take(PAGE_SIZE).cloned().collect();
    let next_offset = offset.saturating_add(items.len());
    let cursor = (next_offset < pool.len()).then(|| {
        cursor::encode(&Cursor {
            snapshot: "fixture".into(),
            seed: 0,
            offset: next_offset,
        })
    });
    FeedResponse {
        items,
        cursor,
        warming: None,
    }
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

        let err = handle_feed(
            &state,
            "did:plc:owner",
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect_err("an opted-out owner must not lay a wall");
        assert!(matches!(err, AppError::LoginRequired(_)));
        assert_eq!(err.status_and_code(), (403, "login_required"));
    }

    /// The whole point of glaze: the same author feed the full wall reads, but
    /// only its image-bearing posts reach the page. A text-only post and a
    /// native-video post from the same author are left off.
    #[tokio::test]
    async fn glaze_lays_only_image_posts() {
        let server = MockServer::start().await;
        // the wall owner is not opted out
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:viewer",
                "handle": "viewer.test"
            })))
            .mount(&server)
            .await;
        // one follow, who becomes the whole cohort
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": "did:plc:friend", "handle": "friend.test"}]
            })))
            .mount(&server)
            .await;

        // three fresh posts from that friend: an image post, a text-only post,
        // and a native-video post. Only the first belongs on a glaze wall.
        let created = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
        let author = serde_json::json!({"did": "did:plc:friend", "handle": "friend.test"});
        let feed = serde_json::json!({ "feed": [
            {"post": {
                "uri": "at://did:plc:friend/app.bsky.feed.post/img",
                "author": author, "record": {"text": "a view", "createdAt": created},
                "embed": {"$type": "app.bsky.embed.images#view",
                    "images": [{"thumb": "https://cdn.test/a.jpg", "alt": "",
                        "aspectRatio": {"width": 4, "height": 3}}]},
                "likeCount": 3, "repostCount": 0
            }},
            {"post": {
                "uri": "at://did:plc:friend/app.bsky.feed.post/txt",
                "author": author, "record": {"text": "just words", "createdAt": created},
                "likeCount": 1, "repostCount": 0
            }},
            {"post": {
                "uri": "at://did:plc:friend/app.bsky.feed.post/vid",
                "author": author, "record": {"text": "watch", "createdAt": created},
                "embed": {"$type": "app.bsky.embed.video#view",
                    "playlist": "https://video.test/p.m3u8"},
                "likeCount": 9, "repostCount": 0
            }}
        ]});
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(feed))
            .mount(&server)
            .await;

        let state = Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            ..Default::default()
        }));

        let page = handle_feed(
            &state,
            "did:plc:viewer",
            None,
            Mode::Glaze,
            FeedIntent::Normal,
        )
        .await
        .expect("a glaze wall must lay");
        assert_eq!(page.items.len(), 1, "only the image post belongs on glaze");
        assert!(
            page.items[0].is_image_post(),
            "and the one brick laid is an image post"
        );
        assert_eq!(
            page.items[0].id(),
            "at://did:plc:friend/app.bsky.feed.post/img"
        );
    }
}

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
use crate::sources::feedref::{self, FeedRef};
use crate::sources::fetch;
use crate::state::AppState;

pub const PAGE_SIZE: usize = 24;

/// What the image wall asks a feed generator for in one call: `getFeed`'s own
/// ceiling, four times what the mixed views ask for.
///
/// Most posts in a general feed carry no image, so a `PAGE_SIZE` request
/// filtered down to its image posts would lay three or four bricks and spend a
/// dozen network calls filling one screen. There is no pool behind a feed wall
/// to accumulate them in, so depth has to come from the request itself.
const GLAZE_FEED_LIMIT: u32 = 100;

/// What the mixed views ask a feed generator for: exactly a page. Derived from
/// `PAGE_SIZE` rather than written out again, so the two cannot drift; the cast
/// is only that a page is a count of bricks and an AppView `limit` is a `u32`.
/// Coming back a few short is normal and fine, because reposts and moderated
/// posts are dropped after the request and the client's pump already retries.
const PAGE_SIZE_LIMIT: u32 = PAGE_SIZE as u32;

/// The `bad_request` payload for a request that names no wall at all. It names
/// both parameters because either one alone would have answered the question,
/// and supplying one of them is the reader's repair.
const NO_TARGET: &str = "actor or feed";

/// The `bad_request` payload for a `?feed=` that will not parse. It names the
/// parameter rather than quoting what arrived: the value is attacker-writable
/// and this message is displayed.
const UNPARSEABLE_FEED: &str = "feed";

/// Which wall a request names: somebody's follow graph, or a feed generator.
///
/// Borrowed rather than owned because both fronts already hold the strings the
/// query string was parsed into, and `handle_feed` never outlives them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedTarget<'a> {
    /// A handle, a DID, or the literal `demo`: whose graph to lay.
    Actor(&'a str),
    /// A feed generator reference, in any of the spellings
    /// [`crate::sources::feedref`] accepts.
    Feed(&'a str),
}

impl<'a> FeedTarget<'a> {
    /// Read the pair of query parameters that name a wall.
    ///
    /// Both halves of the rule live here rather than in each front, and that is
    /// not tidiness: `tests/contract.rs` is a mortar-core integration test and
    /// can reach neither front, and mortar-server has no test module at all, so
    /// a rule spelled out in the axum route is a rule nothing can test. Spelled
    /// once here, it is a unit test and a fixture assert away.
    ///
    /// The lifetime is explicit rather than `'_`: two input references cannot
    /// elide into one output lifetime.
    pub fn from_query(actor: Option<&'a str>, feed: Option<&'a str>) -> Result<Self, AppError> {
        // `feed` wins when both are present, because the two name different
        // walls and one of them has to. It wins rather than erroring so a
        // client that keeps an actor in the URL can still open a feed.
        if let Some(feed) = feed {
            return Ok(Self::Feed(feed));
        }
        if let Some(actor) = actor {
            return Ok(Self::Actor(actor));
        }
        Err(AppError::BadRequest(NO_TARGET))
    }

    /// The wire token for this arm: `"actor"` or `"feed"`, spelled exactly as
    /// the query parameters are. This is the query vocabulary the contract
    /// fixture pins, so a rename here is a wire change.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Actor(_) => "actor",
            Self::Feed(_) => "feed",
        }
    }
}

/// What a feed request is for.
///
/// The wasm front polls `Preview` while a wall warms (each poll lays a fresh,
/// non-committed first screen from the growing pool, which the client reflows),
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
    target: FeedTarget<'_>,
    cursor: Option<&str>,
    mode: Mode,
    intent: FeedIntent,
) -> Result<FeedResponse, AppError> {
    // A feed generator is an algorithm somebody else published, so its wall is
    // laid in FRONT of the snapshot machinery rather than inside it: none of
    // what follows (the owner gate, the pool, the cohort, the waves, grout and
    // the mixer) has anything to do for a feed.
    let actor = match target {
        FeedTarget::Feed(reference) => {
            return feed_wall(state, reference, cursor, mode, intent).await;
        }
        FeedTarget::Actor(actor) => actor,
    };

    let decoded = cursor.and_then(cursor::decode);

    // offline demo wall, kept from M0. Its bricks are fixtures compiled into the
    // wasm, so there is nothing to warm: a preview reports itself already
    // settled, and the client freezes to the real page at once.
    if actor == "demo" {
        // a feed cursor names a position in a generator's ordering, which the
        // compiled-in fixtures are no part of. It is attacker-writable and the
        // demo wall is the one wall reachable without an actor, so meeting one
        // lays from the top like no cursor at all rather than failing the wall.
        let offset = match decoded {
            Some(Cursor::Wall { offset, .. }) => offset,
            Some(Cursor::Feed { .. }) | None => 0,
        };
        let mut page = demo_page(offset, mode);
        if intent == FeedIntent::Preview {
            page.warming = Some(false);
            // a preview's cursor points at the CURRENT screen (not the next
            // page), so the freeze that follows commits from here. Demo warms
            // instantly, so the client freezes on the first poll either way.
            page.cursor = Some(cursor::encode(&Cursor::Wall { seed: 0, offset }));
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
        Some(Cursor::Wall { seed, offset }) => (seed, offset),
        // a feed cursor carries a stranger's ordering and names no position in
        // this snapshot, so the graph path treats it exactly as it treats
        // garbage: a fresh wall, never an error.
        Some(Cursor::Feed { .. }) | None => (snapshot::fresh_seed(&did), 0),
    };

    // A preview never commits and never waits: it lays the current best first
    // screen from a clone of the pool and reports whether more is still on the
    // way. The cursor it hands back carries the same seed, so the next poll (and
    // the freeze) land on this very snapshot rather than rolling a new one.
    if intent == FeedIntent::Preview {
        // the `false` is the refresh flag: no front parses `?refresh=` yet, so
        // every wall is still laid from the warm content caches
        let snap = snapshot::ensure_snapshot(state, &did, seed, mode, false).await;
        let (items, warming) = snapshot::preview_page(&snap, PAGE_SIZE).await;
        return Ok(FeedResponse {
            items,
            cursor: Some(cursor::encode(&Cursor::Wall { seed, offset: 0 })),
            warming: Some(warming),
        });
    }

    // the `false` is the refresh flag, as above
    let snap = snapshot::get_or_build(state, &did, seed, mode, false).await;
    // Freeze commits the first screen immediately: the preview loop already gave
    // the reader the warming reflow, so re-paying the mix wait here is the exact
    // stall reflow exists to remove. Normal (server mode, no preview loop) still
    // waits, so its first page is a proper mix and not just the fast posts.
    let wait_for_mix = intent == FeedIntent::Normal;
    let (items, has_more) = snapshot::get_page(state, &snap, offset, PAGE_SIZE, wait_for_mix).await;
    let next = has_more.then(|| {
        cursor::encode(&Cursor::Wall {
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

/// One page of a feed generator, laid in the generator's own order.
///
/// The whole of a feed wall: parse the reference, read one page, filter it for
/// the view, answer. There is no snapshot to build, no pool to admit into, no
/// cohort to sample and no mixer to run, because the feed already published an
/// order and mason's job here is to lay it rather than to re-rank it.
async fn feed_wall(
    state: &Arc<AppState>,
    reference: &str,
    cursor: Option<&str>,
    mode: Mode,
    intent: FeedIntent,
) -> Result<FeedResponse, AppError> {
    let uri = feed_uri(state, reference).await?;

    // a graph cursor names a position in a snapshot a feed wall has none of, so
    // it is treated exactly as garbage is: the feed from its head, never an
    // error.
    let upstream = match cursor.and_then(cursor::decode) {
        Some(Cursor::Feed { feed }) => Some(feed),
        Some(Cursor::Wall { .. }) | None => None,
    };

    // The depth follows the view, because glaze's filter is aggressive. Both
    // limits are part of the `feed_pages` cache key, so the two views of one
    // feed never serve each other's page.
    let limit = match mode {
        Mode::Wall => PAGE_SIZE_LIMIT,
        Mode::Glaze => GLAZE_FEED_LIMIT,
    };
    let page = fetch::feed_page_cached(state, &uri, upstream.as_deref(), limit).await?;

    let items: Vec<Brick> = match mode {
        // upstream was asked for exactly a page and the mapper only ever drops
        // (reposts, hidden authors), so this truncation is the guard against a
        // generator that ignores `limit`, not the normal path.
        Mode::Wall => page.yield_.bricks.iter().take(PAGE_SIZE).cloned().collect(),
        // EVERY survivor, not the first PAGE_SIZE of them. That is a
        // correctness requirement rather than an optimisation: there is no pool
        // to hold a remainder in, and the cursor mason hands back belongs to the
        // call that fetched these bricks, so a truncated page throws the rest
        // away instead of deferring it.
        Mode::Glaze => page
            .yield_
            .bricks
            .iter()
            .filter(|brick| brick.is_image_post())
            .cloned()
            .collect(),
    };

    if intent == FeedIntent::Preview {
        // One AppView call answers a page, so there is nothing to warm: the
        // preview reports itself settled and the client freezes on its first
        // poll. The cursor it hands back points at the CURRENT screen, so the
        // freeze commits this very page, and the sixty second `feed_pages`
        // entry makes that freeze a cache hit rather than a second round trip.
        //
        // It is re-encoded from the position actually read rather than echoed
        // from the request, so a graph cursor handed to a feed wall is dropped
        // here instead of being returned to the client as though it meant
        // something. For a feed cursor the two are byte identical.
        return Ok(FeedResponse {
            items,
            cursor: upstream.map(|feed| cursor::encode(&Cursor::Feed { feed })),
            warming: Some(false),
        });
    }

    Ok(FeedResponse {
        items,
        // `getFeed` returning no cursor is a feed wall's whole end condition:
        // no pool to drain and no graph to spend.
        cursor: page.next.map(|feed| cursor::encode(&Cursor::Feed { feed })),
        warming: None,
    })
}

/// Turn the `?feed=` parameter into the DID-authority AT-URI `getFeed` is asked
/// for.
async fn feed_uri(state: &Arc<AppState>, reference: &str) -> Result<String, AppError> {
    // A parse, not a fallback. `mode` and `intent` fall back because they are
    // optional decorations on a wall that exists either way; a malformed `feed`
    // names no wall at all, and quietly laying the reader's graph instead would
    // be a different product than the one they asked for.
    let parsed = feedref::parse(reference).ok_or(AppError::BadRequest(UNPARSEABLE_FEED))?;
    match parsed {
        FeedRef::Uri(uri) => Ok(uri),
        FeedRef::NeedsDid { profile, rkey } => {
            // A handle authority (an `at://` spelling or a bsky.app link) owes
            // one resolution hop, through the same `did` cache a wall uses and
            // WITHOUT the wall-owner gate: see `resolve_did`.
            let did = resolve_did(state, &profile).await.map_err(|e| match e {
                // the profile in a feed link is part of the reference, not a
                // wall the reader asked for, so an unknown one is a feed that
                // is not there. `actor_not_found` would hand somebody with a
                // bad feed link a handle box, which repairs nothing.
                AppError::ActorNotFound(_) => AppError::FeedNotFound(reference.to_string()),
                other => other,
            })?;
            Ok(feedref::uri_for(&did, &rkey))
        }
    }
}

/// Resolve an actor to a DID, with no gate on it.
///
/// The resolution half of `resolve_and_gate`, split out because a feed wall has
/// a profile segment to resolve and no owner to gate. `!no-unauthenticated` is
/// a request about a person's own social graph being put on display; a feed
/// generator is a published service, and its creator has not asked anybody not
/// to read it. The posts a feed yields are still filtered per author and per
/// post by the shared mapper, which is the complete coverage there, because a
/// feed cannot yield a blog or a stream for the cohort filter to catch.
///
/// One `getProfile` does what used to take a `resolveHandle` then a separate
/// `getProfile`: the response carries both the DID and the opt-out label, so a
/// cold handle pays one AppView round trip here and leaves the preference
/// cached for the gate above to read.
///
/// This call is load-bearing for resolution, so its failure fails closed
/// (`ActorNotFound` on a 400 or 404, `Upstream` otherwise) exactly as handle
/// resolution always did.
async fn resolve_did(state: &Arc<AppState>, actor: &str) -> Result<String, AppError> {
    // a `did:` actor is already resolved, and a handle read before is cached;
    // neither costs a round trip
    if actor.starts_with("did:") {
        return Ok(actor.to_string());
    }
    if let Some(did) = state.caches.did.get(&actor.to_string()).await {
        return Ok(did);
    }

    match fetch::get_profile(state, actor).await {
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
            Ok(profile.did)
        }
        Err(HttpError::Status(400 | 404)) => Err(AppError::ActorNotFound(actor.to_string())),
        Err(e) => Err(AppError::Upstream(e.to_string())),
    }
}

/// Resolve `actor` to a DID and, in the same breath, honour the owner's
/// logged-out opt-out. Returns the DID, or `LoginRequired` for a sealed wall.
///
/// A wall is somebody's social graph on display; if they asked to be seen only
/// by signed-in visitors, a logged-out mason must not lay it. Their own posts
/// never reach the fill, so this is the one place that preference is checked.
///
/// The fail direction depends on what is already known. Resolution fails closed
/// (see `resolve_did`), because an unresolvable handle has no wall either way.
/// The opt-out fails OPEN: a flaky getProfile is treated as "not opted out", so
/// it can never seal a wall by accident.
async fn resolve_and_gate(state: &Arc<AppState>, actor: &str) -> Result<String, AppError> {
    let did = resolve_did(state, actor).await?;

    // A cold handle's opt-out arrived with its resolution, so this is a cache
    // read rather than a second round trip; so is a wall laid twice in an hour.
    if let Some(opted_out) = state.caches.profiles.get(&did).await {
        return gate(actor, did, opted_out);
    }

    // Nothing cached: a `did:` actor resolves without reading a profile at all,
    // so the opt-out is still outstanding and worth one call of its own.
    match fetch::get_profile(state, &did).await {
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
        cursor::encode(&Cursor::Wall {
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

/// The selection rule, tested where it lives.
///
/// A plain `#[cfg(test)]` module rather than the target-gated one below: this
/// is pure string work with no wiremock and no tokio runtime behind it, so it
/// compiles and runs on wasm32 too, exactly like `sources::feedref`'s.
#[cfg(test)]
mod target_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// The two parameters name different walls, so when both arrive one of them
    /// has to win. `feed` does, which is what lets a client keep an actor in the
    /// URL and still open a feed over it.
    #[test]
    fn feed_wins_when_both_parameters_are_present() {
        let target = FeedTarget::from_query(Some("alice.bsky.social"), Some("at://did:plc:aa/x"))
            .expect("two parameters still name one wall");
        assert_eq!(target, FeedTarget::Feed("at://did:plc:aa/x"));
    }

    /// Either one alone names a wall, and nothing is inferred from the other's
    /// absence.
    #[test]
    fn one_parameter_alone_names_its_own_wall() {
        let actor = FeedTarget::from_query(Some("alice.bsky.social"), None)
            .expect("an actor alone names a wall");
        assert_eq!(actor, FeedTarget::Actor("alice.bsky.social"));

        let feed =
            FeedTarget::from_query(None, Some("at://did:plc:aa/x")).expect("so does a feed alone");
        assert_eq!(feed, FeedTarget::Feed("at://did:plc:aa/x"));
    }

    /// The negative space of the pair: a request naming no wall is a 400, and
    /// the message names both parameters, because either one would have
    /// answered and the reader cannot tell from "actor" alone that a feed would
    /// have done.
    #[test]
    fn neither_parameter_present_is_a_bad_request_naming_both() {
        let error = FeedTarget::from_query(None, None)
            .expect_err("a request naming no wall cannot lay one");
        assert_eq!(error.status_and_code(), (400, "bad_request"));
        let message = error.to_string();
        assert!(
            message.contains("actor") && message.contains("feed"),
            "the message must name both parameters, got {message:?}"
        );
    }

    /// The wire tokens the contract fixture pins, and the vocabulary the query
    /// string is spelled in. A rename here is a wire change.
    #[test]
    fn kind_is_the_query_parameter_it_came_from() {
        assert_eq!(FeedTarget::Actor("alice.bsky.social").kind(), "actor");
        assert_eq!(FeedTarget::Feed("at://did:plc:aa/x").kind(), "feed");
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A cursor is attacker-writable and a feed cursor is now a legal thing for
    /// one to hold, so the demo wall has to survive meeting one. It lays from
    /// the top, exactly as it does with no cursor, and the preview it hands back
    /// is a graph cursor so the freeze that follows resumes on the demo wall's
    /// own terms rather than echoing a stranger's ordering.
    #[tokio::test]
    async fn a_feed_cursor_on_the_demo_wall_lays_from_the_top() {
        let state = Arc::new(AppState::new(Config::default()));
        let feed_cursor = cursor::encode(&Cursor::Feed {
            feed: "3lqk2hj4xyz2t".to_string(),
        });

        let fresh = handle_feed(
            &state,
            FeedTarget::Actor("demo"),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("the demo wall always lays");
        let with_feed_cursor = handle_feed(
            &state,
            FeedTarget::Actor("demo"),
            Some(&feed_cursor),
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("a feed cursor must not fail the demo wall");
        assert!(!fresh.items.is_empty(), "the fixture pool is not empty");
        let laid: Vec<&str> = with_feed_cursor.items.iter().map(Brick::id).collect();
        let from_the_top: Vec<&str> = fresh.items.iter().map(Brick::id).collect();
        assert_eq!(laid, from_the_top, "a feed cursor means offset 0 here");

        let preview = handle_feed(
            &state,
            FeedTarget::Actor("demo"),
            Some(&feed_cursor),
            Mode::Wall,
            FeedIntent::Preview,
        )
        .await
        .expect("a preview must lay too");
        let echoed = cursor::decode(
            preview
                .cursor
                .as_deref()
                .expect("a demo preview always hands back its own screen"),
        )
        .expect("and that cursor decodes");
        assert_eq!(echoed, Cursor::Wall { seed: 0, offset: 0 });
    }

    /// The same wrong-shape case on the real path. A graph wall meeting a feed
    /// cursor has no position to resume from, so it lays a fresh wall the way a
    /// tampered cursor already does, rather than 500ing or panicking on a shape
    /// it cannot read.
    #[tokio::test]
    async fn a_feed_cursor_on_the_graph_wall_lays_a_fresh_wall() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:viewer",
                "handle": "viewer.test"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": "did:plc:friend", "handle": "friend.test"}]
            })))
            .mount(&server)
            .await;
        let created = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"feed": [{
                    "post": {
                        "uri": "at://did:plc:friend/app.bsky.feed.post/1",
                        "author": {"did": "did:plc:friend", "handle": "friend.test"},
                        "record": {"text": "hi", "createdAt": created},
                        "likeCount": 0, "repostCount": 0
                    }
                }]})),
            )
            .mount(&server)
            .await;

        let state = Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            ..Default::default()
        }));

        let feed_cursor = cursor::encode(&Cursor::Feed {
            feed: "3lqk2hj4xyz2t".to_string(),
        });
        let page = handle_feed(
            &state,
            FeedTarget::Actor("did:plc:viewer"),
            Some(&feed_cursor),
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("a feed cursor must not fail a graph wall");
        assert_eq!(
            page.items.iter().map(Brick::id).collect::<Vec<_>>(),
            vec!["at://did:plc:friend/app.bsky.feed.post/1"],
            "the wall is laid from the top, not from a borrowed offset"
        );
        // a one-author graph is spent the moment its single post is laid, so
        // the wall honestly ends rather than echoing back the feed cursor it
        // was handed and inviting a page that can never come.
        assert!(page.cursor.is_none(), "a spent graph ends the wall");
    }

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
            FeedTarget::Actor("did:plc:owner"),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect_err("an opted-out owner must not lay a wall");
        assert!(matches!(err, AppError::LoginRequired(_)));
        assert_eq!(err.status_and_code(), (403, "login_required"));
    }

    /// The wall extends itself: a follow graph bigger than one cohort keeps
    /// yielding past the initial fill. 101 follows means a 100-author cohort
    /// with one author left over; only an extension wave can fetch that last
    /// author, so an endless scroll that lays all 101 posts proves the wave
    /// ran, and the final page reporting no cursor proves a spent graph still
    /// ends the wall honestly.
    #[tokio::test]
    async fn the_scroll_extends_past_the_first_cohort() {
        use wiremock::{Request, Respond};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:viewer",
                "handle": "viewer.test"
            })))
            .mount(&server)
            .await;

        let follows: Vec<serde_json::Value> = (0..101)
            .map(|n| {
                serde_json::json!({"did": format!("did:plc:f{n}"), "handle": format!("f{n}.test")})
            })
            .collect();
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "follows": follows })),
            )
            .mount(&server)
            .await;

        // every author answers with one fresh post of their own, so each
        // author on the wall is one fan-out that actually happened
        struct OnePostEach;
        impl Respond for OnePostEach {
            fn respond(&self, request: &Request) -> ResponseTemplate {
                let actor = request
                    .url
                    .query_pairs()
                    .find(|(k, _)| k == "actor")
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_default();
                let created = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"feed": [{
                    "post": {
                        "uri": format!("at://{actor}/app.bsky.feed.post/1"),
                        "author": {"did": actor, "handle": "a.test"},
                        "record": {"text": "hi", "createdAt": created},
                        "likeCount": 1, "repostCount": 0
                    }
                }]}))
            }
        }
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(OnePostEach)
            .mount(&server)
            .await;

        let state = Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            plc_base: server.uri(),
            streamplace_base: server.uri(),
        }));

        let mut cursor: Option<String> = None;
        let mut laid: Vec<String> = Vec::new();
        let mut ended = false;
        for _ in 0..30 {
            let page = handle_feed(
                &state,
                FeedTarget::Actor("did:plc:viewer"),
                cursor.as_deref(),
                Mode::Wall,
                FeedIntent::Normal,
            )
            .await
            .expect("every page must lay");
            laid.extend(page.items.iter().map(|b| b.id().to_string()));
            match page.cursor {
                Some(next) => cursor = Some(next),
                None => {
                    ended = true;
                    break;
                }
            }
        }

        let distinct: std::collections::HashSet<&str> = laid.iter().map(String::as_str).collect();
        assert_eq!(
            distinct.len(),
            101,
            "all 101 authors' posts laid: the wave fetched the one the cohort missed"
        );
        assert_eq!(laid.len(), 101, "and none of them twice");
        assert!(
            ended,
            "a spent graph must end the wall with no cursor, not spin forever"
        );
    }

    /// A transient author-feed failure must not silence that author for the
    /// wall's whole life. The initial fill's fetch dies (three 5xx, enough to
    /// exhaust the retry loop), so the author is never recorded as fanned and
    /// nothing is cached; the first page's extension wave asks them again,
    /// succeeds, and lays their post. The old behavior lost them forever.
    #[tokio::test]
    async fn a_transiently_failed_author_is_asked_again_by_the_next_wave() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:viewer",
                "handle": "viewer.test"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": "did:plc:flaky", "handle": "flaky.test"}]
            })))
            .mount(&server)
            .await;

        // the fill's one author-feed read: three 500s exhaust the internal
        // retry loop, so the fetch surfaces as a transient failure...
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(ResponseTemplate::new(500).insert_header("retry-after", "0"))
            .up_to_n_times(3)
            .mount(&server)
            .await;
        // ...and the wave's retry finds the author alive with a fresh post
        let created = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"feed": [{
                    "post": {
                        "uri": "at://did:plc:flaky/app.bsky.feed.post/1",
                        "author": {"did": "did:plc:flaky", "handle": "flaky.test"},
                        "record": {"text": "back online", "createdAt": created},
                        "likeCount": 0, "repostCount": 0
                    }
                }]})),
            )
            .mount(&server)
            .await;

        let state = Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            plc_base: server.uri(),
            streamplace_base: server.uri(),
        }));

        let first = handle_feed(
            &state,
            FeedTarget::Actor("did:plc:viewer"),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("the first page must lay");
        assert_eq!(
            first.items.len(),
            1,
            "the wave must recover the transiently failed author's post"
        );
        assert_eq!(
            first.items[0].id(),
            "at://did:plc:flaky/app.bsky.feed.post/1"
        );

        let cursor = first
            .cursor
            .expect("one recovered author is not a spent graph yet");
        let last = handle_feed(
            &state,
            FeedTarget::Actor("did:plc:viewer"),
            Some(&cursor),
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("the last page must answer");
        assert!(last.items.is_empty());
        assert!(
            last.cursor.is_none(),
            "with the author recovered and fanned, the graph is spent and the wall ends"
        );
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
            FeedTarget::Actor("did:plc:viewer"),
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

// The feed wall, driven against a wiremock AppView. A SECOND gated module
// rather than cases in the one above, because everything here reads `getFeed`
// and nothing here builds a snapshot: keeping the two apart is what makes the
// mount list of each test readable as the claim it is making.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod feed_wall_tests {
    use super::*;
    use crate::config::Config;
    use crate::state::AppState;
    use serde_json::{Value, json};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A feed generator's AT-URI in the DID form, which owes no resolution hop.
    /// Most of these tests use it precisely so that mounting `getProfile` would
    /// be mounting something a feed wall must never need.
    const FEED: &str = "at://did:plc:gen/app.bsky.feed.generator/whats-hot";

    /// A fixed timestamp. Nothing on a feed wall is scored, so no assertion here
    /// depends on how old a post is, and a fixed one keeps the fixtures byte
    /// identical run to run.
    const CREATED: &str = "2026-07-10T12:00:00Z";

    fn state_for(server: &MockServer) -> Arc<AppState> {
        // every base points at the mock, so a read this wall should never make
        // fails against a mount that is not there rather than leaving the test
        // machine's network to decide
        Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            plc_base: server.uri(),
            streamplace_base: server.uri(),
        }))
    }

    fn post_uri(rkey: &str) -> String {
        format!("at://did:plc:author/app.bsky.feed.post/{rkey}")
    }

    /// One post as `getFeed` hydrates it: `created` and `likes` vary so a test
    /// can hand the wall an order grout would invert.
    fn post(rkey: &str, created: &str, likes: u64) -> Value {
        json!({"post": {
            "uri": post_uri(rkey),
            "author": {"did": "did:plc:author", "handle": "author.test"},
            "record": {"text": "hello wall", "createdAt": created},
            "likeCount": likes,
            "repostCount": 0
        }})
    }

    /// The same post carrying one image, which is what a glaze wall keeps.
    fn image_post(rkey: &str) -> Value {
        let mut item = post(rkey, CREATED, 0);
        item["post"]["embed"] = json!({
            "$type": "app.bsky.embed.images#view",
            "images": [{"thumb": "https://cdn.test/a.jpg", "alt": "",
                "aspectRatio": {"width": 4, "height": 3}}]
        });
        item
    }

    /// A feed item the mapper must drop, in each of the two ways it can:
    /// somebody else's post the feed surfaced by reposting it, and a post whose
    /// author opted out of being read logged out.
    fn repost(rkey: &str) -> Value {
        let mut item = post(rkey, CREATED, 999);
        item["reason"] = json!({"$type": "app.bsky.feed.defs#reasonRepost"});
        item
    }

    fn post_by_an_opted_out_author(rkey: &str) -> Value {
        let mut item = post(rkey, CREATED, 999);
        item["post"]["author"]["labels"] = json!([{"val": "!no-unauthenticated"}]);
        item
    }

    /// Mount one `getFeed` answer. `limit` and `at` are matched rather than
    /// ignored, so a wall that asked for the wrong depth or paged from the wrong
    /// place finds no mock at all and fails loudly instead of quietly reading
    /// this page.
    async fn feed_answers(
        server: &MockServer,
        limit: u32,
        at: Option<&str>,
        items: Vec<Value>,
        next: Option<&str>,
    ) {
        let mut body = json!({ "feed": items });
        if let Some(next) = next {
            body["cursor"] = json!(next);
        }
        let mut mock = Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getFeed"))
            .and(query_param("limit", limit.to_string()));
        if let Some(at) = at {
            mock = mock.and(query_param("cursor", at));
        }
        mock.respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(server)
            .await;
    }

    fn laid(page: &FeedResponse) -> Vec<&str> {
        page.items.iter().map(Brick::id).collect()
    }

    async fn upstream_paths(server: &MockServer) -> Vec<String> {
        server
            .received_requests()
            .await
            .expect("the mock server records what it was asked")
            .iter()
            .map(|request| request.url.path().to_string())
            .collect()
    }

    /// The whole claim of a feed wall: the generator published an order and
    /// mason lays it in that order. The three surviving posts arrive
    /// oldest-first and least-liked-first, which is precisely the order grout
    /// would invert, so laying them as they came proves no scoring happened.
    ///
    /// The same page also proves the moderation a feed wall inherits for free by
    /// sharing the author feeds' mapper: a repost is dropped (it is not the
    /// reposter's brick) and so is a post whose author opted out of being read
    /// logged out.
    #[tokio::test]
    async fn a_feed_wall_lays_the_generators_own_order() {
        let server = MockServer::start().await;
        feed_answers(
            &server,
            PAGE_SIZE_LIMIT,
            None,
            vec![
                post("first", "2026-07-01T00:00:00Z", 0),
                repost("reposted"),
                post_by_an_opted_out_author("sealed"),
                post("second", "2026-07-05T00:00:00Z", 10),
                post("third", "2026-07-20T00:00:00Z", 900),
            ],
            Some("page2"),
        )
        .await;
        let state = state_for(&server);

        let page = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("a feed wall must lay");

        assert_eq!(
            laid(&page),
            vec![post_uri("first"), post_uri("second"), post_uri("third")],
            "the feed's own order, untouched: grout would have inverted this one"
        );

        // the upstream cursor is the whole of mason's position in the feed's
        // order, and it travels back to the client inside a cursor of its own
        let next = cursor::decode(page.cursor.as_deref().expect("the feed has more"))
            .expect("and it decodes");
        assert_eq!(
            next,
            Cursor::Feed {
                feed: "page2".to_string()
            }
        );
        assert!(
            page.warming.is_none(),
            "a committed page never reports warming"
        );

        // The structural claim, in one assertion: a feed wall reads getFeed and
        // nothing else. No getFollows, no getAuthorFeed, no getProfile, which
        // is what "no snapshot, no cohort, no fill" looks like from outside.
        assert_eq!(
            upstream_paths(&server).await,
            vec!["/xrpc/app.bsky.feed.getFeed"],
            "one page of a feed wall is one AppView call"
        );
    }

    /// `getFeed` returning no cursor is a feed wall's whole end condition: there
    /// is no pool to drain and no graph to spend, so the wall ends exactly when
    /// the feed does.
    #[tokio::test]
    async fn a_feed_wall_ends_when_the_generator_does() {
        let server = MockServer::start().await;
        // the cursor-bearing mock goes up first: the general one would match a
        // request carrying a cursor too, and wiremock takes the first match
        feed_answers(
            &server,
            PAGE_SIZE_LIMIT,
            Some("page2"),
            vec![post("last", CREATED, 0)],
            None,
        )
        .await;
        feed_answers(
            &server,
            PAGE_SIZE_LIMIT,
            None,
            vec![post("first", CREATED, 0)],
            Some("page2"),
        )
        .await;
        let state = state_for(&server);

        let first = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("the first page lays");
        let carried = first.cursor.expect("a feed with more hands back a cursor");

        let last = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            Some(&carried),
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("and the page after it lays too");
        assert_eq!(
            laid(&last),
            vec![post_uri("last")],
            "paging must reach the next page, not re-serve the first"
        );
        assert!(
            last.cursor.is_none(),
            "a feed that ended must end the wall rather than invite a page that can never come"
        );
    }

    /// There is nothing to warm on a feed wall: one AppView call answers a page,
    /// so a preview reports itself already settled and the client freezes on its
    /// first poll. The cursor it hands back points at the CURRENT screen, so the
    /// freeze commits this very page, and the sixty second `feed_pages` entry
    /// makes that freeze a cache hit rather than a second round trip.
    #[tokio::test]
    async fn a_feed_wall_preview_is_already_settled_and_echoes_its_cursor() {
        let server = MockServer::start().await;
        feed_answers(
            &server,
            PAGE_SIZE_LIMIT,
            Some("page2"),
            vec![post("mid", CREATED, 0)],
            Some("page3"),
        )
        .await;
        let state = state_for(&server);

        let incoming = cursor::encode(&Cursor::Feed {
            feed: "page2".to_string(),
        });
        let preview = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            Some(&incoming),
            Mode::Wall,
            FeedIntent::Preview,
        )
        .await
        .expect("a preview must lay");

        assert_eq!(
            preview.warming,
            Some(false),
            "a feed wall is settled the moment it answers"
        );
        assert_eq!(
            preview.cursor.as_deref(),
            Some(incoming.as_str()),
            "the preview's cursor is the screen it just laid, not the one after it"
        );
        assert_eq!(laid(&preview), vec![post_uri("mid")]);
    }

    /// A graph cursor names a position in a snapshot a feed wall has none of, so
    /// it is treated exactly as garbage is: the feed from its head, never an
    /// error, and never a page fetched under somebody else's cursor. The mock
    /// matches on the ABSENCE of a cursor parameter, so a wall that forwarded
    /// this one would find no mock at all.
    #[tokio::test]
    async fn a_graph_cursor_on_a_feed_wall_lays_from_the_head() {
        let server = MockServer::start().await;
        feed_answers(
            &server,
            PAGE_SIZE_LIMIT,
            None,
            vec![post("head", CREATED, 0)],
            None,
        )
        .await;
        let state = state_for(&server);

        let graph_cursor = cursor::encode(&Cursor::Wall {
            seed: 42,
            offset: 96,
        });
        let page = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            Some(&graph_cursor),
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("a graph cursor must not fail a feed wall");
        assert_eq!(laid(&page), vec![post_uri("head")]);

        // and a preview handed one drops it rather than echoing a position that
        // means nothing here back to the client
        let preview = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            Some(&graph_cursor),
            Mode::Wall,
            FeedIntent::Preview,
        )
        .await
        .expect("a preview must lay too");
        assert!(preview.cursor.is_none(), "there is no position to echo");
    }

    /// Glaze over a feed asks for `getFeed`'s ceiling and lays EVERY image post
    /// that survives, not the first `PAGE_SIZE` of them. That is a correctness
    /// requirement rather than an optimisation: the cursor mason hands back
    /// belongs to the call that fetched these bricks, and there is no pool to
    /// hold a remainder in, so a truncated page would throw the rest away.
    ///
    /// The mock matches `limit=100`, so a wall that asked for a page's worth
    /// finds no mock and fails rather than quietly running a quarter as deep.
    #[tokio::test]
    async fn glaze_over_a_feed_lays_every_image_post_rather_than_a_page_of_them() {
        let server = MockServer::start().await;
        // thirty image posts, each followed by a text post: more images than a
        // page holds, and a filter to prove.
        let mut items = Vec::new();
        for n in 0..30 {
            items.push(image_post(&format!("image-{n}")));
            items.push(post(&format!("text-{n}"), CREATED, 0));
        }
        feed_answers(&server, GLAZE_FEED_LIMIT, None, items, None).await;
        let state = state_for(&server);

        let page = handle_feed(
            &state,
            FeedTarget::Feed(FEED),
            None,
            Mode::Glaze,
            FeedIntent::Normal,
        )
        .await
        .expect("a glaze feed wall must lay");

        assert_eq!(page.items.len(), 30, "every image post the page carried");
        assert!(
            page.items.len() > PAGE_SIZE,
            "and more than a page of them, which is the case truncation would eat"
        );
        assert!(
            page.items.iter().all(Brick::is_image_post),
            "a glaze wall is image posts and nothing else"
        );
        assert_eq!(
            laid(&page).first().copied(),
            Some(post_uri("image-0").as_str()),
            "the feed's order survives the filter"
        );
    }

    /// A feed wall has no owner to gate. `!no-unauthenticated` is a request
    /// about a person's own social graph being put on display; a feed generator
    /// is a published service, and its creator has not asked anybody not to read
    /// it. So the creator here is sealed, and the feed still lays.
    ///
    /// This is also the resolution hop: a bsky.app link's profile segment is
    /// resolved to a DID and the AT-URI rebuilt from it, which the `getFeed`
    /// mock matches on.
    #[tokio::test]
    async fn a_sealed_creator_does_not_seal_their_feed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "did": "did:plc:sealed",
                "handle": "sealed.test",
                "labels": [{"val": "!no-unauthenticated"}]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getFeed"))
            .and(query_param(
                "feed",
                "at://did:plc:sealed/app.bsky.feed.generator/whats-hot",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "feed": [post("published", CREATED, 0)]
            })))
            .mount(&server)
            .await;
        let state = state_for(&server);

        let page = handle_feed(
            &state,
            FeedTarget::Feed("https://bsky.app/profile/sealed.test/feed/whats-hot"),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect("a published feed lays whatever its creator set on their own profile");
        assert_eq!(laid(&page), vec![post_uri("published")]);
    }

    /// The negative space of that resolution hop. A feed link naming a profile
    /// the AppView does not know is a feed that is not there, not an actor that
    /// is not there: `actor_not_found` would hand somebody holding a bad feed
    /// link a handle box, which repairs nothing they got wrong.
    #[tokio::test]
    async fn a_feed_link_naming_nobody_is_a_feed_that_is_not_there() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;
        let state = state_for(&server);

        let reference = "https://bsky.app/profile/nobody.test/feed/whats-hot";
        let failure = handle_feed(
            &state,
            FeedTarget::Feed(reference),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect_err("an unresolvable creator is a failure, not a wall");
        match failure {
            AppError::FeedNotFound(named) => assert_eq!(
                named, reference,
                "the error names the reference the reader actually holds"
            ),
            other => panic!("a feed link naming nobody must be a missing feed, got {other:?}"),
        }
    }

    /// The one place a fallback would be tempting, and the reason there is none:
    /// `mode` and `intent` fall back because they decorate a wall that exists
    /// either way, and a malformed `feed` names no wall at all. Laying the actor
    /// beside it instead would be a different product than the one the reader
    /// asked for, so this is a 400 even with a perfectly good actor in hand.
    ///
    /// Nothing is mounted: every one of these is rejected before a socket opens,
    /// which is what the request count at the end asserts.
    #[tokio::test]
    async fn an_unparseable_feed_reference_is_a_bad_request_and_asks_nobody() {
        let server = MockServer::start().await;
        let state = state_for(&server);

        for reference in [
            "nonsense",
            "",
            "javascript:alert(1)",
            // a legal AT-URI, but naming a collection getFeed cannot page
            "at://did:plc:aa/app.bsky.feed.post/3k2a",
            // a lookalike host, which must never reach a network call either
            "https://evilbsky.app/profile/alice.test/feed/whats-hot",
        ] {
            let failure = handle_feed(
                &state,
                FeedTarget::Feed(reference),
                None,
                Mode::Wall,
                FeedIntent::Normal,
            )
            .await
            .expect_err("a reference that names no feed cannot lay one");
            assert_eq!(
                failure.status_and_code(),
                (400, "bad_request"),
                "{reference:?} must be a bad request"
            );
        }

        // and the precedence rule composed with it: `feed` wins, so a request
        // carrying a good actor beside a bad feed is a 400 rather than that
        // actor's wall
        let both = FeedTarget::from_query(Some("did:plc:viewer"), Some("nonsense"))
            .expect("two parameters still name one wall");
        let failure = handle_feed(&state, both, None, Mode::Wall, FeedIntent::Normal)
            .await
            .expect_err("mason must not quietly lay somebody's graph instead");
        assert_eq!(failure.status_and_code(), (400, "bad_request"));

        assert!(
            upstream_paths(&server).await.is_empty(),
            "a malformed reference is rejected before any network call"
        );
    }

    /// `AppError::BadRequest` has two callers and one Display, so the Display
    /// has to be true of both. It used to read "missing required parameter:
    /// {0}", which was honest for the first caller and a lie about the second:
    /// an unparseable `?feed=` was present, and telling the reader it was
    /// missing points the repair at the wrong end of the request.
    ///
    /// Both messages are raised through their real call sites rather than
    /// constructed here, so the assert covers the payload each caller actually
    /// passes as well as the shared Display. The `missing` check is the honesty
    /// claim written down: any reword that reintroduces the old adjective fails
    /// here rather than shipping.
    #[tokio::test]
    async fn a_bad_request_reads_honestly_for_both_of_its_callers() {
        // caller one: a request naming no wall at all. The parameters really are
        // absent, and the message names both because either would have answered.
        let neither = FeedTarget::from_query(None, None)
            .expect_err("a request naming no wall cannot lay one");
        assert_eq!(neither.to_string(), "bad request: actor or feed");

        // caller two: a `?feed=` that was present and would not parse. Nothing
        // is missing, so nothing may say so. Nothing is mounted either, which is
        // the module's idiom: a read this path must never make fails against an
        // absent mount rather than reaching the real AppView.
        let server = MockServer::start().await;
        let state = state_for(&server);
        let malformed = handle_feed(
            &state,
            FeedTarget::Feed("nonsense"),
            None,
            Mode::Wall,
            FeedIntent::Normal,
        )
        .await
        .expect_err("a reference that names no feed cannot lay one");
        assert_eq!(malformed.to_string(), "bad request: feed");

        for message in [neither.to_string(), malformed.to_string()] {
            assert!(
                !message.contains("missing"),
                "one Display serves both callers, and only one of them is about \
                 absence, got {message:?}"
            );
        }
    }
}

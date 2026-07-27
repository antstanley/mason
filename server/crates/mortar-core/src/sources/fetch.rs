//! The fetch-and-cache seam between the sources and the feed engine. Every
//! network read the fill performs goes through here and comes back as bricks
//! (plus the Follow list); algo/ never talks to a source module directly, so
//! swapping an ingestion backend (the v2 Jetstream + SQLite plan) touches this
//! directory and nothing else.
//!
//! Each function is the same shape: consult the matching `Caches` field, fetch
//! on a miss, insert with the TTL the source's failure semantics call for.
//! Failures degrade to empty yields rather than errors; a single author (or
//! source) failing must never sink the wall.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, StreamExt};

use super::{FeedPage, bluesky, pds, standardsite, streamplace};
use crate::cache::TtlCache;
use crate::error::AppError;
use crate::http::HttpError;
use crate::model::{Author, Brick};
use crate::platform;
use crate::state::AppState;

/// Repo-read fan-out. Higher than the author-feed fan-out, because these go to
/// a hundred different PDSes rather than to one rate-limited AppView, and the
/// slowest of them must not hold up the rest.
pub const REPO_FAN_OUT: usize = 32;
/// Pages of the follow graph (100 each) the first wall will wait for. One page
/// is one round trip and 100 follows, already more than the 100-author cohort
/// samples, so the first wall waits for exactly that and no more; each further
/// page is a sequential round trip that fetches nothing while it blocks. The
/// rest of the graph is chased in the background (`FOLLOW_PAGES_MAX`) and the
/// NEXT wall samples the whole of it.
const FOLLOW_PAGES_EAGER: usize = 1;
/// The cap on the whole graph, chased in the background. The cohort sampler
/// has never needed more than this.
const FOLLOW_PAGES_MAX: usize = 20;

// Source-specific TTLs, owned by the seam that picks between them; the core
// cache reuses the negatives as its defaults.
pub const STD_DOCS_POSITIVE_TTL: Duration = Duration::from_secs(900);
pub const STD_DOCS_NEGATIVE_TTL: Duration = Duration::from_secs(24 * 3600);

pub const STREAMS_POSITIVE_TTL: Duration = Duration::from_secs(1800);
pub const STREAMS_NEGATIVE_TTL: Duration = Duration::from_secs(24 * 3600);

/// Fetch a profile view for `actor`, which may be a handle or a DID. A thin
/// pass-through: the caller (the feed gate) owns the caching, because what it
/// caches (the DID, the opt-out) and how it fails differ by what it already
/// knows.
pub async fn get_profile(
    state: &Arc<AppState>,
    actor: &str,
) -> Result<bluesky::Profile, HttpError> {
    bluesky::get_profile(&state.http, &state.config.appview_base, actor).await
}

/// The follow graph, but only as much of it as a waiting person can justify.
///
/// Follows page 100 at a time and each page is a round trip that blocks the
/// next, so a 2000-follow graph costs twenty sequential requests: ten seconds
/// in which not one post has been fetched, and a wall that arrives empty. The
/// cohort samples 100 authors regardless, so a few hundred follows is plenty
/// to build the first wall out of. The rest is fetched behind the user's back
/// and cached, so their NEXT wall samples the whole graph.
pub async fn get_follows_cached(
    state: &Arc<AppState>,
    did: &str,
) -> Result<Arc<Vec<bluesky::Follow>>, AppError> {
    if let Some(follows) = state.caches.follows.get(&did.to_string()).await {
        return Ok(follows);
    }
    let (head, cursor) = bluesky::get_follows(
        &state.http,
        &state.config.appview_base,
        did,
        None,
        FOLLOW_PAGES_EAGER,
    )
    .await
    .map_err(|e| AppError::Upstream(e.to_string()))?;
    let head = Arc::new(head);

    let Some(cursor) = cursor else {
        // the whole graph fitted in the head start; nothing left to chase
        state
            .caches
            .follows
            .insert(did.to_string(), Arc::clone(&head))
            .await;
        return Ok(head);
    };

    // Deliberately NOT cached: a partial graph must never masquerade as the
    // whole one. The task below replaces it with the real thing.
    let rest_state = Arc::clone(state);
    let rest_did = did.to_string();
    let rest_head = Arc::clone(&head);
    platform::spawn(async move {
        let remaining = FOLLOW_PAGES_MAX.saturating_sub(FOLLOW_PAGES_EAGER);
        match bluesky::get_follows(
            &rest_state.http,
            &rest_state.config.appview_base,
            &rest_did,
            Some(cursor),
            remaining,
        )
        .await
        {
            Ok((tail, _)) => {
                let mut whole = rest_head.as_ref().clone();
                whole.extend(tail);
                tracing::debug!("follow graph for {rest_did} completed: {}", whole.len());
                rest_state
                    .caches
                    .follows
                    .insert(rest_did, Arc::new(whole))
                    .await;
            }
            Err(e) => tracing::debug!("completing follow graph for {rest_did} failed: {e}"),
        }
    });

    Ok(head)
}

/// Whether an author-feed failure is worth asking about again. Network
/// trouble and server-side errors (the retry loop's leftovers) are transient;
/// a plain 4xx is the AppView's honest answer (a suspended or deleted repo
/// yields nothing) and deserves the cache.
fn transient(error: &HttpError) -> bool {
    match error {
        HttpError::Transport(_) | HttpError::RetriesExhausted => true,
        HttpError::Status(status) => *status == 429 || *status >= 500,
    }
}

/// What a transiently failed feed read hands back, shared by both readers.
///
/// A refresh must never make the wall worse than the one it replaced, so a
/// refreshed read that fails returns the entry it stepped over: the author did
/// answer, just earlier, and they still count as fanned out. Every ordinary
/// read, and a refreshed read with nothing cached behind it, yields `None`
/// instead, which is the cold-read contract the caller already handles.
///
/// The lookup lives here, inside the failure arm, rather than beside the
/// `refresh` check at the top of each reader: a read succeeds far more often
/// than it fails, and the ordinary path must keep paying exactly one cache
/// lookup rather than two.
async fn refresh_fallback(
    cache: &TtlCache<String, Arc<bluesky::AuthorYield>>,
    author_did: &str,
    refresh: bool,
) -> Option<Arc<bluesky::AuthorYield>> {
    if !refresh {
        return None;
    }
    // an entry that expired while the failing fetch was in flight is gone by
    // now, and that is right: a stale yield is no better than a cold read
    cache.get(&author_did.to_string()).await
}

/// One author's recent posts. `None` is a transient failure with nothing
/// cached: the author never answered, so the caller must not count them as
/// fanned out; a later wave (or the next wall) simply asks again. A refusal
/// caches as an empty yield, exactly like a genuinely quiet author.
///
/// `refresh` is the reader asking for this wall on purpose: the cache is not
/// consulted and the AppView's answer overwrites whatever was there. A
/// refreshed read that fails transiently falls back to the cached yield, so a
/// refresh can never lay a thinner wall than the one it replaced; with nothing
/// cached behind it, a refreshed read behaves exactly like a cold one.
pub async fn author_feed_cached(
    state: &Arc<AppState>,
    author_did: &str,
    refresh: bool,
) -> Option<Arc<bluesky::AuthorYield>> {
    // a refresh steps over the entry rather than serving it: five minutes of
    // cached posts is precisely what the reader asked to get past
    if !refresh && let Some(cached) = state.caches.author_feed.get(&author_did.to_string()).await {
        return Some(cached);
    }
    let yield_ =
        match bluesky::get_author_feed(&state.http, &state.config.appview_base, author_did).await {
            Ok(yield_) => Arc::new(yield_),
            // a single author failing must never sink the wall, but a blip
            // must not be remembered as "this author posts nothing" either
            Err(e) if transient(&e) => {
                tracing::debug!("author feed {author_did} failed: {e}");
                return refresh_fallback(&state.caches.author_feed, author_did, refresh).await;
            }
            Err(e) => {
                tracing::debug!("author feed {author_did} refused: {e}");
                Arc::new(bluesky::AuthorYield { bricks: Vec::new() })
            }
        };
    state
        .caches
        .author_feed
        .insert(author_did.to_string(), Arc::clone(&yield_))
        .await;
    Some(yield_)
}

/// One author's recent MEDIA posts, read deep for the glaze wall. Same shape as
/// `author_feed_cached` (including `None` for a transient failure, and the same
/// `refresh` bypass with the same cached fallback) but a separate endpoint
/// (`posts_with_media`) and a separate cache, so the image wall's deeper read
/// never displaces the full wall's shallow one for the same author.
pub async fn image_feed_cached(
    state: &Arc<AppState>,
    author_did: &str,
    refresh: bool,
) -> Option<Arc<bluesky::AuthorYield>> {
    if !refresh && let Some(cached) = state.caches.image_feed.get(&author_did.to_string()).await {
        return Some(cached);
    }
    let yield_ =
        match bluesky::get_image_feed(&state.http, &state.config.appview_base, author_did).await {
            Ok(yield_) => Arc::new(yield_),
            Err(e) if transient(&e) => {
                tracing::debug!("image feed {author_did} failed: {e}");
                return refresh_fallback(&state.caches.image_feed, author_did, refresh).await;
            }
            Err(e) => {
                tracing::debug!("image feed {author_did} refused: {e}");
                Arc::new(bluesky::AuthorYield { bricks: Vec::new() })
            }
        };
    state
        .caches
        .image_feed
        .insert(author_did.to_string(), Arc::clone(&yield_))
        .await;
    Some(yield_)
}

/// One page of a feed generator, cached for a minute.
///
/// The one CONTENT read on this seam that fails loudly. Every other one is a
/// single author out of a hundred, so it degrades to an empty yield and the wall
/// loses a few bricks; a feed wall's whole ingestion is this single call, so
/// there is no quorum to degrade into and a failure here is the request failing.
/// A 400 or a 404 is the AppView saying it has no such feed, which is the
/// reader's reference to fix rather than an outage, so it is its own code.
///
/// The sixty second entry is what makes the preview-then-freeze pair one network
/// read: the freeze arriving a few hundred milliseconds after the preview asks
/// for the same page, as does a back gesture onto a page just left.
///
/// `refresh` is the reader asking for this wall on purpose, and on a feed wall
/// this entry is the whole of what "new posts" means: there is no author-feed
/// cache behind a generator's ordering, so the two fast content reads a graph
/// wall bypasses have no counterpart here. The entry is skipped and the fresh
/// answer overwrites it, so the freeze behind a refreshed preview is still one
/// network read. Unlike the author feeds this one has no cached fallback: a feed
/// wall's whole ingestion is this single call, so there is no quorum to degrade
/// into and a failure is the request failing, refreshed or not.
pub async fn feed_page_cached(
    state: &Arc<AppState>,
    feed_uri: &str,
    cursor: Option<&str>,
    limit: u32,
    refresh: bool,
) -> Result<FeedPage, AppError> {
    // The LIMIT is part of the key, not just the feed and the cursor: the mixed
    // views ask getFeed for PAGE_SIZE and the glaze wall asks for 100, so a key
    // of (uri, cursor) alone would serve a glaze request the 24-item page a
    // mixed request cached a moment earlier and the image wall would silently
    // run a quarter as deep. The unit separator is what keeps the three parts
    // apart: the cursor is last and the limit is digits, so only a \u{1f} inside
    // the feed uri itself could blur two keys into one, and a feed reference is
    // parsed before it ever reaches here.
    let key = format!(
        "{feed_uri}\u{1f}{limit}\u{1f}{}",
        cursor.unwrap_or_default()
    );
    // a refresh steps over the entry rather than serving it: a minute of cached
    // page is precisely what the reader asked to get past
    if !refresh && let Some(cached) = state.caches.feed_pages.get(&key).await {
        return Ok(cached);
    }
    let (yield_, next) = bluesky::get_feed(
        &state.http,
        &state.config.appview_base,
        feed_uri,
        cursor,
        limit,
    )
    .await
    .map_err(|e| match e {
        // an unknown or withdrawn generator 400s and a well-formed reference to
        // nothing 404s; either way the feed is not there. Distinct from
        // ActorNotFound because the web's repair for that one is a handle box,
        // which is the wrong thing to hand somebody with a bad feed link.
        HttpError::Status(400 | 404) => AppError::FeedNotFound(feed_uri.to_string()),
        other => AppError::Upstream(other.to_string()),
    })?;
    let page = FeedPage {
        yield_: Arc::new(yield_),
        next,
    };
    // cloning a FeedPage clones an Arc and a short cursor, so the entry and the
    // answer are the same page rather than two copies of it
    state.caches.feed_pages.insert(key, page.clone()).await;
    Ok(page)
}

/// Who is live on Streamplace, network-wide. Viewer-independent by
/// construction, which is what makes the single cache key honest: this
/// function must never see the follow graph, or one viewer's friends would be
/// served to the next.
pub async fn live_cached(state: &Arc<AppState>) -> Arc<Vec<streamplace::LiveStream>> {
    if let Some(cached) = state.caches.live.get(&0u8).await {
        return cached;
    }
    let streams = match streamplace::get_live(&state.http, &state.config.streamplace_base).await {
        Ok(streams) => Arc::new(streams),
        Err(e) => {
            tracing::debug!("streamplace live list failed: {e}");
            Arc::new(Vec::new())
        }
    };
    state.caches.live.insert(0u8, Arc::clone(&streams)).await;
    streams
}

/// The still-live stream URIs, if a live list is ALREADY cached. Pruning must
/// never fetch one: that would put a network round trip in the middle of
/// somebody's scroll every time the sixty-second cache lapsed, to answer a
/// question that only matters to the rare wall with a live brick still in its
/// pool. On a lapse this kicks off a background refresh for the NEXT page and
/// returns None, letting this one through.
pub async fn cached_live_uris(state: &Arc<AppState>) -> Option<HashSet<String>> {
    let Some(network) = state.caches.live.get(&0u8).await else {
        let refresh = Arc::clone(state);
        platform::spawn(async move {
            let _ = live_cached(&refresh).await;
        });
        return None;
    };
    Some(network.iter().map(|s| s.uri().to_string()).collect())
}

/// Which of the network's live streams belong to this viewer. Separated out
/// so the filter can be tested without a whole AppState: it is the seam where
/// a shared cache becomes one person's wall, and getting it wrong shows a
/// viewer strangers.
fn followed_live<'a>(
    network: &'a [streamplace::LiveStream],
    follows: &[bluesky::Follow],
) -> Vec<&'a streamplace::LiveStream> {
    // hidden follows are excluded here too: their live stream comes from
    // Streamplace, a source the AppView's labels never reach, so the cohort
    // filter alone would miss it
    let followed: HashSet<&str> = follows
        .iter()
        .filter(|f| !f.hidden())
        .map(|f| f.did.as_str())
        .collect();
    network
        .iter()
        .filter(|s| followed.contains(s.did()))
        .collect()
}

/// The live streams this particular viewer follows, as bricks.
pub async fn live_bricks(state: &Arc<AppState>, follows: &[bluesky::Follow]) -> Vec<Brick> {
    let network = live_cached(state).await;
    // only now, for the handful that survive the filter, is it worth finding
    // out where each repo (and so its poster) lives. Resolve them concurrently
    // rather than one plc round trip at a time; `buffered` bounds the fan-out
    // and preserves input order, so the pool sees the same bricks in the same
    // order the serial version produced.
    let followed: Vec<streamplace::LiveStream> = followed_live(&network, follows)
        .into_iter()
        .cloned()
        .collect();
    stream::iter(followed.into_iter().map(|live| {
        let state = Arc::clone(state);
        async move {
            let pds = pds_cached(&state, live.did()).await;
            live.into_brick(&state.config.streamplace_base, pds.as_deref())
        }
    }))
    .buffered(REPO_FAN_OUT)
    .collect()
    .await
}

/// Where an author's repo lives. Cached for a day: identity moves rarely, and
/// both the blog and the stream reader need the answer for every author.
pub async fn pds_cached(state: &Arc<AppState>, did: &str) -> Option<String> {
    if let Some(cached) = state.caches.pds.get(&did.to_string()).await {
        return Some(cached);
    }
    match pds::resolve(&state.http, &state.config.plc_base, did).await {
        Ok(pds) => {
            state.caches.pds.insert(did.to_string(), pds.clone()).await;
            Some(pds)
        }
        Err(e) => {
            tracing::debug!("pds resolution for {did} failed: {e}");
            None
        }
    }
}

/// One author's archived Streamplace videos.
pub async fn streams_cached(state: &Arc<AppState>, pds: &str, author: &Author) -> Arc<Vec<Brick>> {
    if let Some(cached) = state.caches.streams.get(&author.did).await {
        return cached;
    }
    let bricks =
        match streamplace::get_videos(&state.http, pds, &state.config.streamplace_base, author)
            .await
        {
            Ok(bricks) => Arc::new(bricks),
            Err(e) => {
                // a transient PDS failure is not "this author never streams";
                // caching it would silence them for a day. Skip the insert so
                // the next snapshot simply asks again. A genuine empty repo
                // comes back Ok(empty) and takes the negative TTL below.
                tracing::debug!("streamplace videos for {} failed: {e}", author.did);
                return Arc::new(Vec::new());
            }
        };
    // the same shape as blogs: the few who stream get rechecked within the
    // hour, the many who never will are left alone for a day
    let ttl = if bricks.is_empty() {
        STREAMS_NEGATIVE_TTL
    } else {
        STREAMS_POSITIVE_TTL
    };
    state
        .caches
        .streams
        .insert_with_ttl(author.did.clone(), Arc::clone(&bricks), ttl)
        .await;
    bricks
}

pub async fn std_docs_cached(
    state: &Arc<AppState>,
    pds: &str,
    author: &Author,
) -> Arc<standardsite::StdDocs> {
    if let Some(cached) = state.caches.std_docs.get(&author.did).await {
        return cached;
    }
    let docs = match standardsite::get_documents(&state.http, pds, author).await {
        Ok(result) => Arc::new(standardsite::StdDocs {
            bricks: result.bricks,
            suppressed_posts: result.suppressed_posts,
        }),
        Err(e) => {
            // same as streams: a transient failure must not be remembered for
            // a day as "this author publishes nothing". Skip the insert; only
            // a successful empty listing earns the negative TTL.
            tracing::debug!("standard.site fetch for {} failed: {e}", author.did);
            return Arc::new(standardsite::StdDocs::default());
        }
    };
    // publishers get rechecked soon; the silent majority is cached for a day
    let ttl = if docs.bricks.is_empty() {
        STD_DOCS_NEGATIVE_TTL
    } else {
        STD_DOCS_POSITIVE_TTL
    };
    state
        .caches
        .std_docs
        .insert_with_ttl(author.did.clone(), Arc::clone(&docs), ttl)
        .await;
    docs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The live list is one call for the WHOLE network, cached under a single
    /// key and shared by every viewer on the machine. The filter is therefore
    /// the only thing standing between a viewer and a wall of strangers, and
    /// it must key off the follow graph, not off who asked first.
    fn follow(did: &str) -> bluesky::Follow {
        bluesky::Follow {
            did: did.into(),
            handle: format!("{did}.test"),
            display_name: None,
            avatar: None,
            labels: vec![],
        }
    }

    fn opted_out_follow(did: &str) -> bluesky::Follow {
        let mut f = follow(did);
        f.labels =
            serde_json::from_value(serde_json::json!([{"val": "!no-unauthenticated"}])).unwrap();
        f
    }

    fn live_stream(did: &str) -> streamplace::LiveStream {
        streamplace::LiveStream::for_test(did)
    }

    #[test]
    fn a_viewer_only_sees_the_streams_they_follow() {
        let network = vec![
            live_stream("did:plc:friend"),
            live_stream("did:plc:stranger"),
        ];
        let follows = vec![follow("did:plc:friend")];

        let mine = followed_live(&network, &follows);
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].did(), "did:plc:friend");

        // and someone who follows nobody live gets nothing, rather than
        // inheriting whatever the last viewer's snapshot happened to cache
        assert!(followed_live(&network, &[]).is_empty());
    }

    /// A followed account that opted out of logged-out visibility is kept off
    /// the wall whole: not just their posts (dropped in the author-feed reader)
    /// but their live stream too, which comes from a different source that
    /// never sees the AppView label.
    #[test]
    fn an_opted_out_friend_is_not_shown_live() {
        let network = vec![live_stream("did:plc:friend")];
        let follows = vec![opted_out_follow("did:plc:friend")];
        assert!(
            followed_live(&network, &follows).is_empty(),
            "an opted-out friend's stream must not surface to a logged-out wall"
        );
    }
}

// The refresh bypass on the two fast content reads, driven against a wiremock
// AppView. A SECOND module rather than cases in the one above, because wiremock
// and tokio's runtime are `cfg(not(target_arch = "wasm32"))` dev dependencies:
// under the bare `#[cfg(test)]` above they would break the wasm32 build of
// --all-targets without failing a single test, and `just guard-wasm` is the only
// gate in the repo that would ever see it.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod refresh_tests {
    use super::*;
    use crate::config::Config;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const AUTHOR: &str = "did:plc:aa";
    /// The full wall's skim, and the glaze wall's deep media read. Named here
    /// so a test can answer one of the two reads and leave the other alone.
    const SKIM: &str = "posts_no_replies";
    const DEEP_MEDIA: &str = "posts_with_media";

    fn state_for(server: &MockServer) -> Arc<AppState> {
        Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            ..Default::default()
        }))
    }

    /// A brick's id is its at-uri, so an rkey is how a test tells the wall it
    /// was handed from the wall it asked to replace.
    fn post_uri(rkey: &str) -> String {
        format!("at://{AUTHOR}/app.bsky.feed.post/{rkey}")
    }

    /// One post, under `filter`, until the server is reset.
    async fn answers_with(server: &MockServer, filter: &str, rkey: &str) {
        let body = serde_json::json!({"feed": [{
            "post": {
                "uri": post_uri(rkey),
                "author": {"did": AUTHOR, "handle": "a.test"},
                "record": {"text": "hello wall", "createdAt": "2026-07-10T12:00:00Z"},
                "likeCount": 0,
                "repostCount": 0
            }
        }]});
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .and(query_param("filter", filter))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(server)
            .await;
    }

    /// Every attempt 5xxs, so the three-attempt retry loop hands back a 503 and
    /// the reader classifies it transient. This is the failure a refresh must
    /// not let thin the wall.
    async fn always_5xx(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(ResponseTemplate::new(503))
            .mount(server)
            .await;
    }

    fn only_brick(yield_: &bluesky::AuthorYield) -> &str {
        assert_eq!(yield_.bricks.len(), 1, "the fixture feed carries one post");
        yield_.bricks[0].id()
    }

    /// The whole point of the flag: five minutes of cached posts is exactly what
    /// a reader asking for a new wall wants to get past, so a refreshed read
    /// steps over a live entry, reaches the AppView, and leaves the newer answer
    /// behind it for everyone who reads after.
    #[tokio::test]
    async fn a_refreshed_read_reaches_past_a_fresh_cache_entry() {
        let server = MockServer::start().await;
        answers_with(&server, SKIM, "1").await;
        let state = state_for(&server);

        let cold = author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("a live AppView answers");
        assert_eq!(only_brick(&cold), post_uri("1"));

        // the wall moves on; only a refresh may see it
        server.reset().await;
        answers_with(&server, SKIM, "2").await;

        let cached = author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("the entry is still live");
        assert_eq!(
            only_brick(&cached),
            post_uri("1"),
            "an ordinary read must still be served from cache, or the flag is doing nothing"
        );

        let refreshed = author_feed_cached(&state, AUTHOR, true)
            .await
            .expect("a refreshed read reaches the AppView");
        assert_eq!(only_brick(&refreshed), post_uri("2"));

        let after = state
            .caches
            .author_feed
            .get(&AUTHOR.to_string())
            .await
            .expect("and the refreshed answer is what stays behind");
        assert_eq!(
            only_brick(&after),
            post_uri("2"),
            "the AppView answer must overwrite the entry the refresh stepped over"
        );
    }

    /// A refresh must never make the wall worse. Returning `None` here would
    /// drop the author from the wall AND leave them unfanned, so a flaky moment
    /// during a refresh would visibly thin the wall the reader just asked to
    /// improve.
    #[tokio::test]
    async fn a_refreshed_read_that_fails_falls_back_to_the_cached_yield() {
        let server = MockServer::start().await;
        answers_with(&server, SKIM, "1").await;
        let state = state_for(&server);
        author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("a live AppView answers");

        server.reset().await;
        always_5xx(&server).await;

        let refreshed = author_feed_cached(&state, AUTHOR, true)
            .await
            .expect("a refreshed read that fails falls back rather than vanishing");
        assert_eq!(
            only_brick(&refreshed),
            post_uri("1"),
            "the author did answer, just earlier"
        );
        let survived = state
            .caches
            .author_feed
            .get(&AUTHOR.to_string())
            .await
            .expect("and the older entry survives a failed refresh");
        assert_eq!(only_brick(&survived), post_uri("1"));
    }

    /// The negative space of the fallback. With nothing behind it there is
    /// nothing to fall back to, so a refreshed read is a cold read: `None`, the
    /// author is not counted as fanned out, and the blip is not remembered as
    /// "this author posts nothing".
    #[tokio::test]
    async fn a_refreshed_read_with_nothing_cached_fails_like_a_cold_one() {
        let server = MockServer::start().await;
        always_5xx(&server).await;
        let state = state_for(&server);

        assert!(
            author_feed_cached(&state, AUTHOR, true).await.is_none(),
            "a refresh cannot invent a yield it never had"
        );
        assert!(
            state
                .caches
                .author_feed
                .get(&AUTHOR.to_string())
                .await
                .is_none(),
            "and a transient failure is never cached, refreshed or not"
        );
    }

    /// The glaze wall's read has both behaviours too, against its OWN cache.
    /// The two feeds are deliberately kept apart, and a bypass or a fallback
    /// that reached the wrong one would hand the image wall the full wall's
    /// skim: text posts on a wall of pictures.
    #[tokio::test]
    async fn a_refreshed_image_read_bypasses_and_falls_back_to_its_own_cache() {
        let server = MockServer::start().await;
        answers_with(&server, DEEP_MEDIA, "image-1").await;
        answers_with(&server, SKIM, "skim").await;
        let state = state_for(&server);
        image_feed_cached(&state, AUTHOR, false)
            .await
            .expect("the deep media read answers");
        author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("and so does the skim, into the other cache");

        server.reset().await;
        answers_with(&server, DEEP_MEDIA, "image-2").await;

        let refreshed = image_feed_cached(&state, AUTHOR, true)
            .await
            .expect("a refreshed image read reaches the AppView");
        assert_eq!(
            only_brick(&refreshed),
            post_uri("image-2"),
            "the deep media read steps over its live entry too"
        );

        server.reset().await;
        always_5xx(&server).await;

        let fell_back = image_feed_cached(&state, AUTHOR, true)
            .await
            .expect("a refreshed image read falls back too");
        assert_eq!(
            only_brick(&fell_back),
            post_uri("image-2"),
            "the image feed falls back to the image cache, not the author feed's"
        );
    }

    /// What makes the claim in 05-caching-and-persistence.md true: a refreshed
    /// read is an insert like any other, so the next persist cycle captures the
    /// fresher data rather than the data the refresh replaced.
    #[tokio::test]
    async fn a_refreshed_read_leaves_the_cache_dirty() {
        let server = MockServer::start().await;
        answers_with(&server, SKIM, "1").await;
        let state = state_for(&server);
        author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("a live AppView answers");

        // stand in for a persist cycle: it takes the flag, leaving the cache
        // clean, and everything after this is the refresh's own doing
        assert!(
            state.caches.author_feed.take_dirty(),
            "the cold read dirties the cache"
        );

        server.reset().await;
        answers_with(&server, SKIM, "2").await;
        author_feed_cached(&state, AUTHOR, true)
            .await
            .expect("a refreshed read reaches the AppView");

        assert!(
            state.caches.author_feed.is_dirty(),
            "a refreshed read must dirty the cache, or its newer posts are never persisted"
        );
    }
}

// The feed-page cache, driven against a wiremock AppView. A THIRD module for the
// same reason there is a second one: wiremock and tokio's runtime are
// `cfg(not(target_arch = "wasm32"))` dev dependencies, so a bare `#[cfg(test)]`
// would break the wasm32 build of --all-targets without failing a single test,
// and `just guard-wasm` is the only gate in the repo that would ever see it.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod feed_page_tests {
    use super::*;
    use crate::config::Config;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A feed generator's AT-URI, as a parsed reference hands one over.
    const FEED: &str = "at://did:plc:gen/app.bsky.feed.generator/whats-hot";
    /// What the mixed views ask a feed for (`PAGE_SIZE`) and what the glaze wall
    /// asks the same feed for (getFeed's ceiling). The two limits are the reason
    /// the key carries one.
    const MIXED: u32 = 24;
    const GLAZE: u32 = 100;

    fn state_for(server: &MockServer) -> Arc<AppState> {
        Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            ..Default::default()
        }))
    }

    /// A brick's id is its at-uri, so an rkey is how a test tells one fixture
    /// page from another.
    fn post_uri(rkey: &str) -> String {
        format!("at://did:plc:aa/app.bsky.feed.post/{rkey}")
    }

    /// One post under `limit` (and, when `at` is given, only for that incoming
    /// cursor), followed by `next`.
    ///
    /// `expect(1)` on every mock, verified when the server drops: a cache that
    /// missed shows up here as a second request rather than as a passing
    /// assertion about identical fixture content.
    async fn answers(
        server: &MockServer,
        limit: u32,
        at: Option<&str>,
        rkey: &str,
        next: Option<&str>,
    ) {
        let mut body = serde_json::json!({"feed": [{
            "post": {
                "uri": post_uri(rkey),
                "author": {"did": "did:plc:aa", "handle": "a.test"},
                "record": {"text": "hello wall", "createdAt": "2026-07-10T12:00:00Z"},
                "likeCount": 0,
                "repostCount": 0
            }
        }]});
        if let Some(next) = next {
            body["cursor"] = serde_json::json!(next);
        }
        let mut mock = Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getFeed"))
            .and(query_param("limit", limit.to_string()));
        if let Some(at) = at {
            mock = mock.and(query_param("cursor", at));
        }
        mock.respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(server)
            .await;
    }

    /// Every attempt answers `status`. `retry-after: 0` because a retryable
    /// status (the 500 below) costs three attempts, and a real backoff in the
    /// middle of a test buys nothing.
    async fn always(server: &MockServer, status: u16) {
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getFeed"))
            .respond_with(ResponseTemplate::new(status).insert_header("retry-after", "0"))
            .mount(server)
            .await;
    }

    fn only_brick(page: &FeedPage) -> &str {
        assert_eq!(
            page.yield_.bricks.len(),
            1,
            "each fixture page carries one post"
        );
        page.yield_.bricks[0].id()
    }

    async fn upstream_reads(server: &MockServer) -> usize {
        server
            .received_requests()
            .await
            .expect("the mock server records what it was asked")
            .len()
    }

    /// The whole point of the cache. A first screen is a preview poll and then a
    /// freeze over the same page, and a back gesture is a third read of it, so
    /// without an entry the cheapest wall mason lays would cost three AppView
    /// calls to show one page.
    #[tokio::test]
    async fn the_second_read_of_a_page_never_reaches_the_appview() {
        let server = MockServer::start().await;
        answers(&server, MIXED, None, "1", Some("page2")).await;
        let state = state_for(&server);

        let first = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .expect("the AppView answers");
        let second = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .expect("and the cache answers after it");

        assert_eq!(only_brick(&first), post_uri("1"));
        assert_eq!(
            only_brick(&second),
            post_uri("1"),
            "the same page comes back"
        );
        assert_eq!(
            second.next.as_deref(),
            Some("page2"),
            "cursor and all: a page served without its cursor could not be paged past"
        );
        assert_eq!(
            upstream_reads(&server).await,
            1,
            "one page, one upstream read"
        );
    }

    /// The limit is in the key because the two views read the same feed to
    /// different depths. Without it the glaze wall is handed the mixed wall's
    /// 24-item page, lays the three or four images in it, and runs a quarter as
    /// deep with nothing anywhere saying so.
    #[tokio::test]
    async fn two_limits_of_one_page_do_not_collide() {
        let server = MockServer::start().await;
        answers(&server, MIXED, None, "mixed", None).await;
        answers(&server, GLAZE, None, "glaze", None).await;
        let state = state_for(&server);

        let mixed = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .expect("the mixed views ask for a page");
        let glaze = feed_page_cached(&state, FEED, None, GLAZE, false)
            .await
            .expect("and the glaze wall asks the same feed deeper");

        assert_eq!(only_brick(&mixed), post_uri("mixed"));
        assert_eq!(
            only_brick(&glaze),
            post_uri("glaze"),
            "the deep read must get its own page, not the shallow one cached a moment earlier"
        );
        assert_eq!(
            upstream_reads(&server).await,
            2,
            "two depths of one feed are two upstream reads"
        );
    }

    /// The cursor is in the key too, which is what lets a reader page: the
    /// second page of a feed is a different entry from the first, not a hit on
    /// it.
    #[tokio::test]
    async fn a_second_cursor_is_a_second_page() {
        let server = MockServer::start().await;
        // the cursor-bearing mock is mounted first: the general one would match
        // a request carrying a cursor too, and wiremock takes the first match
        answers(&server, MIXED, Some("page2"), "second", None).await;
        answers(&server, MIXED, None, "first", Some("page2")).await;
        let state = state_for(&server);

        let first = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .expect("a fresh feed wall starts with no cursor");
        let second = feed_page_cached(&state, FEED, first.next.as_deref(), MIXED, false)
            .await
            .expect("and pages on the cursor the first page carried");

        assert_eq!(only_brick(&first), post_uri("first"));
        assert_eq!(
            only_brick(&second),
            post_uri("second"),
            "paging must reach the next page, not re-serve the first"
        );
        assert!(second.next.is_none(), "and this feed has ended");
    }

    /// An unknown or withdrawn feed generator. `getFeed` 400s on one, and this
    /// is the reader holding a reference to nothing rather than an outage, so it
    /// is `FeedNotFound` and the web can offer the picker instead of a handle
    /// box.
    #[tokio::test]
    async fn a_400_is_a_feed_that_is_not_there() {
        let server = MockServer::start().await;
        always(&server, 400).await;
        let state = state_for(&server);

        let failure = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .err()
            .expect("a 400 is a failure, not a page");
        match failure {
            AppError::FeedNotFound(uri) => assert_eq!(
                uri, FEED,
                "the error names the feed the reader actually asked for"
            ),
            other => panic!("a 400 must be a missing feed, got {other:?}"),
        }
    }

    /// And a 404 the same, so a feed wall reports the same thing whichever of
    /// the two an AppView chooses for a reference it cannot serve.
    #[tokio::test]
    async fn a_404_is_a_feed_that_is_not_there() {
        let server = MockServer::start().await;
        always(&server, 404).await;
        let state = state_for(&server);

        let failure = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .err()
            .expect("a 404 is a failure, not a page");
        match failure {
            AppError::FeedNotFound(uri) => assert_eq!(uri, FEED),
            other => panic!("a 404 must be a missing feed, got {other:?}"),
        }
    }

    /// The other side of that line. A feed generator is a third-party service
    /// with its own uptime, and one falling over is not the reader's reference
    /// being wrong: telling them "no such feed" would send them to fix a link
    /// that is fine.
    #[tokio::test]
    async fn a_500_is_an_upstream_failure() {
        let server = MockServer::start().await;
        always(&server, 500).await;
        let state = state_for(&server);

        let failure = feed_page_cached(&state, FEED, None, MIXED, false)
            .await
            .err()
            .expect("a 500 is a failure, not a page");
        assert!(
            matches!(failure, AppError::Upstream(_)),
            "a server-side failure must not read as a missing feed, got {failure:?}"
        );
    }
}

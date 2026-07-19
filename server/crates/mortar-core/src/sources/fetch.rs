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

use futures::stream::{self, StreamExt};

use super::{bluesky, pds, standardsite, streamplace};
use crate::cache::{
    STD_DOCS_NEGATIVE_TTL, STD_DOCS_POSITIVE_TTL, STREAMS_NEGATIVE_TTL, STREAMS_POSITIVE_TTL,
};
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

pub async fn author_feed_cached(
    state: &Arc<AppState>,
    author_did: &str,
) -> Arc<bluesky::AuthorYield> {
    if let Some(cached) = state.caches.author_feed.get(&author_did.to_string()).await {
        return cached;
    }
    let yield_ =
        match bluesky::get_author_feed(&state.http, &state.config.appview_base, author_did).await {
            Ok(yield_) => Arc::new(yield_),
            Err(e) => {
                // a single author failing must never sink the wall
                tracing::debug!("author feed {author_did} failed: {e}");
                Arc::new(bluesky::AuthorYield { bricks: Vec::new() })
            }
        };
    state
        .caches
        .author_feed
        .insert(author_did.to_string(), Arc::clone(&yield_))
        .await;
    yield_
}

/// One author's recent MEDIA posts, read deep for the glaze wall. Same shape as
/// `author_feed_cached` but a separate endpoint (`posts_with_media`) and a
/// separate cache, so the image wall's deeper read never displaces the full
/// wall's shallow one for the same author.
pub async fn image_feed_cached(
    state: &Arc<AppState>,
    author_did: &str,
) -> Arc<bluesky::AuthorYield> {
    if let Some(cached) = state.caches.image_feed.get(&author_did.to_string()).await {
        return cached;
    }
    let yield_ =
        match bluesky::get_image_feed(&state.http, &state.config.appview_base, author_did).await {
            Ok(yield_) => Arc::new(yield_),
            Err(e) => {
                tracing::debug!("image feed {author_did} failed: {e}");
                Arc::new(bluesky::AuthorYield { bricks: Vec::new() })
            }
        };
    state
        .caches
        .image_feed
        .insert(author_did.to_string(), Arc::clone(&yield_))
        .await;
    yield_
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

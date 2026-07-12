//! The heart of mortar: a snapshot is one user's wall-in-progress. Built on
//! first request with a first-paint threshold (respond as soon as enough
//! authors have answered), filled to completion by a background task, and
//! paged immutably; bricks already served never move.
//!
//! The cursor carries (snapshot id, seed, offset). If a snapshot is evicted
//! mid-scroll (cache TTL natively; instance death in the service worker),
//! the same seed rebuilds a closely-matching wall; continuity is
//! best-effort, determinism of jitter is exact.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures::stream::{self, StreamExt};
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use tokio::sync::{Mutex, Notify};
use xxhash_rust::xxh3::xxh3_64_with_seed;

use super::{mix, score};
use crate::cache::{STD_DOCS_NEGATIVE_TTL, STD_DOCS_POSITIVE_TTL, StdDocs};
use crate::error::AppError;
use crate::model::{Author, Brick};
use crate::platform::{self, Instant, SystemTime};
use crate::sources::bluesky::AuthorYield;
use crate::sources::{bluesky, standardsite, steam};
use crate::state::AppState;

/// Sampled authors per snapshot; never fan out to the whole follow graph.
const COHORT_SIZE: usize = 100;
/// Of which: authors that yielded content in recent snapshots.
const KNOWN_ACTIVE: usize = 60;
/// First paint: respond once the pool holds bricks from this many DISTINCT
/// authors. It used to count bricks, which is how a single chatty account
/// could own the whole first screen: 30 of its posts cleared a brick-count
/// gate long before anyone else's feed arrived.
const FIRST_PAINT_AUTHORS: usize = 12;
/// No author may hold more than this many bricks in the pool. The mixer's
/// diversity window can only space authors out if the pool HAS other authors.
const MAX_BRICKS_PER_AUTHOR: usize = 4;
/// …or this much time has passed, whichever comes first.
const FIRST_PAINT_DEADLINE: Duration = Duration::from_secs(3);
/// Per-slot caps on bricks held per snapshot: posts / blogs / Bluesky videos
/// / Steam trailers. Posts arrive by the thousand and must not crowd out the
/// rarer kinds, and trailers hydrate last so they get a reserved slot that
/// Bluesky videos can't fill first.
const KIND_CAPS: [usize; 4] = [500, 60, 30, 13];
/// Author-feed fan-out concurrency (the rate limiter is the real governor).
const FAN_OUT: usize = 16;
/// New Steam games hydrated per snapshot; the storefront API throttles hard.
const STEAM_MENTIONS_PER_SNAPSHOT: usize = 10;
/// Featured trailers mixed in as exploration filler.
const STEAM_FEATURED_PER_SNAPSHOT: usize = 3;
/// Storefront concurrency.
const STEAM_FAN_OUT: usize = 2;

fn kind_slot(brick: &Brick) -> usize {
    match brick {
        Brick::Post(_) => 0,
        Brick::Blog(_) => 1,
        Brick::Video(v) if v.source == crate::model::VideoSource::Bluesky => 2,
        Brick::Video(_) => 3,
    }
}

pub struct Snapshot {
    pub id: String,
    pub seed: u64,
    inner: Mutex<Inner>,
    progress: Notify,
}

impl Snapshot {
    fn new(id: String, seed: u64) -> Self {
        Self {
            id,
            seed,
            inner: Mutex::new(Inner {
                pool: Vec::new(),
                wall: Vec::new(),
                seen: HashSet::new(),
                kind_counts: [0; 4],
                author_counts: HashMap::new(),
                warming: true,
            }),
            progress: Notify::new(),
        }
    }
}

struct Inner {
    pool: Vec<Brick>,
    wall: Vec<Brick>,
    seen: HashSet<String>,
    /// pool+wall population per kind, checked against KIND_CAPS
    kind_counts: [usize; 4],
    /// bricks admitted per author, checked against MAX_BRICKS_PER_AUTHOR
    author_counts: HashMap<String, usize>,
    warming: bool,
}

impl Inner {
    /// Insert into the pool unless it is a duplicate, stale, over its kind's
    /// cap, or its author already holds their share.
    fn admit(&mut self, brick: &Brick, now: chrono::DateTime<Utc>) {
        if !score::is_fresh(brick, now) {
            return;
        }
        let slot = kind_slot(brick);
        if self.kind_counts[slot] >= KIND_CAPS[slot] {
            return;
        }
        let author = score::author_key(brick).to_string();
        let held = self.author_counts.entry(author).or_insert(0);
        if *held >= MAX_BRICKS_PER_AUTHOR {
            return;
        }
        if self.seen.insert(brick.id().to_string()) {
            *held += 1;
            self.kind_counts[slot] += 1;
            self.pool.push(brick.clone());
        }
    }

    /// Distinct authors currently represented in the pool and on the wall.
    fn distinct_authors(&self) -> usize {
        self.author_counts.values().filter(|n| **n > 0).count()
    }
}

pub fn snapshot_id(did: &str, seed: u64) -> String {
    format!("{:016x}", xxh3_64_with_seed(did.as_bytes(), seed))
}

pub fn fresh_seed(did: &str) -> u64 {
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    xxh3_64_with_seed(did.as_bytes(), millis)
}

/// Fetch-or-create under one cache lock: exactly one caller wins the insert
/// and spawns the background fill; everyone gets the same Arc. Then all
/// callers wait for the first-paint threshold (a no-op once warm).
pub async fn get_or_build(
    state: &Arc<AppState>,
    did: &str,
    seed: u64,
) -> Result<Arc<Snapshot>, AppError> {
    let id = snapshot_id(did, seed);
    let (snapshot, inserted) = state
        .caches
        .snapshots
        .get_or_insert_with(id.clone(), || Arc::new(Snapshot::new(id.clone(), seed)))
        .await;

    if inserted {
        let fill_state = Arc::clone(state);
        let fill_snapshot = Arc::clone(&snapshot);
        let viewer = did.to_string();
        platform::spawn(async move {
            fill(fill_state, fill_snapshot, viewer, seed).await;
        });
    }

    // first-paint threshold: enough bricks pooled, or deadline
    let deadline = Instant::now() + FIRST_PAINT_DEADLINE;
    loop {
        {
            let inner = snapshot.inner.lock().await;
            if !inner.warming || inner.distinct_authors() >= FIRST_PAINT_AUTHORS {
                break;
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        let _ = platform::timeout(remaining, snapshot.progress.notified()).await;
    }

    Ok(snapshot)
}

/// The background fill: follows → cohort fan-out (+ concurrent featured
/// trailers) → mentioned-game trailers → warming off. Any follow-graph
/// failure leaves an empty (but terminated) snapshot rather than an error -
/// actor existence was already checked by resolve.
async fn fill(state: Arc<AppState>, snapshot: Arc<Snapshot>, viewer: String, seed: u64) {
    let started = Instant::now();

    let follows = match get_follows_cached(&state, &viewer).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("follows fetch for {viewer} failed: {e}");
            let mut inner = snapshot.inner.lock().await;
            inner.warming = false;
            drop(inner);
            snapshot.progress.notify_waiters();
            return;
        }
    };
    let cohort = sample_cohort(&state, &viewer, &follows, seed).await;
    tracing::debug!(
        "snapshot {}: {} follows, cohort of {}",
        snapshot.id,
        follows.len(),
        cohort.len()
    );

    // featured trailers don't depend on the cohort; hydrate them
    // concurrently with the author fan-out so they make the first pages
    let featured_fill = async {
        if !state.config.steam_enabled {
            return;
        }
        let featured = featured_sample(&state, seed).await;
        let bricks = hydrate_trailers(&state, featured).await;
        tracing::debug!("steam featured yielded {} trailer bricks", bricks.len());
        let now = Utc::now();
        let mut inner = snapshot.inner.lock().await;
        for brick in &bricks {
            inner.admit(brick, now);
        }
        drop(inner);
        snapshot.progress.notify_waiters();
    };

    let authors_fill = async {
        let mut feeds = stream::iter(cohort.into_iter().map(|author| {
            let state = Arc::clone(&state);
            async move {
                let (yield_, docs) = tokio::join!(
                    author_feed_cached(&state, &author.did),
                    std_docs_cached(&state, &author),
                );
                (author, yield_, docs)
            }
        }))
        .buffer_unordered(FAN_OUT);

        let mut answered = 0usize;
        let mut yielding_authors: Vec<String> = Vec::new();
        let mut mentioned: Vec<u64> = Vec::new();
        while let Some((author, yield_, docs)) = feeds.next().await {
            answered += 1;
            if !yield_.bricks.is_empty() || !docs.bricks.is_empty() {
                yielding_authors.push(author.did);
            }
            for appid in &yield_.steam_appids {
                if !mentioned.contains(appid) {
                    mentioned.push(*appid);
                }
            }
            {
                let mut inner = snapshot.inner.lock().await;
                let inner = &mut *inner;
                // bskyPostRef suppression: the blog card wins over its
                // cross-posted skeet, whether the post came first or later
                for uri in &docs.suppressed_posts {
                    if inner.seen.insert(uri.clone()) {
                        // post not pooled yet; the insert blocks it later
                    } else {
                        inner.pool.retain(|b| b.id() != uri);
                    }
                }
                let now = Utc::now();
                for brick in docs.bricks.iter().chain(yield_.bricks.iter()) {
                    inner.admit(brick, now);
                }
            }
            snapshot.progress.notify_waiters();
        }
        (answered, yielding_authors, mentioned)
    };

    let ((answered, yielding_authors, mut mentioned_appids), ()) =
        futures::join!(authors_fill, featured_fill);

    // games the cohort talked about; these do need the posts first
    mentioned_appids.truncate(STEAM_MENTIONS_PER_SNAPSHOT);
    if state.config.steam_enabled && !mentioned_appids.is_empty() {
        tracing::debug!("steam mentions: hydrating {}", mentioned_appids.len());
        let bricks = hydrate_trailers(&state, mentioned_appids).await;
        let now = Utc::now();
        let mut inner = snapshot.inner.lock().await;
        for brick in &bricks {
            inner.admit(brick, now);
        }
    }

    {
        let mut inner = snapshot.inner.lock().await;
        inner.warming = false;
    }
    snapshot.progress.notify_waiters();
    tracing::debug!(
        "snapshot {} warmed: {answered} authors in {:?}",
        snapshot.id,
        started.elapsed()
    );

    // remember who yielded, for the next snapshot's cohort
    record_activity(&state, &viewer, yielding_authors).await;
}

/// Serve one page, laying new bricks from the pool as needed. Waits briefly
/// while warming if the wall is still too short.
pub async fn get_page(snapshot: &Snapshot, offset: usize, size: usize) -> (Vec<Brick>, bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        {
            let mut guard = snapshot.inner.lock().await;
            let inner = &mut *guard;
            let wanted = offset + size;
            if inner.wall.len() < wanted && !inner.pool.is_empty() {
                let missing = wanted - inner.wall.len();
                mix::lay(
                    &mut inner.pool,
                    &mut inner.wall,
                    missing,
                    snapshot.seed,
                    Utc::now(),
                );
            }
            let exhausted = !inner.warming && inner.pool.is_empty();
            if inner.wall.len() >= wanted || exhausted {
                let end = wanted.min(inner.wall.len());
                let items = inner
                    .wall
                    .get(offset.min(end)..end)
                    .map(<[Brick]>::to_vec)
                    .unwrap_or_default();
                let has_more = !inner.pool.is_empty() || inner.warming;
                return (items, has_more);
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            // serve a short page rather than hang; scroll retries
            let guard = snapshot.inner.lock().await;
            let end = (offset + size).min(guard.wall.len());
            let items = guard
                .wall
                .get(offset.min(end)..end)
                .map(<[Brick]>::to_vec)
                .unwrap_or_default();
            return (items, guard.warming || !guard.pool.is_empty());
        }
        let _ = platform::timeout(remaining, snapshot.progress.notified()).await;
    }
}

async fn get_follows_cached(
    state: &Arc<AppState>,
    did: &str,
) -> Result<Arc<Vec<bluesky::Follow>>, AppError> {
    if let Some(follows) = state.caches.follows.get(&did.to_string()).await {
        return Ok(follows);
    }
    let follows = bluesky::get_follows(&state.http, &state.config.appview_base, did)
        .await
        .map(Arc::new)
        .map_err(|e| AppError::Upstream(e.to_string()))?;
    state
        .caches
        .follows
        .insert(did.to_string(), Arc::clone(&follows))
        .await;
    Ok(follows)
}

async fn author_feed_cached(state: &Arc<AppState>, author_did: &str) -> Arc<AuthorYield> {
    if let Some(cached) = state.caches.author_feed.get(&author_did.to_string()).await {
        return cached;
    }
    let yield_ =
        match bluesky::get_author_feed(&state.http, &state.config.appview_base, author_did).await {
            Ok(yield_) => Arc::new(yield_),
            Err(e) => {
                // a single author failing must never sink the wall
                tracing::debug!("author feed {author_did} failed: {e}");
                Arc::new(AuthorYield {
                    bricks: Vec::new(),
                    steam_appids: Vec::new(),
                })
            }
        };
    state
        .caches
        .author_feed
        .insert(author_did.to_string(), Arc::clone(&yield_))
        .await;
    yield_
}

/// Hydrate a batch of appids into trailer bricks (bounded concurrency).
async fn hydrate_trailers(state: &Arc<AppState>, appids: Vec<u64>) -> Vec<Brick> {
    stream::iter(appids.into_iter().map(|appid| {
        let state = Arc::clone(state);
        async move { steam_trailers_cached(&state, appid).await }
    }))
    .buffer_unordered(STEAM_FAN_OUT)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .flat_map(|bricks| bricks.iter().cloned().collect::<Vec<_>>())
    .collect()
}

async fn steam_trailers_cached(state: &Arc<AppState>, appid: u64) -> Arc<Vec<Brick>> {
    if let Some(cached) = state.caches.steam_trailers.get(&appid).await {
        return cached;
    }
    let hydrated_at = Utc::now().to_rfc3339();
    let bricks = match steam::get_trailers(
        &state.http,
        &state.config.steam_store_base,
        appid,
        &hydrated_at,
    )
    .await
    {
        Ok(bricks) => Arc::new(bricks),
        Err(e) => {
            tracing::debug!("steam appdetails {appid} failed: {e}");
            Arc::new(Vec::new())
        }
    };
    state
        .caches
        .steam_trailers
        .insert(appid, Arc::clone(&bricks))
        .await;
    bricks
}

/// A small seeded sample of featured releases; exploration filler so video
/// bricks exist even when nobody you follow talks about games.
async fn featured_sample(state: &Arc<AppState>, seed: u64) -> Vec<u64> {
    let featured = match state.caches.steam_featured.get(&0u8).await {
        Some(cached) => cached,
        None => {
            let ids = match steam::get_featured(&state.http, &state.config.steam_store_base).await {
                Ok(ids) => Arc::new(ids),
                Err(e) => {
                    tracing::debug!("steam featured failed: {e}");
                    Arc::new(Vec::new())
                }
            };
            state
                .caches
                .steam_featured
                .insert(0u8, Arc::clone(&ids))
                .await;
            ids
        }
    };
    let mut ids: Vec<u64> = featured.as_ref().clone();
    // sample from a small stable prefix, not the whole list: successive
    // snapshots then reuse already-hydrated trailers (cache hits) instead of
    // hydrating 3 cold appids after the wall has already been laid
    ids.truncate(STEAM_FEATURED_PER_SNAPSHOT * 2);
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x57EA);
    ids.shuffle(&mut rng);
    ids.truncate(STEAM_FEATURED_PER_SNAPSHOT);
    ids
}

/// Cohort: up to KNOWN_ACTIVE authors that yielded content before, topped up
/// with a seeded-random sample of the rest so refreshes rotate through the
/// whole follow graph.
async fn sample_cohort(
    state: &Arc<AppState>,
    viewer: &str,
    follows: &[bluesky::Follow],
    seed: u64,
) -> Vec<Author> {
    let known_active = state
        .caches
        .activity
        .get(&viewer.to_string())
        .await
        .unwrap_or_default();
    let by_did: std::collections::HashMap<&str, &bluesky::Follow> =
        follows.iter().map(|f| (f.did.as_str(), f)).collect();

    let mut cohort: Vec<Author> = known_active
        .iter()
        .filter_map(|did| by_did.get(did.as_str()))
        .take(KNOWN_ACTIVE)
        .map(|f| Author::from(*f))
        .collect();

    let chosen: HashSet<&str> = cohort.iter().map(|a| a.did.as_str()).collect();
    let mut rest: Vec<&bluesky::Follow> = follows
        .iter()
        .filter(|f| !chosen.contains(f.did.as_str()))
        .collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    rest.shuffle(&mut rng);
    cohort.extend(
        rest.into_iter()
            .take(COHORT_SIZE.saturating_sub(cohort.len()))
            .map(Author::from),
    );
    cohort
}

async fn std_docs_cached(state: &Arc<AppState>, author: &Author) -> Arc<StdDocs> {
    if let Some(cached) = state.caches.std_docs.get(&author.did).await {
        return cached;
    }
    let docs = match standardsite::get_documents(&state.http, &state.config.plc_base, author).await
    {
        Ok(result) => Arc::new(StdDocs {
            bricks: result.bricks,
            suppressed_posts: result.suppressed_posts,
        }),
        Err(e) => {
            tracing::debug!("standard.site fetch for {} failed: {e}", author.did);
            Arc::new(StdDocs {
                bricks: Vec::new(),
                suppressed_posts: Vec::new(),
            })
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

async fn record_activity(state: &Arc<AppState>, viewer: &str, mut yielding: Vec<String>) {
    if yielding.is_empty() {
        return;
    }
    if let Some(previous) = state.caches.activity.get(&viewer.to_string()).await {
        let fresh: HashSet<String> = yielding.iter().cloned().collect();
        yielding.extend(
            previous
                .iter()
                .filter(|d| !fresh.contains(d.as_str()))
                .cloned(),
        );
    }
    yielding.truncate(300);
    state
        .caches
        .activity
        .insert(viewer.to_string(), Arc::new(yielding))
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Author, PostBrick};

    fn post(id: usize, author: usize, hours_old: i64) -> Brick {
        Brick::Post(PostBrick {
            id: format!("post-{id}"),
            url: String::new(),
            author: Author {
                did: format!("did:plc:a{author}"),
                handle: format!("a{author}.test"),
                display_name: None,
                avatar: None,
            },
            text: "t".into(),
            created_at: (Utc::now() - chrono::TimeDelta::hours(hours_old)).to_rfc3339(),
            like_count: 0,
            repost_count: 0,
            images: vec![],
            external: None,
        })
    }

    fn inner() -> Inner {
        Inner {
            pool: Vec::new(),
            wall: Vec::new(),
            seen: HashSet::new(),
            kind_counts: [0; 4],
            author_counts: HashMap::new(),
            warming: true,
        }
    }

    /// The root cause of the one-author wall: nothing stopped a chatty account
    /// pouring its whole feed into the pool, and the mixer cannot space out
    /// authors that are not there.
    #[test]
    fn one_author_cannot_flood_the_pool() {
        let mut i = inner();
        let now = Utc::now();
        for n in 0..30 {
            i.admit(&post(n, 0, 1), now);
        }
        assert_eq!(i.pool.len(), MAX_BRICKS_PER_AUTHOR);
    }

    /// First paint used to gate on brick count, so 30 bricks from one author
    /// could open the wall. It gates on distinct authors now.
    #[test]
    fn distinct_authors_is_what_opens_the_wall() {
        let mut i = inner();
        let now = Utc::now();
        for n in 0..30 {
            i.admit(&post(n, 0, 1), now);
        }
        assert_eq!(
            i.distinct_authors(),
            1,
            "one loud author is still one author"
        );

        for author in 1..FIRST_PAINT_AUTHORS {
            i.admit(&post(100 + author, author, 1), now);
        }
        assert_eq!(i.distinct_authors(), FIRST_PAINT_AUTHORS);
    }

    /// Stale bricks never enter the pool, so they cannot be laid.
    #[test]
    fn stale_bricks_are_not_admitted() {
        let mut i = inner();
        let now = Utc::now();
        i.admit(&post(1, 0, 2), now); // 2 hours old
        i.admit(&post(2, 1, 71), now); // just inside the 72h window
        i.admit(&post(3, 2, 73), now); // just outside it
        assert_eq!(i.pool.len(), 2);
    }
}

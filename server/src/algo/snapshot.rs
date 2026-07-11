//! The heart of mortar: a snapshot is one user's wall-in-progress. Built on
//! first request with a first-paint threshold (respond as soon as enough
//! authors have answered), filled to completion by a background task, and
//! paged immutably — bricks already served never move.
//!
//! The cursor carries (snapshot id, seed, offset). If a snapshot is evicted
//! mid-scroll, the same seed rebuilds a closely-matching wall from warm
//! caches — continuity is best-effort, determinism of jitter is exact.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use chrono::Utc;
use futures::stream::{self, StreamExt};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tokio::sync::{Mutex, Notify};
use xxhash_rust::xxh3::xxh3_64_with_seed;

use super::mix;
use crate::cache::StdDocs;
use crate::error::AppError;
use crate::model::{Author, Brick};
use crate::sources::{bluesky, standardsite};
use crate::state::AppState;

/// Sampled authors per snapshot — never fan out to the whole follow graph.
const COHORT_SIZE: usize = 100;
/// Of which: authors that yielded content in recent snapshots.
const KNOWN_ACTIVE: usize = 60;
/// First paint: respond once this many authors have answered…
const FIRST_PAINT_AUTHORS: usize = 40;
/// …or this much time has passed, whichever comes first.
const FIRST_PAINT_DEADLINE: Duration = Duration::from_secs(3);
/// Hard cap on bricks held per snapshot.
const SNAPSHOT_CAP: usize = 600;
/// Author-feed fan-out concurrency (the rate limiter is the real governor).
const FAN_OUT: usize = 16;

pub struct Snapshot {
    pub id: String,
    pub seed: u64,
    inner: Mutex<Inner>,
    progress: Notify,
}

struct Inner {
    pool: Vec<Brick>,
    wall: Vec<Brick>,
    seen: HashSet<String>,
    warming: bool,
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

pub async fn get_or_build(
    state: &Arc<AppState>,
    did: &str,
    seed: u64,
) -> Result<Arc<Snapshot>, AppError> {
    let id = snapshot_id(did, seed);
    let state_bg = Arc::clone(state);
    let did_owned = did.to_string();
    let id_key = id.clone();
    state
        .caches
        .snapshots
        .try_get_with(id_key, async move { build(state_bg, did_owned, seed, id).await })
        .await
        .map_err(|e: Arc<AppError>| match e.as_ref() {
            AppError::ActorNotFound(a) => AppError::ActorNotFound(a.clone()),
            other => AppError::Upstream(other.to_string()),
        })
}

async fn build(
    state: Arc<AppState>,
    did: String,
    seed: u64,
    id: String,
) -> Result<Arc<Snapshot>, AppError> {
    let follows = get_follows_cached(&state, &did).await?;
    let cohort = sample_cohort(&state, &did, &follows, seed).await;
    tracing::debug!(
        "snapshot {id}: {} follows, cohort of {}",
        follows.len(),
        cohort.len()
    );

    let snapshot = Arc::new(Snapshot {
        id,
        seed,
        inner: Mutex::new(Inner {
            pool: Vec::new(),
            wall: Vec::new(),
            seen: HashSet::new(),
            warming: !cohort.is_empty(),
        }),
        progress: Notify::new(),
    });

    if cohort.is_empty() {
        return Ok(snapshot);
    }

    // background fill — holds its own Arcs, outlives this request
    let fill_snapshot = Arc::clone(&snapshot);
    let fill_state = Arc::clone(&state);
    let viewer = did.clone();
    tokio::spawn(async move {
        let started = Instant::now();
        let mut answered = 0usize;
        let mut yielding_authors: Vec<String> = Vec::new();

        let mut feeds = stream::iter(cohort.into_iter().map(|author| {
            let state = Arc::clone(&fill_state);
            async move {
                let (bricks, docs) = tokio::join!(
                    author_feed_cached(&state, &author.did),
                    std_docs_cached(&state, &author),
                );
                (author, bricks, docs)
            }
        }))
        .buffer_unordered(FAN_OUT);

        while let Some((author, bricks, docs)) = feeds.next().await {
            answered += 1;
            if !bricks.is_empty() || !docs.bricks.is_empty() {
                yielding_authors.push(author.did);
            }
            {
                let mut inner = fill_snapshot.inner.lock().await;
                let inner = &mut *inner;
                // bskyPostRef suppression: the blog card wins over its
                // cross-posted skeet, whether the post came first or later
                for uri in &docs.suppressed_posts {
                    if inner.seen.insert(uri.clone()) {
                        // post not pooled yet — the insert blocks it later
                    } else {
                        inner.pool.retain(|b| b.id() != uri);
                    }
                }
                for brick in docs.bricks.iter().chain(bricks.iter()) {
                    if inner.pool.len() + inner.wall.len() >= SNAPSHOT_CAP {
                        break;
                    }
                    if inner.seen.insert(brick.id().to_string()) {
                        inner.pool.push(brick.clone());
                    }
                }
            }
            fill_snapshot.progress.notify_waiters();
        }

        {
            let mut inner = fill_snapshot.inner.lock().await;
            inner.warming = false;
        }
        fill_snapshot.progress.notify_waiters();
        tracing::debug!(
            "snapshot {} warmed: {answered} authors in {:?}",
            fill_snapshot.id,
            started.elapsed()
        );

        // remember who yielded, for the next snapshot's cohort
        record_activity(&fill_state, &viewer, yielding_authors).await;
    });

    // first-paint threshold: enough authors answered, or deadline
    let deadline = Instant::now() + FIRST_PAINT_DEADLINE;
    loop {
        {
            let inner = snapshot.inner.lock().await;
            let enough = inner.pool.len() + inner.wall.len();
            if !inner.warming || enough >= FIRST_PAINT_AUTHORS * 4 {
                break;
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        let _ = tokio::time::timeout(remaining, snapshot.progress.notified()).await;
    }

    Ok(snapshot)
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
                mix::lay(&mut inner.pool, &mut inner.wall, missing, snapshot.seed, Utc::now());
            }
            let exhausted = !inner.warming && inner.pool.is_empty();
            if inner.wall.len() >= wanted || exhausted {
                let end = wanted.min(inner.wall.len());
                let items =
                    inner.wall.get(offset.min(end)..end).map(<[Brick]>::to_vec).unwrap_or_default();
                let has_more = !inner.pool.is_empty() || inner.warming;
                return (items, has_more);
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            // serve a short page rather than hang; scroll retries
            let guard = snapshot.inner.lock().await;
            let end = (offset + size).min(guard.wall.len());
            let items =
                guard.wall.get(offset.min(end)..end).map(<[Brick]>::to_vec).unwrap_or_default();
            return (items, guard.warming || !guard.pool.is_empty());
        }
        let _ = tokio::time::timeout(remaining, snapshot.progress.notified()).await;
    }
}

async fn get_follows_cached(
    state: &Arc<AppState>,
    did: &str,
) -> Result<Arc<Vec<bluesky::Follow>>, AppError> {
    let http = &state.http;
    let base = state.config.appview_base.clone();
    let did_owned = did.to_string();
    state
        .caches
        .follows
        .try_get_with(did.to_string(), async move {
            bluesky::get_follows(http, &base, &did_owned).await.map(Arc::new)
        })
        .await
        .map_err(|e: Arc<crate::http::HttpError>| AppError::Upstream(e.to_string()))
}

async fn author_feed_cached(state: &Arc<AppState>, author_did: &str) -> Arc<Vec<Brick>> {
    let http = &state.http;
    let base = state.config.appview_base.clone();
    let did_owned = author_did.to_string();
    state
        .caches
        .author_feed
        .get_with(author_did.to_string(), async move {
            match bluesky::get_author_feed(http, &base, &did_owned).await {
                Ok(bricks) => Arc::new(bricks),
                Err(e) => {
                    // a single author failing must never sink the wall
                    tracing::debug!("author feed {did_owned} failed: {e}");
                    Arc::new(Vec::new())
                }
            }
        })
        .await
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
    let known_active = state.caches.activity.get(viewer).await.unwrap_or_default();
    let by_did: std::collections::HashMap<&str, &bluesky::Follow> =
        follows.iter().map(|f| (f.did.as_str(), f)).collect();

    let mut cohort: Vec<Author> = known_active
        .iter()
        .filter_map(|did| by_did.get(did.as_str()))
        .take(KNOWN_ACTIVE)
        .map(|f| Author::from(*f))
        .collect();

    let chosen: HashSet<&str> = cohort.iter().map(|a| a.did.as_str()).collect();
    let mut rest: Vec<&bluesky::Follow> =
        follows.iter().filter(|f| !chosen.contains(f.did.as_str())).collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    rest.shuffle(&mut rng);
    cohort.extend(
        rest.into_iter().take(COHORT_SIZE.saturating_sub(cohort.len())).map(Author::from),
    );
    cohort
}

async fn std_docs_cached(state: &Arc<AppState>, author: &Author) -> Arc<StdDocs> {
    let http = &state.http;
    let plc_base = state.config.plc_base.clone();
    let author = author.clone();
    state
        .caches
        .std_docs
        .get_with(author.did.clone(), async move {
            match standardsite::get_documents(http, &plc_base, &author).await {
                Ok(result) => Arc::new(StdDocs {
                    bricks: result.bricks,
                    suppressed_posts: result.suppressed_posts,
                }),
                Err(e) => {
                    tracing::debug!("standard.site fetch for {} failed: {e}", author.did);
                    Arc::new(StdDocs { bricks: Vec::new(), suppressed_posts: Vec::new() })
                }
            }
        })
        .await
}

async fn record_activity(state: &Arc<AppState>, viewer: &str, mut yielding: Vec<String>) {
    if yielding.is_empty() {
        return;
    }
    if let Some(previous) = state.caches.activity.get(viewer).await {
        let fresh: HashSet<String> = yielding.iter().cloned().collect();
        yielding.extend(previous.iter().filter(|d| !fresh.contains(d.as_str())).cloned());
    }
    yielding.truncate(300);
    state.caches.activity.insert(viewer.to_string(), Arc::new(yielding)).await;
}

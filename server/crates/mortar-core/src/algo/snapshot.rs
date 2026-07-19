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
use crate::cache::{
    STD_DOCS_NEGATIVE_TTL, STD_DOCS_POSITIVE_TTL, STREAMS_NEGATIVE_TTL, STREAMS_POSITIVE_TTL,
    StdDocs,
};
use crate::error::AppError;
use crate::mode::Mode;
use crate::model::{Author, Brick};
use crate::platform::{self, Instant, SystemTime};
use crate::sources::bluesky::AuthorYield;
use crate::sources::{bluesky, pds, standardsite, streamplace};
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
/// No author may hold more than this many bricks of ONE KIND in the pool. The
/// mixer's diversity window can only space authors out if the pool has other
/// authors in it.
///
/// Per kind, not per author, and the distinction is load-bearing: posts arrive
/// from a fast endpoint and blogs and streams from slow ones, so a flat
/// per-author cap is spent entirely on skeets before a prolific author's blog
/// has even been fetched, and their blog is then turned away at the door. The
/// flooding this guards against is a chatty account drowning the pool in
/// posts, and that is still exactly what it stops.
const MAX_BRICKS_PER_AUTHOR: usize = 4;
/// The glaze wall is one kind from one source, so a chatty account cannot crowd
/// out the rarer kinds (there are none) — only other image posters. It reads
/// `posts_with_media` deep, so it can afford to let a prolific image account
/// bring more of itself than the mixed wall does; the diversity window still
/// spaces them out.
const GLAZE_MAX_BRICKS_PER_AUTHOR: usize = 8;
/// How far back the glaze wall reaches. The full wall holds posts to a 72h
/// window ("what the people you follow are making, present tense"); an image
/// wall is a gallery, not a timeline, so it keeps a month of a follow's images
/// rather than three days of them.
const GLAZE_MAX_AGE_HOURS: f64 = 24.0 * 30.0;
/// …or this much time has passed, whichever comes first.
const FIRST_PAINT_DEADLINE: Duration = Duration::from_secs(3);
/// Per-slot caps on bricks held per snapshot, by mix kind (see `mix::KINDS`).
/// Posts arrive by the thousand and must not crowd out the rarer kinds.
const KIND_CAPS: [usize; mix::KINDS] = [500, 60, 30, 20, 5];
/// Author-feed fan-out concurrency (the AppView rate limiter, 10/s, is the
/// real governor here).
const FAN_OUT: usize = 16;
/// Repo-read fan-out. Higher, because these go to a hundred different PDSes
/// rather than to one rate-limited AppView, and the slowest of them must not
/// hold up the rest.
const REPO_FAN_OUT: usize = 32;
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
/// The fans that supply the rare kinds: repo reads, and the live list.
const SLOW_FANS: usize = 2;
/// How long the FIRST page will wait for those two before laying anyway,
/// measured from when the SNAPSHOT was created, not from when the page request
/// arrives. The two waits used to stack: `get_or_build` blocked up to
/// `FIRST_PAINT_DEADLINE` for a dozen authors, and only then did `get_page`
/// start this clock, so a cold wall could stare at skeletons for the sum of the
/// two. Anchoring to creation folds the first-paint wait inside this budget: a
/// wall of nothing but posts is not mason, but neither is one that never
/// arrives, and this bounds the whole opening wait to `MIX_DEADLINE`.
const MIX_DEADLINE: Duration = Duration::from_secs(6);

pub struct Snapshot {
    pub id: String,
    pub seed: u64,
    /// When this snapshot was created, so the first page's mix wait can be
    /// bounded from the moment the fill began rather than restarted when the
    /// page request lands.
    created: Instant,
    inner: Mutex<Inner>,
    progress: Notify,
}

impl Snapshot {
    fn new(id: String, seed: u64, mode: Mode) -> Self {
        // glaze reads a single source (Bluesky posts): there are no rare-kind
        // fans to wait for, so its first page never defers laying for a better
        // mix. The full wall waits for the two slow fans (repos and live).
        let (slow_fans, max_age_hours, max_per_author) = match mode {
            Mode::Wall => (SLOW_FANS, None, MAX_BRICKS_PER_AUTHOR),
            Mode::Glaze => (0, Some(GLAZE_MAX_AGE_HOURS), GLAZE_MAX_BRICKS_PER_AUTHOR),
        };
        Self {
            id,
            seed,
            created: Instant::now(),
            inner: Mutex::new(Inner {
                pool: Vec::new(),
                wall: Vec::new(),
                seen: HashSet::new(),
                kind_counts: [0; mix::KINDS],
                author_counts: HashMap::new(),
                warming: true,
                slow_fans,
                max_age_hours,
                max_per_author,
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
    kind_counts: [usize; mix::KINDS],
    /// bricks admitted per (author, kind), checked against
    /// MAX_BRICKS_PER_AUTHOR
    author_counts: HashMap<(String, usize), usize>,
    warming: bool,
    /// Fans that supply the rare kinds and have not finished: the repo reads
    /// (every blog, every archived stream) and the live list. Posts arrive
    /// from one fast endpoint and always win the race, so a wall laid the
    /// instant it *could* be laid is a wall of nothing but posts. The first
    /// page waits, briefly, for these.
    slow_fans: usize,
    /// Admission window in hours. `None` uses the per-kind default
    /// (`score::is_fresh`); the glaze wall sets it to reach further back.
    max_age_hours: Option<f64>,
    /// How many bricks of one kind a single author may hold in the pool.
    max_per_author: usize,
}

impl Inner {
    /// Insert into the pool unless it is a duplicate, stale, over its kind's
    /// cap, or its author already holds their share.
    fn admit(&mut self, brick: &Brick, now: chrono::DateTime<Utc>) {
        let fresh = match self.max_age_hours {
            Some(max) => score::within_age(brick, now, max),
            None => score::is_fresh(brick, now),
        };
        if !fresh {
            return;
        }
        let slot = mix::kind_index(brick);
        if self.kind_counts[slot] >= KIND_CAPS[slot] {
            return;
        }
        let author = score::author_key(brick).to_string();
        let held = self.author_counts.entry((author, slot)).or_insert(0);
        if *held >= self.max_per_author {
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
        self.author_counts
            .iter()
            .filter(|(_, held)| **held > 0)
            .map(|((did, _), _)| did.as_str())
            .collect::<HashSet<_>>()
            .len()
    }
}

pub fn snapshot_id(did: &str, seed: u64, mode: Mode) -> String {
    // the mode tag namespaces the id so a glaze wall and a full wall for the
    // same actor and seed can never collide in the snapshot cache.
    format!(
        "{}-{:016x}",
        mode.tag(),
        xxh3_64_with_seed(did.as_bytes(), seed)
    )
}

pub fn fresh_seed(did: &str) -> u64 {
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    xxh3_64_with_seed(did.as_bytes(), millis)
}

/// Fetch-or-create under one cache lock: exactly one caller wins the insert and
/// spawns the background fill; everyone gets the same Arc. No waiting — the
/// caller decides whether to block for first paint. The preview loop wants this
/// bare: it returns the current pool the instant it is asked, however thin.
pub async fn ensure_snapshot(
    state: &Arc<AppState>,
    did: &str,
    seed: u64,
    mode: Mode,
) -> Arc<Snapshot> {
    let id = snapshot_id(did, seed, mode);
    let (snapshot, inserted) = state
        .caches
        .snapshots
        .get_or_insert_with(id.clone(), || {
            Arc::new(Snapshot::new(id.clone(), seed, mode))
        })
        .await;

    if inserted {
        let fill_state = Arc::clone(state);
        let fill_snapshot = Arc::clone(&snapshot);
        let viewer = did.to_string();
        platform::spawn(async move {
            fill(fill_state, fill_snapshot, viewer, seed, mode).await;
        });
    }

    snapshot
}

/// `ensure_snapshot`, then block until the first-paint threshold (enough
/// distinct authors pooled, or the deadline). A no-op once warm. The committing
/// paths use this so a page is never laid from an empty pool.
pub async fn get_or_build(
    state: &Arc<AppState>,
    did: &str,
    seed: u64,
    mode: Mode,
) -> Arc<Snapshot> {
    let snapshot = ensure_snapshot(state, did, seed, mode).await;

    // first-paint threshold: enough bricks pooled, or deadline
    let deadline = Instant::now() + FIRST_PAINT_DEADLINE;
    loop {
        // registered BEFORE the state check: Notify only wakes futures that
        // exist (and are enabled) when it fires, so a notify slipping in
        // between the lock release and the await would otherwise be lost and
        // stall first paint until the deadline
        let mut notified = std::pin::pin!(snapshot.progress.notified());
        notified.as_mut().enable();
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
        let _ = platform::timeout(remaining, notified).await;
    }

    snapshot
}

/// The current best first screen, laid from a CLONE of the pool so nothing is
/// committed. Polled while a wall warms, it reflows as the pool grows: the same
/// pool and seed always yield the same arrangement (the mixer is pure), so the
/// screen only moves when new bricks actually arrive. Returns the bricks and
/// whether the wall is still warming.
///
/// Only meaningful before the wall is frozen, which is the only time it is
/// called: at that point nothing has been laid, so laying `size` from the pool
/// clone is the whole first screen. The real pool is never touched, so a
/// preview can never race the commit that follows it.
pub async fn preview_page(snapshot: &Snapshot, size: usize) -> (Vec<Brick>, bool) {
    let inner = snapshot.inner.lock().await;
    let mut pool = inner.pool.clone();
    let mut wall = Vec::new();
    mix::lay(&mut pool, &mut wall, size, snapshot.seed, Utc::now());
    (wall, inner.warming)
}

/// The background fill: follows → cohort fan-out, with the live list running
/// concurrently → warming off. Any follow-graph failure leaves an empty (but
/// terminated) snapshot rather than an error; actor existence was already
/// checked by resolve.
async fn fill(
    state: Arc<AppState>,
    snapshot: Arc<Snapshot>,
    viewer: String,
    seed: u64,
    mode: Mode,
) {
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
    // per-viewer activity is namespaced by mode, so browsing the image wall
    // never reshapes the full wall's known-active cohort, and vice versa.
    let activity_key = activity_key(&viewer, mode);
    let cohort = sample_cohort(&state, &activity_key, &follows, seed).await;
    tracing::debug!(
        "snapshot {}: {} follows, cohort of {}",
        snapshot.id,
        follows.len(),
        cohort.len()
    );

    let (answered, yielding_authors) = match mode {
        // The image wall reads one source and admits one kind. It leans on the
        // shared author-feed cache (moderation and `!warn` blur are applied
        // there, so glaze inherits them), keeps only image posts, and skips the
        // repo reads and the live list entirely.
        Mode::Glaze => {
            fan_out_authors(&state, &snapshot, &cohort, true, Brick::is_image_post).await
        }

        // The full wall: posts from the AppView, blogs and archived streams from
        // a hundred PDSes, and the live list, all fanned out at once.
        Mode::Wall => {
            // Who is live is one call for the whole network, and it does not
            // depend on the cohort: a friend streaming right now belongs on the
            // wall whether or not this snapshot's random sample happened to pick
            // them. It runs alongside the fan-out so it lands in time for the
            // first paint.
            let live_fill = async {
                let bricks = live_bricks(&state, &follows).await;
                if !bricks.is_empty() {
                    tracing::debug!("{} of the follow graph is live", bricks.len());
                    let now = Utc::now();
                    let mut inner = snapshot.inner.lock().await;
                    for brick in bricks.iter() {
                        inner.admit(brick, now);
                    }
                }
                finish_slow_fan(&snapshot).await;
            };

            // Posts and repo reads are fanned out SEPARATELY, and this is the
            // whole reason a cold wall paints at all.
            //
            // They used to share one task per author, which meant an author's
            // posts were not admitted until plc.directory and two PDS
            // listRecords had also answered for them. Posts are 68% of the wall
            // and come from one fast, rate-limited endpoint; blogs and archived
            // streams are a handful of bricks and come from a hundred different
            // PDSes at a hundred different speeds. Coupling them held the fast
            // source hostage to the slow one: a 100-author fill took 17s, so a
            // viewer with a large follow graph got an EMPTY first page and had
            // to wait for a second request to see anything. Split, the posts
            // land at AppView speed and the rest catches up behind them.
            let posts_fill = fan_out_authors(&state, &snapshot, &cohort, false, |_| true);

            let repos_fill = async {
                let mut repos = stream::iter(cohort.iter().cloned().map(|author| {
                    let state = Arc::clone(&state);
                    async move {
                        // blogs and archived streams both read the author's repo,
                        // so they share one identity lookup rather than racing
                        // for two
                        let Some(pds) = pds_cached(&state, &author.did).await else {
                            return (author, Arc::new(StdDocs::default()), Arc::new(Vec::new()));
                        };
                        let (docs, streams) = tokio::join!(
                            std_docs_cached(&state, &pds, &author),
                            streams_cached(&state, &pds, &author),
                        );
                        (author, docs, streams)
                    }
                }))
                .buffer_unordered(REPO_FAN_OUT);

                let mut yielding: Vec<String> = Vec::new();
                while let Some((author, docs, streams)) = repos.next().await {
                    if !docs.bricks.is_empty() || !streams.is_empty() {
                        yielding.push(author.did);
                    }
                    {
                        let mut inner = snapshot.inner.lock().await;
                        let inner = &mut *inner;
                        // bskyPostRef suppression: the blog card wins over its
                        // cross-posted skeet, whether the post came first or
                        // later. Now that posts race ahead, "later" is the
                        // common case: the skeet is usually already pooled, and
                        // gets withdrawn here.
                        for uri in &docs.suppressed_posts {
                            if inner.seen.insert(uri.clone()) {
                                // post not pooled yet; the insert blocks it later
                            } else {
                                inner.pool.retain(|b| b.id() != uri);
                            }
                        }
                        let now = Utc::now();
                        for brick in docs.bricks.iter().chain(streams.iter()) {
                            inner.admit(brick, now);
                        }
                    }
                    snapshot.progress.notify_waiters();
                }
                finish_slow_fan(&snapshot).await;
                yielding
            };

            let ((answered, mut yielding), repo_authors, ()) =
                futures::join!(posts_fill, repos_fill, live_fill);
            yielding.extend(repo_authors);
            (answered, yielding)
        }
    };

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

    // remember who yielded, for the next snapshot's cohort (mode-namespaced)
    record_activity(&state, &activity_key, yielding_authors).await;
}

/// Namespace a viewer's activity by mode. The full wall keeps the bare did (so
/// entries persisted before glaze existed still match), while glaze prefixes it,
/// keeping the two walls' known-active cohorts from bleeding into each other.
fn activity_key(viewer: &str, mode: Mode) -> String {
    match mode {
        Mode::Wall => viewer.to_string(),
        Mode::Glaze => format!("{}:{viewer}", mode.tag()),
    }
}

/// Fan out author feeds across the cohort, admitting the bricks that pass
/// `keep`. The full wall keeps everything; glaze keeps only image posts. Returns
/// (authors answered, authors that yielded at least one kept brick) so the
/// caller can warm-start the next cohort with them.
async fn fan_out_authors(
    state: &Arc<AppState>,
    snapshot: &Arc<Snapshot>,
    cohort: &[Author],
    deep_media: bool,
    keep: impl Fn(&Brick) -> bool,
) -> (usize, Vec<String>) {
    let mut feeds = stream::iter(cohort.iter().cloned().map(|author| {
        let state = Arc::clone(state);
        async move {
            // glaze reads the author's media deep (posts_with_media); the full
            // wall skims their last thirty posts. Separate caches, so neither
            // read clobbers the other's.
            let yield_ = if deep_media {
                image_feed_cached(&state, &author.did).await
            } else {
                author_feed_cached(&state, &author.did).await
            };
            (author, yield_)
        }
    }))
    .buffer_unordered(FAN_OUT);

    let mut answered = 0usize;
    let mut yielding: Vec<String> = Vec::new();
    while let Some((author, yield_)) = feeds.next().await {
        answered += 1;
        let mut kept_any = false;
        {
            let now = Utc::now();
            let mut inner = snapshot.inner.lock().await;
            for brick in yield_.bricks.iter().filter(|b| keep(b)) {
                kept_any = true;
                inner.admit(brick, now);
            }
        }
        if kept_any {
            yielding.push(author.did);
        }
        snapshot.progress.notify_waiters();
    }
    (answered, yielding)
}

/// One of the two rare-kind fans has finished; a page waiting for a mixed pool
/// has one less thing to wait for.
async fn finish_slow_fan(snapshot: &Snapshot) {
    {
        let mut inner = snapshot.inner.lock().await;
        inner.slow_fans = inner.slow_fans.saturating_sub(1);
    }
    snapshot.progress.notify_waiters();
}

/// Streams end. A live brick is admitted once, during the fill, and then waits
/// in the pool for the snapshot's whole half-hour life; without this, a stream
/// that finished twenty minutes ago would still be laid with a LIVE badge and
/// a playlist that 404s.
///
/// Only bricks still in the pool can be withdrawn. Ones already laid stay
/// where they are, because the wall never moves; the player is the last line
/// of defence for those, and says so.
async fn drop_ended_streams(state: &Arc<AppState>, snapshot: &Snapshot) {
    {
        // the overwhelming majority of walls have no live brick at all, and
        // must not pay anything to discover that
        let inner = snapshot.inner.lock().await;
        if !inner.pool.iter().any(score::is_live) {
            return;
        }
    }

    // Prune against a live list we ALREADY hold. Fetching one here would put a
    // network round trip in the middle of somebody's scroll every time the
    // sixty-second cache lapsed, to answer a question that only matters to the
    // rare wall with a live brick still in its pool. Refresh it for the next
    // page instead, and let this one through.
    let Some(network) = state.caches.live.get(&0u8).await else {
        let refresh = Arc::clone(state);
        platform::spawn(async move {
            let _ = live_cached(&refresh).await;
        });
        return;
    };
    let still_live: HashSet<&str> = network.iter().map(|s| s.uri()).collect();

    let mut inner = snapshot.inner.lock().await;
    let before = inner.pool.len();
    inner
        .pool
        .retain(|brick| !score::is_live(brick) || still_live.contains(brick.id()));
    let dropped = before - inner.pool.len();
    if dropped > 0 {
        tracing::debug!("snapshot {}: {dropped} stream(s) ended", snapshot.id);
    }
    // kind_counts and author_counts are admission budgets, not a census, and
    // are deliberately left alone: an ended stream does not buy its author a
    // fresh slot on a wall that has already been laid around them.
}

/// Serve one page, laying new bricks from the pool as needed. Waits briefly
/// while warming if the wall is still too short.
pub async fn get_page(
    state: &Arc<AppState>,
    snapshot: &Snapshot,
    offset: usize,
    size: usize,
    wait_for_mix: bool,
) -> (Vec<Brick>, bool) {
    drop_ended_streams(state, snapshot).await;
    let started = Instant::now();
    let deadline = started + Duration::from_secs(8);
    // the offset rides in on the cursor, and a cursor is attacker-writable:
    // an offset near usize::MAX would overflow the addition below. Treat it
    // as the end of the feed rather than panic (debug) or wrap (release).
    let Some(wanted) = offset.checked_add(size) else {
        return (Vec::new(), false);
    };
    // anchored to snapshot creation, so the first-paint wait already spent in
    // get_or_build counts against this budget rather than stacking on top of it
    let mix_deadline = snapshot.created + MIX_DEADLINE;
    loop {
        let awaiting_mix;
        // registered before the state check, same as get_or_build: a brick
        // admitted between the lock release and the await must still wake
        // this page, or it serves short after a full deadline of nothing
        let mut notified = std::pin::pin!(snapshot.progress.notified());
        notified.as_mut().enable();
        {
            let mut guard = snapshot.inner.lock().await;
            let inner = &mut *guard;

            // Bricks are laid once and never move, so laying is the moment the
            // pool's composition becomes the wall's composition. Posts arrive
            // from one fast endpoint and blogs and streams from a hundred slow
            // ones, so laying the instant 24 bricks exist would freeze a FIRST
            // wall of pure posts before a single blog had a chance to arrive.
            //
            // Only the first page. Nobody is watching a blank screen decide
            // what it wants to be on page four; they are mid-scroll, they have
            // hit the bottom, and they are waiting. A snapshot rebuilt after
            // the service worker was reaped mid-scroll is warming all over
            // again, and making that reader wait six seconds for a better
            // blog-to-post ratio is a bad trade every time.
            // …unless the caller is a freeze, which never waits: the client's
            // preview loop already served the warming reflow, so re-paying the
            // mix wait here is the exact stall reflow exists to remove.
            awaiting_mix = wait_for_mix
                && offset == 0
                && inner.warming
                && inner.slow_fans > 0
                && Instant::now() < mix_deadline;

            if !awaiting_mix {
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
        }
        // wake on the next brick, or on whichever deadline is next: the mix
        // deadline only defers laying, the hard one ends the page
        let wake = if awaiting_mix { mix_deadline } else { deadline };
        let remaining = wake.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            if awaiting_mix {
                continue; // waited long enough for the rare kinds; lay anyway
            }
            // serve a short page rather than hang; scroll retries. One last
            // lay first: bricks that arrived during the wait belong on it
            let mut guard = snapshot.inner.lock().await;
            let inner = &mut *guard;
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
            let end = wanted.min(inner.wall.len());
            let items = inner
                .wall
                .get(offset.min(end)..end)
                .map(<[Brick]>::to_vec)
                .unwrap_or_default();
            return (items, inner.warming || !inner.pool.is_empty());
        }
        let _ = platform::timeout(remaining, notified).await;
    }
}

/// The follow graph, but only as much of it as a waiting person can justify.
///
/// Follows page 100 at a time and each page is a round trip that blocks the
/// next, so a 2000-follow graph costs twenty sequential requests: ten seconds
/// in which not one post has been fetched, and a wall that arrives empty. The
/// cohort samples 100 authors regardless, so a few hundred follows is plenty
/// to build the first wall out of. The rest is fetched behind the user's back
/// and cached, so their NEXT wall samples the whole graph.
async fn get_follows_cached(
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
                Arc::new(AuthorYield { bricks: Vec::new() })
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
async fn image_feed_cached(state: &Arc<AppState>, author_did: &str) -> Arc<AuthorYield> {
    if let Some(cached) = state.caches.image_feed.get(&author_did.to_string()).await {
        return cached;
    }
    let yield_ =
        match bluesky::get_image_feed(&state.http, &state.config.appview_base, author_did).await {
            Ok(yield_) => Arc::new(yield_),
            Err(e) => {
                tracing::debug!("image feed {author_did} failed: {e}");
                Arc::new(AuthorYield { bricks: Vec::new() })
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
async fn live_cached(state: &Arc<AppState>) -> Arc<Vec<streamplace::LiveStream>> {
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
async fn live_bricks(state: &Arc<AppState>, follows: &[bluesky::Follow]) -> Vec<Brick> {
    let network = live_cached(state).await;
    let mut bricks = Vec::new();
    for stream in followed_live(&network, follows) {
        // only now, for the handful that survive the filter, is it worth
        // finding out where their repo (and so their poster) lives
        let pds = pds_cached(state, stream.did()).await;
        bricks.push(
            stream
                .clone()
                .into_brick(&state.config.streamplace_base, pds.as_deref()),
        );
    }
    bricks
}

/// Where an author's repo lives. Cached for a day: identity moves rarely, and
/// both the blog and the stream reader need the answer for every author.
async fn pds_cached(state: &Arc<AppState>, did: &str) -> Option<String> {
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
async fn streams_cached(state: &Arc<AppState>, pds: &str, author: &Author) -> Arc<Vec<Brick>> {
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

/// Cohort: up to KNOWN_ACTIVE authors that yielded content before, topped up
/// with a seeded-random sample of the rest so refreshes rotate through the
/// whole follow graph.
async fn sample_cohort(
    state: &Arc<AppState>,
    activity_key: &str,
    follows: &[bluesky::Follow],
    seed: u64,
) -> Vec<Author> {
    let known_active = state
        .caches
        .activity
        .get(&activity_key.to_string())
        .await
        .unwrap_or_default();
    // Hidden follows never enter the cohort, so none of their content is
    // fetched: not posts, and not the blogs and archived streams that the
    // author-feed label filter cannot see. This is the single choke point that
    // keeps a logged-out opt-out (and an adult-labelled account) off every
    // source at once.
    let by_did: std::collections::HashMap<&str, &bluesky::Follow> = follows
        .iter()
        .filter(|f| !f.hidden())
        .map(|f| (f.did.as_str(), f))
        .collect();

    let mut cohort: Vec<Author> = known_active
        .iter()
        .filter_map(|did| by_did.get(did.as_str()))
        .take(KNOWN_ACTIVE)
        .map(|f| Author::from(*f))
        .collect();

    let chosen: HashSet<&str> = cohort.iter().map(|a| a.did.as_str()).collect();
    let mut rest: Vec<&bluesky::Follow> = follows
        .iter()
        .filter(|f| !f.hidden() && !chosen.contains(f.did.as_str()))
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

async fn std_docs_cached(state: &Arc<AppState>, pds: &str, author: &Author) -> Arc<StdDocs> {
    if let Some(cached) = state.caches.std_docs.get(&author.did).await {
        return cached;
    }
    let docs = match standardsite::get_documents(&state.http, pds, author).await {
        Ok(result) => Arc::new(StdDocs {
            bricks: result.bricks,
            suppressed_posts: result.suppressed_posts,
        }),
        Err(e) => {
            // same as streams: a transient failure must not be remembered for
            // a day as "this author publishes nothing". Skip the insert; only
            // a successful empty listing earns the negative TTL.
            tracing::debug!("standard.site fetch for {} failed: {e}", author.did);
            return Arc::new(StdDocs::default());
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

async fn record_activity(state: &Arc<AppState>, activity_key: &str, mut yielding: Vec<String>) {
    if yielding.is_empty() {
        return;
    }
    // an author who yielded both posts and a blog is reported by both fans;
    // a duplicate here would waste one of the next cohort's known-active slots
    let mut once = HashSet::new();
    yielding.retain(|did| once.insert(did.clone()));
    if let Some(previous) = state.caches.activity.get(&activity_key.to_string()).await {
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
        .insert(activity_key.to_string(), Arc::new(yielding))
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Author, PostBrick, VideoBrick, VideoSource};

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
            blur: None,
        })
    }

    fn blog(id: usize, author: usize) -> Brick {
        Brick::Blog(crate::model::BlogBrick {
            id: format!("blog-{id}"),
            url: String::new(),
            author: Author {
                did: format!("did:plc:a{author}"),
                handle: format!("a{author}.test"),
                display_name: None,
                avatar: None,
            },
            title: "t".into(),
            description: None,
            cover_image: None,
            publication: crate::model::Publication {
                name: "p".into(),
                url: String::new(),
                icon: None,
            },
            tags: vec![],
            published_at: Utc::now().to_rfc3339(),
        })
    }

    fn live_stream(did: &str) -> streamplace::LiveStream {
        streamplace::LiveStream::for_test(did)
    }

    fn inner() -> Inner {
        Inner {
            pool: Vec::new(),
            wall: Vec::new(),
            seen: HashSet::new(),
            kind_counts: [0; mix::KINDS],
            author_counts: HashMap::new(),
            warming: true,
            slow_fans: SLOW_FANS,
            max_age_hours: None,
            max_per_author: MAX_BRICKS_PER_AUTHOR,
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

    /// Posts reach the pool seconds before blogs and archived streams do, from
    /// a faster endpoint. A per-author cap is therefore always spent on posts
    /// first, and a flat one would turn a prolific blogger's own blog away
    /// from their own wall. The cap is per author PER KIND for exactly this
    /// reason.
    #[test]
    fn a_prolific_poster_can_still_bring_a_blog() {
        let mut i = inner();
        let now = Utc::now();
        for n in 0..10 {
            i.admit(&post(n, 0, 1), now);
        }
        assert_eq!(i.pool.len(), MAX_BRICKS_PER_AUTHOR, "posts are capped");

        i.admit(&blog(100, 0), now);
        assert_eq!(
            i.pool.len(),
            MAX_BRICKS_PER_AUTHOR + 1,
            "the blog was turned away by a quota its author's own skeets had eaten"
        );
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

    /// A glaze snapshot has no slow fans to wait for (it reads one source), so
    /// its first page never defers laying; a full-wall snapshot waits for the
    /// two rare-kind fans.
    #[tokio::test]
    async fn glaze_waits_for_no_slow_fans() {
        let wall = Snapshot::new("w".into(), 1, Mode::Wall);
        let glaze = Snapshot::new("g".into(), 1, Mode::Glaze);
        assert_eq!(wall.inner.lock().await.slow_fans, SLOW_FANS);
        assert_eq!(glaze.inner.lock().await.slow_fans, 0);
    }

    /// The glaze wall reaches further back and lets one author bring more of
    /// itself: a two-week-old image is past the full wall's 72h window but stays
    /// on glaze, and a prolific poster contributes up to the glaze cap, not the
    /// full wall's four.
    #[tokio::test]
    async fn glaze_reaches_further_back_and_lets_authors_bring_more() {
        let glaze = Snapshot::new("g".into(), 1, Mode::Glaze);
        let mut inner = glaze.inner.lock().await;
        let now = Utc::now();

        inner.admit(&post(1, 0, 24 * 14), now); // 14 days old
        assert_eq!(
            inner.pool.len(),
            1,
            "an image well past the full wall's 72h window stays on glaze"
        );

        for n in 2..20 {
            inner.admit(&post(n, 0, 1), now);
        }
        assert_eq!(
            inner.pool.len(),
            GLAZE_MAX_BRICKS_PER_AUTHOR,
            "one author fills up to the glaze cap, not the full wall's four"
        );
    }

    /// The mode is folded into the snapshot id, so a glaze wall and a full wall
    /// for the same actor and seed never collide in the snapshot cache.
    #[test]
    fn snapshot_id_separates_the_two_walls() {
        let wall = snapshot_id("did:plc:aa", 7, Mode::Wall);
        let glaze = snapshot_id("did:plc:aa", 7, Mode::Glaze);
        assert_ne!(wall, glaze);
        assert_eq!(
            wall,
            snapshot_id("did:plc:aa", 7, Mode::Wall),
            "stable per mode"
        );
    }

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

    /// The cohort is where posts, blogs and archived streams are all fanned out
    /// from, so dropping an opted-out author here is what keeps their non-post
    /// bricks off the wall as well.
    #[tokio::test]
    async fn the_cohort_excludes_opted_out_follows() {
        let state = Arc::new(AppState::new(crate::config::Config::default()));
        let follows = vec![follow("did:plc:open"), opted_out_follow("did:plc:sealed")];
        let cohort = sample_cohort(&state, "did:plc:viewer", &follows, 1).await;
        let dids: Vec<&str> = cohort.iter().map(|a| a.did.as_str()).collect();
        assert_eq!(
            dids,
            vec!["did:plc:open"],
            "the opted-out author must be sampled out"
        );
    }

    /// An account labelled adult is kept off a logged-out wall whole, the same
    /// way an opted-out one is: dropped from the cohort before any of its
    /// content is fetched.
    #[tokio::test]
    async fn the_cohort_excludes_adult_accounts() {
        let state = Arc::new(AppState::new(crate::config::Config::default()));
        let mut adult = follow("did:plc:adult");
        adult.labels = serde_json::from_value(serde_json::json!([{"val": "porn"}])).unwrap();
        let follows = vec![follow("did:plc:open"), adult];
        let cohort = sample_cohort(&state, "did:plc:viewer", &follows, 1).await;
        let dids: Vec<&str> = cohort.iter().map(|a| a.did.as_str()).collect();
        assert_eq!(
            dids,
            vec!["did:plc:open"],
            "the adult account must be sampled out"
        );
    }

    fn live_brick(uri: &str, did: &str) -> Brick {
        Brick::Video(VideoBrick {
            id: uri.into(),
            url: String::new(),
            author: Author {
                did: did.into(),
                handle: format!("{did}.test"),
                display_name: None,
                avatar: None,
            },
            title: "on air".into(),
            poster: None,
            playlist: String::new(),
            aspect_ratio: None,
            source: VideoSource::Streamplace,
            created_at: Utc::now().to_rfc3339(),
            like_count: 0,
            live: true,
            viewer_count: Some(2),
            duration_ms: None,
            activity: None,
            blur: None,
        })
    }

    /// The first page waits for the slow sources so the wall does not open on
    /// nothing but skeets. A LATER page must not: the reader is mid-scroll and
    /// has hit the bottom. This bites hardest in the service worker, which is
    /// reaped after ~30s idle: pausing a scroll kills it, and the next page
    /// rebuilds a warming snapshot from the cursor, so every deep page would
    /// pay the first-paint tax again.
    #[tokio::test]
    async fn a_later_page_never_waits_for_the_mix() {
        let state = Arc::new(AppState::new(crate::config::Config::default()));
        let snapshot = Snapshot::new("s".into(), 1, Mode::Wall);
        {
            let mut inner = snapshot.inner.lock().await;
            let now = Utc::now();
            for n in 0..60 {
                inner.admit(&post(n, n % 15, 1), now);
            }
            assert!(
                inner.warming && inner.slow_fans > 0,
                "the snapshot must still be warming for this test to mean anything"
            );
        }

        let began = Instant::now();
        let (items, _) = get_page(&state, &snapshot, 24, 24, true).await;
        assert_eq!(items.len(), 24);
        assert!(
            began.elapsed() < Duration::from_secs(1),
            "a scrolling reader waited {:?} for a better blog-to-post ratio",
            began.elapsed()
        );
    }

    /// A preview lays the first screen from a CLONE of the pool: the same pool
    /// gives the same screen (the mixer is pure), and the real pool is never
    /// spent, so the freeze that follows still has every brick to lay.
    #[tokio::test]
    async fn a_preview_lays_without_spending_the_pool() {
        let snapshot = Snapshot::new("s".into(), 1, Mode::Wall);
        {
            let mut inner = snapshot.inner.lock().await;
            let now = Utc::now();
            for n in 0..60 {
                inner.admit(&post(n, n % 15, 1), now);
            }
        }
        let pooled = snapshot.inner.lock().await.pool.len();

        let (first, warming) = preview_page(&snapshot, 24).await;
        assert_eq!(first.len(), 24);
        assert!(warming, "a warming snapshot reports itself warming");

        let (again, _) = preview_page(&snapshot, 24).await;
        let ids = |w: &[Brick]| w.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        assert_eq!(ids(&first), ids(&again), "same pool, same preview");
        assert_eq!(
            snapshot.inner.lock().await.pool.len(),
            pooled,
            "the preview must not spend the real pool"
        );
    }

    /// The whole point of the reflow: a brick arriving mid-warm changes the
    /// first screen. A blog joining a wall of nothing but posts is pulled onto
    /// the screen by the mixer's need factor, so the arrangement moves.
    #[tokio::test]
    async fn a_preview_reflows_when_a_brick_arrives() {
        let snapshot = Snapshot::new("s".into(), 3, Mode::Wall);
        {
            let mut inner = snapshot.inner.lock().await;
            let now = Utc::now();
            for n in 0..24 {
                inner.admit(&post(n, n % 12, 1), now);
            }
        }
        let (before, _) = preview_page(&snapshot, 24).await;
        {
            let mut inner = snapshot.inner.lock().await;
            inner.admit(&blog(500, 20), Utc::now());
        }
        let (after, _) = preview_page(&snapshot, 24).await;

        let ids = |w: &[Brick]| w.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        assert_ne!(
            ids(&before),
            ids(&after),
            "a new brick should reflow the screen"
        );
        assert!(
            after.iter().any(|b| b.id() == "blog-500"),
            "the blog the wall was starved of should join the first screen"
        );
    }

    /// A freeze commits the first page at once even while the snapshot is warming
    /// with slow fans still out: the preview loop already served the reader the
    /// warming reflow, so `wait_for_mix = false` must not defer laying.
    #[tokio::test]
    async fn a_freeze_commits_the_first_page_without_waiting() {
        let state = Arc::new(AppState::new(crate::config::Config::default()));
        let snapshot = Snapshot::new("s".into(), 1, Mode::Wall);
        {
            let mut inner = snapshot.inner.lock().await;
            let now = Utc::now();
            for n in 0..60 {
                inner.admit(&post(n, n % 15, 1), now);
            }
            assert!(
                inner.warming && inner.slow_fans > 0,
                "the snapshot must still be warming for this test to mean anything"
            );
        }

        let began = Instant::now();
        let (items, _) = get_page(&state, &snapshot, 0, 24, false).await;
        assert_eq!(items.len(), 24);
        assert!(
            began.elapsed() < Duration::from_secs(1),
            "a freeze waited {:?} for the mix instead of committing at once",
            began.elapsed()
        );
    }

    /// Streams end. A live brick is admitted once and then waits in the pool
    /// for half an hour; laying it after its stream has finished puts a LIVE
    /// badge on a dead playlist.
    #[tokio::test]
    async fn an_ended_stream_is_withdrawn_from_the_pool() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/place.stream.live.getLiveUsers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "streams": [{
                    "uri": "at://did:plc:still/place.stream.livestream/3on",
                    "author": { "did": "did:plc:still", "handle": "still.test" },
                    "record": { "title": "on air", "createdAt": "2026-07-12T12:00:00Z" }
                }]
            })))
            .mount(&server)
            .await;

        let state = Arc::new(AppState::new(crate::config::Config {
            streamplace_base: server.uri(),
            ..Default::default()
        }));
        let snapshot = Snapshot::new("s".into(), 1, Mode::Wall);
        let now = Utc::now();
        {
            let mut inner = snapshot.inner.lock().await;
            inner.admit(
                &live_brick(
                    "at://did:plc:still/place.stream.livestream/3on",
                    "did:plc:still",
                ),
                now,
            );
            inner.admit(
                &live_brick(
                    "at://did:plc:gone/place.stream.livestream/3off",
                    "did:plc:gone",
                ),
                now,
            );
            inner.admit(&post(1, 9, 1), now);
            assert_eq!(inner.pool.len(), 3);
        }

        // the fill populates this; the prune reads it rather than fetching one
        // mid-scroll
        live_cached(&state).await;
        drop_ended_streams(&state, &snapshot).await;

        let inner = snapshot.inner.lock().await;
        let ids: Vec<&str> = inner.pool.iter().map(Brick::id).collect();
        assert_eq!(
            ids,
            vec!["at://did:plc:still/place.stream.livestream/3on", "post-1"],
            "the ended stream must go, and nothing else with it"
        );
    }

    /// A livestream record is written once and reused for months. The wall
    /// admits it on the strength of the stream being live, not on the age of
    /// the record; the age test would throw away the best brick on the wall.
    #[test]
    fn a_live_stream_is_admitted_however_old_its_record_is() {
        let mut i = inner();
        let now = Utc::now();

        let ancient = Brick::Video(VideoBrick {
            id: "live-1".into(),
            url: String::new(),
            author: Author {
                did: "did:plc:streamer".into(),
                handle: "streamer.test".into(),
                display_name: None,
                avatar: None,
            },
            title: "still going".into(),
            poster: None,
            playlist: String::new(),
            aspect_ratio: None,
            source: VideoSource::Streamplace,
            created_at: (now - chrono::TimeDelta::days(120)).to_rfc3339(),
            like_count: 0,
            live: true,
            viewer_count: Some(4),
            duration_ms: None,
            activity: None,
            blur: None,
        });
        i.admit(&ancient, now);
        assert_eq!(i.pool.len(), 1, "a live stream aged out of its own wall");

        // the very same timestamp, no longer live, is past the 90-day window
        // for an archived stream and drops. `live` is the whole difference.
        let Brick::Video(mut ended) = ancient else {
            unreachable!()
        };
        ended.live = false;
        ended.id = "vod-1".into();
        i.admit(&Brick::Video(ended), now);
        assert_eq!(i.pool.len(), 1, "a 120-day archived stream is not fresh");
    }
}

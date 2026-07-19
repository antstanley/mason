//! The background fill: everything that pours bricks into a warming snapshot.
//! Follows in, cohort fanned out across the sources (author feeds, repo reads,
//! the live list), activity recorded for the next wall. Orchestration only:
//! this module never takes the snapshot's inner mutex; every pool mutation
//! goes through `Snapshot`'s admission methods.

use std::sync::Arc;

use futures::stream::{self, StreamExt};

use super::cohort;
use super::snapshot::Snapshot;
use crate::mode::Mode;
use crate::model::{Author, Brick};
use crate::platform::Instant;
use crate::sources::{StdDocs, fetch};
use crate::state::AppState;

/// Author-feed fan-out concurrency (the AppView rate limiter, 10/s, is the
/// real governor here).
const FAN_OUT: usize = 16;

/// The background fill: follows → cohort fan-out, with the live list running
/// concurrently → warming off. Any follow-graph failure leaves an empty (but
/// terminated) snapshot rather than an error; actor existence was already
/// checked by resolve.
pub async fn fill(state: Arc<AppState>, snapshot: Arc<Snapshot>) {
    let started = Instant::now();
    let (viewer, seed, mode) = (snapshot.viewer.clone(), snapshot.seed, snapshot.mode);

    let follows = match fetch::get_follows_cached(&state, &viewer).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("follows fetch for {viewer} failed: {e}");
            snapshot.finish_warming().await;
            return;
        }
    };
    // per-viewer activity is namespaced by mode, so browsing the image wall
    // never reshapes the full wall's known-active cohort, and vice versa.
    let activity_key = cohort::activity_key(&viewer, mode);
    let cohort = cohort::sample_cohort(&state, &activity_key, &follows, seed).await;
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
                let bricks = fetch::live_bricks(&state, &follows).await;
                if !bricks.is_empty() {
                    tracing::debug!("{} of the follow graph is live", bricks.len());
                    snapshot.admit_all(bricks.iter()).await;
                }
                snapshot.finish_slow_fan().await;
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
                let yielding = fan_out_repos(&state, &snapshot, &cohort).await;
                snapshot.finish_slow_fan().await;
                yielding
            };

            let ((answered, mut yielding), repo_authors, ()) =
                futures::join!(posts_fill, repos_fill, live_fill);
            yielding.extend(repo_authors);
            (answered, yielding)
        }
    };

    // Only the authors that ANSWERED are remembered as fanned out, and before
    // warming ends so no wave can trigger against a half-recorded set. One
    // whose fetch failed transiently was never really asked: the failure is
    // not cached, so the next wave (which excludes only the fanned) simply
    // asks them again.
    snapshot.record_fanned(&answered).await;
    snapshot.finish_warming().await;
    tracing::debug!(
        "snapshot {} warmed: {} authors in {:?}",
        snapshot.id,
        answered.len(),
        started.elapsed()
    );

    // remember who yielded, for the next snapshot's cohort (mode-namespaced)
    cohort::record_activity(&state, &activity_key, yielding_authors).await;
}

/// One extension wave: fan out to the next slice of the follow graph this
/// wall has never asked, so an endless scroll quarries the whole graph rather
/// than ending at the first cohort. Triggered by `get_page` when the pool
/// runs low; the snapshot's `extending` flag keeps waves single-file, and the
/// caller has already set it. The live list is not re-read here: a wave feeds
/// a wall long past its first paint, and ended streams are pruned separately.
pub async fn extend(state: Arc<AppState>, snapshot: Arc<Snapshot>) {
    let started = Instant::now();
    let follows = match fetch::get_follows_cached(&state, &snapshot.viewer).await {
        Ok(f) => f,
        Err(e) => {
            // transient, so the graph is NOT marked spent: a later page retries
            tracing::debug!("wave follows fetch for {} failed: {e}", snapshot.viewer);
            snapshot.finish_extension(false).await;
            return;
        }
    };
    let fanned = snapshot.fanned().await;
    let wave = cohort::next_wave(&follows, snapshot.seed, &fanned);
    if wave.is_empty() {
        tracing::debug!("snapshot {}: follow graph spent", snapshot.id);
        snapshot.finish_extension(true).await;
        return;
    }
    // the admission caps were budgeted for one cohort; each wave brings that
    // budget again so its authors are not turned away at a full door
    snapshot.raise_caps().await;

    let (answered, yielding) = match snapshot.mode {
        Mode::Glaze => fan_out_authors(&state, &snapshot, &wave, true, Brick::is_image_post).await,
        Mode::Wall => {
            let posts = fan_out_authors(&state, &snapshot, &wave, false, |_| true);
            let repos = fan_out_repos(&state, &snapshot, &wave);
            let ((answered, mut yielding), repo_authors) = futures::join!(posts, repos);
            yielding.extend(repo_authors);
            (answered, yielding)
        }
    };

    // as in the initial fill: only the authors that answered count as fanned,
    // so a transient blip is asked again by the next wave rather than costing
    // this wall a hundred authors for its whole life. Recorded before the
    // extension is finished, so the next wave can never race a stale set.
    snapshot.record_fanned(&answered).await;
    snapshot.finish_extension(false).await;
    tracing::debug!(
        "snapshot {}: wave of {} authors in {:?}",
        snapshot.id,
        wave.len(),
        started.elapsed()
    );
    let activity_key = cohort::activity_key(&snapshot.viewer, snapshot.mode);
    cohort::record_activity(&state, &activity_key, yielding).await;
}

/// Fan the repo reads (blogs and archived streams) across a cohort, admitting
/// as they land. Returns the authors that yielded. Shared by the initial fill
/// and the extension waves; only the initial fill counts it as a slow fan.
async fn fan_out_repos(
    state: &Arc<AppState>,
    snapshot: &Arc<Snapshot>,
    cohort: &[Author],
) -> Vec<String> {
    let mut repos = stream::iter(cohort.iter().cloned().map(|author| {
        let state = Arc::clone(state);
        async move {
            // blogs and archived streams both read the author's repo, so they
            // share one identity lookup rather than racing for two
            let Some(pds) = fetch::pds_cached(&state, &author.did).await else {
                return (author, Arc::new(StdDocs::default()), Arc::new(Vec::new()));
            };
            let (docs, streams) = tokio::join!(
                fetch::std_docs_cached(&state, &pds, &author),
                fetch::streams_cached(&state, &pds, &author),
            );
            (author, docs, streams)
        }
    }))
    .buffer_unordered(fetch::REPO_FAN_OUT);

    let mut yielding: Vec<String> = Vec::new();
    while let Some((author, docs, streams)) = repos.next().await {
        if !docs.bricks.is_empty() || !streams.is_empty() {
            yielding.push(author.did);
        }
        snapshot.admit_repo_yield(&docs, &streams).await;
        snapshot.notify_progress();
    }
    yielding
}

/// Fan out author feeds across the cohort, admitting the bricks that pass
/// `keep`. The full wall keeps everything; glaze keeps only image posts.
/// Returns (authors that answered, authors that yielded at least one kept
/// brick): the first is what `record_fanned` remembers, the second warm-starts
/// the next cohort. An author whose fetch failed transiently is in neither
/// list, so a later wave asks them again.
async fn fan_out_authors(
    state: &Arc<AppState>,
    snapshot: &Arc<Snapshot>,
    cohort: &[Author],
    deep_media: bool,
    keep: impl Fn(&Brick) -> bool,
) -> (Vec<Author>, Vec<String>) {
    let mut feeds = stream::iter(cohort.iter().cloned().map(|author| {
        let state = Arc::clone(state);
        async move {
            // glaze reads the author's media deep (posts_with_media); the full
            // wall skims their last thirty posts. Separate caches, so neither
            // read clobbers the other's.
            let yield_ = if deep_media {
                fetch::image_feed_cached(&state, &author.did).await
            } else {
                fetch::author_feed_cached(&state, &author.did).await
            };
            (author, yield_)
        }
    }))
    .buffer_unordered(FAN_OUT);

    let mut answered: Vec<Author> = Vec::new();
    let mut yielding: Vec<String> = Vec::new();
    while let Some((author, yield_)) = feeds.next().await {
        let Some(yield_) = yield_ else {
            continue; // transient failure: never answered, never fanned
        };
        // `keep` is a pure filter, so what survives it is known before any
        // admission; the batch is then admitted under one lock hold
        let kept: Vec<&Brick> = yield_.bricks.iter().filter(|b| keep(b)).collect();
        if !kept.is_empty() {
            yielding.push(author.did.clone());
        }
        answered.push(author);
        snapshot.admit_all(kept).await;
        snapshot.notify_progress();
    }
    (answered, yielding)
}

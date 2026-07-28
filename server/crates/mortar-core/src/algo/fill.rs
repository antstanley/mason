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
///
/// This is the one place `snapshot.refresh` is honoured: the wall the reader
/// asked for on purpose is the wall whose author feeds are re-read rather than
/// served from the five-minute content caches. The waves that follow are not
/// (see `extend`), so a refresh costs exactly one cohort however far the reader
/// then scrolls.
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
            fan_out_authors(
                &state,
                &snapshot,
                &cohort,
                true,
                snapshot.refresh,
                Brick::is_image_post,
            )
            .await
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
            let posts_fill =
                fan_out_authors(&state, &snapshot, &cohort, false, snapshot.refresh, |_| {
                    true
                });

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

    // A wave is never refreshed, and the two literal `false`s below are that
    // rule. The snapshot's own flag is deliberately not read here, even on a
    // wall the reader did ask to refresh: a wave asks authors this wall has
    // never asked, so there is nothing cached for it to bypass, and honouring
    // the flag per wave would multiply a refresh's cost by the length of the
    // scroll. Only the initial fill pays it.
    let (answered, yielding) = match snapshot.mode {
        Mode::Glaze => {
            fan_out_authors(&state, &snapshot, &wave, true, false, Brick::is_image_post).await
        }
        Mode::Wall => {
            let posts = fan_out_authors(&state, &snapshot, &wave, false, false, |_| true);
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
///
/// `refresh` is a PARAMETER rather than a read of the snapshot's own flag, and
/// that is the whole reason it is spelled out at each of the four call sites:
/// `extend` hands this function the same `Arc<Snapshot>` that `fill` does, so a
/// version that read the field could not tell a wave from a fill and would
/// re-read the AppView on every wave of a refreshed wall.
async fn fan_out_authors(
    state: &Arc<AppState>,
    snapshot: &Arc<Snapshot>,
    cohort: &[Author],
    deep_media: bool,
    refresh: bool,
    keep: impl Fn(&Brick) -> bool,
) -> (Vec<Author>, Vec<String>) {
    let mut feeds = stream::iter(cohort.iter().cloned().map(|author| {
        let state = Arc::clone(state);
        async move {
            // glaze reads the author's media deep (posts_with_media); the full
            // wall skims their last thirty posts. Separate caches, so neither
            // read clobbers the other's, and `refresh` bypasses whichever of
            // the two this wall reads.
            let yield_ = if deep_media {
                fetch::image_feed_cached(&state, &author.did, refresh).await
            } else {
                fetch::author_feed_cached(&state, &author.did, refresh).await
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

// Where the refresh flag goes, and where it stops, driven against a wiremock
// AppView. The FIRST test module in this file, so it has nothing to copy its
// gating from: `#[cfg(all(test, not(target_arch = "wasm32")))]` rather than a
// bare `#[cfg(test)]` because wiremock and tokio's runtime are
// `cfg(not(target_arch = "wasm32"))` dev dependencies. Under a bare gate this
// module would break the wasm32 build of `--all-targets` without failing a
// single test, and `just guard-wasm` is the only gate in the repo that would
// ever see it.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod refresh_tests {
    use super::*;
    use crate::algo::snapshot::{self, for_test};
    use crate::config::Config;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const VIEWER: &str = "did:plc:viewer";
    /// The viewer's one follow, and so the whole cohort and the whole of the
    /// first wave.
    const AUTHOR: &str = "did:plc:aa";
    const AUTHOR_FEED: &str = "/xrpc/app.bsky.feed.getAuthorFeed";

    fn state_for(server: &MockServer) -> Arc<AppState> {
        // every base points at the mock, so a read this test does not expect
        // fails against a mount that is not there rather than leaving the test
        // machine's network to decide
        Arc::new(AppState::new(Config {
            appview_base: server.uri(),
            plc_base: server.uri(),
            streamplace_base: server.uri(),
        }))
    }

    /// A brick's id is its at-uri, so an rkey is how a test tells the answer a
    /// refresh went and got from the one it was handed out of the cache.
    fn post_uri(rkey: &str) -> String {
        format!("at://{AUTHOR}/app.bsky.feed.post/{rkey}")
    }

    async fn follows_answer(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": AUTHOR, "handle": "a.test"}]
            })))
            .mount(server)
            .await;
    }

    /// One fresh post from the author. `times` is how many reads this answer
    /// serves before the next mount takes over, which is how a test hands the
    /// warm-up one wall and whatever reads the AppView after it a newer one.
    async fn author_feed_answers(server: &MockServer, rkey: &str, times: Option<u64>) {
        let created = (chrono::Utc::now() - chrono::TimeDelta::hours(1)).to_rfc3339();
        let body = serde_json::json!({"feed": [{
            "post": {
                "uri": post_uri(rkey),
                "author": {"did": AUTHOR, "handle": "a.test"},
                "record": {"text": "hello wall", "createdAt": created},
                "likeCount": 0,
                "repostCount": 0
            }
        }]});
        let mock = Mock::given(method("GET"))
            .and(path(AUTHOR_FEED))
            .respond_with(ResponseTemplate::new(200).set_body_json(body));
        match times {
            Some(times) => mock.up_to_n_times(times).mount(server).await,
            None => mock.mount(server).await,
        }
    }

    /// How many times the AppView's author feed was actually read. The whole
    /// claim of both tests below is a count of these, because a cache bypass is
    /// invisible in the answer and visible only in the traffic.
    async fn author_feed_reads(server: &MockServer) -> usize {
        server
            .received_requests()
            .await
            .expect("the mock server records what it was asked")
            .iter()
            .filter(|request| request.url.path() == AUTHOR_FEED)
            .count()
    }

    async fn cached_brick(state: &Arc<AppState>) -> String {
        let yield_ = state
            .caches
            .author_feed
            .get(&AUTHOR.to_string())
            .await
            .expect("the author's feed is cached");
        assert_eq!(yield_.bricks.len(), 1, "the fixture feed carries one post");
        yield_.bricks[0].id().to_string()
    }

    /// The flag's whole route: a `refresh` snapshot's fill is spawned DETACHED
    /// by `ensure_snapshot`, long after the request that asked for it has
    /// returned, so the flag has to travel on the snapshot to reach the two
    /// cache reads at the far end of it.
    #[tokio::test]
    async fn a_refreshed_walls_fill_re_reads_the_author_feed() {
        let server = MockServer::start().await;
        follows_answer(&server).await;
        // the warm entry the refresh has to step over, and then the newer wall
        // that only a re-read can see
        author_feed_answers(&server, "1", Some(1)).await;
        author_feed_answers(&server, "2", None).await;
        let state = state_for(&server);

        fetch::author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("a live AppView answers");
        assert_eq!(author_feed_reads(&server).await, 1, "the cold read");
        assert_eq!(cached_brick(&state).await, post_uri("1"));

        // get_or_build spawns the fill through ensure_snapshot and comes back
        // once warming is over, which is the fill having run to the end
        let refreshing = snapshot::get_or_build(&state, VIEWER, 1, Mode::Wall, true).await;
        assert!(refreshing.refresh, "the flag rides on the snapshot");

        assert_eq!(
            author_feed_reads(&server).await,
            2,
            "a refreshed fill must reach the AppView, or the wall it lays is the one it replaced"
        );
        assert_eq!(
            cached_brick(&state).await,
            post_uri("2"),
            "and the newer answer is what stays behind for everyone reading after"
        );
    }

    /// The rule no signature can enforce. `extend` is handed the same
    /// `Arc<Snapshot>` the fill was, so a fan-out that read the snapshot's flag
    /// itself could not tell a wave from a fill: every wave of a refreshed wall
    /// would re-fan a hundred rate-limited reads, and a refresh would cost more
    /// the longer the reader scrolled.
    ///
    /// The wave's author is warm ALREADY, because the content caches are shared
    /// across snapshots: another wall a minute ago, or another tab, is enough.
    /// Without that entry a wave honouring the flag and a wave ignoring it would
    /// both reach the AppView and this test would prove nothing.
    #[tokio::test]
    async fn a_wave_of_a_refreshed_wall_never_re_reads_an_author_feed() {
        let server = MockServer::start().await;
        follows_answer(&server).await;
        author_feed_answers(&server, "1", Some(1)).await;
        // mounted so that a wave which DID refresh would succeed and be caught
        // by the newer rkey, rather than merely failing to find a mock
        author_feed_answers(&server, "2", None).await;
        let state = state_for(&server);

        fetch::author_feed_cached(&state, AUTHOR, false)
            .await
            .expect("a live AppView answers");
        let warmed = author_feed_reads(&server).await;
        assert_eq!(warmed, 1, "warming the entry is one read, and the baseline");

        let refreshing = Arc::new(for_test("s", VIEWER, 1, Mode::Wall, true));
        assert!(refreshing.refresh, "a wave of a REFRESHED wall is the case");
        extend(Arc::clone(&state), Arc::clone(&refreshing)).await;

        assert!(
            refreshing.fanned().await.contains(AUTHOR),
            "the wave must actually have asked about the author, or zero reads proves nothing"
        );
        assert_eq!(
            author_feed_reads(&server).await,
            warmed,
            "a wave must be served from the content caches even on a refreshed wall"
        );
        assert_eq!(
            cached_brick(&state).await,
            post_uri("1"),
            "and the entry it read is the one that was already there"
        );
    }
}

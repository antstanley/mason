//! Who a snapshot fans out to. The cohort samples the follow graph (known
//! yielders first, a seeded-random top-up of the rest), and activity remembers
//! which authors yielded content so the next cohort warm-starts with them.
//! Reads and writes the shared activity cache only; this module never sees a
//! snapshot, let alone its inner mutex.

use std::collections::HashSet;
use std::sync::Arc;

use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;

use crate::mode::Mode;
use crate::model::Author;
use crate::sources::Follow;
use crate::state::AppState;

/// Sampled authors per snapshot; never fan out to the whole follow graph.
const COHORT_SIZE: usize = 100;
/// Of which: authors that yielded content in recent snapshots.
const KNOWN_ACTIVE: usize = 60;

/// Namespace a viewer's activity by mode. The full wall keeps the bare did (so
/// entries persisted before glaze existed still match), while glaze prefixes it,
/// keeping the two walls' known-active cohorts from bleeding into each other.
pub fn activity_key(viewer: &str, mode: Mode) -> String {
    match mode {
        Mode::Wall => viewer.to_string(),
        Mode::Glaze => format!("{}:{viewer}", mode.tag()),
    }
}

/// Cohort: up to KNOWN_ACTIVE authors that yielded content before, topped up
/// with a seeded-random sample of the rest so refreshes rotate through the
/// whole follow graph.
pub async fn sample_cohort(
    state: &Arc<AppState>,
    activity_key: &str,
    follows: &[Follow],
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
    let by_did: std::collections::HashMap<&str, &Follow> = follows
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
    let mut rest: Vec<&Follow> = follows
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

/// The next wave of authors to fan out to, once scrolling has drained what
/// the earlier waves yielded: the follow graph minus everyone already fanned
/// out, seeded-shuffled, capped at COHORT_SIZE. Deterministic given the same
/// follows and fanned set, so a snapshot rebuilt from its cursor walks its
/// waves in the same order. Empty means the graph is spent.
pub fn next_wave(follows: &[Follow], seed: u64, fanned: &HashSet<String>) -> Vec<Author> {
    let mut rest: Vec<&Follow> = follows
        .iter()
        .filter(|f| !f.hidden() && !fanned.contains(f.did.as_str()))
        .collect();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    rest.shuffle(&mut rng);
    rest.truncate(COHORT_SIZE);
    rest.into_iter().map(Author::from).collect()
}

pub async fn record_activity(state: &Arc<AppState>, activity_key: &str, mut yielding: Vec<String>) {
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn follow(did: &str) -> Follow {
        Follow {
            did: did.into(),
            handle: format!("{did}.test"),
            display_name: None,
            avatar: None,
            labels: vec![],
        }
    }

    fn opted_out_follow(did: &str) -> Follow {
        let mut f = follow(did);
        f.labels =
            serde_json::from_value(serde_json::json!([{"val": "!no-unauthenticated"}])).unwrap();
        f
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

    /// A wave is the graph minus everyone already asked: it never repeats an
    /// author, it is stable for a seed (a rebuilt snapshot walks the same
    /// waves), and once the graph is spent it comes back empty, which is how
    /// the wall learns it can genuinely end.
    #[test]
    fn waves_walk_the_graph_without_repeats_and_then_end() {
        let follows: Vec<Follow> = (0..250).map(|n| follow(&format!("did:plc:f{n}"))).collect();
        let mut fanned: HashSet<String> = HashSet::new();

        let first = next_wave(&follows, 7, &fanned);
        assert_eq!(first.len(), COHORT_SIZE);
        assert_eq!(
            next_wave(&follows, 7, &fanned)
                .iter()
                .map(|a| a.did.as_str())
                .collect::<Vec<_>>(),
            first.iter().map(|a| a.did.as_str()).collect::<Vec<_>>(),
            "the same seed and fanned set must give the same wave"
        );

        fanned.extend(first.iter().map(|a| a.did.clone()));
        let second = next_wave(&follows, 7, &fanned);
        assert_eq!(second.len(), COHORT_SIZE);
        assert!(
            second.iter().all(|a| !fanned.contains(&a.did)),
            "a wave must never repeat an author already fanned out to"
        );

        fanned.extend(second.iter().map(|a| a.did.clone()));
        let third = next_wave(&follows, 7, &fanned);
        assert_eq!(third.len(), 50, "the remainder of a 250-follow graph");

        fanned.extend(third.iter().map(|a| a.did.clone()));
        assert!(
            next_wave(&follows, 7, &fanned).is_empty(),
            "a spent graph must come back empty"
        );
    }

    /// Hidden follows are excluded from waves exactly as from the first
    /// cohort: the wave is a fan-out list, and fanning out to an opted-out
    /// account would fetch content the wall must never lay.
    #[test]
    fn a_wave_excludes_hidden_follows() {
        let follows = vec![follow("did:plc:open"), opted_out_follow("did:plc:sealed")];
        let wave = next_wave(&follows, 1, &HashSet::new());
        let dids: Vec<&str> = wave.iter().map(|a| a.did.as_str()).collect();
        assert_eq!(dids, vec!["did:plc:open"]);
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
}

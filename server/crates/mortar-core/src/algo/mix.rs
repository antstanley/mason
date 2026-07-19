//! The mixer lays bricks one at a time: each pick maximizes
//! grout × type-need × jitter among candidates outside the author-diversity
//! window. Ranking is within-kind (grout) while the need factor pulls the
//! wall toward the target kind ratio; kinds are never compared by raw score.
//!
//! Pure and deterministic given (pool, wall, seed, now): laying 20 then 20
//! equals laying 40, which is what makes cursor pagination stable.

use chrono::{DateTime, Utc};
use xxhash_rust::xxh3::xxh3_64_with_seed;

use super::score;
use crate::model::{Brick, VideoSource};

/// Target share of the wall per mix kind: post / blog / Bluesky video /
/// archived stream / live stream. Each kind competes only with itself; an
/// archived stream has no engagement signal, so ranking it against a liked
/// Bluesky video would bury it at the bottom of every wall.
const TARGET: [f64; KINDS] = [0.68, 0.15, 0.09, 0.05, 0.03];
pub const KINDS: usize = 5;
/// The kind that is happening right now.
const LIVE: usize = 4;
/// A brick's author may not reappear within this many trailing bricks
/// (soft; ignored when every candidate is inside the window).
const AUTHOR_WINDOW: usize = 8;

pub fn kind_index(brick: &Brick) -> usize {
    match brick {
        Brick::Post(_) => 0,
        Brick::Blog(_) => 1,
        Brick::Video(v) if v.source == VideoSource::Bluesky => 2,
        Brick::Video(v) if v.live => LIVE,
        Brick::Video(_) => 3,
    }
}

/// Deterministic wobble in [0.85, 1.15]; the wall feels alive across
/// seeds but identical within one.
fn jitter(seed: u64, id: &str) -> f64 {
    let h = xxh3_64_with_seed(id.as_bytes(), seed);
    0.85 + 0.30 * ((h % 10_000) as f64 / 10_000.0)
}

/// How much the wall currently wants each kind: target share over actual
/// share. Scale-free, so it composes with grout by multiplication.
fn need(wall_counts: &[usize; KINDS], laid: usize) -> [f64; KINDS] {
    let mut need = [0.0; KINDS];
    for k in 0..KINDS {
        let actual = if laid == 0 {
            0.0
        } else {
            wall_counts[k] as f64 / laid as f64
        };
        need[k] = TARGET[k] / (actual + 0.05);
    }
    need
}

/// Pick and remove the next brick from the pool. Returns None when the pool
/// is empty.
///
/// Two-step so kinds are never compared by raw grout (engagement-boosted
/// posts would drown blogs and trailers): first choose the KIND by
/// need × jitter, then the best brick WITHIN that kind by grout × jitter.
pub fn lay_next(
    pool: &mut Vec<Brick>,
    wall: &[Brick],
    seed: u64,
    now: DateTime<Utc>,
) -> Option<Brick> {
    if pool.is_empty() {
        return None;
    }

    let recent_authors: Vec<&str> = wall
        .iter()
        .rev()
        .take(AUTHOR_WINDOW)
        .map(score::author_key)
        .collect();
    let mut counts = [0usize; KINDS];
    for brick in wall {
        counts[kind_index(brick)] += 1;
    }
    let need = need(&counts, wall.len());
    let position = wall.len();

    // grout parses created_at off a date string, so scoring inside the max_by
    // comparators would parse each brick's date many times per lay_next (and
    // preview re-lays the whole ~600-brick pool every 350ms). now and seed are
    // fixed across this call, so each brick's score is a constant: compute it
    // once here, indexed by pool position, and the comparators just read it.
    // Identical value as before, so ranking and determinism are unchanged.
    let scores: Vec<f64> = pool
        .iter()
        .map(|b| score::grout(b, now) * jitter(seed, b.id()))
        .collect();

    // best candidate per kind, honoring the author window; None if the kind
    // has no eligible candidate
    let leader = |kind: usize, respect_window: bool| -> Option<usize> {
        pool.iter()
            .enumerate()
            .filter(|(_, b)| kind_index(b) == kind)
            .filter(|(_, b)| !respect_window || !recent_authors.contains(&score::author_key(b)))
            .max_by(|(ia, _), (ib, _)| scores[*ia].total_cmp(&scores[*ib]))
            .map(|(i, _)| i)
    };

    // A live stream is the only brick on the wall with a deadline: it is
    // happening while you look at it, and tomorrow it is gone. The first one
    // the pool can offer skips the kind lottery and opens the wall; the need
    // factor spaces out any others (rare: it means two people you follow are
    // streaming at once).
    if counts[LIVE] == 0
        && let Some(index) = leader(LIVE, false)
    {
        return Some(pool.remove(index));
    }

    let pick = |respect_window: bool| -> Option<usize> {
        (0..KINDS)
            .filter_map(|kind| {
                leader(kind, respect_window).map(|index| {
                    let weight = need[kind] * jitter(seed, &format!("{position}:{kind}"));
                    (index, weight)
                })
            })
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
    };

    // The diversity window is a hard constraint while any other author has a
    // brick to offer. When it truly cannot be honoured (a wall built from one
    // author's feed), fall back to the author holding the FEWEST bricks on the
    // wall so far, not to the highest score: scoring again would just re-pick
    // the dominant author, which is how a first page ended up belonging to one
    // person.
    let index = pick(true).or_else(|| {
        let mut wall_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for brick in wall {
            *wall_counts.entry(score::author_key(brick)).or_insert(0) += 1;
        }
        pool.iter()
            .enumerate()
            .min_by(|(ia, a), (ib, b)| {
                let laid = |x: &Brick| *wall_counts.get(score::author_key(x)).unwrap_or(&0);
                laid(a)
                    .cmp(&laid(b))
                    .then_with(|| scores[*ib].total_cmp(&scores[*ia]))
            })
            .map(|(i, _)| i)
    })?;
    Some(pool.remove(index))
}

/// Lay `count` bricks (or until the pool runs dry).
pub fn lay(
    pool: &mut Vec<Brick>,
    wall: &mut Vec<Brick>,
    count: usize,
    seed: u64,
    now: DateTime<Utc>,
) {
    for _ in 0..count {
        match lay_next(pool, wall, seed, now) {
            Some(brick) => wall.push(brick),
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::collections::{HashMap, HashSet};

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-11T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn author(n: usize) -> Author {
        Author {
            did: format!("did:plc:a{n}"),
            handle: format!("a{n}.test"),
            display_name: None,
            avatar: None,
        }
    }

    fn ts(i: usize) -> String {
        format!("2026-07-10T{:02}:{:02}:00Z", i / 60 % 24, i % 60)
    }

    fn post(i: usize, author_n: usize) -> Brick {
        Brick::Post(PostBrick {
            id: format!("post-{i}"),
            url: String::new(),
            author: author(author_n),
            text: "t".into(),
            created_at: ts(i),
            like_count: (i * 13 % 300) as u64,
            repost_count: 0,
            images: vec![],
            external: None,
            blur: None,
        })
    }

    fn blog(i: usize, author_n: usize) -> Brick {
        Brick::Blog(BlogBrick {
            id: format!("blog-{i}"),
            url: String::new(),
            author: author(author_n),
            title: "t".into(),
            description: None,
            cover_image: None,
            publication: Publication {
                name: "p".into(),
                url: String::new(),
                icon: None,
            },
            tags: vec![],
            published_at: ts(i),
        })
    }

    fn video(i: usize, author_n: usize, source: VideoSource) -> Brick {
        Brick::Video(VideoBrick {
            id: format!("video-{i}"),
            url: String::new(),
            author: author(author_n),
            title: "t".into(),
            poster: None,
            playlist: String::new(),
            aspect_ratio: None,
            source,
            created_at: ts(i),
            like_count: 0,
            live: false,
            viewer_count: None,
            duration_ms: None,
            activity: None,
            captions: Vec::new(),
            blur: None,
        })
    }

    fn live(i: usize, author_n: usize, viewers: u64) -> Brick {
        match video(i, author_n, VideoSource::Streamplace) {
            Brick::Video(mut v) => {
                v.live = true;
                v.viewer_count = Some(viewers);
                Brick::Video(v)
            }
            other => other,
        }
    }

    /// 300 candidates over 30 authors, roughly matching the target mix
    fn big_pool() -> Vec<Brick> {
        let mut pool = Vec::new();
        for i in 0..300 {
            let a = i % 30;
            pool.push(match i % 20 {
                3 | 10 | 17 => blog(i, a),
                6 | 13 => video(i, a, VideoSource::Bluesky),
                19 => video(i, a, VideoSource::Streamplace),
                _ => post(i, a),
            });
        }
        pool
    }

    #[test]
    fn same_seed_same_wall() {
        let (mut p1, mut p2) = (big_pool(), big_pool());
        let (mut w1, mut w2) = (Vec::new(), Vec::new());
        lay(&mut p1, &mut w1, 100, 42, now());
        lay(&mut p2, &mut w2, 100, 42, now());
        let ids = |w: &[Brick]| w.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        assert_eq!(ids(&w1), ids(&w2));
    }

    #[test]
    fn different_seed_different_wall() {
        let (mut p1, mut p2) = (big_pool(), big_pool());
        let (mut w1, mut w2) = (Vec::new(), Vec::new());
        lay(&mut p1, &mut w1, 100, 1, now());
        lay(&mut p2, &mut w2, 100, 2, now());
        let ids = |w: &[Brick]| w.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        assert_ne!(ids(&w1), ids(&w2));
    }

    #[test]
    fn pagination_is_stable() {
        // laying 24 then 24 must equal laying 48 in one call
        let mut p1 = big_pool();
        let mut w1 = Vec::new();
        lay(&mut p1, &mut w1, 24, 7, now());
        lay(&mut p1, &mut w1, 24, 7, now());

        let mut p2 = big_pool();
        let mut w2 = Vec::new();
        lay(&mut p2, &mut w2, 48, 7, now());

        let ids = |w: &[Brick]| w.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        assert_eq!(ids(&w1), ids(&w2));
    }

    /// The live queue-jump is a new early return inside lay_next, so it has to
    /// obey the rule the whole cursor rests on: laying 24 then 24 must equal
    /// laying 48. It reads only (pool, wall), so it does; this is the test that
    /// notices if it ever starts reading a clock or a counter instead.
    #[test]
    fn pagination_is_stable_with_a_live_brick() {
        let mut p1 = big_pool();
        p1.push(live(900, 7, 42));
        let mut w1 = Vec::new();
        lay(&mut p1, &mut w1, 24, 7, now());
        lay(&mut p1, &mut w1, 24, 7, now());

        let mut p2 = big_pool();
        p2.push(live(900, 7, 42));
        let mut w2 = Vec::new();
        lay(&mut p2, &mut w2, 48, 7, now());

        let ids = |w: &[Brick]| w.iter().map(|b| b.id().to_string()).collect::<Vec<_>>();
        assert_eq!(ids(&w1), ids(&w2));
    }

    #[test]
    fn author_never_repeats_within_window_when_alternatives_exist() {
        let mut pool = big_pool(); // 30 authors ≫ window of 8
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 200, 3, now());
        for window in wall.windows(AUTHOR_WINDOW + 1) {
            let authors: HashSet<_> = window.iter().map(score::author_key).collect();
            assert_eq!(authors.len(), window.len(), "author repeated within window");
        }
    }

    #[test]
    fn ratio_converges_to_target() {
        let mut pool = big_pool();
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 200, 9, now());
        let mut counts = [0usize; KINDS];
        for b in &wall {
            counts[kind_index(b)] += 1;
        }
        let share = |k: usize| counts[k] as f64 / wall.len() as f64;
        assert!((share(0) - TARGET[0]).abs() < 0.10, "posts {}", share(0));
        assert!((share(1) - TARGET[1]).abs() < 0.08, "blogs {}", share(1));
        assert!(
            (share(2) - TARGET[2]).abs() < 0.06,
            "bsky videos {}",
            share(2)
        );
        assert!((share(3) - TARGET[3]).abs() < 0.04, "streams {}", share(3));
    }

    /// The whole point of pinning live high: a stream that is happening right
    /// now is worthless three screens down, and the mixer's need factor alone
    /// would bury it there (posts open every wall at 14x the need of anything
    /// else).
    #[test]
    fn a_live_stream_opens_the_wall() {
        for seed in 0..20 {
            let mut pool = big_pool();
            pool.push(live(900, 7, 42));
            let mut wall = Vec::new();
            lay(&mut pool, &mut wall, 24, seed, now());
            assert_eq!(
                wall[0].id(),
                "video-900",
                "seed {seed} buried the live stream"
            );
        }
    }

    /// It jumps the queue exactly once. A live brick that keeps winning would
    /// be a wall of one stream.
    #[test]
    fn live_does_not_take_over_the_wall() {
        let mut pool = big_pool();
        pool.push(live(900, 7, 42));
        pool.push(live(901, 8, 9));
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 48, 3, now());

        let live_laid = wall.iter().filter(|b| kind_index(b) == LIVE).count();
        assert_eq!(wall[0].id(), "video-900", "the busiest live stream leads");
        assert!(
            (1..=3).contains(&live_laid),
            "{live_laid} live bricks in 48 is not a wall, it is a channel"
        );
    }

    /// A live stream has no date worth trusting (its record may be months
    /// old), so it must not be ranked by one. Viewers are its recency.
    #[test]
    fn among_live_streams_the_busiest_leads() {
        let mut pool = vec![live(1, 1, 3), live(2, 2, 500), live(3, 3, 40)];
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 1, 11, now());
        assert_eq!(wall[0].id(), "video-2");
    }

    #[test]
    fn starved_kind_degrades_gracefully() {
        // all-post pool must still fill the wall
        let mut pool: Vec<Brick> = (0..50).map(|i| post(i, i % 20)).collect();
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 50, 5, now());
        assert_eq!(wall.len(), 50);
    }
    #[test]
    fn streams_pooled_late_still_get_laid() {
        // wall already has 100 bricks laid with no streams available, then 3
        // join the pool (a slow PDS answered); they must appear in the next 48
        let mut pool: Vec<Brick> = Vec::new();
        for i in 0..200 {
            let a = i % 30;
            pool.push(match i % 20 {
                3 | 10 | 17 => blog(i, a),
                6 | 13 => video(i, a, VideoSource::Bluesky),
                _ => post(i, a),
            });
        }
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 100, 11, now());
        assert!(wall.iter().all(|b| kind_index(b) != 3));

        for i in 900..903 {
            pool.push(video(i, i % 30, VideoSource::Streamplace));
        }
        lay(&mut pool, &mut wall, 48, 11, now());
        let streams = wall.iter().filter(|b| kind_index(b) == 3).count();
        assert!(streams >= 2, "only {streams} streams laid in 48 bricks");
    }

    /// Half of the fix for the wall whose first page belonged to one person.
    /// The mixer cannot un-dominate a dominated pool (see the admission cap in
    /// snapshot.rs, which is the other half); what it MUST do is space out the
    /// authors it is given. Pool here is what admission really yields: at most
    /// four bricks per author.
    #[test]
    fn no_author_owns_the_first_page() {
        let mut pool = Vec::new();
        for author in 0..10 {
            for n in 0..4 {
                pool.push(post(author * 10 + n, author));
            }
        }
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 24, 4, now());

        let mut counts: HashMap<&str, usize> = HashMap::new();
        for brick in &wall {
            *counts.entry(score::author_key(brick)).or_insert(0) += 1;
        }
        let loudest = *counts.values().max().unwrap();
        assert!(
            loudest <= 4,
            "one author took {loudest} of the first {} bricks: {counts:?}",
            wall.len()
        );
        assert!(
            counts.len() >= 7,
            "only {} authors on the page",
            counts.len()
        );
    }

    /// Even when the pool holds NOTHING but one author, the wall should still
    /// be laid rather than hang. Domination is only acceptable when it is the
    /// literal truth about the pool.
    #[test]
    fn single_author_pool_still_lays() {
        let mut pool: Vec<Brick> = (0..10).map(|i| post(i, 7)).collect();
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 10, 1, now());
        assert_eq!(wall.len(), 10);
    }

    /// The relaxed path must spread the load, not re-pick the top scorer.
    #[test]
    fn relaxation_prefers_the_least_represented_author() {
        // two authors only, so the 8-brick window cannot be honoured
        let mut pool: Vec<Brick> = (0..6).map(|i| post(i, 0)).collect();
        pool.extend((6..12).map(|i| post(i, 1)));
        let mut wall = Vec::new();
        lay(&mut pool, &mut wall, 12, 2, now());

        let mut counts: HashMap<&str, usize> = HashMap::new();
        for brick in &wall {
            *counts.entry(score::author_key(brick)).or_insert(0) += 1;
        }
        assert_eq!(counts.len(), 2);
        let (min, max) = (
            *counts.values().min().unwrap(),
            *counts.values().max().unwrap(),
        );
        assert!(max - min <= 1, "load was not spread: {counts:?}");
    }
}

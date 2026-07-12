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
/// Steam trailer. Trailers are their own kind; they have no engagement
/// signal, so ranking them against liked Bluesky videos would bury them at
/// the bottom of every wall.
const TARGET: [f64; 4] = [0.70, 0.15, 0.10, 0.05];
const KINDS: usize = 4;
/// A brick's author may not reappear within this many trailing bricks
/// (soft; ignored when every candidate is inside the window).
const AUTHOR_WINDOW: usize = 8;

fn kind_index(brick: &Brick) -> usize {
    match brick {
        Brick::Post(_) => 0,
        Brick::Blog(_) => 1,
        Brick::Video(v) if v.source == VideoSource::Bluesky => 2,
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

    // best candidate per kind, honoring the author window; None if the kind
    // has no eligible candidate
    let leader = |kind: usize, respect_window: bool| -> Option<usize> {
        pool.iter()
            .enumerate()
            .filter(|(_, b)| kind_index(b) == kind)
            .filter(|(_, b)| !respect_window || !recent_authors.contains(&score::author_key(b)))
            .max_by(|(_, a), (_, b)| {
                let s = |x: &Brick| score::grout(x, now) * jitter(seed, x.id());
                s(a).total_cmp(&s(b))
            })
            .map(|(i, _)| i)
    };

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

    // hard diversity constraint, relaxed only when nothing qualifies
    let index = pick(true).or_else(|| pick(false))?;
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
    use std::collections::HashSet;

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
        let steam = source == VideoSource::Steam;
        Brick::Video(VideoBrick {
            id: format!("video-{i}"),
            url: String::new(),
            author: (!steam).then(|| author(author_n)),
            title: "t".into(),
            poster: None,
            playlist: String::new(),
            aspect_ratio: None,
            source,
            game: None,
            created_at: ts(i),
            like_count: 0,
        })
    }

    /// 300 candidates over 30 authors, roughly matching the target mix
    fn big_pool() -> Vec<Brick> {
        let mut pool = Vec::new();
        for i in 0..300 {
            let a = i % 30;
            pool.push(match i % 20 {
                3 | 10 | 17 => blog(i, a),
                6 | 13 => video(i, a, VideoSource::Bluesky),
                19 => video(i, a, VideoSource::Steam),
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
        assert!((share(0) - 0.70).abs() < 0.10, "posts {}", share(0));
        assert!((share(1) - 0.15).abs() < 0.08, "blogs {}", share(1));
        assert!((share(2) - 0.10).abs() < 0.06, "bsky videos {}", share(2));
        assert!((share(3) - 0.05).abs() < 0.04, "trailers {}", share(3));
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
    fn trailers_pooled_late_still_get_laid() {
        // wall already has 100 bricks laid with no trailers available,
        // then 3 trailers join the pool; they must appear in the next 48
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
            pool.push(video(i, 0, VideoSource::Steam));
        }
        lay(&mut pool, &mut wall, 48, 11, now());
        let trailers = wall.iter().filter(|b| kind_index(b) == 3).count();
        assert!(trailers >= 2, "only {trailers} trailers laid in 48 bricks");
    }
}

//! The grout score. Pure functions; no clocks, no IO; `now` is always a
//! parameter so tests are exact.

use chrono::{DateTime, Utc};

use crate::model::Brick;

/// Half-life in hours per brick kind: posts churn fast, blogs simmer,
/// trailers are near-evergreen.
pub fn half_life_hours(brick: &Brick) -> f64 {
    match brick {
        Brick::Post(_) => 24.0,
        Brick::Blog(_) => 24.0 * 7.0,
        Brick::Video(_) => 24.0 * 30.0,
    }
}

pub fn created_at(brick: &Brick) -> Option<DateTime<Utc>> {
    let raw = match brick {
        Brick::Post(b) => &b.created_at,
        Brick::Blog(b) => &b.published_at,
        Brick::Video(b) => &b.created_at,
    };
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

pub fn author_key(brick: &Brick) -> &str {
    match brick {
        Brick::Post(b) => &b.author.did,
        Brick::Blog(b) => &b.author.did,
        // Steam trailers have no atproto author; group them under one key so
        // the diversity window also spaces trailers apart
        Brick::Video(b) => b.author.as_ref().map(|a| a.did.as_str()).unwrap_or("steam"),
    }
}

fn engagement(brick: &Brick) -> f64 {
    match brick {
        Brick::Post(b) => (b.like_count + 2 * b.repost_count) as f64,
        // no comparable signal for blogs/trailers; neutral
        Brick::Blog(_) | Brick::Video(_) => 0.0,
    }
}

/// recency_decay × engagement_boost. Only meaningful relative to bricks of
/// the same kind; cross-kind balance is the mixer's job, not the score's.
pub fn grout(brick: &Brick, now: DateTime<Utc>) -> f64 {
    let age_hours = created_at(brick)
        .map(|t| (now - t).num_seconds().max(0) as f64 / 3600.0)
        .unwrap_or(f64::MAX / 2.0);
    let decay = 0.5_f64.powf(age_hours / half_life_hours(brick));
    let boost = 1.0 + (1.0 + engagement(brick)).ln();
    decay * boost
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Author, Brick, PostBrick};
    use chrono::TimeDelta;

    fn post(created_at: &str, likes: u64, reposts: u64) -> Brick {
        Brick::Post(PostBrick {
            id: format!("post-{created_at}-{likes}"),
            url: String::new(),
            author: Author {
                did: "did:plc:x".into(),
                handle: "x.test".into(),
                display_name: None,
                avatar: None,
            },
            text: "t".into(),
            created_at: created_at.into(),
            like_count: likes,
            repost_count: reposts,
            images: vec![],
            external: None,
        })
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-11T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn older_scores_strictly_lower() {
        let newer = post("2026-07-10T23:00:00Z", 5, 0);
        let older = post("2026-07-10T11:00:00Z", 5, 0);
        assert!(grout(&newer, now()) > grout(&older, now()));
    }

    #[test]
    fn decay_halves_at_half_life() {
        let fresh = post("2026-07-11T00:00:00Z", 0, 0);
        let one_half_life = post("2026-07-10T00:00:00Z", 0, 0); // 24h old
        let ratio = grout(&one_half_life, now()) / grout(&fresh, now());
        assert!((ratio - 0.5).abs() < 1e-9, "ratio was {ratio}");
    }

    #[test]
    fn engagement_is_log_shaped_and_never_beats_big_recency_gap() {
        // 10x half-life age gap ⇒ 2^10 decay gap; even a viral post
        // (100k likes ⇒ boost ≈ 12.5) cannot close it
        let fresh_quiet = post("2026-07-11T00:00:00Z", 0, 0);
        let stale_viral = post(
            &(now() - TimeDelta::hours(240)).to_rfc3339(),
            100_000,
            10_000,
        );
        assert!(grout(&fresh_quiet, now()) > grout(&stale_viral, now()));
    }

    #[test]
    fn reposts_weigh_double() {
        let liked = post("2026-07-11T00:00:00Z", 10, 0);
        let reposted = post("2026-07-11T00:00:00Z", 0, 6);
        assert!(grout(&reposted, now()) > grout(&liked, now()));
    }

    #[test]
    fn unparseable_timestamp_sinks_to_bottom() {
        let bad = post("not-a-date", 1000, 0);
        let ok = post("2026-07-01T00:00:00Z", 0, 0);
        assert!(grout(&ok, now()) > grout(&bad, now()));
    }
}

//! The grout score. Pure functions; no clocks, no IO; `now` is always a
//! parameter so tests are exact.

use chrono::{DateTime, Utc};

use crate::model::{Brick, VideoSource};

/// Half-life in hours per brick kind: posts churn fast, blogs simmer, an
/// archived stream is nearly evergreen. Shorter than the medium's shelf life
/// on purpose, so the freshest brick of a kind clearly outranks yesterday's.
pub fn half_life_hours(brick: &Brick) -> f64 {
    match brick {
        Brick::Post(_) => 12.0,
        Brick::Blog(_) => 24.0 * 3.0,
        // a Bluesky video IS a post, and ages like one
        Brick::Video(v) if v.source == VideoSource::Bluesky => 12.0,
        // an archived stream is an hours-long thing somebody made; it stays
        // worth watching long after a skeet about it would have expired
        Brick::Video(_) => 24.0 * 14.0,
    }
}

/// Nothing older than this is worth a slot on the wall. A hard window, not a
/// soft preference: decay alone leaves a week-old post technically eligible,
/// and on a quiet follow graph it will surface. mason is for what the people
/// you follow are making, present tense.
pub fn max_age_hours(brick: &Brick) -> f64 {
    match brick {
        Brick::Post(_) => 72.0,
        Brick::Blog(_) => 24.0 * 14.0,
        // same window as any other post: a video from three months ago is not
        // "what the people you follow are making" (this is how the wall ended
        // up 42% video: stale clips filled the gap left by expired text posts)
        Brick::Video(v) if v.source == VideoSource::Bluesky => 72.0,
        Brick::Video(_) => 24.0 * 90.0,
    }
}

/// A stream that is happening right now. It is the only brick with a deadline,
/// which earns it both an exemption from the age window and the top of the
/// wall (see `mix::lay_next`).
pub fn is_live(brick: &Brick) -> bool {
    matches!(brick, Brick::Video(v) if v.live)
}

/// Bricks with an unparseable date are treated as stale: better to drop one
/// than to let it sit at the top of a wall forever with an infinite age.
pub fn is_fresh(brick: &Brick, now: DateTime<Utc>) -> bool {
    // "live" is a fact about the present, not a claim about a timestamp. A
    // streamer who has been broadcasting on the same record since March is
    // still broadcasting.
    if is_live(brick) {
        return true;
    }
    match created_at(brick) {
        Some(t) => (now - t).num_seconds().max(0) as f64 / 3600.0 <= max_age_hours(brick),
        None => false,
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
        Brick::Video(b) => &b.author.did,
    }
}

fn engagement(brick: &Brick) -> f64 {
    match brick {
        Brick::Post(b) => (b.like_count + 2 * b.repost_count) as f64,
        // a live stream's audience is its engagement, and it is the only kind
        // whose signal is being generated as you look at it
        Brick::Video(b) if b.live => b.viewer_count.unwrap_or(0) as f64,
        Brick::Video(b) => b.like_count as f64,
        // no comparable signal for blogs; neutral
        Brick::Blog(_) => 0.0,
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
        let one_half_life = post("2026-07-10T12:00:00Z", 0, 0); // 12h old
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

    #[test]
    fn posts_older_than_72_hours_are_not_fresh() {
        assert!(is_fresh(
            &post(&(now() - TimeDelta::hours(71)).to_rfc3339(), 0, 0),
            now()
        ));
        assert!(!is_fresh(
            &post(&(now() - TimeDelta::hours(73)).to_rfc3339(), 0, 0),
            now()
        ));
    }

    #[test]
    fn an_unparseable_date_is_stale_not_immortal() {
        // it used to score as age f64::MAX/2, which is merely a low score;
        // it must be dropped instead, or it lingers on every wall forever
        assert!(!is_fresh(&post("not-a-date", 9999, 9999), now()));
    }

    #[test]
    fn recency_bias_is_steep_enough_to_matter() {
        // a day-old post must rank below a fresh one even when it was popular:
        // 24h is two half-lives now, so decay is 0.25 against a x(1+ln) boost
        let fresh_quiet = post(&now().to_rfc3339(), 0, 0);
        let day_old_popular = post(&(now() - TimeDelta::hours(24)).to_rfc3339(), 40, 0);
        assert!(grout(&fresh_quiet, now()) < grout(&day_old_popular, now()));

        // but three days old (the window edge) loses to a fresh quiet post
        let stale_popular = post(&(now() - TimeDelta::hours(71)).to_rfc3339(), 40, 0);
        assert!(grout(&fresh_quiet, now()) > grout(&stale_popular, now()));
    }
}

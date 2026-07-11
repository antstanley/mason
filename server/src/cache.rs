use std::sync::Arc;
use std::time::{Duration, Instant};

use moka::future::Cache;
use moka::Expiry;

use crate::algo::snapshot::Snapshot;
use crate::model::Brick;
use crate::sources::bluesky::{AuthorYield, Follow};

/// One author's standard.site yield.
pub struct StdDocs {
    pub bricks: Vec<Brick>,
    /// post URIs suppressed via bskyPostRef — the blog card wins
    pub suppressed_posts: Vec<String>,
}

/// Most follows publish no standard.site records — cache that emptiness for
/// a day, but recheck actual publishers every 15 minutes.
struct StdDocsExpiry;

impl Expiry<String, Arc<StdDocs>> for StdDocsExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &Arc<StdDocs>,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(if value.bricks.is_empty() {
            Duration::from_secs(24 * 3600)
        } else {
            Duration::from_secs(900)
        })
    }
}

/// All in-memory state. TTLs per the plan; capacities keep a small
/// deployment bounded.
pub struct Caches {
    /// handle → did
    pub did: Cache<String, String>,
    /// did → follows
    pub follows: Cache<String, Arc<Vec<Follow>>>,
    /// author did → their recent bricks + mentioned Steam games
    pub author_feed: Cache<String, Arc<AuthorYield>>,
    /// author did → their standard.site documents (negative results live 24h)
    pub std_docs: Cache<String, Arc<StdDocs>>,
    /// appid → trailer bricks
    pub steam_trailers: Cache<u64, Arc<Vec<Brick>>>,
    /// featured appids (single-key)
    pub steam_featured: Cache<u8, Arc<Vec<u64>>>,
    /// snapshot id → live snapshot
    pub snapshots: Cache<String, Arc<Snapshot>>,
    /// viewer did → authors that yielded content recently (for cohort sampling)
    pub activity: Cache<String, Arc<Vec<String>>>,
}

impl Caches {
    pub fn new() -> Self {
        Self {
            did: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(24 * 3600))
                .build(),
            follows: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            author_feed: Cache::builder()
                .max_capacity(20_000)
                .time_to_live(Duration::from_secs(300))
                .build(),
            std_docs: Cache::builder()
                .max_capacity(20_000)
                .expire_after(StdDocsExpiry)
                .build(),
            steam_trailers: Cache::builder()
                .max_capacity(5_000)
                .time_to_live(Duration::from_secs(24 * 3600))
                .build(),
            steam_featured: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_secs(6 * 3600))
                .build(),
            snapshots: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(1800))
                .build(),
            activity: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(24 * 3600))
                .build(),
        }
    }
}

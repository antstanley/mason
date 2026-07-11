use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

use crate::algo::snapshot::Snapshot;
use crate::model::Brick;
use crate::sources::bluesky::Follow;

/// All in-memory state. TTLs per the plan; capacities keep a small
/// deployment bounded.
pub struct Caches {
    /// handle → did
    pub did: Cache<String, String>,
    /// did → follows
    pub follows: Cache<String, Arc<Vec<Follow>>>,
    /// author did → their recent bricks
    pub author_feed: Cache<String, Arc<Vec<Brick>>>,
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

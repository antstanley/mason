//! In-memory TTL caches. Hand-rolled (moka doesn't support wasm): a plain
//! HashMap behind an async Mutex with per-entry expiry and a soft capacity.
//! Values are Arcs everywhere, so `get` clones are cheap.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use crate::algo::snapshot::Snapshot;
use crate::model::Brick;
use crate::platform::Instant;
use crate::sources::bluesky::{AuthorYield, Follow};
use crate::sources::streamplace::LiveStream;

/// One author's standard.site yield.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct StdDocs {
    pub bricks: Vec<Brick>,
    /// post URIs suppressed via bskyPostRef; the blog card wins
    pub suppressed_posts: Vec<String>,
}

pub struct TtlCache<K, V> {
    entries: Mutex<HashMap<K, Entry<V>>>,
    default_ttl: Duration,
    max_capacity: usize,
}

struct Entry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Eq + Hash + Clone, V: Clone> TtlCache<K, V> {
    pub fn new(default_ttl: Duration, max_capacity: usize) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            default_ttl,
            max_capacity,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.lock().await;
        match entries.get(key) {
            Some(entry) if entry.expires_at > Instant::now() => Some(entry.value.clone()),
            Some(_) => {
                entries.remove(key);
                None
            }
            None => None,
        }
    }

    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await;
    }

    /// Atomic fetch-or-create under the cache lock. Returns (value, inserted);
    /// exactly one concurrent caller observes inserted == true.
    pub async fn get_or_insert_with(&self, key: K, make: impl FnOnce() -> V) -> (V, bool) {
        let mut entries = self.entries.lock().await;
        if let Some(entry) = entries.get(&key)
            && entry.expires_at > Instant::now()
        {
            return (entry.value.clone(), false);
        }
        let value = make();
        self.trim(&mut entries);
        entries.insert(
            key,
            Entry {
                value: value.clone(),
                expires_at: Instant::now() + self.default_ttl,
            },
        );
        (value, true)
    }

    /// Keep the map bounded before an insert: drop expired entries, then, if
    /// still at or over capacity, evict soonest-to-expire entries until under
    /// it. Shared by every insert path so the cache stays bounded however it
    /// was filled (snapshots arrive only through get_or_insert_with, and each
    /// pins an Arc of up to a few hundred bricks, so an untrimmed path grows
    /// without bound in a long-lived server).
    fn trim(&self, entries: &mut HashMap<K, Entry<V>>) {
        if entries.len() < self.max_capacity {
            return;
        }
        let now = Instant::now();
        entries.retain(|_, e| e.expires_at > now);
        if entries.len() < self.max_capacity {
            return;
        }
        // Still over the ceiling with everything live: evict the soonest-to-
        // expire entries in one batch. Removing a single min per insert costs
        // an O(n) scan on every insert once a 20k cache is full; dropping a
        // slice (~10%) amortizes that over the inserts that follow.
        let batch = (self.max_capacity / 10).max(1);
        let target = self.max_capacity.saturating_sub(batch);
        let drop_count = entries.len() - target;
        let mut by_expiry: Vec<(Instant, K)> = entries
            .iter()
            .map(|(k, e)| (e.expires_at, k.clone()))
            .collect();
        // soonest-to-expire first
        by_expiry.sort_by_key(|(expires_at, _)| *expires_at);
        for (_, key) in by_expiry.into_iter().take(drop_count) {
            entries.remove(&key);
        }
    }

    /// Live entries as (key, unwrapped value, absolute unix-ms expiry) -
    /// Instants don't survive process death, wall-clock timestamps do.
    pub async fn export_map<T>(&self, unwrap: impl Fn(&V) -> T) -> Vec<(K, T, u64)> {
        let now = Instant::now();
        let now_unix = crate::platform::unix_now_ms();
        let entries = self.entries.lock().await;
        entries
            .iter()
            .filter(|(_, e)| e.expires_at > now)
            .map(|(k, e)| {
                let remaining = e.expires_at.saturating_duration_since(now);
                (
                    k.clone(),
                    unwrap(&e.value),
                    now_unix + remaining.as_millis() as u64,
                )
            })
            .collect()
    }

    /// Restore exported entries, dropping anything already expired.
    pub async fn import_map<T>(&self, entries: Vec<(K, T, u64)>, wrap: impl Fn(T) -> V) {
        let now_unix = crate::platform::unix_now_ms();
        for (key, value, expires_unix) in entries {
            if expires_unix > now_unix {
                let ttl = Duration::from_millis(expires_unix - now_unix);
                self.insert_with_ttl(key, wrap(value), ttl).await;
            }
        }
    }

    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut entries = self.entries.lock().await;
        self.trim(&mut entries);
        entries.insert(
            key,
            Entry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );
    }
}

const HOUR: Duration = Duration::from_secs(3600);

pub const STD_DOCS_POSITIVE_TTL: Duration = Duration::from_secs(900);
pub const STD_DOCS_NEGATIVE_TTL: Duration = Duration::from_secs(24 * 3600);

pub const STREAMS_POSITIVE_TTL: Duration = Duration::from_secs(1800);
pub const STREAMS_NEGATIVE_TTL: Duration = Duration::from_secs(24 * 3600);
/// Long enough to serve a whole snapshot's fan-out from one call, short enough
/// that "live" stays true.
pub const LIVE_TTL: Duration = Duration::from_secs(60);

/// All in-memory state. TTLs per the plan; capacities keep a small
/// deployment bounded.
pub struct Caches {
    /// handle → did (24h)
    pub did: TtlCache<String, String>,
    /// did → follows (1h)
    pub follows: TtlCache<String, Arc<Vec<Follow>>>,
    /// author did → their recent bricks (5min)
    pub author_feed: TtlCache<String, Arc<AuthorYield>>,
    /// author did → their recent MEDIA bricks, read deeper (posts_with_media,
    /// 100) for the glaze wall (5min). Kept apart from `author_feed` so the two
    /// walls' different reads of the same author never clobber each other.
    pub image_feed: TtlCache<String, Arc<AuthorYield>>,
    /// author did → standard.site docs; publishers 15min, negatives 24h
    /// (callers pick the TTL via insert_with_ttl)
    pub std_docs: TtlCache<String, Arc<StdDocs>>,
    /// author did → their PDS endpoint (24h). Identity moves rarely, and
    /// every repo we read (blogs, streams, blobs) needs the answer.
    pub pds: TtlCache<String, String>,
    /// author did → their archived Streamplace videos; streamers 30min,
    /// negatives 24h (most people have never streamed)
    pub streams: TtlCache<String, Arc<Vec<Brick>>>,
    /// the whole live network, single key (60s). It is the one thing on the
    /// wall with a deadline, so it is the one thing barely cached. Streams,
    /// not bricks: what is cached here is true for every viewer, and the
    /// per-viewer filter happens downstream.
    pub live: TtlCache<u8, Arc<Vec<LiveStream>>>,
    /// wall-owner did → did they opt out of logged-out visibility (1h). One
    /// getProfile per cold wall, no more; the owner's own profile is not
    /// otherwise fetched by the fill.
    pub profiles: TtlCache<String, bool>,
    /// snapshot id → live snapshot (30min)
    pub snapshots: TtlCache<String, Arc<Snapshot>>,
    /// viewer did → authors that yielded content recently (24h)
    pub activity: TtlCache<String, Arc<Vec<String>>>,
}

impl Default for Caches {
    fn default() -> Self {
        Self::new()
    }
}

impl Caches {
    pub fn new() -> Self {
        Self {
            did: TtlCache::new(24 * HOUR, 10_000),
            follows: TtlCache::new(HOUR, 1_000),
            author_feed: TtlCache::new(Duration::from_secs(300), 20_000),
            image_feed: TtlCache::new(Duration::from_secs(300), 20_000),
            std_docs: TtlCache::new(STD_DOCS_NEGATIVE_TTL, 20_000),
            pds: TtlCache::new(24 * HOUR, 20_000),
            streams: TtlCache::new(STREAMS_NEGATIVE_TTL, 20_000),
            live: TtlCache::new(LIVE_TTL, 1),
            profiles: TtlCache::new(HOUR, 10_000),
            snapshots: TtlCache::new(Duration::from_secs(1800), 500),
            activity: TtlCache::new(24 * HOUR, 1_000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn expires_after_ttl() {
        let cache: TtlCache<u32, u32> = TtlCache::new(Duration::from_millis(30), 10);
        cache.insert(1, 11).await;
        assert_eq!(cache.get(&1).await, Some(11));
        crate::platform::sleep(Duration::from_millis(50)).await;
        assert_eq!(cache.get(&1).await, None);
    }

    #[tokio::test]
    async fn per_entry_ttl_beats_default() {
        let cache: TtlCache<u32, u32> = TtlCache::new(Duration::from_secs(3600), 10);
        cache
            .insert_with_ttl(1, 11, Duration::from_millis(30))
            .await;
        cache.insert(2, 22).await;
        crate::platform::sleep(Duration::from_millis(50)).await;
        assert_eq!(cache.get(&1).await, None, "short-ttl entry must expire");
        assert_eq!(
            cache.get(&2).await,
            Some(22),
            "default-ttl entry must survive"
        );
    }

    #[tokio::test]
    async fn capacity_trim_keeps_cache_bounded() {
        let cache: TtlCache<u32, u32> = TtlCache::new(Duration::from_secs(3600), 3);
        for i in 0..10 {
            cache.insert(i, i).await;
        }
        let entries = cache.entries.lock().await;
        assert!(entries.len() <= 3, "cache grew to {}", entries.len());
    }
}

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

/// One author's standard.site yield.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct StdDocs {
    pub bricks: Vec<Brick>,
    /// post URIs suppressed via bskyPostRef — the blog card wins
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
        entries.insert(
            key,
            Entry {
                value: value.clone(),
                expires_at: Instant::now() + self.default_ttl,
            },
        );
        (value, true)
    }

    /// Live entries as (key, unwrapped value, absolute unix-ms expiry) —
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
        if entries.len() >= self.max_capacity {
            let now = Instant::now();
            entries.retain(|_, e| e.expires_at > now);
            // still over: drop soonest-to-expire entries
            while entries.len() >= self.max_capacity {
                let Some(key) = entries
                    .iter()
                    .min_by_key(|(_, e)| e.expires_at)
                    .map(|(k, _)| k.clone())
                else {
                    break;
                };
                entries.remove(&key);
            }
        }
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

/// All in-memory state. TTLs per the plan; capacities keep a small
/// deployment bounded.
pub struct Caches {
    /// handle → did (24h)
    pub did: TtlCache<String, String>,
    /// did → follows (1h)
    pub follows: TtlCache<String, Arc<Vec<Follow>>>,
    /// author did → their recent bricks + mentioned Steam games (5min)
    pub author_feed: TtlCache<String, Arc<AuthorYield>>,
    /// author did → standard.site docs; publishers 15min, negatives 24h
    /// (callers pick the TTL via insert_with_ttl)
    pub std_docs: TtlCache<String, Arc<StdDocs>>,
    /// appid → trailer bricks (24h)
    pub steam_trailers: TtlCache<u64, Arc<Vec<Brick>>>,
    /// featured appids, single key (6h)
    pub steam_featured: TtlCache<u8, Arc<Vec<u64>>>,
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
            std_docs: TtlCache::new(STD_DOCS_NEGATIVE_TTL, 20_000),
            steam_trailers: TtlCache::new(24 * HOUR, 5_000),
            steam_featured: TtlCache::new(6 * HOUR, 1),
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

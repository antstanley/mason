//! Cache persistence for the service-worker build. Browsers reap an idle
//! service worker after ~30s; without persistence every wake-up means a
//! cold refetch of the whole cohort. The SW exports these caches to
//! IndexedDB after serving a page and imports them on startup, turning the
//! post-idle rebuild from seconds of network fan-out into milliseconds.
//!
//! Each cache is exported on its own, under its own IndexedDB key, and only
//! when its dirty flag says it changed since the last export: persistence
//! cost scales with what changed, not with everything cached. A session
//! holding hundreds of warm AuthorYields no longer deep-clones them all per
//! page just because a handle resolved.
//!
//! Persisted: did, follows, author_feed, image_feed (the glaze wall's deep
//! media read), std_docs, pds, streams, profiles (the wall owner's logged-out
//! opt-out), and activity. Each is a warm cache that a cold wall would
//! otherwise repay in network round trips.
//!
//! NOT persisted: the live list (60 seconds from being a lie, one call to
//! rebuild) and snapshots themselves (they hold locks and in-flight state, and
//! the seed-carrying cursor already rebuilds them deterministically from the
//! warm caches above).

use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::cache::Caches;
use crate::model::Brick;
// yield types come through the sources seam, never a source submodule directly
use crate::sources::{AuthorYield, Follow, StdDocs};

/// Bump when any persisted shape changes; imports of older payloads are
/// discarded per cache (they're just caches). v3 was the last whole-bundle
/// shape under a single key; v4 is one payload per cache.
pub const VERSION: u32 = 4;

/// Every persistable cache, in export order. The service worker iterates this
/// list on startup to import whatever survived in IndexedDB.
pub const CACHE_NAMES: [&str; 9] = [
    "did",
    "follows",
    "author_feed",
    "image_feed",
    "std_docs",
    "pds",
    "streams",
    "profiles",
    "activity",
];

type Entries<K, V> = Vec<(K, V, u64)>;

/// One cache's survival kit: its live entries plus the version of the shape
/// they were written in.
#[derive(Serialize, serde::Deserialize)]
struct PersistedCache<K, V> {
    version: u32,
    entries: Entries<K, V>,
}

fn to_json<K: Serialize, V: Serialize>(entries: Entries<K, V>) -> Option<String> {
    serde_json::to_string(&PersistedCache {
        version: VERSION,
        entries,
    })
    .ok()
}

fn from_json<K: DeserializeOwned, V: DeserializeOwned>(json: &str) -> Option<Entries<K, V>> {
    let payload: PersistedCache<K, V> = serde_json::from_str(json).ok()?;
    if payload.version != VERSION {
        tracing::debug!("discarding persisted cache v{}", payload.version);
        return None;
    }
    Some(payload.entries)
}

/// Names of the caches written to since their last export. Does not clear the
/// flags; `export_cache` does, so a crash between the two calls costs nothing.
pub fn dirty_cache_names(caches: &Caches) -> Vec<&'static str> {
    let flags = [
        caches.did.is_dirty(),
        caches.follows.is_dirty(),
        caches.author_feed.is_dirty(),
        caches.image_feed.is_dirty(),
        caches.std_docs.is_dirty(),
        caches.pds.is_dirty(),
        caches.streams.is_dirty(),
        caches.profiles.is_dirty(),
        caches.activity.is_dirty(),
    ];
    CACHE_NAMES
        .iter()
        .zip(flags)
        .filter_map(|(name, dirty)| dirty.then_some(*name))
        .collect()
}

/// Serialize one cache by name, clearing its dirty flag first (an insert
/// racing the export re-sets the flag, so nothing is lost, at worst
/// re-exported). Returns None for an unknown name or a serialization failure.
pub async fn export_cache(caches: &Caches, name: &str) -> Option<String> {
    match name {
        "did" => {
            caches.did.take_dirty();
            to_json(caches.did.export_map(Clone::clone).await)
        }
        "follows" => {
            caches.follows.take_dirty();
            to_json(caches.follows.export_map(|v| v.as_ref().clone()).await)
        }
        "author_feed" => {
            caches.author_feed.take_dirty();
            to_json(caches.author_feed.export_map(|v| v.as_ref().clone()).await)
        }
        "image_feed" => {
            caches.image_feed.take_dirty();
            to_json(caches.image_feed.export_map(|v| v.as_ref().clone()).await)
        }
        "std_docs" => {
            caches.std_docs.take_dirty();
            to_json(caches.std_docs.export_map(|v| v.as_ref().clone()).await)
        }
        "pds" => {
            caches.pds.take_dirty();
            to_json(caches.pds.export_map(Clone::clone).await)
        }
        "streams" => {
            caches.streams.take_dirty();
            to_json(caches.streams.export_map(|v| v.as_ref().clone()).await)
        }
        "profiles" => {
            caches.profiles.take_dirty();
            to_json(caches.profiles.export_map(Clone::clone).await)
        }
        "activity" => {
            caches.activity.take_dirty();
            to_json(caches.activity.export_map(|v| v.as_ref().clone()).await)
        }
        _ => None,
    }
}

/// Restore one previously exported cache. Anything unparseable, stale, or
/// written by another VERSION is silently discarded; it's only a cache.
/// Importing does not mark the cache dirty: what was just read back from
/// IndexedDB is, by definition, already persisted.
pub async fn import_cache(caches: &Caches, name: &str, json: &str) {
    match name {
        "did" => {
            if let Some(entries) = from_json(json) {
                caches.did.import_map(entries, |v: String| v).await;
            }
        }
        "follows" => {
            if let Some(entries) = from_json::<String, Vec<Follow>>(json) {
                caches.follows.import_map(entries, Arc::new).await;
            }
        }
        "author_feed" => {
            if let Some(entries) = from_json::<String, AuthorYield>(json) {
                caches.author_feed.import_map(entries, Arc::new).await;
            }
        }
        "image_feed" => {
            if let Some(entries) = from_json::<String, AuthorYield>(json) {
                caches.image_feed.import_map(entries, Arc::new).await;
            }
        }
        "std_docs" => {
            if let Some(entries) = from_json::<String, StdDocs>(json) {
                caches.std_docs.import_map(entries, Arc::new).await;
            }
        }
        "pds" => {
            if let Some(entries) = from_json(json) {
                caches.pds.import_map(entries, |v: String| v).await;
            }
        }
        "streams" => {
            if let Some(entries) = from_json::<String, Vec<Brick>>(json) {
                caches.streams.import_map(entries, Arc::new).await;
            }
        }
        "profiles" => {
            if let Some(entries) = from_json(json) {
                caches.profiles.import_map(entries, |v: bool| v).await;
            }
        }
        "activity" => {
            if let Some(entries) = from_json::<String, Vec<String>>(json) {
                caches.activity.import_map(entries, Arc::new).await;
            }
        }
        _ => {}
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Export every dirty cache and import the payloads into a fresh Caches,
    /// the way the service worker does across a reap.
    async fn round_trip(source: &Caches) -> Caches {
        let restored = Caches::new();
        for name in dirty_cache_names(source) {
            let json = export_cache(source, name).await.expect("export succeeds");
            import_cache(&restored, name, &json).await;
        }
        restored
    }

    #[tokio::test]
    async fn round_trip_restores_live_entries() {
        let source = Caches::new();
        source
            .did
            .insert("alice.test".into(), "did:plc:alice".into())
            .await;
        source
            .follows
            .insert(
                "did:plc:alice".into(),
                Arc::new(vec![Follow {
                    did: "did:plc:bob".into(),
                    handle: "bob.test".into(),
                    display_name: None,
                    avatar: None,
                    labels: vec![],
                }]),
            )
            .await;
        source
            .activity
            .insert("did:plc:alice".into(), Arc::new(vec!["did:plc:bob".into()]))
            .await;

        let restored = round_trip(&source).await;
        assert_eq!(
            restored.did.get(&"alice.test".to_string()).await.as_deref(),
            Some("did:plc:alice")
        );
        let follows = restored
            .follows
            .get(&"did:plc:alice".to_string())
            .await
            .unwrap();
        assert_eq!(follows[0].handle, "bob.test");
        let activity = restored
            .activity
            .get(&"did:plc:alice".to_string())
            .await
            .unwrap();
        assert_eq!(activity[0], "did:plc:bob");
    }

    #[tokio::test]
    async fn image_feed_and_profiles_survive_the_trip() {
        let source = Caches::new();
        source
            .image_feed
            .insert(
                "did:plc:alice".into(),
                Arc::new(AuthorYield { bricks: vec![] }),
            )
            .await;
        source.profiles.insert("did:plc:alice".into(), true).await;

        let restored = round_trip(&source).await;

        assert!(
            restored
                .image_feed
                .get(&"did:plc:alice".to_string())
                .await
                .is_some(),
            "the glaze wall's media read must persist"
        );
        assert_eq!(
            restored.profiles.get(&"did:plc:alice".to_string()).await,
            Some(true),
            "the wall owner's logged-out opt-out must persist"
        );
    }

    #[tokio::test]
    async fn only_dirty_caches_are_listed_and_export_clears_the_flag() {
        let caches = Caches::new();
        assert!(
            dirty_cache_names(&caches).is_empty(),
            "a fresh Caches has nothing to persist"
        );

        caches
            .did
            .insert("alice.test".into(), "did:plc:alice".into())
            .await;
        caches.profiles.insert("did:plc:alice".into(), true).await;
        assert_eq!(dirty_cache_names(&caches), vec!["did", "profiles"]);

        export_cache(&caches, "did").await.expect("did exports");
        assert_eq!(
            dirty_cache_names(&caches),
            vec!["profiles"],
            "exporting did must clear only did's flag"
        );

        export_cache(&caches, "profiles").await.unwrap();
        assert!(dirty_cache_names(&caches).is_empty());

        caches
            .follows
            .insert("did:plc:alice".into(), Arc::new(vec![]))
            .await;
        assert_eq!(
            dirty_cache_names(&caches),
            vec!["follows"],
            "a fresh insert re-dirties exactly its own cache"
        );
    }

    #[tokio::test]
    async fn partial_export_round_trips_the_dirty_cache_only() {
        let source = Caches::new();
        source
            .did
            .insert("alice.test".into(), "did:plc:alice".into())
            .await;
        source
            .pds
            .insert("did:plc:alice".into(), "pds".into())
            .await;
        // simulate an earlier persist cycle: both flags cleared
        export_cache(&source, "did").await.unwrap();
        export_cache(&source, "pds").await.unwrap();

        // only did changes afterwards
        source
            .did
            .insert("bob.test".into(), "did:plc:bob".into())
            .await;
        let restored = round_trip(&source).await;

        assert!(
            restored.did.get(&"bob.test".to_string()).await.is_some(),
            "the changed cache must round-trip"
        );
        assert!(
            restored
                .pds
                .get(&"did:plc:alice".to_string())
                .await
                .is_none(),
            "the clean cache must not have been re-exported"
        );
    }

    #[tokio::test]
    async fn import_does_not_mark_the_cache_dirty() {
        let source = Caches::new();
        source
            .did
            .insert("alice.test".into(), "did:plc:alice".into())
            .await;
        let json = export_cache(&source, "did").await.unwrap();

        let restored = Caches::new();
        import_cache(&restored, "did", &json).await;
        assert!(
            restored.did.get(&"alice.test".to_string()).await.is_some(),
            "import must restore the entry"
        );
        assert!(
            dirty_cache_names(&restored).is_empty(),
            "a wake-up import must not trigger a full re-export"
        );
    }

    #[tokio::test]
    async fn expired_entries_do_not_survive_the_trip() {
        let source = Caches::new();
        source
            .did
            .insert_with_ttl("gone.test".into(), "did:plc:gone".into(), Duration::ZERO)
            .await;
        source
            .did
            .insert("kept.test".into(), "did:plc:kept".into())
            .await;
        crate::platform::sleep(Duration::from_millis(10)).await;

        let json = export_cache(&source, "did").await.unwrap();
        let entries: PersistedCache<String, String> = serde_json::from_str(&json).unwrap();
        assert_eq!(
            entries.entries.len(),
            1,
            "expired entry must not be exported"
        );

        let restored = Caches::new();
        import_cache(&restored, "did", &json).await;
        assert!(restored.did.get(&"gone.test".to_string()).await.is_none());
        assert!(restored.did.get(&"kept.test".to_string()).await.is_some());
    }

    #[tokio::test]
    async fn version_mismatch_is_discarded() {
        let payload = PersistedCache {
            version: 999,
            entries: vec![(
                "alice.test".to_string(),
                "did:plc:alice".to_string(),
                u64::MAX,
            )],
        };
        let json = serde_json::to_string(&payload).unwrap();

        let restored = Caches::new();
        import_cache(&restored, "did", &json).await;
        assert!(restored.did.get(&"alice.test".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn unknown_name_and_garbage_are_ignored() {
        let caches = Caches::new();
        assert!(export_cache(&caches, "kiln").await.is_none());
        import_cache(&caches, "kiln", "{}").await;
        import_cache(&caches, "did", "not json").await;
        assert!(dirty_cache_names(&caches).is_empty());
    }
}

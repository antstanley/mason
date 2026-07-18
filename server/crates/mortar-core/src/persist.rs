//! Cache persistence for the service-worker build. Browsers reap an idle
//! service worker after ~30s; without persistence every wake-up means a
//! cold refetch of the whole cohort. The SW exports this bundle to
//! IndexedDB after serving a page and imports it on startup, turning the
//! post-idle rebuild from seconds of network fan-out into milliseconds.
//!
//! Snapshots themselves are NOT persisted; they hold locks and in-flight
//! state, and the seed-carrying cursor already rebuilds them
//! deterministically from these warm caches.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::cache::{Caches, StdDocs};
use crate::model::Brick;
use crate::sources::bluesky::{AuthorYield, Follow};

/// Bump when any persisted shape changes; imports of older bundles are
/// discarded wholesale (they're just caches).
pub const VERSION: u32 = 2;

type Entries<K, V> = Vec<(K, V, u64)>;

/// The live list is deliberately absent: it is 60 seconds from being a lie,
/// and it costs one call to rebuild.
#[derive(Serialize, Deserialize)]
pub struct PersistedCaches {
    pub version: u32,
    pub did: Entries<String, String>,
    pub follows: Entries<String, Vec<Follow>>,
    pub author_feed: Entries<String, AuthorYield>,
    pub std_docs: Entries<String, StdDocs>,
    pub pds: Entries<String, String>,
    pub streams: Entries<String, Vec<Brick>>,
    pub activity: Entries<String, Vec<String>>,
}

pub async fn export(caches: &Caches) -> PersistedCaches {
    PersistedCaches {
        version: VERSION,
        did: caches.did.export_map(Clone::clone).await,
        follows: caches.follows.export_map(|v| v.as_ref().clone()).await,
        author_feed: caches.author_feed.export_map(|v| v.as_ref().clone()).await,
        std_docs: caches.std_docs.export_map(|v| v.as_ref().clone()).await,
        pds: caches.pds.export_map(Clone::clone).await,
        streams: caches.streams.export_map(|v| v.as_ref().clone()).await,
        activity: caches.activity.export_map(|v| v.as_ref().clone()).await,
    }
}

pub async fn import(caches: &Caches, persisted: PersistedCaches) {
    if persisted.version != VERSION {
        tracing::debug!("discarding persisted caches v{}", persisted.version);
        return;
    }
    caches.did.import_map(persisted.did, |v| v).await;
    caches.follows.import_map(persisted.follows, Arc::new).await;
    caches
        .author_feed
        .import_map(persisted.author_feed, Arc::new)
        .await;
    caches
        .std_docs
        .import_map(persisted.std_docs, Arc::new)
        .await;
    caches.pds.import_map(persisted.pds, |v| v).await;
    caches.streams.import_map(persisted.streams, Arc::new).await;
    caches
        .activity
        .import_map(persisted.activity, Arc::new)
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

        let json = serde_json::to_string(&export(&source).await).unwrap();

        let restored = Caches::new();
        import(&restored, serde_json::from_str(&json).unwrap()).await;
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

        let bundle = export(&source).await;
        assert_eq!(bundle.did.len(), 1, "expired entry must not be exported");

        let restored = Caches::new();
        import(&restored, bundle).await;
        assert!(restored.did.get(&"gone.test".to_string()).await.is_none());
        assert!(restored.did.get(&"kept.test".to_string()).await.is_some());
    }

    #[tokio::test]
    async fn version_mismatch_is_discarded() {
        let source = Caches::new();
        source
            .did
            .insert("alice.test".into(), "did:plc:alice".into())
            .await;
        let mut bundle = export(&source).await;
        bundle.version = 999;

        let restored = Caches::new();
        import(&restored, bundle).await;
        assert!(restored.did.get(&"alice.test".to_string()).await.is_none());
    }
}

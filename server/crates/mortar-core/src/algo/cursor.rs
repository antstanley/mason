use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};

/// Opaque pagination cursor. Carries the seed so an evicted snapshot can be
/// rebuilt deterministically from warm caches mid-scroll.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Cursor {
    /// Snapshot cache key
    pub snapshot: String,
    /// Seed for the deterministic shuffle/jitter
    pub seed: u64,
    /// Next item offset within the snapshot
    pub offset: usize,
}

pub fn encode(cursor: &Cursor) -> String {
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(cursor).expect("cursor serializes"))
}

/// Garbage or tampered input yields None; callers fall back to a fresh feed,
/// never a 500.
pub fn decode(raw: &str) -> Option<Cursor> {
    let bytes = URL_SAFE_NO_PAD.decode(raw).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn roundtrip() {
        let c = Cursor {
            snapshot: "abc123".into(),
            seed: 42,
            offset: 96,
        };
        assert_eq!(decode(&encode(&c)), Some(c));
    }

    #[test]
    fn garbage_is_none() {
        assert_eq!(decode("not!!!valid###base64"), None);
        assert_eq!(decode(""), None);
        // valid base64, invalid json
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode(b"{\"nope\":1}")), None);
    }
}

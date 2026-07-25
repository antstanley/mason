use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};

/// Opaque pagination cursor. Carries the seed so an evicted snapshot can be
/// rebuilt deterministically from warm caches mid-scroll.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Cursor {
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
            seed: 42,
            offset: 96,
        };
        assert_eq!(decode(&encode(&c)), Some(c));
    }

    /// Every cursor mason issued before the snapshot field was dropped carries a
    /// stray "snapshot" key, and a reader mid-scroll across a deploy hands one
    /// straight back. Cursor has no deny_unknown_fields, so serde must ignore the
    /// key rather than fail the decode and drop that reader onto a fresh wall at
    /// the offset they had already scrolled past.
    #[test]
    fn a_cursor_carrying_the_removed_snapshot_key_still_decodes_to_its_seed_and_offset() {
        let legacy = URL_SAFE_NO_PAD
            .encode(br#"{"snapshot":"wall-0123456789abcdef","seed":42,"offset":96}"#);
        assert_eq!(
            decode(&legacy),
            Some(Cursor {
                seed: 42,
                offset: 96
            })
        );
    }

    #[test]
    fn garbage_is_none() {
        assert_eq!(decode("not!!!valid###base64"), None);
        assert_eq!(decode(""), None);
        // valid base64, invalid json
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode(b"{\"nope\":1}")), None);
        // a required field missing, not just an unknown one
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode(b"{\"seed\":42}")), None);
    }
}

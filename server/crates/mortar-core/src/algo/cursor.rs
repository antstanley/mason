use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};

/// Opaque pagination cursor, one shape per wall source.
///
/// The two are distinguished structurally rather than by a discriminator field,
/// which is what lets every cursor mason issued before feed walls existed still
/// decode to `Wall`.
///
/// **The declared order is load-bearing.** serde tries an untagged enum's
/// variants top to bottom and takes the first that fits, so `Feed` is declared
/// first: `feed` is required and has no default, so a `{seed, offset}` payload
/// cannot match `Feed` and the order is unambiguous rather than a preference.
/// Neither variant denies unknown fields, deliberately, because that is what
/// makes a legacy cursor carrying the long-dropped `snapshot` key keep its
/// offset instead of dropping a mid-scroll reader onto a fresh wall.
///
/// The same structural decode is why a field added to either shape later could
/// make one start matching the other: keep `feed` required, keep `Feed` first,
/// and land any new field with a round-trip test for both shapes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Cursor {
    /// A feed wall. The generator holds the order, so there is no snapshot to
    /// rebuild and nothing to carry but mason's position in it.
    Feed {
        /// The upstream `app.bsky.feed.getFeed` cursor, verbatim and opaque
        feed: String,
    },
    /// A graph wall. Carries the seed so an evicted snapshot can be rebuilt
    /// deterministically from warm caches mid-scroll.
    Wall {
        /// Seed for the deterministic shuffle/jitter
        seed: u64,
        /// Next item offset within the snapshot
        offset: usize,
    },
}

pub fn encode(cursor: &Cursor) -> String {
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(cursor).expect("cursor serializes"))
}

/// Garbage or tampered input yields None; callers fall back to a fresh feed,
/// never a 500.
///
/// A well-formed cursor of the *other* wall's shape decodes happily, because
/// `decode` has no idea which wall is asking. Refusing a shape it cannot use is
/// each caller's job, and every caller treats it as no cursor at all: a fresh
/// wall, exactly as garbage gets.
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
        let c = Cursor::Wall {
            seed: 42,
            offset: 96,
        };
        assert_eq!(decode(&encode(&c)), Some(c));
    }

    #[test]
    fn a_feed_cursor_roundtrips() {
        let c = Cursor::Feed {
            feed: "3lqk2hj4xyz2t".to_string(),
        };
        assert_eq!(decode(&encode(&c)), Some(c));
    }

    /// The untagged decode is structural, so the only thing keeping the two
    /// shapes apart is that neither one's payload satisfies the other's required
    /// fields. If that ever stopped holding, a feed cursor would silently page a
    /// graph wall from a garbage offset (or the reverse), which is a wrong wall
    /// rather than a loud failure. Assert the shapes both ways round.
    #[test]
    fn neither_shape_decodes_as_the_other() {
        let feed = decode(&encode(&Cursor::Feed {
            feed: "upstream".to_string(),
        }))
        .expect("a feed cursor decodes");
        assert!(matches!(feed, Cursor::Feed { .. }));

        let wall = decode(&encode(&Cursor::Wall {
            seed: 7,
            offset: 24,
        }))
        .expect("a graph cursor decodes");
        assert!(matches!(wall, Cursor::Wall { .. }));
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
            Some(Cursor::Wall {
                seed: 42,
                offset: 96
            })
        );
    }

    /// The new shape's negative space, kept in its own test because
    /// `garbage_is_none` is the structural guard on the untagged decode and is
    /// worth leaving exactly as it was. A payload that mentions `feed` but is
    /// not the feed shape has to fall through to `Wall` and fail there too,
    /// rather than decoding to half a cursor.
    #[test]
    fn a_malformed_feed_cursor_is_none() {
        // feed present but not a string, so neither variant fits
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode(br#"{"feed":42}"#)), None);
        // an empty object names neither shape
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode(b"{}")), None);
        // half of each shape is still neither of them
        assert_eq!(
            decode(&URL_SAFE_NO_PAD.encode(br#"{"feed":null,"seed":42}"#)),
            None
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

//! Which wall a request asks for. The default `Wall` mixes every kind from
//! every source; `Glaze` is an image-only wall built from Bluesky posts alone.
//!
//! The mode is a client preference, threaded from the query string all the way
//! into the snapshot's fill, and folded into the snapshot's cache key so a
//! glaze wall and a full wall for the same actor never share a snapshot. The
//! author-feed cache underneath IS shared, moderation and blur intact, so a
//! glaze request over an actor already browsed reuses that warm data for free.

/// The two walls mason can lay for one actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    /// The full wall: posts, blogs, and video, mixed by the grout ratio.
    #[default]
    Wall,
    /// The image wall: Bluesky posts carrying images, and nothing else.
    Glaze,
}

impl Mode {
    /// Read the `mode` query value. Anything unrecognised (or absent) is the
    /// default `Wall`, so a stray parameter can never break a feed request.
    pub fn from_query(value: Option<&str>) -> Self {
        match value {
            Some("glaze") => Mode::Glaze,
            _ => Mode::Wall,
        }
    }

    /// A short, stable token folded into the snapshot id and the per-viewer
    /// activity key, keeping the two walls in separate cache namespaces.
    pub fn tag(self) -> &'static str {
        match self {
            Mode::Wall => "wall",
            Mode::Glaze => "glaze",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glaze_is_the_only_named_mode_the_rest_default_to_wall() {
        assert_eq!(Mode::from_query(Some("glaze")), Mode::Glaze);
        assert_eq!(Mode::from_query(None), Mode::Wall);
        assert_eq!(Mode::from_query(Some("wall")), Mode::Wall);
        assert_eq!(Mode::from_query(Some("nonsense")), Mode::Wall);
        assert_eq!(Mode::default(), Mode::Wall);
    }

    #[test]
    fn tags_are_distinct() {
        assert_ne!(Mode::Wall.tag(), Mode::Glaze.tag());
    }
}

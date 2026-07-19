use serde::{Deserialize, Serialize};

/// A brick is one card on the Wall. Internally-tagged so the web client gets
/// a clean discriminated union on `kind`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Brick {
    Post(PostBrick),
    Blog(BlogBrick),
    Video(VideoBrick),
}

impl Brick {
    pub fn id(&self) -> &str {
        match self {
            Brick::Post(b) => &b.id,
            Brick::Blog(b) => &b.id,
            Brick::Video(b) => &b.id,
        }
    }

    /// A brick fit for a glaze wall: a Bluesky post carrying at least one image.
    /// Native-video posts and text- or link-only posts are not image bricks, so
    /// neither reaches the image wall.
    pub fn is_image_post(&self) -> bool {
        matches!(self, Brick::Post(p) if !p.images.is_empty())
    }

    /// Cover this brick's media behind a reveal. Only posts and native videos
    /// carry a blur; blogs and archived streams come from sources the Bluesky
    /// labels never reach, so there is nothing to set on them.
    pub fn set_blur(&mut self, blur: Option<Blur>) {
        match self {
            Brick::Post(b) => b.blur = blur,
            Brick::Video(b) => b.blur = blur,
            Brick::Blog(_) => {}
        }
    }
}

/// Why a brick's media is covered on a logged-out wall. Only the soft-warn
/// tier reaches here: anything hard-hidden (adult, `!hide`, an opted-out
/// account) is dropped upstream, so a blurred brick can always be revealed.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Blur {
    /// The label that triggered the cover, e.g. `!warn`.
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageEmbed {
    pub src: String,
    pub alt: String,
    pub aspect_ratio: Option<AspectRatio>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEmbed {
    pub uri: String,
    pub title: String,
    pub description: String,
    pub thumb: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostBrick {
    pub id: String,
    /// Link to the post on bsky.app
    pub url: String,
    pub author: Author,
    pub text: String,
    pub created_at: String,
    pub like_count: u64,
    pub repost_count: u64,
    #[serde(default)]
    pub images: Vec<ImageEmbed>,
    pub external: Option<ExternalEmbed>,
    /// Set when a `!warn` label covers the media behind a reveal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur: Option<Blur>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlogBrick {
    pub id: String,
    /// Canonical URL: publication.url + document path
    pub url: String,
    pub author: Author,
    pub title: String,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub publication: Publication,
    #[serde(default)]
    pub tags: Vec<String>,
    pub published_at: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoSource {
    Bluesky,
    /// stream.place: atproto livestreaming. Live now, or an archived VOD.
    Streamplace,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoBrick {
    pub id: String,
    pub url: String,
    pub author: Author,
    pub title: String,
    pub poster: Option<String>,
    /// HLS m3u8 URL: Bluesky's `playlist`, or Streamplace's
    /// `place.stream.playback.*` (live or archived)
    pub playlist: String,
    pub aspect_ratio: Option<AspectRatio>,
    pub source: VideoSource,
    pub created_at: String,
    #[serde(default)]
    pub like_count: u64,
    /// Streamplace only: this stream is happening RIGHT NOW. The most
    /// valuable brick on the wall, and the only one with a deadline.
    #[serde(default)]
    pub live: bool,
    /// Viewers watching a live stream.
    pub viewer_count: Option<u64>,
    /// Length of an archived stream.
    pub duration_ms: Option<u64>,
    /// What the streamer says they are doing ("music", a game, …).
    pub activity: Option<String>,
    /// Set when a `!warn` label covers the poster behind a reveal. Only native
    /// Bluesky videos are ever labelled; Streamplace bricks leave this None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur: Option<Blur>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeedResponse {
    pub items: Vec<Brick>,
    /// None when the wall has no more bricks
    pub cursor: Option<String>,
    /// Only set on a preview response: whether the wall is still warming (more
    /// bricks are arriving). The client polls previews and reflows the first
    /// screen while this is true, then freezes it. Absent on committed pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warming: Option<bool>,
}

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
    Steam,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub appid: u64,
    pub name: String,
    pub header_image: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoBrick {
    pub id: String,
    pub url: String,
    /// None for Steam trailers (no atproto author)
    pub author: Option<Author>,
    pub title: String,
    pub poster: Option<String>,
    /// HLS m3u8 URL; Bluesky `playlist` or Steam `hls_h264`
    pub playlist: String,
    pub aspect_ratio: Option<AspectRatio>,
    pub source: VideoSource,
    pub game: Option<GameInfo>,
    pub created_at: String,
    #[serde(default)]
    pub like_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeedResponse {
    pub items: Vec<Brick>,
    /// None when the wall has no more bricks
    pub cursor: Option<String>,
}

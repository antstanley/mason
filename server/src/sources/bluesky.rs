//! Bluesky AppView ingestion: handle resolution, follow graph, and author
//! feeds mapped into bricks. All URLs are built from `Config::appview_base`
//! so wiremock tests can stand in for the real network.

use serde::Deserialize;

use crate::http::{Bucket, Http, HttpError};
use crate::model::{AspectRatio, Author, Brick, ExternalEmbed, ImageEmbed, PostBrick, VideoBrick, VideoSource};

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub did: String,
    pub handle: String,
}

pub async fn resolve_handle(http: &Http, base: &str, handle: &str) -> Result<String, HttpError> {
    #[derive(Deserialize)]
    struct Resolved {
        did: String,
    }
    let url = format!("{base}/xrpc/com.atproto.identity.resolveHandle?handle={handle}");
    Ok(http.get_json::<Resolved>(&url, Bucket::Appview).await?.did)
}

/// Full follow list, paginated 100 at a time (AppView max).
pub async fn get_follows(http: &Http, base: &str, did: &str) -> Result<Vec<Follow>, HttpError> {
    #[derive(Deserialize)]
    struct FollowsPage {
        follows: Vec<Follow>,
        cursor: Option<String>,
    }

    let mut follows = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let mut url = format!("{base}/xrpc/app.bsky.graph.getFollows?actor={did}&limit=100");
        if let Some(c) = &cursor {
            url.push_str(&format!("&cursor={c}"));
        }
        let page: FollowsPage = http.get_json(&url, Bucket::Appview).await?;
        follows.extend(page.follows);
        cursor = page.cursor;
        // hard stop at 2000 follows — the cohort sampler doesn't need more
        if cursor.is_none() || follows.len() >= 2000 {
            return Ok(follows);
        }
    }
}

/// One author's recent posts as bricks. Replies excluded upstream; reposts
/// (reason != null) dropped here so nothing is double-counted.
pub async fn get_author_feed(http: &Http, base: &str, did: &str) -> Result<Vec<Brick>, HttpError> {
    let url = format!(
        "{base}/xrpc/app.bsky.feed.getAuthorFeed?actor={did}&limit=30&filter=posts_no_replies"
    );
    let page: AuthorFeed = http.get_json(&url, Bucket::Appview).await?;
    Ok(page
        .feed
        .into_iter()
        .filter(|item| item.reason.is_none())
        .filter_map(|item| post_to_brick(item.post))
        .collect())
}

#[derive(Deserialize)]
struct AuthorFeed {
    feed: Vec<FeedItem>,
}

#[derive(Deserialize)]
struct FeedItem {
    post: PostView,
    reason: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostView {
    uri: String,
    author: AuthorView,
    record: PostRecord,
    embed: Option<EmbedView>,
    #[serde(default)]
    like_count: u64,
    #[serde(default)]
    repost_count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorView {
    did: String,
    handle: String,
    display_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostRecord {
    #[serde(default)]
    text: String,
    created_at: String,
    #[serde(default)]
    facets: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(tag = "$type")]
enum EmbedView {
    #[serde(rename = "app.bsky.embed.images#view")]
    Images { images: Vec<ImageView> },
    #[serde(rename = "app.bsky.embed.video#view")]
    Video(VideoView),
    #[serde(rename = "app.bsky.embed.external#view")]
    External { external: ExternalView },
    #[serde(rename = "app.bsky.embed.recordWithMedia#view")]
    RecordWithMedia { media: Box<EmbedView> },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageView {
    thumb: String,
    #[serde(default)]
    alt: String,
    aspect_ratio: Option<AspectRatioView>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoView {
    playlist: String,
    thumbnail: Option<String>,
    aspect_ratio: Option<AspectRatioView>,
}

#[derive(Deserialize)]
struct AspectRatioView {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExternalView {
    uri: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
    thumb: Option<String>,
}

impl From<AspectRatioView> for AspectRatio {
    fn from(v: AspectRatioView) -> Self {
        AspectRatio { width: v.width, height: v.height }
    }
}

fn bsky_url(handle: &str, uri: &str) -> String {
    let rkey = uri.rsplit('/').next().unwrap_or_default();
    format!("https://bsky.app/profile/{handle}/post/{rkey}")
}

/// Map a post view to a brick. Posts whose media is a native video become
/// video bricks; everything else is a post brick.
fn post_to_brick(post: PostView) -> Option<Brick> {
    let author = Author {
        did: post.author.did,
        handle: post.author.handle,
        display_name: post.author.display_name,
        avatar: post.author.avatar,
    };
    let url = bsky_url(&author.handle, &post.uri);

    // unwrap recordWithMedia to its media half
    let embed = match post.embed {
        Some(EmbedView::RecordWithMedia { media }) => Some(*media),
        other => other,
    };

    match embed {
        Some(EmbedView::Video(video)) => Some(Brick::Video(VideoBrick {
            id: post.uri.clone(),
            url,
            author: Some(author),
            title: post.record.text,
            poster: video.thumbnail,
            playlist: video.playlist,
            aspect_ratio: video.aspect_ratio.map(Into::into),
            source: VideoSource::Bluesky,
            game: None,
            created_at: post.record.created_at,
            like_count: post.like_count,
        })),
        embed => {
            let (images, external) = match embed {
                Some(EmbedView::Images { images }) => (
                    images
                        .into_iter()
                        .map(|img| ImageEmbed {
                            src: img.thumb,
                            alt: img.alt,
                            aspect_ratio: img.aspect_ratio.map(Into::into),
                        })
                        .collect(),
                    None,
                ),
                Some(EmbedView::External { external }) => (
                    Vec::new(),
                    Some(ExternalEmbed {
                        uri: external.uri,
                        title: external.title,
                        description: external.description,
                        thumb: external.thumb,
                    }),
                ),
                _ => (Vec::new(), None),
            };
            // text-only posts with no media and no text are not wall-worthy
            if post.record.text.is_empty() && images.is_empty() && external.is_none() {
                return None;
            }
            let _ = &post.record.facets; // facets consumed by the Steam source in M5
            Some(Brick::Post(PostBrick {
                id: post.uri,
                url,
                author,
                text: post.record.text,
                created_at: post.record.created_at,
                like_count: post.like_count,
                repost_count: post.repost_count,
                images,
                external,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Http;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn post_json(uri: &str, embed: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "post": {
                "uri": uri,
                "author": {"did": "did:plc:aa", "handle": "a.test", "displayName": "A"},
                "record": {"text": "hello wall", "createdAt": "2026-07-10T12:00:00Z"},
                "embed": embed,
                "likeCount": 7,
                "repostCount": 2
            }
        })
    }

    #[tokio::test]
    async fn author_feed_parses_video_and_drops_reposts() {
        let server = MockServer::start().await;
        let video_embed = serde_json::json!({
            "$type": "app.bsky.embed.video#view",
            "playlist": "https://video.bsky.app/hls/playlist.m3u8",
            "thumbnail": "https://video.bsky.app/thumb.jpg",
            "aspectRatio": {"width": 16, "height": 9}
        });
        let mut repost = post_json("at://did:plc:bb/app.bsky.feed.post/2", serde_json::Value::Null);
        repost["reason"] = serde_json::json!({"$type": "app.bsky.feed.defs#reasonRepost"});

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .and(query_param("filter", "posts_no_replies"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "feed": [post_json("at://did:plc:aa/app.bsky.feed.post/1", video_embed), repost]
            })))
            .mount(&server)
            .await;

        let bricks = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa").await.unwrap();
        assert_eq!(bricks.len(), 1, "repost must be dropped");
        match &bricks[0] {
            Brick::Video(v) => {
                assert_eq!(v.playlist, "https://video.bsky.app/hls/playlist.m3u8");
                assert_eq!(v.poster.as_deref(), Some("https://video.bsky.app/thumb.jpg"));
                assert_eq!(v.aspect_ratio.unwrap().width, 16);
                assert_eq!(v.source, VideoSource::Bluesky);
            }
            other => panic!("expected video brick, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn follows_pagination_threads_cursor() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .and(query_param("cursor", "page2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": "did:plc:cc", "handle": "c.test"}]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": "did:plc:bb", "handle": "b.test"}],
                "cursor": "page2"
            })))
            .mount(&server)
            .await;

        let follows = get_follows(&Http::new(), &server.uri(), "did:plc:aa").await.unwrap();
        assert_eq!(follows.len(), 2);
        assert_eq!(follows[1].did, "did:plc:cc");
    }

    #[tokio::test]
    async fn retries_on_429_then_succeeds() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.identity.resolveHandle"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.identity.resolveHandle"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"did": "did:plc:aa"})),
            )
            .mount(&server)
            .await;

        let did = resolve_handle(&Http::new(), &server.uri(), "a.test").await.unwrap();
        assert_eq!(did, "did:plc:aa");
    }
}

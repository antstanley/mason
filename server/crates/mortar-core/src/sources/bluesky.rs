//! Bluesky AppView ingestion: handle resolution, follow graph, and author
//! feeds mapped into bricks. All URLs are built from `Config::appview_base`
//! so wiremock tests can stand in for the real network.

use serde::{Deserialize, Serialize};

use crate::http::{Bucket, Http, HttpError};
use crate::model::{
    AspectRatio, Author, Blur, Brick, ExternalEmbed, ImageEmbed, PostBrick, VideoBrick, VideoSource,
};

/// One author's recent posts, videos among them.
#[derive(Serialize, Deserialize, Clone)]
pub struct AuthorYield {
    pub bricks: Vec<Brick>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub did: String,
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    /// Self-labels, carried on every profile view getFollows returns. This is
    /// what lets an opted-out account be dropped from the cohort BEFORE any of
    /// its content (posts, blogs, streams) is ever fetched.
    #[serde(default)]
    pub labels: Vec<Label>,
}

impl Follow {
    /// Whether a logged-out mason must leave this followed account off the wall
    /// entirely: they opted out of logged-out visibility, or their account is
    /// flagged adult or graphic. Excluding them from the cohort skips every
    /// source at once (posts, blogs, archived streams, live).
    pub fn hidden(&self) -> bool {
        hidden_from_logged_out(&self.labels)
    }
}

impl From<&Follow> for Author {
    fn from(f: &Follow) -> Self {
        Author {
            did: f.did.clone(),
            handle: f.handle.clone(),
            display_name: f.display_name.clone(),
            avatar: f.avatar.clone(),
        }
    }
}

/// The reserved self-label an account sets to say "only signed-in people may
/// see me". mason is a logged-out reader by design, so wherever this label
/// rides along we treat it as a hard no. It is a request, not a guarantee (the
/// public AppView still serves the content), which is exactly why the client
/// has to honour it: nothing upstream does it for us.
const NO_UNAUTHENTICATED: &str = "!no-unauthenticated";

/// Labels that keep a subject off a logged-out wall entirely, mirroring what a
/// logged-out Bluesky viewer is shown: the reserved hard-hide, the logged-out
/// opt-out, and the adult-flagged media (which needs a signed-in, adult
/// account). `nudity` is deliberately absent: it carries no adult flag and
/// Bluesky shows it to logged-out viewers, so we do too. Labeler labels (the
/// default moderation service) and self-labels both land in the same `labels`
/// array, so this one check covers both.
const HIDDEN_LABELS: [&str; 5] = [
    "!hide",
    NO_UNAUTHENTICATED,
    "porn",
    "sexual",
    "graphic-media",
];

/// Labels that keep the subject but cover its media behind a reveal, again as
/// Bluesky does for a logged-out viewer. Anything in `HIDDEN_LABELS` is dropped
/// before this tier is consulted, so a blurred brick can always be revealed.
const WARN_LABELS: [&str; 1] = ["!warn"];

/// One label as the AppView reports it. Only `val` matters to us; the rest of
/// the object (src, uri, cts, …) is discarded.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Label {
    val: String,
}

/// Whether this set of labels carries the logged-out opt-out. Kept distinct
/// from `hidden_from_logged_out` because only this one means "sign in to view":
/// it is what seals a wall owner, where a hard-hide or adult label would not.
fn wants_auth(labels: &[Label]) -> bool {
    labels.iter().any(|l| l.val == NO_UNAUTHENTICATED)
}

/// Whether a subject carrying these labels must be kept off a logged-out wall
/// entirely. Used for authors (in the cohort and live filters) and for
/// individual posts, so a hard-hidden or adult post from an otherwise-visible
/// account is dropped too.
fn hidden_from_logged_out(labels: &[Label]) -> bool {
    labels
        .iter()
        .any(|l| HIDDEN_LABELS.contains(&l.val.as_str()))
}

/// Whether these labels ask for the media to be covered behind a reveal rather
/// than dropped. Only the soft-warn tier; hard-hides never reach here.
fn warned_for_logged_out(labels: &[Label]) -> bool {
    labels.iter().any(|l| WARN_LABELS.contains(&l.val.as_str()))
}

pub async fn resolve_handle(http: &Http, base: &str, handle: &str) -> Result<String, HttpError> {
    #[derive(Deserialize)]
    struct Resolved {
        did: String,
    }
    let url = format!("{base}/xrpc/com.atproto.identity.resolveHandle?handle={handle}");
    Ok(http.get_json::<Resolved>(&url, Bucket::Appview).await?.did)
}

/// Whether an account has opted out of being shown to logged-out viewers.
///
/// The follow-graph and author-feed paths never fetch the wall owner's OWN
/// profile (their posts are not on their wall), so their opt-out never rides
/// along; this is the one call that surfaces it. The label lives in the
/// `labels` array of `getProfile`.
pub async fn get_profile_optout(http: &Http, base: &str, actor: &str) -> Result<bool, HttpError> {
    #[derive(Deserialize)]
    struct ProfileView {
        #[serde(default)]
        labels: Vec<Label>,
    }
    let url = format!("{base}/xrpc/app.bsky.actor.getProfile?actor={actor}");
    let profile: ProfileView = http.get_json(&url, Bucket::Appview).await?;
    Ok(wants_auth(&profile.labels))
}

/// Follow-graph pages (100 at a time, the AppView maximum), threaded through
/// the cursor.
///
/// Each page is a round trip that cannot start until the previous one lands,
/// so `max_pages` is the caller's patience: someone with 2000 follows costs
/// twenty sequential requests, and a wall that waits for all of them has not
/// begun to fetch a single post ten seconds in. The returned cursor is `Some`
/// when there is more graph behind it, so a caller can take a head start now
/// and finish the job later.
pub async fn get_follows(
    http: &Http,
    base: &str,
    did: &str,
    from: Option<String>,
    max_pages: usize,
) -> Result<(Vec<Follow>, Option<String>), HttpError> {
    #[derive(Deserialize)]
    struct FollowsPage {
        follows: Vec<Follow>,
        cursor: Option<String>,
    }

    let mut follows = Vec::new();
    let mut cursor = from;
    for _ in 0..max_pages {
        let mut url = format!("{base}/xrpc/app.bsky.graph.getFollows?actor={did}&limit=100");
        if let Some(c) = &cursor {
            url.push_str(&format!("&cursor={c}"));
        }
        let page: FollowsPage = http.get_json(&url, Bucket::Appview).await?;
        follows.extend(page.follows);
        cursor = page.cursor;
        if cursor.is_none() {
            break;
        }
    }
    Ok((follows, cursor))
}

/// One author's recent posts as bricks. Replies are excluded upstream; reposts
/// (reason != null) are dropped here so nothing is double-counted.
pub async fn get_author_feed(http: &Http, base: &str, did: &str) -> Result<AuthorYield, HttpError> {
    let url = format!(
        "{base}/xrpc/app.bsky.feed.getAuthorFeed?actor={did}&limit=30&filter=posts_no_replies"
    );
    let page: AuthorFeed = http.get_json(&url, Bucket::Appview).await?;

    let bricks = page
        .feed
        .into_iter()
        .filter(|item| item.reason.is_none())
        // drop anything a logged-out viewer must not see at all: an author who
        // opted out or is hard-hidden or adult (yields nothing, like a private
        // feed), and any single post the same is true of
        .filter(|item| {
            !hidden_from_logged_out(&item.post.author.labels)
                && !hidden_from_logged_out(&item.post.labels)
        })
        .filter_map(|item| {
            // a soft `!warn`, on the post or its account, covers the media
            // behind a reveal instead of dropping the brick
            let warned = warned_for_logged_out(&item.post.author.labels)
                || warned_for_logged_out(&item.post.labels);
            let mut brick = post_to_brick(item.post)?;
            if warned {
                brick.set_blur(Some(Blur {
                    label: "!warn".into(),
                }));
            }
            Some(brick)
        })
        .collect();
    Ok(AuthorYield { bricks })
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
    /// Per-post labels: where adult/graphic self-labels and moderation-labeler
    /// labels land, on posts from accounts that are otherwise visible.
    #[serde(default)]
    labels: Vec<Label>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthorView {
    did: String,
    handle: String,
    display_name: Option<String>,
    avatar: Option<String>,
    /// Self-labels ride along on every post's author; this is where a followed
    /// account's logged-out opt-out reaches us.
    #[serde(default)]
    labels: Vec<Label>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostRecord {
    #[serde(default)]
    text: String,
    created_at: String,
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
        AspectRatio {
            width: v.width,
            height: v.height,
        }
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
            author,
            title: post.record.text,
            poster: video.thumbnail,
            playlist: video.playlist,
            aspect_ratio: video.aspect_ratio.map(Into::into),
            source: VideoSource::Bluesky,
            created_at: post.record.created_at,
            like_count: post.like_count,
            live: false,
            viewer_count: None,
            duration_ms: None,
            activity: None,
            blur: None,
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
                blur: None,
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
        let mut repost = post_json(
            "at://did:plc:bb/app.bsky.feed.post/2",
            serde_json::Value::Null,
        );
        repost["reason"] = serde_json::json!({"$type": "app.bsky.feed.defs#reasonRepost"});

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .and(query_param("filter", "posts_no_replies"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "feed": [post_json("at://did:plc:aa/app.bsky.feed.post/1", video_embed), repost]
            })))
            .mount(&server)
            .await;

        let AuthorYield { bricks, .. } = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa")
            .await
            .unwrap();
        assert_eq!(bricks.len(), 1, "repost must be dropped");
        match &bricks[0] {
            Brick::Video(v) => {
                assert_eq!(v.playlist, "https://video.bsky.app/hls/playlist.m3u8");
                assert_eq!(
                    v.poster.as_deref(),
                    Some("https://video.bsky.app/thumb.jpg")
                );
                assert_eq!(v.aspect_ratio.unwrap().width, 16);
                assert_eq!(v.source, VideoSource::Bluesky);
            }
            other => panic!("expected video brick, got {other:?}"),
        }
    }

    /// A followed account that opted out of logged-out visibility yields no
    /// bricks: the label rides on each post's author, and we drop it there.
    #[tokio::test]
    async fn author_feed_drops_a_no_unauthenticated_author() {
        let server = MockServer::start().await;
        let mut opted_out = post_json(
            "at://did:plc:aa/app.bsky.feed.post/1",
            serde_json::Value::Null,
        );
        opted_out["post"]["author"]["labels"] = serde_json::json!([{"val": "!no-unauthenticated"}]);

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "feed": [opted_out]
            })))
            .mount(&server)
            .await;

        let AuthorYield { bricks, .. } = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa")
            .await
            .unwrap();
        assert!(
            bricks.is_empty(),
            "a logged-out opt-out must keep the author off the wall"
        );
    }

    /// Adult and graphic media is dropped for a logged-out wall, whether the
    /// label sits on the post itself or comes from the moderation labeler. Here
    /// the account is otherwise visible; only the flagged post goes.
    #[tokio::test]
    async fn author_feed_drops_adult_posts() {
        let server = MockServer::start().await;
        let clean = post_json(
            "at://did:plc:aa/app.bsky.feed.post/1",
            serde_json::Value::Null,
        );
        let mut adult = post_json(
            "at://did:plc:aa/app.bsky.feed.post/2",
            serde_json::Value::Null,
        );
        adult["post"]["labels"] = serde_json::json!([{"val": "porn"}]);

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "feed": [clean, adult]
            })))
            .mount(&server)
            .await;

        let AuthorYield { bricks, .. } = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa")
            .await
            .unwrap();
        assert_eq!(bricks.len(), 1, "only the unlabelled post survives");
        assert_eq!(bricks[0].id(), "at://did:plc:aa/app.bsky.feed.post/1");
    }

    /// nudity is not adult-flagged, and Bluesky shows it to logged-out viewers,
    /// so mason keeps it too: shown, and not even blurred.
    #[tokio::test]
    async fn author_feed_shows_nudity() {
        let server = MockServer::start().await;
        let mut nude = post_json(
            "at://did:plc:aa/app.bsky.feed.post/1",
            serde_json::Value::Null,
        );
        nude["post"]["labels"] = serde_json::json!([{"val": "nudity"}]);

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "feed": [nude] })),
            )
            .mount(&server)
            .await;

        let AuthorYield { bricks, .. } = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa")
            .await
            .unwrap();
        assert_eq!(bricks.len(), 1, "nudity stays on a logged-out wall");
        match &bricks[0] {
            Brick::Post(p) => assert!(p.blur.is_none(), "and it is not blurred"),
            other => panic!("expected a post brick, got {other:?}"),
        }
    }

    /// a soft `!warn` keeps the post but covers its media behind a reveal.
    #[tokio::test]
    async fn author_feed_blurs_warned_posts() {
        let server = MockServer::start().await;
        let mut warned = post_json(
            "at://did:plc:aa/app.bsky.feed.post/1",
            serde_json::Value::Null,
        );
        warned["post"]["labels"] = serde_json::json!([{"val": "!warn"}]);

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "feed": [warned] })),
            )
            .mount(&server)
            .await;

        let AuthorYield { bricks, .. } = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa")
            .await
            .unwrap();
        assert_eq!(bricks.len(), 1, "a warned post is kept, not dropped");
        match &bricks[0] {
            Brick::Post(p) => assert_eq!(
                p.blur.as_ref().map(|b| b.label.as_str()),
                Some("!warn"),
                "its media is covered behind a reveal"
            ),
            other => panic!("expected a post brick, got {other:?}"),
        }
    }

    /// a moderator `!hide` drops the post outright, like an adult one.
    #[tokio::test]
    async fn author_feed_drops_hidden_posts() {
        let server = MockServer::start().await;
        let mut hidden = post_json(
            "at://did:plc:aa/app.bsky.feed.post/1",
            serde_json::Value::Null,
        );
        hidden["post"]["labels"] = serde_json::json!([{"val": "!hide"}]);

        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.feed.getAuthorFeed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "feed": [hidden] })),
            )
            .mount(&server)
            .await;

        let AuthorYield { bricks, .. } = get_author_feed(&Http::new(), &server.uri(), "did:plc:aa")
            .await
            .unwrap();
        assert!(bricks.is_empty(), "a hard-hidden post is not laid");
    }

    #[tokio::test]
    async fn get_profile_optout_reads_the_self_label() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .and(query_param("actor", "opted.test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:opt",
                "handle": "opted.test",
                "labels": [{"val": "!no-unauthenticated"}]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.actor.getProfile"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "did": "did:plc:open",
                "handle": "open.test"
            })))
            .mount(&server)
            .await;

        assert!(
            get_profile_optout(&Http::new(), &server.uri(), "opted.test")
                .await
                .unwrap(),
            "the self-label means opted out"
        );
        assert!(
            !get_profile_optout(&Http::new(), &server.uri(), "open.test")
                .await
                .unwrap(),
            "no labels means visible to everyone"
        );
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

        let (follows, cursor) = get_follows(&Http::new(), &server.uri(), "did:plc:aa", None, 10)
            .await
            .unwrap();
        assert_eq!(follows.len(), 2);
        assert_eq!(follows[1].did, "did:plc:cc");
        assert!(
            cursor.is_none(),
            "the graph ended, so there is nothing to chase"
        );
    }

    /// A big follow graph costs one blocking round trip per 100 follows, so a
    /// caller must be able to take a head start and come back for the rest.
    #[tokio::test]
    async fn a_bounded_fetch_stops_early_and_hands_back_the_cursor() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/app.bsky.graph.getFollows"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "follows": [{"did": "did:plc:bb", "handle": "b.test"}],
                "cursor": "more"
            })))
            .mount(&server)
            .await;

        let (follows, cursor) = get_follows(&Http::new(), &server.uri(), "did:plc:aa", None, 2)
            .await
            .unwrap();
        assert_eq!(
            follows.len(),
            2,
            "exactly max_pages pages, not the whole graph"
        );
        assert_eq!(
            cursor.as_deref(),
            Some("more"),
            "the rest of the graph must be reachable later"
        );
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

        let did = resolve_handle(&Http::new(), &server.uri(), "a.test")
            .await
            .unwrap();
        assert_eq!(did, "did:plc:aa");
    }
}

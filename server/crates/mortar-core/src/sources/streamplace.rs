//! Streamplace ingestion: atproto livestreaming (stream.place).
//!
//! Two shapes of content, reached two different ways:
//!
//! - **Live right now.** One call to `place.stream.live.getLiveUsers` returns
//!   everyone streaming anywhere, and the caller intersects that with the
//!   viewer's follow graph. Far cheaper than asking every author in turn
//!   whether they happen to be live, and the whole network is small enough
//!   that one page covers it.
//! - **Archived streams.** `place.stream.video` records live in the author's
//!   own repo, exactly like standard.site documents, and are listed the same
//!   way.
//!
//! Playback is HLS from `place.stream.playback.*`, which serves
//! `access-control-allow-origin: *`, so the wasm build reaches it straight
//! from the browser and the existing hls.js player handles both kinds.

use serde::Deserialize;

use crate::http::{Bucket, Http, HttpError};
use crate::model::{AspectRatio, Author, Brick, VideoBrick, VideoSource};
use crate::sources::pds::blob_url;

/// Archived streams read per author. They are long, rare, and long-lived; a
/// handful each is plenty.
const VOD_LIMIT: usize = 10;
/// The whole live network in one call (the endpoint's maximum).
const LIVE_LIMIT: usize = 100;

const WIDESCREEN: AspectRatio = AspectRatio {
    width: 16,
    height: 9,
};

/// at-uris are full of `:` and `/`, none of which may be read as query
/// structure.
fn urlencode(raw: &str) -> String {
    raw.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// The creation time encoded in a record key.
///
/// `createdAt` is required by the lexicon but only populated server-side from
/// some version onward, and older archived streams simply do not have it.
/// Dropping them would throw away real content, and an atproto rkey is a TID:
/// a base32-sortable u64 whose top bits are microseconds since the epoch. The
/// record carries its own birthday whether or not anyone wrote it down.
fn tid_created_at(rkey: &str) -> Option<String> {
    const ALPHABET: &[u8] = b"234567abcdefghijklmnopqrstuvwxyz";
    if rkey.len() != 13 {
        return None;
    }
    let mut bits: u64 = 0;
    for byte in rkey.bytes() {
        let value = ALPHABET.iter().position(|c| *c == byte)?;
        bits = (bits << 5) | value as u64;
    }
    // top bit is always 0, low 10 bits are a clock id; the middle is micros
    let micros = (bits >> 10) & ((1 << 53) - 1);
    chrono::DateTime::from_timestamp_micros(micros as i64).map(|t| t.to_rfc3339())
}

/// Where a viewer goes to watch. stream.place takes either a handle or a DID
/// in the profile position, and a handle is the one a human recognises.
fn watch_url(base: &str, author: &Author, rkey: Option<&str>) -> String {
    let who = if author.handle.is_empty() {
        &author.did
    } else {
        &author.handle
    };
    match rkey {
        Some(rkey) => format!("{base}/{who}/{rkey}"),
        None => format!("{base}/{who}"),
    }
}

// --- live -------------------------------------------------------------------

/// One person streaming right now, anywhere on the network.
///
/// Deliberately not a Brick yet. This is a fact about Streamplace, identical
/// for every viewer, which is what makes it safe to cache once for the whole
/// machine. Turning it into a brick is a fact about ONE viewer (do they follow
/// this person, and where does that person's poster live), and the two must
/// not be conflated: a cache of already-filtered bricks would serve the first
/// viewer's friends to the second one.
#[derive(Clone)]
pub struct LiveStream {
    author: Author,
    uri: String,
    title: String,
    url: Option<String>,
    thumb_cid: Option<String>,
    /// The heartbeat if there is one; a livestream record is created once and
    /// then reused for months, so its createdAt is roughly the day the
    /// streamer signed up, not the day this stream began. Without the
    /// heartbeat a four-month-old record ages out of the wall while its owner
    /// is live on it.
    created_at: String,
    activity: Option<String>,
    viewers: Option<u64>,
}

impl LiveStream {
    pub fn did(&self) -> &str {
        &self.author.did
    }

    /// The at-uri of the livestream record, which is also the brick's id.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    #[cfg(test)]
    pub fn for_test(did: &str) -> Self {
        Self {
            author: Author {
                did: did.into(),
                handle: format!("{did}.test"),
                display_name: None,
                avatar: None,
            },
            uri: format!("at://{did}/place.stream.livestream/3abc"),
            title: "live now".into(),
            url: None,
            thumb_cid: None,
            created_at: "2026-07-12T13:00:00Z".into(),
            activity: None,
            viewers: Some(1),
        }
    }

    /// The poster is a blob in the streamer's own repo, so a brick needs to
    /// know where that repo is. `None` just means no poster.
    pub fn into_brick(self, base: &str, pds: Option<&str>) -> Brick {
        let poster = match (self.thumb_cid.as_deref(), pds) {
            (Some(cid), Some(pds)) => Some(blob_url(pds, self.did(), cid)),
            _ => None,
        };
        Brick::Video(VideoBrick {
            url: self
                .url
                .unwrap_or_else(|| watch_url(base, &self.author, None)),
            playlist: format!(
                "{base}/xrpc/place.stream.playback.getLivePlaylist?streamer={}",
                self.author.did
            ),
            id: self.uri,
            poster,
            title: self.title,
            created_at: self.created_at,
            author: self.author,
            aspect_ratio: Some(WIDESCREEN),
            source: VideoSource::Streamplace,
            like_count: 0,
            live: true,
            viewer_count: self.viewers,
            duration_ms: None,
            activity: self.activity,
            blur: None,
        })
    }
}

/// Everyone live on Streamplace right now. One call covers the whole network,
/// which is why the caller intersects it with the follow graph rather than
/// asking each author in turn whether they happen to be live.
pub async fn get_live(http: &Http, base: &str) -> Result<Vec<LiveStream>, HttpError> {
    #[derive(Deserialize)]
    struct LiveUsers {
        #[serde(default)]
        streams: Vec<LiveView>,
    }

    let url = format!("{base}/xrpc/place.stream.live.getLiveUsers?limit={LIVE_LIMIT}");
    let page: LiveUsers = http.get_json(&url, Bucket::Unmetered).await?;

    Ok(page
        .streams
        .into_iter()
        .filter_map(|view| {
            let record = view.record?;
            Some(LiveStream {
                author: Author {
                    did: view.author.did,
                    handle: view.author.handle,
                    display_name: view.author.display_name,
                    avatar: view.author.avatar,
                },
                uri: view.uri,
                title: record.title,
                url: record.url,
                thumb_cid: record.thumb.as_ref().and_then(BlobRef::link),
                created_at: record.last_seen_at.unwrap_or(record.created_at),
                activity: record.activity.and_then(Activity::label),
                viewers: view.viewer_count.map(|v| v.count),
            })
        })
        .collect())
}

// --- archived streams -------------------------------------------------------

/// One author's archived streams, read straight from their repo.
pub async fn get_videos(
    http: &Http,
    pds: &str,
    base: &str,
    author: &Author,
) -> Result<Vec<Brick>, HttpError> {
    let url = format!(
        "{pds}/xrpc/com.atproto.repo.listRecords?repo={}&collection=place.stream.video&limit={VOD_LIMIT}",
        author.did
    );
    let listing: ListRecords = match http.get_json(&url, Bucket::Unmetered).await {
        Ok(listing) => listing,
        // a repo that has never held the collection 400s on some PDS
        // implementations; that is just "this person does not stream"
        Err(HttpError::Status(400 | 404)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    Ok(listing
        .records
        .into_iter()
        .filter_map(|envelope| {
            let record: VideoRecord = match serde_json::from_value(envelope.value) {
                Ok(record) => record,
                Err(e) => {
                    tracing::debug!("skipping unparseable place.stream.video: {e}");
                    return None;
                }
            };
            let rkey = envelope.uri.rsplit('/').next().unwrap_or_default();
            let created_at = record.created_at.or_else(|| tid_created_at(rkey))?;
            Some(Brick::Video(VideoBrick {
                url: watch_url(base, author, Some(rkey)),
                playlist: format!(
                    "{base}/xrpc/place.stream.playback.getVideoPlaylist?uri={}",
                    urlencode(&envelope.uri)
                ),
                poster: record
                    .thumb
                    .as_ref()
                    .and_then(BlobRef::link)
                    .map(|cid| blob_url(pds, &author.did, &cid)),
                id: envelope.uri,
                title: record.title,
                author: author.clone(),
                aspect_ratio: Some(WIDESCREEN),
                source: VideoSource::Streamplace,
                created_at,
                like_count: 0,
                live: false,
                viewer_count: None,
                duration_ms: record.duration_ms,
                activity: record.activity.and_then(Activity::label),
                blur: None,
            }))
        })
        .collect())
}

// --- wire types -------------------------------------------------------------

#[derive(Deserialize)]
struct ListRecords {
    records: Vec<RecordEnvelope>,
}

#[derive(Deserialize)]
struct RecordEnvelope {
    uri: String,
    value: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoRecord {
    title: String,
    created_at: Option<String>,
    thumb: Option<BlobRef>,
    duration_ms: Option<u64>,
    activity: Option<Activity>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiveView {
    uri: String,
    author: LiveAuthor,
    record: Option<LiveRecord>,
    viewer_count: Option<ViewerCount>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiveAuthor {
    did: String,
    #[serde(default)]
    handle: String,
    display_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiveRecord {
    title: String,
    url: Option<String>,
    created_at: String,
    /// The stream's heartbeat, refreshed while it is running.
    last_seen_at: Option<String>,
    thumb: Option<BlobRef>,
    activity: Option<Activity>,
}

#[derive(Deserialize)]
struct ViewerCount {
    count: u64,
}

/// Either a labelled activity ("music") or a game record; both carry something
/// a human can read.
#[derive(Deserialize)]
struct Activity {
    label: Option<String>,
    name: Option<String>,
}

impl Activity {
    fn label(self) -> Option<String> {
        self.label.or(self.name).filter(|s| !s.is_empty())
    }
}

#[derive(Deserialize)]
struct BlobRef {
    #[serde(rename = "ref")]
    reference: Option<serde_json::Value>,
}

impl BlobRef {
    fn link(&self) -> Option<String> {
        self.reference
            .as_ref()?
            .get("$link")?
            .as_str()
            .map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn author() -> Author {
        Author {
            did: "did:plc:streamer".into(),
            handle: "streamer.test".into(),
            display_name: None,
            avatar: None,
        }
    }

    #[test]
    fn at_uris_are_encoded_into_the_playlist_query() {
        assert_eq!(
            urlencode("at://did:plc:abc/place.stream.video/3xyz"),
            "at%3A%2F%2Fdid%3Aplc%3Aabc%2Fplace.stream.video%2F3xyz"
        );
    }

    /// Real archived streams in the wild carry no createdAt (they predate the
    /// server-side field), and dropping them threw away real content. The rkey
    /// is a TID and knows its own birthday.
    #[test]
    fn an_rkey_carries_the_date_the_record_forgot() {
        // a real Streamplace rkey whose record also carries the matching
        // createdAt, so it pins the decode against ground truth to the second
        assert_eq!(
            tid_created_at("3mqh6xxje5q2x").as_deref(),
            Some("2026-07-12T12:37:15.807862+00:00"),
        );

        assert_eq!(tid_created_at("not-a-tid"), None);
        assert_eq!(
            tid_created_at("1111111111111"),
            None,
            "1 is not in the TID alphabet"
        );
    }

    #[tokio::test]
    async fn live_streams_become_live_bricks() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/place.stream.live.getLiveUsers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "streams": [{
                    "uri": "at://did:plc:streamer/place.stream.livestream/3abc",
                    "author": { "did": "did:plc:streamer", "handle": "streamer.test" },
                    "record": {
                        "title": "CD-DA Radio",
                        "url": "https://stream.place/did:plc:streamer",
                        "createdAt": "2026-03-04T04:56:36Z",
                        "lastSeenAt": "2026-07-12T13:05:10Z",
                        "thumb": { "$type": "blob", "ref": { "$link": "bafyTHUMB" } },
                        "activity": { "$type": "place.stream.defs#activityLabel", "label": "music" }
                    },
                    "viewerCount": { "count": 42 }
                }]
            })))
            .mount(&server)
            .await;

        let streams = get_live(&Http::new(), &server.uri()).await.unwrap();
        assert_eq!(streams.len(), 1);
        let brick = streams
            .into_iter()
            .next()
            .unwrap()
            .into_brick(&server.uri(), Some("https://pds.test"));
        match brick {
            Brick::Video(v) => {
                assert!(v.live);
                assert_eq!(v.viewer_count, Some(42));
                assert_eq!(v.activity.as_deref(), Some("music"));
                assert_eq!(v.source, VideoSource::Streamplace);
                assert_eq!(
                    v.created_at, "2026-07-12T13:05:10Z",
                    "a live brick is dated by its heartbeat, not by when its record was made"
                );
                assert!(
                    v.playlist
                        .ends_with("getLivePlaylist?streamer=did:plc:streamer")
                );
                assert!(v.poster.as_deref().unwrap().contains("bafyTHUMB"));
            }
            other => panic!("expected a video brick, got {other:?}"),
        }
    }

    /// A streamer whose repo we cannot locate still belongs on the wall; only
    /// their poster is missing.
    #[tokio::test]
    async fn an_unresolvable_repo_costs_the_poster_not_the_brick() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/place.stream.live.getLiveUsers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "streams": [{
                    "uri": "at://did:plc:streamer/place.stream.livestream/3abc",
                    "author": { "did": "did:plc:streamer", "handle": "streamer.test" },
                    "record": {
                        "title": "still on",
                        "createdAt": "2026-07-12T12:00:00Z",
                        "thumb": { "$type": "blob", "ref": { "$link": "bafyTHUMB" } }
                    }
                }]
            })))
            .mount(&server)
            .await;

        let streams = get_live(&Http::new(), &server.uri()).await.unwrap();
        match streams.into_iter().next().unwrap().into_brick("b", None) {
            Brick::Video(v) => {
                assert!(v.live);
                assert!(v.poster.is_none());
            }
            other => panic!("expected a video brick, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn archived_streams_become_vod_bricks() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.repo.listRecords"))
            .and(query_param("collection", "place.stream.video"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "records": [{
                    "uri": "at://did:plc:streamer/place.stream.video/3vod",
                    "value": {
                        "$type": "place.stream.video",
                        "title": "Narrative strands",
                        "createdAt": "2026-03-27T22:25:00.000Z",
                        "durationMs": 940000,
                        "thumb": { "$type": "blob", "ref": { "$link": "bafyVOD" } },
                        "source": { "$type": "place.stream.media.defs#sourceClip" }
                    }
                }]
            })))
            .mount(&server)
            .await;

        let bricks = get_videos(
            &Http::new(),
            &server.uri(),
            "https://stream.place",
            &author(),
        )
        .await
        .unwrap();
        match &bricks[..] {
            [Brick::Video(v)] => {
                assert!(!v.live);
                assert_eq!(v.duration_ms, Some(940_000));
                assert_eq!(v.url, "https://stream.place/streamer.test/3vod");
                assert!(v.playlist.contains("getVideoPlaylist?uri=at%3A%2F%2F"));
                assert!(v.poster.as_deref().unwrap().contains("bafyVOD"));
            }
            other => panic!("expected one video brick, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_repo_with_no_streams_is_empty_not_an_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.repo.listRecords"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "InvalidRequest"
            })))
            .mount(&server)
            .await;

        let bricks = get_videos(
            &Http::new(),
            &server.uri(),
            "https://stream.place",
            &author(),
        )
        .await
        .unwrap();
        assert!(bricks.is_empty());
    }
}

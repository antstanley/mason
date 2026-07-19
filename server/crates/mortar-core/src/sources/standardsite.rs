//! standard.site blog ingestion: DID doc → PDS → listRecords
//! site.standard.document (+ its site.standard.publication). Blog cards are
//! metadata + link-out only; the `content` union is platform-specific
//! (Leaflet, pckt, WordPress all differ) and is never rendered.

use serde::Deserialize;

use crate::http::{Bucket, Http, HttpError};
use crate::model::{Author, BlogBrick, Brick, Publication};
use crate::sources::pds::blob_url;
use crate::sources::util::is_http_url;

/// (bricks, suppressed post uris from bskyPostRef; the blog card wins over
/// its cross-posted skeet)
pub struct StandardSiteYield {
    pub bricks: Vec<Brick>,
    pub suppressed_posts: Vec<String>,
}

/// One author's standard.site yield, as cached and persisted.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct StdDocs {
    pub bricks: Vec<Brick>,
    /// post URIs suppressed via bskyPostRef; the blog card wins
    pub suppressed_posts: Vec<String>,
}

pub async fn get_documents(
    http: &Http,
    pds: &str,
    author: &Author,
) -> Result<StandardSiteYield, HttpError> {
    let url = format!(
        "{pds}/xrpc/com.atproto.repo.listRecords?repo={}&collection=site.standard.document&limit=25",
        author.did
    );
    let listing: ListRecords = match http.get_json(&url, Bucket::Unmetered).await {
        Ok(l) => l,
        // repos without the collection 400 on some PDS implementations -
        // that's just "no blog here"
        Err(HttpError::Status(400 | 404)) => {
            return Ok(StandardSiteYield {
                bricks: Vec::new(),
                suppressed_posts: Vec::new(),
            });
        }
        Err(e) => return Err(e),
    };

    let mut bricks = Vec::new();
    let mut suppressed_posts = Vec::new();
    // Every document points at a publication, and it is nearly always the SAME
    // publication: a blogger has one blog. Fetching it per document meant 25
    // sequential getRecord calls for one author, which is what made the repo
    // fan-out take twenty seconds and left the first wall with no blogs on it.
    let mut publications: std::collections::HashMap<String, Publication> =
        std::collections::HashMap::new();
    for envelope in listing.records {
        let Some(doc) = parse_document(envelope.value) else {
            continue;
        };
        let Some(site) = doc.site.clone() else {
            continue;
        };
        let publication = match publications.get(&site) {
            Some(known) => known.clone(),
            None => {
                let fetched = fetch_publication(http, pds, &author.did, &site).await;
                publications.insert(site.clone(), fetched.clone());
                fetched
            }
        };
        let url = canonical_url(&doc, &publication);
        if let Some(uri) = doc.bsky_post_ref.as_ref().and_then(|r| r.uri(&author.did)) {
            suppressed_posts.push(uri);
        }
        bricks.push(Brick::Blog(BlogBrick {
            id: envelope.uri,
            url,
            author: author.clone(),
            title: doc.title,
            description: doc.description.filter(|d| !d.is_empty()),
            cover_image: doc
                .cover_image
                .and_then(|blob| blob.link())
                .map(|cid| blob_url(pds, &author.did, &cid)),
            publication,
            tags: doc.tags,
            published_at: doc.published_at,
        }));
    }
    Ok(StandardSiteYield {
        bricks,
        suppressed_posts,
    })
}

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
struct DocumentRecord {
    title: String,
    /// AT-URI of the publication record (repo part may be a handle),
    /// or a plain https URL. Required by the lexicon but absent in some
    /// real records; those are skipped (a card that links nowhere is
    /// not wall-worthy).
    site: Option<String>,
    published_at: String,
    path: Option<String>,
    description: Option<String>,
    cover_image: Option<BlobRef>,
    #[serde(default)]
    tags: Vec<String>,
    bsky_post_ref: Option<BskyPostRef>,
}

/// Officially a strongRef `{uri, cid}`, but some publishers write a bare
/// string (an rkey or at-uri).
#[derive(Deserialize)]
#[serde(untagged)]
enum BskyPostRef {
    Strong(StrongRef),
    Bare(String),
}

impl BskyPostRef {
    /// Full post at-uri when derivable; bare rkeys resolve against the
    /// author's repo.
    fn uri(&self, author_did: &str) -> Option<String> {
        match self {
            BskyPostRef::Strong(r) => Some(r.uri.clone()),
            BskyPostRef::Bare(s) if s.starts_with("at://") => Some(s.clone()),
            BskyPostRef::Bare(rkey) if !rkey.is_empty() => {
                Some(format!("at://{author_did}/app.bsky.feed.post/{rkey}"))
            }
            BskyPostRef::Bare(_) => None,
        }
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

#[derive(Deserialize)]
struct StrongRef {
    uri: String,
}

/// Log-and-skip on unknown shapes; the lexicon is young, parse defensively.
fn parse_document(value: serde_json::Value) -> Option<DocumentRecord> {
    match serde_json::from_value(value) {
        Ok(doc) => Some(doc),
        Err(e) => {
            tracing::debug!("skipping unparseable site.standard.document: {e}");
            None
        }
    }
}

async fn fetch_publication(http: &Http, pds: &str, fallback_repo: &str, site: &str) -> Publication {
    // site may be a plain https URL; publication is implied
    if let Some(rest) = site.strip_prefix("https://") {
        let host = rest.split('/').next().unwrap_or(rest);
        return Publication {
            name: host.to_string(),
            url: site.to_string(),
            icon: None,
        };
    }

    // at://repo/site.standard.publication/rkey (repo may be handle or did)
    let mut parts = site.strip_prefix("at://").unwrap_or(site).splitn(3, '/');
    let repo = parts.next().unwrap_or(fallback_repo);
    let _collection = parts.next();
    let rkey = parts.next().unwrap_or("self");

    #[derive(Deserialize)]
    struct RecordResponse {
        value: PublicationRecord,
    }
    #[derive(Deserialize)]
    struct PublicationRecord {
        name: String,
        url: String,
    }

    let url = format!(
        "{pds}/xrpc/com.atproto.repo.getRecord?repo={repo}&collection=site.standard.publication&rkey={rkey}"
    );
    match http
        .get_json::<RecordResponse>(&url, Bucket::Unmetered)
        .await
    {
        Ok(r) => Publication {
            name: r.value.name,
            url: r.value.url,
            icon: None,
        },
        Err(e) => {
            tracing::debug!("publication fetch failed for {site}: {e}");
            Publication {
                name: "blog".into(),
                url: String::new(),
                icon: None,
            }
        }
    }
}

fn canonical_url(doc: &DocumentRecord, publication: &Publication) -> String {
    let base = publication.url.trim_end_matches('/');
    let url = match &doc.path {
        Some(path) if !base.is_empty() => format!("{base}{path}"),
        _ if !base.is_empty() => base.to_string(),
        _ => String::new(),
    };
    // the publication url is third-party; a link-out card must never carry a
    // javascript:/data: scheme to the anchor. Drop anything but http(s).
    if is_http_url(&url) {
        url
    } else {
        String::new()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::http::Http;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn author() -> Author {
        Author {
            did: "did:plc:blogger".into(),
            handle: "blogger.test".into(),
            display_name: None,
            avatar: None,
        }
    }

    #[tokio::test]
    async fn documents_become_blog_bricks_with_canonical_urls() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.repo.listRecords"))
            .and(query_param("collection", "site.standard.document"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "records": [
                    {
                        "uri": "at://did:plc:blogger/site.standard.document/aaa",
                        "value": {
                            "$type": "site.standard.document",
                            "title": "Why bricks?",
                            "site": "at://did:plc:blogger/site.standard.publication/self",
                            "publishedAt": "2026-07-01T00:00:00Z",
                            "path": "/why-bricks/",
                            "description": "A manifesto",
                            "coverImage": {"$type": "blob", "ref": {"$link": "bafyCOVER"}, "mimeType": "image/png", "size": 1},
                            "tags": ["walls"],
                            "bskyPostRef": {"uri": "at://did:plc:blogger/app.bsky.feed.post/xpost", "cid": "bafyPOST"},
                            "content": {"$type": "pub.leaflet.content", "whatever": true}
                        }
                    },
                    {
                        "uri": "at://did:plc:blogger/site.standard.document/bbb",
                        "value": {"$type": "site.standard.document", "totally": "malformed"}
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.repo.getRecord"))
            .and(query_param("collection", "site.standard.publication"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "value": {"name": "The Daily Brick", "url": "https://blog.example.com/"}
            })))
            .mount(&server)
            .await;

        let result = get_documents(&Http::new(), &server.uri(), &author())
            .await
            .unwrap();
        assert_eq!(
            result.bricks.len(),
            1,
            "malformed record must be skipped, not fatal"
        );
        assert_eq!(
            result.suppressed_posts,
            vec!["at://did:plc:blogger/app.bsky.feed.post/xpost"]
        );
        match &result.bricks[0] {
            Brick::Blog(b) => {
                assert_eq!(b.title, "Why bricks?");
                assert_eq!(b.url, "https://blog.example.com/why-bricks/");
                assert_eq!(b.publication.name, "The Daily Brick");
                assert!(b.cover_image.as_deref().unwrap().contains("getBlob"));
                assert!(b.cover_image.as_deref().unwrap().contains("bafyCOVER"));
            }
            other => panic!("expected blog brick, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn no_collection_is_empty_not_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.repo.listRecords"))
            .respond_with(ResponseTemplate::new(400).set_body_json(
                serde_json::json!({"error": "InvalidRequest", "message": "no such collection"}),
            ))
            .mount(&server)
            .await;

        let result = get_documents(&Http::new(), &server.uri(), &author())
            .await
            .unwrap();
        assert!(result.bricks.is_empty());
    }

    #[test]
    fn a_non_http_publication_url_yields_no_link() {
        let doc = DocumentRecord {
            title: "Trap".into(),
            site: None,
            published_at: "2026-07-01T00:00:00Z".into(),
            path: Some("/x".into()),
            description: None,
            cover_image: None,
            tags: Vec::new(),
            bsky_post_ref: None,
        };
        let hostile = Publication {
            name: "evil".into(),
            url: "javascript:alert(1)".into(),
            icon: None,
        };
        assert_eq!(canonical_url(&doc, &hostile), "");

        let ok = Publication {
            name: "blog".into(),
            url: "https://blog.example.com".into(),
            icon: None,
        };
        assert_eq!(canonical_url(&doc, &ok), "https://blog.example.com/x");
    }

    #[tokio::test]
    async fn https_site_skips_publication_fetch() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/xrpc/com.atproto.repo.listRecords"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "records": [{
                    "uri": "at://did:plc:blogger/site.standard.document/ccc",
                    "value": {
                        "title": "Plain site",
                        "site": "https://plain.example.com",
                        "publishedAt": "2026-07-02T00:00:00Z",
                        "path": "/post/"
                    }
                }]
            })))
            .mount(&server)
            .await;

        let result = get_documents(&Http::new(), &server.uri(), &author())
            .await
            .unwrap();
        match &result.bricks[0] {
            Brick::Blog(b) => {
                assert_eq!(b.publication.name, "plain.example.com");
                assert_eq!(b.url, "https://plain.example.com/post/");
            }
            other => panic!("expected blog brick, got {other:?}"),
        }
    }
}

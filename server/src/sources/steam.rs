//! Steam trailer ingestion. Two feeds into the wall:
//! 1. games the people you follow are talking about (store links extracted
//!    from their posts' text, richtext facets, and link cards)
//! 2. a small evergreen pool of featured releases as exploration filler
//!
//! The storefront API is unofficial and throttles hard: one appid per
//! appdetails call, callers bound concurrency to 2, results cached 24h.

use serde::Deserialize;

use crate::http::{Bucket, Http, HttpError};
use crate::model::{AspectRatio, Brick, GameInfo, VideoBrick, VideoSource};

/// Pull Steam appids out of any text fragment (post text, facet URI, link
/// card URL). Substring parse, no regex needed.
pub fn extract_appids(fragments: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<u64> {
    const MARKER: &str = "store.steampowered.com/app/";
    let mut appids = Vec::new();
    for fragment in fragments {
        let mut rest = fragment.as_ref();
        while let Some(at) = rest.find(MARKER) {
            rest = &rest[at + MARKER.len()..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(id) = digits.parse::<u64>()
                && !appids.contains(&id)
            {
                appids.push(id);
            }
        }
    }
    appids
}

/// Trailer bricks for one game. Empty (not an error) for age-gated or
/// delisted titles, and for games without HLS trailers.
pub async fn get_trailers(
    http: &Http,
    store_base: &str,
    appid: u64,
    hydrated_at: &str,
) -> Result<Vec<Brick>, HttpError> {
    let url = format!("{store_base}/api/appdetails?appids={appid}");
    let mut body: serde_json::Map<String, serde_json::Value> =
        http.get_json(&url, Bucket::Unmetered).await?;

    let Some(entry) = body.remove(&appid.to_string()) else {
        return Ok(Vec::new());
    };
    let entry: AppEntry = match serde_json::from_value(entry) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("appdetails {appid} unparseable: {e}");
            return Ok(Vec::new());
        }
    };
    let Some(data) = entry.data.filter(|_| entry.success) else {
        return Ok(Vec::new());
    };

    Ok(data
        .movies
        .iter()
        // prefer the trailer Steam itself highlights, fall back to the first
        .filter(|m| m.hls_h264.is_some())
        .max_by_key(|m| m.highlight)
        .map(|movie| {
            Brick::Video(VideoBrick {
                id: format!("steam-{appid}-{}", movie.id),
                url: format!("https://store.steampowered.com/app/{appid}/"),
                author: None,
                title: movie.name.clone(),
                poster: movie.thumbnail.clone().map(force_https),
                playlist: force_https(movie.hls_h264.clone().unwrap_or_default()),
                aspect_ratio: Some(AspectRatio {
                    width: 16,
                    height: 9,
                }),
                source: VideoSource::Steam,
                game: Some(GameInfo {
                    appid,
                    name: data.name.clone(),
                    header_image: data.header_image.clone().map(force_https),
                }),
                created_at: hydrated_at.to_string(),
                like_count: 0,
            })
        })
        .into_iter()
        .collect())
}

/// Appids of currently featured games (new releases + top sellers).
pub async fn get_featured(http: &Http, store_base: &str) -> Result<Vec<u64>, HttpError> {
    #[derive(Deserialize)]
    struct Featured {
        #[serde(default)]
        new_releases: Category,
        #[serde(default)]
        top_sellers: Category,
    }
    #[derive(Deserialize, Default)]
    struct Category {
        #[serde(default)]
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        id: u64,
    }

    let url = format!("{store_base}/api/featuredcategories");
    let featured: Featured = http.get_json(&url, Bucket::Unmetered).await?;
    let mut ids: Vec<u64> = featured
        .new_releases
        .items
        .iter()
        .chain(featured.top_sellers.items.iter())
        .map(|i| i.id)
        .collect();
    ids.dedup();
    ids.truncate(20);
    Ok(ids)
}

fn force_https(url: String) -> String {
    // the storefront still hands out http:// URLs sometimes
    url.strip_prefix("http://")
        .map(|rest| format!("https://{rest}"))
        .unwrap_or(url)
}

#[derive(Deserialize)]
struct AppEntry {
    success: bool,
    data: Option<AppData>,
}

#[derive(Deserialize)]
struct AppData {
    name: String,
    header_image: Option<String>,
    #[serde(default)]
    movies: Vec<Movie>,
}

#[derive(Deserialize)]
struct Movie {
    id: u64,
    name: String,
    thumbnail: Option<String>,
    hls_h264: Option<String>,
    #[serde(default)]
    highlight: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::Http;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn extracts_appids_from_mixed_fragments() {
        let ids = extract_appids([
            "check this out store.steampowered.com/app/570/Dota_2/ so good",
            "https://store.steampowered.com/app/2358720",
            "no link here",
            "dupe: store.steampowered.com/app/570",
        ]);
        assert_eq!(ids, vec![570, 2358720]);
    }

    #[tokio::test]
    async fn appdetails_maps_highlight_trailer_and_forces_https() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/appdetails"))
            .and(query_param("appids", "570"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "570": {
                    "success": true,
                    "data": {
                        "name": "Dota 2",
                        "header_image": "http://cdn.example.com/header.jpg",
                        "movies": [
                            {"id": 1, "name": "Old teaser", "thumbnail": "https://t/1.jpg",
                             "hls_h264": "https://v/1.m3u8", "highlight": false},
                            {"id": 2, "name": "Join the Battle", "thumbnail": "http://t/2.jpg",
                             "hls_h264": "http://v/2.m3u8", "highlight": true}
                        ]
                    }
                }
            })))
            .mount(&server)
            .await;

        let bricks = get_trailers(&Http::new(), &server.uri(), 570, "2026-07-11T00:00:00Z")
            .await
            .unwrap();
        assert_eq!(bricks.len(), 1);
        match &bricks[0] {
            Brick::Video(v) => {
                assert_eq!(v.title, "Join the Battle");
                assert_eq!(v.playlist, "https://v/2.m3u8");
                assert_eq!(v.poster.as_deref(), Some("https://t/2.jpg"));
                assert_eq!(v.source, VideoSource::Steam);
                assert_eq!(v.game.as_ref().unwrap().name, "Dota 2");
                assert!(
                    v.game
                        .as_ref()
                        .unwrap()
                        .header_image
                        .as_deref()
                        .unwrap()
                        .starts_with("https://")
                );
            }
            other => panic!("expected video brick, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unsuccessful_appdetails_is_empty_not_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/appdetails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "999": {"success": false}
            })))
            .mount(&server)
            .await;

        let bricks = get_trailers(&Http::new(), &server.uri(), 999, "2026-07-11T00:00:00Z")
            .await
            .unwrap();
        assert!(bricks.is_empty());
    }
}

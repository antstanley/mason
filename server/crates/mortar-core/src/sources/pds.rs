//! DID document → PDS endpoint.
//!
//! Every repo-reading source needs this (standard.site documents, Streamplace
//! videos, blobs for both), so it lives here rather than in any one of them,
//! and the snapshot caches the answer: one author costs one identity lookup no
//! matter how many collections we go on to read from their repo.

use serde::Deserialize;

use crate::http::{Bucket, Http, HttpError};

/// The PDS endpoint from the DID document: plc.directory for did:plc,
/// `/.well-known/did.json` for did:web.
pub async fn resolve(http: &Http, plc_base: &str, did: &str) -> Result<String, HttpError> {
    #[derive(Deserialize)]
    struct DidDoc {
        #[serde(default)]
        service: Vec<Service>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Service {
        id: String,
        service_endpoint: serde_json::Value,
    }

    let url = if let Some(domain) = did.strip_prefix("did:web:") {
        format!("https://{domain}/.well-known/did.json")
    } else {
        format!("{plc_base}/{did}")
    };
    let doc: DidDoc = http.get_json(&url, Bucket::Unmetered).await?;
    doc.service
        .into_iter()
        .find(|s| s.id.ends_with("atproto_pds"))
        .and_then(|s| s.service_endpoint.as_str().map(String::from))
        .ok_or(HttpError::Status(404))
}

/// A blob in someone's repo, addressed by content hash.
pub fn blob_url(pds: &str, did: &str, cid: &str) -> String {
    format!("{pds}/xrpc/com.atproto.sync.getBlob?did={did}&cid={cid}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn finds_the_pds_service_in_a_did_doc() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/did:plc:someone"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "service": [
                    { "id": "#atproto_labeler", "serviceEndpoint": "https://labels.test" },
                    { "id": "#atproto_pds", "serviceEndpoint": "https://pds.test" }
                ]
            })))
            .mount(&server)
            .await;

        let pds = resolve(&Http::new(), &server.uri(), "did:plc:someone")
            .await
            .unwrap();
        assert_eq!(pds, "https://pds.test");
    }

    #[tokio::test]
    async fn a_did_doc_without_a_pds_is_a_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/did:plc:homeless"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "service": [] })),
            )
            .mount(&server)
            .await;

        let err = resolve(&Http::new(), &server.uri(), "did:plc:homeless")
            .await
            .unwrap_err();
        assert!(matches!(err, HttpError::Status(404)), "got {err:?}");
    }
}

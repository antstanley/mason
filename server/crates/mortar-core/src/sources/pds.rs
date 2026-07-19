//! DID document → PDS endpoint.
//!
//! Every repo-reading source needs this (standard.site documents, Streamplace
//! videos, blobs for both), so it lives here rather than in any one of them,
//! and the snapshot caches the answer: one author costs one identity lookup no
//! matter how many collections we go on to read from their repo.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::Deserialize;

use crate::http::{Bucket, Http, HttpError};

/// The PDS endpoint from the DID document: plc.directory for did:plc,
/// `/.well-known/did.json` for did:web.
///
/// Both the did:web domain and the serviceEndpoint come out of an untrusted DID
/// document, and in server mode the native binary fetches them verbatim, so a
/// hostile document could aim mortar at the machine's own loopback, the cloud
/// metadata endpoint, or an internal host. Both are vetted before any request
/// leaves for them (see `validate_host` / `validate_endpoint`).
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
        if domain.contains('@') {
            return Err(HttpError::Status(403));
        }
        validate_host(host_of(domain)).await?;
        format!("https://{domain}/.well-known/did.json")
    } else {
        format!("{plc_base}/{did}")
    };
    let doc: DidDoc = http.get_json(&url, Bucket::Unmetered).await?;
    let endpoint = doc
        .service
        .into_iter()
        .find(|s| s.id.ends_with("atproto_pds"))
        .and_then(|s| s.service_endpoint.as_str().map(String::from))
        .ok_or(HttpError::Status(404))?;
    validate_endpoint(&endpoint).await?;
    Ok(endpoint)
}

/// The serviceEndpoint must be an https URL whose host is not an internal one.
/// http/file/data and friends are rejected outright, as is any userinfo (which
/// can hide the real host from a naive read).
async fn validate_endpoint(endpoint: &str) -> Result<(), HttpError> {
    let rest = strip_https_scheme(endpoint).ok_or(HttpError::Status(403))?;
    // the authority is everything up to the first path/query/fragment delimiter
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    if authority.contains('@') {
        return Err(HttpError::Status(403));
    }
    validate_host(host_of(authority)).await
}

/// Cheap-then-thorough host vetting. The literal/loopback string checks run in
/// both builds so the wasm engine rejects obviously bad hosts too; the native
/// build additionally resolves the name and rejects any private answer (a
/// hostname that points at a loopback or metadata IP). DNS lives behind a
/// native cfg because wasm has no `std::net` resolver: there the browser is the
/// client and does the fetching, so SSRF is not mortar's to prevent.
async fn validate_host(host: &str) -> Result<(), HttpError> {
    if host_is_blocked(host) {
        return Err(HttpError::Status(403));
    }
    #[cfg(not(target_arch = "wasm32"))]
    resolves_to_public_ip(host).await?;
    Ok(())
}

/// Case-insensitively require and strip an `https://` scheme, returning the rest
/// of the URL. Anything else (http, file, data, ...) yields `None`.
fn strip_https_scheme(url: &str) -> Option<&str> {
    let trimmed = url.trim();
    if trimmed.len() >= 8 && trimmed[..8].eq_ignore_ascii_case("https://") {
        Some(&trimmed[8..])
    } else {
        None
    }
}

/// The host out of an authority, dropping any `:port` and unwrapping a bracketed
/// IPv6 literal. For a did:web domain (colon-separated) this yields the first
/// segment, which is the host.
fn host_of(authority: &str) -> &str {
    if let Some(rest) = authority.strip_prefix('[') {
        return rest.split(']').next().unwrap_or(rest);
    }
    authority.split(':').next().unwrap_or(authority)
}

/// String and IP-literal host checks, no DNS: loopback and metadata hostnames,
/// and any literal address in a private, loopback, link-local, or ULA range.
fn host_is_blocked(host: &str) -> bool {
    let host = host.trim().trim_end_matches('.');
    if host.is_empty() {
        return true;
    }
    let lower = host.to_ascii_lowercase();
    if lower == "localhost" || lower.ends_with(".localhost") || lower.ends_with(".local") {
        return true;
    }
    if let Ok(v4) = lower.parse::<Ipv4Addr>() {
        return ip_is_blocked(IpAddr::V4(v4));
    }
    if let Ok(v6) = lower.parse::<Ipv6Addr>() {
        return ip_is_blocked(IpAddr::V6(v6));
    }
    false
}

/// Whether an address sits in a range mortar must never be steered onto:
/// loopback, unspecified, private (10/172.16-31/192.168, fc00::/7), or
/// link-local (169.254, fe80::/10, and thus the 169.254.169.254 metadata IP).
fn ip_is_blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() {
                return true;
            }
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return ip_is_blocked(IpAddr::V4(mapped));
            }
            let seg = v6.segments();
            // ULA fc00::/7 or link-local fe80::/10
            (seg[0] & 0xfe00) == 0xfc00 || (seg[0] & 0xffc0) == 0xfe80
        }
    }
}

/// Native DNS guard: resolve the host and reject if any answer is a private
/// address. An unresolvable host is allowed through unchanged, because a host
/// the resolver cannot reach is one the follow-up fetch cannot reach either.
#[cfg(not(target_arch = "wasm32"))]
async fn resolves_to_public_ip(host: &str) -> Result<(), HttpError> {
    use std::net::ToSocketAddrs;

    let query = format!("{host}:443");
    let addrs = tokio::task::spawn_blocking(move || {
        query
            .to_socket_addrs()
            .map(|it| it.collect::<Vec<_>>())
            .unwrap_or_default()
    })
    .await
    .map_err(|e| HttpError::Transport(e.to_string()))?;
    if addrs.iter().any(|addr| ip_is_blocked(addr.ip())) {
        return Err(HttpError::Status(403));
    }
    Ok(())
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

    #[tokio::test]
    async fn a_metadata_ip_endpoint_is_rejected() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/did:plc:evil"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "service": [
                    { "id": "#atproto_pds", "serviceEndpoint": "https://169.254.169.254/latest/meta-data" }
                ]
            })))
            .mount(&server)
            .await;

        let err = resolve(&Http::new(), &server.uri(), "did:plc:evil")
            .await
            .unwrap_err();
        assert!(
            matches!(err, HttpError::Status(403)),
            "an endpoint aimed at the metadata IP must be refused, got {err:?}"
        );
    }

    #[tokio::test]
    async fn endpoint_validation_gates_scheme_and_host() {
        // an ordinary https host passes: pds.test never resolves, so the native
        // DNS guard is a no-op and only the cheap checks apply
        assert!(validate_endpoint("https://pds.test").await.is_ok());
        // non-https schemes are refused outright
        assert!(matches!(
            validate_endpoint("http://example.com").await,
            Err(HttpError::Status(403))
        ));
        // loopback and private literals are refused before any DNS
        assert!(matches!(
            validate_endpoint("https://localhost/x").await,
            Err(HttpError::Status(403))
        ));
        assert!(matches!(
            validate_endpoint("https://10.0.0.5").await,
            Err(HttpError::Status(403))
        ));
        // userinfo cannot smuggle a private host past the read
        assert!(matches!(
            validate_endpoint("https://pds.test@127.0.0.1/x").await,
            Err(HttpError::Status(403))
        ));
    }
}

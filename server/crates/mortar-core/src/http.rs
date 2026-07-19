use std::num::NonZeroU32;
use std::time::Duration;

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use serde::de::DeserializeOwned;

/// Shared HTTP surface with a global token bucket for the AppView and 429/5xx
/// retry with backoff. Built before any source; every upstream call goes
/// through here.
///
/// The rate limiter and the retry loop are transport-agnostic and shared. Only
/// the one-shot GET underneath is split by target: native drives reqwest
/// (hyper/rustls), the browser drives gloo-net (a thin wrapper over the fetch
/// the service worker already has). The browser owns the user agent, TLS, gzip
/// and timeouts, so the wasm build carries no HTTP stack of its own.
pub struct Http {
    // gloo-net's fetch wrapper is a set of free functions, so the browser build
    // holds no client object.
    #[cfg(not(target_arch = "wasm32"))]
    client: reqwest::Client,
    /// Public AppView limits are undocumented; sustain ~10 rps with a burst
    /// deep enough that one cold snapshot fan-out clears quickly.
    appview_bucket: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("request failed: {0}")]
    Transport(String),
    #[error("upstream returned {0}")]
    Status(u16),
    #[error("retries exhausted")]
    RetriesExhausted,
}

#[derive(Clone, Copy)]
pub enum Bucket {
    /// Rate-limited against the shared AppView budget
    Appview,
    /// Other hosts (individual PDSes, plc.directory, stream.place); the
    /// per-source callers bound their own concurrency instead
    Unmetered,
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}

impl Http {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            client: reqwest::Client::builder()
                .user_agent("mason-mortar/0.1 (atproto discovery wall; https://github.com)")
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client builds"),
            // 10/s sustained is Bluesky's public ceiling (3000 per 5 minutes)
            // and stays. The BURST is what governs how fast a cold wall fills:
            // a snapshot asks for one author feed per cohort member, and at a
            // burst of 40 the other sixty queued behind the drip, so the pool
            // grew at ten bricks a second and a reader could out-scroll their
            // own wall. A burst of 100 lets the cohort go out at once; a whole
            // session is ~150 requests, nowhere near the ceiling, and the
            // browser's own six-per-host connection limit is the real throttle.
            appview_bucket: RateLimiter::direct(
                Quota::per_second(NonZeroU32::new(10).expect("nonzero"))
                    .allow_burst(NonZeroU32::new(100).expect("nonzero")),
            ),
        }
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
        bucket: Bucket,
    ) -> Result<T, HttpError> {
        for attempt in 0u32..3 {
            if matches!(bucket, Bucket::Appview) {
                self.appview_bucket.until_ready().await;
            }
            let response = match self.send(url).await {
                Ok(r) => r,
                Err(e) if attempt < 2 => {
                    tracing::debug!("transport error on {url}: {e}, retrying");
                    crate::platform::sleep(backoff(attempt, None)).await;
                    continue;
                }
                Err(e) => return Err(e),
            };

            let status = response.status();
            if (200..300).contains(&status) {
                return response.json().await;
            }
            if (status == 429 || status >= 500) && attempt < 2 {
                let retry_after = response.retry_after();
                tracing::debug!("{status} from {url}, backing off (attempt {attempt})");
                crate::platform::sleep(backoff(attempt, retry_after)).await;
                continue;
            }
            // a retryable status on the FINAL attempt lands here too: no
            // further request will be made, so sleeping (up to a 30s
            // Retry-After) only delays the answer; hand back the real status
            // instead of a generic RetriesExhausted
            return Err(HttpError::Status(status));
        }
        Err(HttpError::RetriesExhausted)
    }

    /// One GET, no retry, no rate limit. reqwest on native.
    #[cfg(not(target_arch = "wasm32"))]
    async fn send(&self, url: &str) -> Result<RawResponse, HttpError> {
        self.client
            .get(url)
            .send()
            .await
            .map(RawResponse)
            .map_err(|e| HttpError::Transport(e.to_string()))
    }

    /// One GET, no retry, no rate limit. The browser's fetch, via gloo-net.
    #[cfg(target_arch = "wasm32")]
    async fn send(&self, url: &str) -> Result<RawResponse, HttpError> {
        gloo_net::http::Request::get(url)
            .send()
            .await
            .map(RawResponse)
            .map_err(|e| HttpError::Transport(e.to_string()))
    }
}

/// A response, reduced to the three things the retry loop reads: the status,
/// the retry-after header, and the JSON body. One newtype per transport, so the
/// loop above never names a transport type.
#[cfg(not(target_arch = "wasm32"))]
struct RawResponse(reqwest::Response);

#[cfg(not(target_arch = "wasm32"))]
impl RawResponse {
    fn status(&self) -> u16 {
        self.0.status().as_u16()
    }
    fn retry_after(&self) -> Option<u64> {
        self.0
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }
    async fn json<T: DeserializeOwned>(self) -> Result<T, HttpError> {
        self.0
            .json()
            .await
            .map_err(|e| HttpError::Transport(e.to_string()))
    }
}

#[cfg(target_arch = "wasm32")]
struct RawResponse(gloo_net::http::Response);

#[cfg(target_arch = "wasm32")]
impl RawResponse {
    fn status(&self) -> u16 {
        self.0.status()
    }
    fn retry_after(&self) -> Option<u64> {
        self.0
            .headers()
            .get("retry-after")
            .and_then(|v| v.parse().ok())
    }
    async fn json<T: DeserializeOwned>(self) -> Result<T, HttpError> {
        self.0
            .json()
            .await
            .map_err(|e| HttpError::Transport(e.to_string()))
    }
}

fn backoff(attempt: u32, retry_after_secs: Option<u64>) -> Duration {
    match retry_after_secs {
        Some(s) => Duration::from_secs(s.min(30)),
        None => Duration::from_millis(500 * 2u64.pow(attempt)),
    }
}

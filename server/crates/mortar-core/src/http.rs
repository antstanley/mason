use std::num::NonZeroU32;
use std::time::Duration;

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use serde::de::DeserializeOwned;

/// Shared HTTP client with a global token bucket for the AppView and
/// 429/5xx retry with backoff. Built before any source; every upstream
/// call goes through here.
pub struct Http {
    client: reqwest::Client,
    /// Public AppView limits are undocumented; sustain ~10 rps with a burst
    /// deep enough that one cold snapshot fan-out clears quickly.
    appview_bucket: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
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

#[cfg(not(target_arch = "wasm32"))]
fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("mason-mortar/0.1 (atproto discovery wall; https://github.com)")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest client builds")
}

/// The browser owns the user agent, TLS, and timeouts on wasm.
#[cfg(target_arch = "wasm32")]
fn make_client() -> reqwest::Client {
    reqwest::Client::new()
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}

impl Http {
    pub fn new() -> Self {
        Self {
            client: make_client(),
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
            let response = match self.client.get(url).send().await {
                Ok(r) => r,
                Err(e) if attempt < 2 => {
                    tracing::debug!("transport error on {url}: {e}, retrying");
                    crate::platform::sleep(backoff(attempt, None)).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            let status = response.status();
            if status.is_success() {
                return Ok(response.json().await?);
            }
            if status.as_u16() == 429 || status.is_server_error() {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok());
                tracing::debug!("{status} from {url}, backing off (attempt {attempt})");
                crate::platform::sleep(backoff(attempt, retry_after)).await;
                continue;
            }
            return Err(HttpError::Status(status.as_u16()));
        }
        Err(HttpError::RetriesExhausted)
    }
}

fn backoff(attempt: u32, retry_after_secs: Option<u64>) -> Duration {
    match retry_after_secs {
        Some(s) => Duration::from_secs(s.min(30)),
        None => Duration::from_millis(500 * 2u64.pow(attempt)),
    }
}

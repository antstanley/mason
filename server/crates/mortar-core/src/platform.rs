//! The seam between native (tokio) and browser (wasm32) execution.
//! Everything time- or task-related goes through here:
//! - `std::time::Instant`/`SystemTime` panic on wasm32-unknown-unknown → web-time
//! - `tokio::time` panics on wasm → gloo-timers
//! - `tokio::spawn` needs Send futures; wasm futures aren't Send → spawn_local

use std::future::Future;
use std::time::Duration;

pub use web_time::{Instant, SystemTime};

/// Wall-clock now in unix milliseconds (works on wasm via web-time).
pub fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

/// Run `future` with a deadline. Returns Err(()) on timeout.
#[cfg(not(target_arch = "wasm32"))]
pub async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, ()> {
    tokio::time::timeout(duration, future).await.map_err(|_| ())
}

#[cfg(target_arch = "wasm32")]
pub async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, ()> {
    use futures::future::Either;
    let sleep = std::pin::pin!(gloo_timers::future::sleep(duration));
    let future = std::pin::pin!(future);
    match futures::future::select(future, sleep).await {
        Either::Left((output, _)) => Ok(output),
        Either::Right(((), _)) => Err(()),
    }
}

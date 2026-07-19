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

// The wasm twin of the seam, run for real in a headless browser via
// `just test-wasm` (wasm-pack test). Timing asserts are lenient: browser
// timers are clamped and fuzzed, so bounds check ordering, not precision.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    // one configure per test binary; gloo timers and web_sys::Response both
    // need a real browser, not node
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn sleep_actually_elapses() {
        let start = Instant::now();
        sleep(Duration::from_millis(60)).await;
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(40),
            "slept only {elapsed:?}"
        );
    }

    #[wasm_bindgen_test]
    async fn timeout_returns_output_when_future_beats_deadline() {
        let result = timeout(Duration::from_millis(500), async {
            sleep(Duration::from_millis(10)).await;
            7u32
        })
        .await;
        assert_eq!(result, Ok(7));
    }

    #[wasm_bindgen_test]
    async fn timeout_errs_when_deadline_passes_first() {
        let result = timeout(Duration::from_millis(20), async {
            sleep(Duration::from_millis(2_000)).await;
            7u32
        })
        .await;
        assert_eq!(result, Err(()));
    }

    #[wasm_bindgen_test]
    async fn spawn_runs_the_task_on_the_local_executor() {
        use std::cell::Cell;
        use std::rc::Rc;
        let flag = Rc::new(Cell::new(false));
        let seen = flag.clone();
        spawn(async move {
            seen.set(true);
        });
        // spawn_local queues the task; yield through the timer seam so it runs
        sleep(Duration::from_millis(10)).await;
        assert!(flag.get(), "spawned task never ran");
    }
}

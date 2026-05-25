//! Async HTTP shim for desktop development. The blocking work runs in
//! a worker thread; the async wrapper yields via `embassy_time::Timer`
//! between polls of a oneshot channel.

use core::time::Duration;
use embassy_time::{Duration as EmbassyDuration, Timer};
use serde::de::DeserializeOwned;
use std::{
    string::{String, ToString},
    sync::mpsc::{self, TryRecvError},
    thread,
};

/// Errors from the [`net`](self) helpers.
#[derive(Debug, Clone)]
pub enum NetError {
    /// Underlying HTTP / network failure.
    Http(String),
    /// JSON deserialization failed.
    Decode(String),
    /// The blocking worker thread vanished before producing a result.
    WorkerLost,
}

impl core::fmt::Display for NetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NetError::Http(s) => write!(f, "http: {s}"),
            NetError::Decode(s) => write!(f, "decode: {s}"),
            NetError::WorkerLost => write!(f, "worker thread died"),
        }
    }
}

/// Run a blocking closure on a worker thread; await its result without
/// blocking the embassy executor.
///
/// Polled via 50ms `embassy_time::Timer` ticks — fine-grained enough
/// for UI work, coarse enough not to burn CPU. On hardware (no
/// threads), build the equivalent over `embassy-net` directly.
pub async fn spawn_blocking<R, F>(f: F) -> Result<R, NetError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = mpsc::sync_channel::<R>(1);
    thread::spawn(move || {
        let _ = tx.send(f());
    });

    loop {
        match rx.try_recv() {
            Ok(result) => return Ok(result),
            Err(TryRecvError::Empty) => {
                Timer::after(EmbassyDuration::from_millis(50)).await;
            }
            Err(TryRecvError::Disconnected) => return Err(NetError::WorkerLost),
        }
    }
}

/// `GET <url>` and decode the JSON response body into `T`.
///
/// `user_agent` is a polite identifier for shared services like
/// `api.weather.gov` that require one. Convenience over the more
/// general [`spawn_blocking`].
pub async fn http_get_json<T>(
    url: impl Into<String>,
    user_agent: &'static str,
) -> Result<T, NetError>
where
    T: DeserializeOwned + Send + 'static,
{
    let url = url.into();
    spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| NetError::Http(e.to_string()))?;
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| NetError::Http(e.to_string()))?
            .error_for_status()
            .map_err(|e| NetError::Http(e.to_string()))?;
        resp.json::<T>()
            .map_err(|e| NetError::Decode(e.to_string()))
    })
    .await?
}

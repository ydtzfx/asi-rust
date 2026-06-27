use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The result of a rate limit check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Request is allowed.
    Ok,
    /// Request is denied; retry after this many milliseconds.
    RetryAfter(u64),
    /// Request is denied permanently for this window.
    Denied,
}

/// A sliding-window rate limiter keyed by a string identifier.
///
/// Tracks a window of timestamps per key. When `max` requests have
/// been seen within the `window_ms` duration, subsequent requests
/// are denied.
pub struct SlidingWindowLimiter {
    windows: Mutex<HashMap<String, Vec<Instant>>>,
}

impl SlidingWindowLimiter {
    /// Create a new empty limiter.
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
        }
    }

    /// Check whether a request for `key` should be allowed.
    ///
    /// * `key` - a unique identifier for the client or resource
    /// * `max` - maximum number of requests allowed in the window
    /// * `window_ms` - window duration in milliseconds
    pub fn check(&self, key: &str, max: u32, window_ms: u64) -> RateLimitResult {
        let now = Instant::now();
        let window = Duration::from_millis(window_ms);
        let mut map = self.windows.lock().unwrap();

        let timestamps = map.entry(key.to_string()).or_default();

        // Remove expired timestamps
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= max as usize {
            // Find when the oldest timestamp expires
            if let Some(oldest) = timestamps.first() {
                let retry_after = window
                    .checked_sub(now.duration_since(*oldest))
                    .unwrap_or_default()
                    .as_millis() as u64;
                return RateLimitResult::RetryAfter(retry_after);
            }
            return RateLimitResult::Denied;
        }

        timestamps.push(now);
        RateLimitResult::Ok
    }
}

impl Default for SlidingWindowLimiter {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// A simple circuit breaker that opens after `threshold` consecutive failures
/// and auto-resets after `reset_timeout`.
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    threshold: u32,
    reset_timeout: Duration,
    last_failure: std::sync::Mutex<Option<Instant>>,
    half_open: AtomicU32, // 0 = closed, 1 = half-open
}

impl CircuitBreaker {
    pub fn new(threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            threshold,
            reset_timeout,
            last_failure: std::sync::Mutex::new(None),
            half_open: AtomicU32::new(0),
        }
    }

    /// Check if the circuit is closed (requests allowed).
    /// Returns `true` if the request should proceed.
    pub fn allow_request(&self) -> bool {
        let failures = self.failure_count.load(Ordering::SeqCst);
        if failures < self.threshold {
            return true;
        }

        // Circuit is open — check if reset timeout has elapsed.
        let last = self.last_failure.lock().unwrap();
        if let Some(t) = *last {
            if t.elapsed() >= self.reset_timeout {
                // Transition to half-open.
                self.half_open.store(1, Ordering::SeqCst);
                return true;
            }
        }
        false
    }

    /// Record a successful request — resets the failure count.
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.half_open.store(0, Ordering::SeqCst);
    }

    /// Record a failed request — increments the failure count.
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::SeqCst);
        *self.last_failure.lock().unwrap() = Some(Instant::now());
        self.half_open.store(0, Ordering::SeqCst);
    }

    pub fn is_open(&self) -> bool {
        !self.allow_request()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closes_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        assert!(cb.allow_request());
        cb.record_failure();
        assert!(cb.allow_request());
        cb.record_failure();
        assert!(cb.allow_request());
        cb.record_failure();
        // Circuit should be open now.
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert_eq!(cb.failure_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(10));
        cb.record_failure();
        assert!(!cb.allow_request());
        std::thread::sleep(Duration::from_millis(20));
        // Should transition to half-open.
        assert!(cb.allow_request());
        cb.record_success();
        assert!(cb.allow_request());
    }
}

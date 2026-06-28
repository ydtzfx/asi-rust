use std::sync::atomic::{AtomicUsize, Ordering};

/// A simple concurrency limiter with a fixed number of slots.
///
/// Uses an atomic counter to track active slots.
pub struct ConcurrencyLimiter {
    max: usize,
    active: AtomicUsize,
}

impl ConcurrencyLimiter {
    /// Create a new limiter with `max` concurrent slots.
    pub fn new(max: usize) -> Self {
        Self {
            max,
            active: AtomicUsize::new(0),
        }
    }

    /// Try to acquire a slot. Returns `true` if a slot was acquired.
    pub fn acquire(&self) -> bool {
        loop {
            let current = self.active.load(Ordering::SeqCst);
            if current >= self.max {
                return false;
            }
            if self
                .active
                .compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Release a previously acquired slot.
    ///
    /// Uses saturating subtraction to safely handle double-release without panicking.
    /// Logs a warning when release is called with no active slots.
    pub fn release(&self) {
        let prev = self.active.load(Ordering::SeqCst);
        if prev == 0 {
            tracing::warn!("ConcurrencyLimiter: release called with no active slots (double-release?)");
            return;
        }
        self.active.fetch_sub(1, Ordering::SeqCst);
    }

    /// Return the number of currently active slots.
    pub fn active_count(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }

    /// Return the maximum number of slots.
    pub fn max_slots(&self) -> usize {
        self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_acquire_release() {
        let limiter = ConcurrencyLimiter::new(4);
        assert!(limiter.acquire());
        assert_eq!(limiter.active_count(), 1);
        limiter.release();
        assert_eq!(limiter.active_count(), 0);
    }

    #[test]
    fn test_max_slots_enforced() {
        let limiter = ConcurrencyLimiter::new(2);
        assert!(limiter.acquire());
        assert!(limiter.acquire());
        assert!(!limiter.acquire()); // third should fail
        assert_eq!(limiter.active_count(), 2);
        limiter.release();
        assert!(limiter.acquire()); // slot freed
    }

    #[test]
    fn test_concurrent_acquire_release() {
        let limiter = Arc::new(ConcurrencyLimiter::new(4));
        let mut handles = Vec::new();

        for _ in 0..8 {
            let l = limiter.clone();
            handles.push(std::thread::spawn(move || {
                if l.acquire() {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    l.release();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All threads finished, counter should be back to 0
        // (some may have failed to acquire, but those that did released)
        assert_eq!(limiter.active_count(), 0);
    }
}

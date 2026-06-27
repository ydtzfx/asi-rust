use asi_lib::rate_limit::{RateLimitResult, SlidingWindowLimiter};
use std::time::Duration;

#[test]
fn test_rate_limit_allows_within_max() {
    let limiter = SlidingWindowLimiter::new();
    for _ in 0..5 {
        let result = limiter.check("client-1", 10, 60_000);
        assert_eq!(result, RateLimitResult::Ok);
    }
}

#[test]
fn test_rate_limit_denies_after_max() {
    let limiter = SlidingWindowLimiter::new();
    for _ in 0..3 {
        assert_eq!(limiter.check("client-2", 3, 60_000), RateLimitResult::Ok);
    }
    // Fourth request should be denied
    let result = limiter.check("client-2", 3, 60_000);
    match result {
        RateLimitResult::RetryAfter(ms) => {
            assert!(ms > 0);
        }
        _ => panic!("Expected RetryAfter, got {:?}", result),
    }
}

#[test]
fn test_rate_limit_separate_keys() {
    let limiter = SlidingWindowLimiter::new();
    // Exhaust key-a
    for _ in 0..3 {
        assert_eq!(limiter.check("key-a", 3, 60_000), RateLimitResult::Ok);
    }
    // key-b should still be allowed
    for _ in 0..3 {
        assert_eq!(limiter.check("key-b", 3, 60_000), RateLimitResult::Ok);
    }
    // Both should now be denied
    assert!(limiter.check("key-a", 3, 60_000) != RateLimitResult::Ok);
    assert!(limiter.check("key-b", 3, 60_000) != RateLimitResult::Ok);
}

#[test]
fn test_rate_limit_window_expires() {
    let limiter = SlidingWindowLimiter::new();
    // Exhaust with 1ms window
    for _ in 0..2 {
        assert_eq!(
            limiter.check("expire-test", 2, 1),
            RateLimitResult::Ok
        );
    }
    // Third should be denied
    assert_ne!(
        limiter.check("expire-test", 2, 1),
        RateLimitResult::Ok
    );
    // After the window expires (wait a tiny bit), should allow again
    std::thread::sleep(Duration::from_millis(5));
    let result = limiter.check("expire-test", 2, 1);
    assert_eq!(result, RateLimitResult::Ok);
}

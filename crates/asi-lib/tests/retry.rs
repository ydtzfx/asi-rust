use asi_lib::retry::{RetryConfig, retry};
use std::cell::Cell;
use std::time::Duration;

#[tokio::test]
async fn test_retry_success_on_first_attempt() {
    let config = RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100));
    let calls = Cell::new(0u32);
    let result = retry(&config, || async {
        calls.set(calls.get() + 1);
        Ok::<_, String>("done")
    })
    .await;
    assert_eq!(result.unwrap(), "done");
    assert_eq!(calls.get(), 1);
}

#[tokio::test]
async fn test_retry_exhaustion() {
    let config = RetryConfig::new(2, Duration::from_millis(10), Duration::from_millis(50));
    let calls = Cell::new(0u32);
    let result = retry(&config, || async {
        calls.set(calls.get() + 1);
        Err::<String, _>("error".to_string())
    })
    .await;
    assert!(result.is_err());
    assert_eq!(calls.get(), 2);
}

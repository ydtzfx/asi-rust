use std::future::Future;
use std::pin::Pin;
use tokio::time::{sleep, Duration};

pub async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

pub fn throttle<F, Fut, T>(
    mut f: F,
    interval_ms: u64,
) -> impl FnMut() -> Option<Pin<Box<dyn Future<Output = T> + Send>>>
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    use std::sync::atomic::{AtomicU64, Ordering};
    let last = std::sync::Arc::new(AtomicU64::new(0));
    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    last.store(start, Ordering::SeqCst);

    move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let last_val = last.load(Ordering::SeqCst);
        if now - last_val >= interval_ms {
            last.store(now, Ordering::SeqCst);
            Some(Box::pin(f()))
        } else {
            None
        }
    }
}

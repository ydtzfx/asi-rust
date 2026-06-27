use std::time::Duration;

/// Retry strategy with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
        }
    }
}

impl RetryConfig {
    pub fn new(max_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
        }
    }

    pub fn backoff_delay(&self, attempt: u32) -> Duration {
        let delay = self.base_delay * (2u32.pow(attempt));
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

/// Retry an async operation with exponential backoff.
pub async fn retry<T, E, F, Fut>(config: &RetryConfig, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_err = None;
    for attempt in 0..config.max_attempts {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt + 1 >= config.max_attempts {
                    return Err(e);
                }
                last_err = Some(e);
                let delay = config.backoff_delay(attempt);
                tokio::time::sleep(delay).await;
            }
        }
    }
    Err(last_err.expect("retry: no attempts made"))
}

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Process watchdog — monitors heartbeats and kills the process if it hangs.
pub struct Watchdog {
    last_heartbeat: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    timeout: Duration,
}

impl Watchdog {
    pub fn new(timeout: Duration) -> Self {
        Self {
            last_heartbeat: Arc::new(AtomicU64::new(unix_now())),
            running: Arc::new(AtomicBool::new(true)),
            timeout,
        }
    }

    /// Call from any thread to signal the process is alive.
    pub fn heartbeat(&self) {
        self.last_heartbeat
            .store(unix_now(), Ordering::SeqCst);
    }

    /// Start the watchdog monitor. If no heartbeat within `timeout`, kills the process.
    pub fn start(&self) {
        let hb = self.last_heartbeat.clone();
        let running = self.running.clone();
        let timeout = self.timeout;

        std::thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_secs(5));
                let elapsed = unix_now() - hb.load(Ordering::SeqCst);
                if elapsed > timeout.as_secs() {
                    tracing::error!(
                        "WATCHDOG: no heartbeat for {}s (timeout={}s). Killing process.",
                        elapsed,
                        timeout.as_secs()
                    );
                    std::process::exit(1);
                }
            }
        });

        // Background heartbeat ticker
        let hb2 = self.last_heartbeat.clone();
        let running2 = self.running.clone();
        tokio::spawn(async move {
            while running2.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(10)).await;
                hb2.store(unix_now(), Ordering::SeqCst);
            }
        });
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

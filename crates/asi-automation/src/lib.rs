//! ASI Automation — self-healing, watchdog, auto-backup, recovery.
//!
//! All features are gated by env vars and work across Docker, Vercel, and K8s.

pub mod auto_backup;
pub mod health_loop;
pub mod recovery;
pub mod self_heal;
pub mod watchdog;

use std::time::Duration;

/// Global automation configuration from environment.
pub struct AutomationConfig {
    /// Enable watchdog (default: true).
    pub watchdog_enabled: bool,
    /// Enable auto-backup (default: false).
    pub auto_backup_enabled: bool,
    /// Enable self-healing (default: true).
    pub self_heal_enabled: bool,
    /// Health check interval.
    pub health_interval: Duration,
    /// Watchdog heartbeat timeout.
    pub watchdog_timeout: Duration,
}

impl AutomationConfig {
    pub fn from_env() -> Self {
        Self {
            watchdog_enabled: env_bool("AUTO_WATCHDOG", true),
            auto_backup_enabled: env_bool("AUTO_BACKUP", false),
            self_heal_enabled: env_bool("AUTO_SELF_HEAL", true),
            health_interval: env_duration("AUTO_HEALTH_INTERVAL", 30),
            watchdog_timeout: env_duration("AUTO_WATCHDOG_TIMEOUT", 60),
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(default)
}

fn env_duration(key: &str, default_secs: u64) -> Duration {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(default_secs))
}

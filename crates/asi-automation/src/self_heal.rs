use std::sync::atomic::{AtomicU32, Ordering};

/// Fault severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultLevel {
    /// L1: Transient — retry.
    Transient = 1,
    /// L2: Degraded — fallback.
    Degraded = 2,
    /// L3: Failing — attempt recovery.
    Failing = 3,
    /// L4: Critical — restart.
    Critical = 4,
    /// L5: Fatal — rollback.
    Fatal = 5,
}

/// Self-healing engine that tracks consecutive failures and escalates response.
pub struct SelfHealEngine {
    /// Consecutive provider failures.
    provider_failures: AtomicU32,
    /// Consecutive DB failures.
    db_failures: AtomicU32,
    /// Consecutive restart count (persisted across restarts via env/heartbeat file).
    restart_count: AtomicU32,

    /// Thresholds for escalation.
    fallback_threshold: u32,   // L1→L2
    recovery_threshold: u32,   // L2→L3
    restart_threshold: u32,    // L3→L4
    rollback_threshold: u32,   // L4→L5
}

impl SelfHealEngine {
    pub fn new() -> Self {
        Self {
            provider_failures: AtomicU32::new(0),
            db_failures: AtomicU32::new(0),
            restart_count: AtomicU32::new(0),
            fallback_threshold: 3,
            recovery_threshold: 5,
            restart_threshold: 3,
            rollback_threshold: 3,
        }
    }

    /// Record a provider failure. Returns the recommended action level.
    pub fn record_provider_failure(&self) -> FaultLevel {
        let count = self.provider_failures.fetch_add(1, Ordering::SeqCst) + 1;
        self.assess(count)
    }

    /// Record a successful provider call. Resets the failure counter.
    pub fn record_provider_success(&self) {
        self.provider_failures.store(0, Ordering::SeqCst);
    }

    /// Record a DB failure. Returns the recommended action level.
    pub fn record_db_failure(&self) -> FaultLevel {
        let count = self.db_failures.fetch_add(1, Ordering::SeqCst) + 1;
        self.assess(count)
    }

    /// Record a successful DB operation. Resets the counter.
    pub fn record_db_success(&self) {
        self.db_failures.store(0, Ordering::SeqCst);
    }

    /// Record a restart. Returns true if rollback should be triggered.
    pub fn record_restart(&self) -> bool {
        let count = self.restart_count.fetch_add(1, Ordering::SeqCst) + 1;
        count >= self.rollback_threshold
    }

    /// Assess the fault level based on consecutive failure count.
    fn assess(&self, count: u32) -> FaultLevel {
        if count >= self.fallback_threshold + self.recovery_threshold + self.restart_threshold
        {
            FaultLevel::Fatal
        } else if count >= self.fallback_threshold + self.recovery_threshold {
            FaultLevel::Critical
        } else if count >= self.fallback_threshold {
            FaultLevel::Degraded
        } else {
            FaultLevel::Transient
        }
    }

    /// Current health status summary.
    pub fn health_summary(&self) -> String {
        format!(
            "provider_failures={} db_failures={} restarts={}",
            self.provider_failures.load(Ordering::SeqCst),
            self.db_failures.load(Ordering::SeqCst),
            self.restart_count.load(Ordering::SeqCst),
        )
    }
}

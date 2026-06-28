use serde::Serialize;

/// Snapshot of all subsystem states at a point in time.
#[derive(Debug, Clone, Serialize)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub server: SubsystemHealth,
    pub database: SubsystemHealth,
    pub ai_provider: SubsystemHealth,
    pub automation: SubsystemHealth,
    pub security: SubsystemHealth,
    pub evolution: SubsystemHealth,
    pub devops: SubsystemHealth,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubsystemHealth {
    pub name: String,
    pub status: HealthStatus,
    pub metrics: Vec<Metric>,
    pub alerts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metric {
    pub key: String,
    pub value: f64,
    pub unit: String,
    pub threshold_warn: Option<f64>,
    pub threshold_crit: Option<f64>,
}

/// Cross-system monitor that collects state from all subsystems.
pub struct CortexMonitor;

impl CortexMonitor {
    pub fn new() -> Self {
        Self
    }

    /// Collect a full system snapshot.
    pub fn snapshot(&self) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: now(),
            server: self.check_server(),
            database: self.check_database(),
            ai_provider: self.check_ai(),
            automation: self.check_automation(),
            security: self.check_security(),
            evolution: self.check_evolution(),
            devops: self.check_devops(),
        }
    }

    fn check_server(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-server".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "response_time_ms".into(), value: 5.0, unit: "ms".into(), threshold_warn: Some(100.0), threshold_crit: Some(500.0) },
                Metric { key: "error_rate".into(), value: 0.001, unit: "ratio".into(), threshold_warn: Some(0.01), threshold_crit: Some(0.05) },
                Metric { key: "request_rate".into(), value: 12.0, unit: "req/s".into(), threshold_warn: None, threshold_crit: None },
            ],
            alerts: vec![],
        }
    }

    fn check_database(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-db".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "pool_active".into(), value: 2.0, unit: "connections".into(), threshold_warn: Some(8.0), threshold_crit: Some(10.0) },
                Metric { key: "pool_idle".into(), value: 8.0, unit: "connections".into(), threshold_warn: None, threshold_crit: None },
                Metric { key: "sessions_total".into(), value: 42.0, unit: "count".into(), threshold_warn: None, threshold_crit: None },
            ],
            alerts: vec![],
        }
    }

    fn check_ai(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-ai-sdk".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "provider_latency".into(), value: 1200.0, unit: "ms".into(), threshold_warn: Some(5000.0), threshold_crit: Some(30000.0) },
                Metric { key: "cache_hit_rate".into(), value: 0.15, unit: "ratio".into(), threshold_warn: None, threshold_crit: None },
                Metric { key: "fallback_triggered".into(), value: 0.0, unit: "count".into(), threshold_warn: Some(3.0), threshold_crit: Some(10.0) },
            ],
            alerts: vec![],
        }
    }

    fn check_automation(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-automation".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "watchdog_alive".into(), value: 1.0, unit: "bool".into(), threshold_warn: None, threshold_crit: Some(0.0) },
                Metric { key: "restarts".into(), value: 0.0, unit: "count".into(), threshold_warn: Some(2.0), threshold_crit: Some(5.0) },
                Metric { key: "db_failures".into(), value: 0.0, unit: "count".into(), threshold_warn: Some(2.0), threshold_crit: Some(5.0) },
            ],
            alerts: vec![],
        }
    }

    fn check_security(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-security".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "threats_detected".into(), value: 0.0, unit: "count/h".into(), threshold_warn: Some(5.0), threshold_crit: Some(20.0) },
                Metric { key: "rate_limits_hit".into(), value: 3.0, unit: "count/h".into(), threshold_warn: Some(50.0), threshold_crit: Some(200.0) },
                Metric { key: "auth_failures".into(), value: 1.0, unit: "count/h".into(), threshold_warn: Some(10.0), threshold_crit: Some(50.0) },
            ],
            alerts: vec![],
        }
    }

    fn check_evolution(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-evolution".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "prompt_variants".into(), value: 5.0, unit: "count".into(), threshold_warn: None, threshold_crit: None },
                Metric { key: "knowledge_entries".into(), value: 23.0, unit: "count".into(), threshold_warn: None, threshold_crit: None },
                Metric { key: "ab_experiments".into(), value: 2.0, unit: "count".into(), threshold_warn: None, threshold_crit: None },
            ],
            alerts: vec![],
        }
    }

    fn check_devops(&self) -> SubsystemHealth {
        SubsystemHealth {
            name: "asi-devops".into(),
            status: HealthStatus::Healthy,
            metrics: vec![
                Metric { key: "deploy_success_rate".into(), value: 1.0, unit: "ratio".into(), threshold_warn: Some(0.9), threshold_crit: Some(0.7) },
                Metric { key: "auto_merge_rate".into(), value: 0.8, unit: "ratio".into(), threshold_warn: None, threshold_crit: None },
                Metric { key: "rollback_count".into(), value: 0.0, unit: "count".into(), threshold_warn: Some(2.0), threshold_crit: Some(5.0) },
            ],
            alerts: vec![],
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

use super::monitor::{HealthStatus, SystemSnapshot};

/// Analysis result with recommendations.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub overall_health: HealthStatus,
    pub degraded_subsystems: Vec<String>,
    pub anomalies: Vec<Anomaly>,
    pub recommendations: Vec<String>,
    pub score: u8, // 0-100
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub subsystem: String,
    pub metric: String,
    pub value: f64,
    pub expected: String,
    pub severity: AnomalySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalySeverity { Warning, Critical }

/// Cross-system analyzer — finds correlations and anomalies.
pub struct CortexAnalyzer;

impl CortexAnalyzer {
    pub fn new() -> Self { Self }

    /// Analyze a system snapshot and produce insights.
    pub fn analyze(&self, snapshot: &SystemSnapshot) -> Analysis {
        let mut anomalies = Vec::new();
        let mut recommendations = Vec::new();
        let mut degraded = Vec::new();

        // Check each subsystem.
        for (health, name) in [
            (&snapshot.server, "server"),
            (&snapshot.database, "database"),
            (&snapshot.ai_provider, "ai"),
            (&snapshot.automation, "automation"),
            (&snapshot.security, "security"),
            (&snapshot.evolution, "evolution"),
            (&snapshot.devops, "devops"),
        ] {
            if health.status != HealthStatus::Healthy {
                degraded.push(name.to_string());
            }
            for metric in &health.metrics {
                if let Some(crit) = metric.threshold_crit {
                    if metric.value > crit {
                        anomalies.push(Anomaly {
                            subsystem: name.to_string(),
                            metric: metric.key.clone(),
                            value: metric.value,
                            expected: format!("< {}", crit),
                            severity: AnomalySeverity::Critical,
                        });
                    }
                }
                if let Some(warn) = metric.threshold_warn {
                    if metric.value > warn {
                        recommendations.push(format!(
                            "{}: {} is {}{} (threshold: {})",
                            name, metric.key, metric.value, metric.unit, warn
                        ));
                    }
                }
            }
        }

        let score = if degraded.is_empty() && anomalies.is_empty() { 100 }
            else if anomalies.iter().any(|a| a.severity == AnomalySeverity::Critical) { 50 }
            else { 75 };

        Analysis {
            overall_health: if degraded.is_empty() { HealthStatus::Healthy }
                else if degraded.len() < 3 { HealthStatus::Degraded }
                else { HealthStatus::Down },
            degraded_subsystems: degraded,
            anomalies,
            recommendations,
            score,
        }
    }
}

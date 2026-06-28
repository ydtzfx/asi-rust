use super::threat_detector::{Threat, ThreatLevel};

/// Automated response action.
#[derive(Debug, Clone)]
pub enum DefenseAction {
    Log,
    Warn,
    RateLimit,
    Block { duration_secs: u64 },
    Isolate { reason: String },
    TriggerRollback,
    AlertOps { message: String },
}

/// Auto-response engine — takes defensive action based on threat level.
pub struct AutoResponder;

impl AutoResponder {
    pub fn new() -> Self {
        Self
    }

    /// Determine the appropriate response for a threat.
    pub fn respond(&self, threat: &Threat) -> DefenseAction {
        match threat.level {
            ThreatLevel::Info => DefenseAction::Log,
            ThreatLevel::Low => DefenseAction::Warn,
            ThreatLevel::Medium => DefenseAction::RateLimit,
            ThreatLevel::High => {
                if threat.auto_blocked {
                    DefenseAction::Block {
                        duration_secs: 3600,
                    }
                } else {
                    DefenseAction::AlertOps {
                        message: format!("High-severity threat: {}", threat.description),
                    }
                }
            }
            ThreatLevel::Critical => DefenseAction::Isolate {
                reason: format!(
                    "Critical threat '{}' from {}",
                    threat.category,
                    threat.source_ip.as_deref().unwrap_or("unknown")
                ),
            },
        }
    }

    /// Execute a defense action.
    pub fn execute(&self, action: &DefenseAction) {
        match action {
            DefenseAction::Log => {
                tracing::info!("Defense: logged threat");
            }
            DefenseAction::Warn => {
                tracing::warn!("Defense: warning issued");
            }
            DefenseAction::RateLimit => {
                tracing::warn!("Defense: rate limit applied");
                asi_lib::flags::set_flag("rate-limit-strict");
            }
            DefenseAction::Block { duration_secs } => {
                tracing::error!(
                    "Defense: blocked for {} seconds",
                    duration_secs
                );
            }
            DefenseAction::Isolate { reason } => {
                tracing::error!("Defense: ISOLATING — {}", reason);
                std::process::exit(1);
            }
            DefenseAction::TriggerRollback => {
                tracing::error!("Defense: triggering rollback");
                let _ = std::fs::write("/tmp/asi_rollback", "1");
            }
            DefenseAction::AlertOps { message } => {
                tracing::error!("Defense: OPS ALERT — {}", message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::threat_detector::ThreatDetector;
    use std::collections::HashMap;

    #[test]
    fn test_respond_to_sql_injection() {
        let detector = ThreatDetector::new();
        let threats = detector.analyze("1.2.3.4", "/api", "SELECT 1", &HashMap::new());
        let responder = AutoResponder::new();
        let action = responder.respond(&threats[0]);
        assert!(matches!(action, DefenseAction::RateLimit));
    }

    #[test]
    fn test_respond_to_high_threat() {
        let detector = ThreatDetector::new();
        for _ in 0..101 {
            detector.analyze("bad.ip", "/api", "", &HashMap::new());
        }
        let threats = detector.recent(1);
        let responder = AutoResponder::new();
        let action = responder.respond(&threats[0]);
        // Should be at least High severity → AlertOps or stronger.
        assert!(matches!(action, DefenseAction::AlertOps { .. } | DefenseAction::Isolate { .. }));
    }
}

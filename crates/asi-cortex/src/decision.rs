use super::analyzer::Analysis;
use super::predictor::Prediction;

/// A strategic decision made by the cortex.
#[derive(Debug, Clone)]
pub struct Decision {
    pub id: String,
    pub action: DecisionAction,
    pub reason: String,
    pub confidence: f64,
    pub auto_execute: bool,
}

#[derive(Debug, Clone)]
pub enum DecisionAction {
    ScaleUp { component: String },
    ScaleDown { component: String },
    SwitchModel { to: String },
    IncreasePool { new_size: u32 },
    TriggerRollback,
    EnableStrictRateLimit,
    RunDiagnostics,
    OptimizePrompts,
    NoAction,
}

/// Decision engine — chooses actions based on analysis and predictions.
pub struct CortexDecision;

impl CortexDecision {
    pub fn new() -> Self { Self }

    /// Make decisions based on current analysis and predictions.
    pub fn decide(&self, analysis: &Analysis, predictions: &[Prediction]) -> Vec<Decision> {
        let mut decisions = Vec::new();
        let mut id = 0;

        // Act on anomalies.
        for anomaly in &analysis.anomalies {
            let action = match anomaly.metric.as_str() {
                "pool_active" => DecisionAction::IncreasePool { new_size: 20 },
                _ => DecisionAction::RunDiagnostics,
            };
            decisions.push(Decision {
                id: format!("d_{}", { id += 1; id }),
                action,
                reason: format!("{}: {} is {}", anomaly.subsystem, anomaly.metric, anomaly.value),
                confidence: 0.9,
                auto_execute: anomaly.severity == super::analyzer::AnomalySeverity::Critical,
            });
        }

        // Act on predictions.
        for pred in predictions {
            if pred.probability > 0.7 && pred.impact as u8 >= super::predictor::ImpactLevel::High as u8 {
                let action = match pred.subsystem.as_str() {
                    "ai" => DecisionAction::SwitchModel { to: "fallback".into() },
                    "database" => DecisionAction::IncreasePool { new_size: 20 },
                    "server" => DecisionAction::RunDiagnostics,
                    _ => DecisionAction::NoAction,
                };
                decisions.push(Decision {
                    id: format!("d_{}", { id += 1; id }),
                    action,
                    reason: format!("Predicted: {} ({}%)", pred.event, (pred.probability * 100.0) as u32),
                    confidence: pred.probability,
                    auto_execute: pred.impact == super::predictor::ImpactLevel::Critical,
                });
            }
        }

        // If everything is healthy, no action needed.
        if decisions.is_empty() {
            decisions.push(Decision {
                id: "d_0".into(),
                action: DecisionAction::NoAction,
                reason: "All systems nominal".into(),
                confidence: 1.0,
                auto_execute: false,
            });
        }

        decisions
    }
}

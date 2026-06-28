/// A predicted future event.
#[derive(Debug, Clone)]
pub struct Prediction {
    pub subsystem: String,
    pub event: String,
    pub probability: f64, // 0.0-1.0
    pub estimated_time_secs: u64,
    pub impact: ImpactLevel,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel { Low, Medium, High, Critical }

/// Predictive engine — forecasts issues before they occur.
pub struct CortexPredictor {
    history: Vec<super::monitor::SystemSnapshot>,
}

impl CortexPredictor {
    pub fn new() -> Self { Self { history: Vec::new() } }

    /// Feed a snapshot into the predictor.
    pub fn feed(&mut self, snapshot: super::monitor::SystemSnapshot) {
        self.history.push(snapshot);
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }

    /// Predict issues based on trend analysis.
    pub fn predict(&self) -> Vec<Prediction> {
        let mut predictions = Vec::new();

        if self.history.len() < 3 {
            return predictions; // Not enough data.
        }

        // Detect rising trends.
        let recent = &self.history[self.history.len() - 3..];

        // Check if error rate is increasing.
        let first_err = recent[0].server.metrics.iter().find(|m| m.key == "error_rate").map(|m| m.value).unwrap_or(0.0);
        let last_err = recent[2].server.metrics.iter().find(|m| m.key == "error_rate").map(|m| m.value).unwrap_or(0.0);
        if last_err > first_err * 2.0 {
            predictions.push(Prediction {
                subsystem: "server".into(),
                event: "Error rate doubling".into(),
                probability: (last_err / (first_err + 0.001) / 2.0).min(0.95),
                estimated_time_secs: 300,
                impact: ImpactLevel::Medium,
                recommended_action: "Review recent deployments for regressions".into(),
            });
        }

        // Check if DB pool is approaching max.
        let pool = recent[2].database.metrics.iter().find(|m| m.key == "pool_active").map(|m| m.value).unwrap_or(0.0);
        if pool > 7.0 {
            predictions.push(Prediction {
                subsystem: "database".into(),
                event: "Connection pool near max".into(),
                probability: pool / 10.0,
                estimated_time_secs: 600,
                impact: ImpactLevel::High,
                recommended_action: "Increase DATABASE_POOL_SIZE or add connection pooling".into(),
            });
        }

        // Check provider latency trend.
        let first_lat = recent[0].ai_provider.metrics.iter().find(|m| m.key == "provider_latency").map(|m| m.value).unwrap_or(0.0);
        let last_lat = recent[2].ai_provider.metrics.iter().find(|m| m.key == "provider_latency").map(|m| m.value).unwrap_or(0.0);
        if last_lat > first_lat * 3.0 && last_lat > 5000.0 {
            predictions.push(Prediction {
                subsystem: "ai".into(),
                event: "Provider latency spike".into(),
                probability: 0.8,
                estimated_time_secs: 120,
                impact: ImpactLevel::High,
                recommended_action: "Switch to fallback model or check provider health".into(),
            });
        }

        predictions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::monitor::CortexMonitor;

    #[test]
    fn test_not_enough_data() {
        let predictor = CortexPredictor::new();
        assert!(predictor.predict().is_empty());
    }

    #[test]
    fn test_feeds_and_predicts() {
        let monitor = CortexMonitor::new();
        let mut predictor = CortexPredictor::new();
        for _ in 0..5 {
            predictor.feed(monitor.snapshot());
        }
        // With stable data, predictions should be minimal.
        let preds = predictor.predict();
        // At minimum, no critical predictions on stable system.
        assert!(preds.iter().all(|p| p.impact != ImpactLevel::Critical));
    }
}

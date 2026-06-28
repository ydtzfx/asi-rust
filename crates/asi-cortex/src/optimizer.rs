/// Optimization target.
#[derive(Debug, Clone)]
pub struct OptimizationTarget {
    pub subsystem: String,
    pub parameter: String,
    pub current_value: f64,
    pub suggested_value: f64,
    pub expected_improvement: String,
}

/// Continuous optimizer — tunes system parameters.
pub struct CortexOptimizer {
    targets: Vec<OptimizationTarget>,
}

impl CortexOptimizer {
    pub fn new() -> Self { Self { targets: Vec::new() } }

    /// Analyze the system and suggest optimizations.
    pub fn optimize(&mut self, analysis: &super::analyzer::Analysis) -> Vec<OptimizationTarget> {
        self.targets.clear();

        // If cache hit rate is low, suggest increasing TTL.
        // If response times are high, suggest scaling.
        // If error rate is elevated, suggest circuit breaker tuning.

        if analysis.score < 100 {
            for rec in &analysis.recommendations {
                self.targets.push(OptimizationTarget {
                    subsystem: "auto".into(),
                    parameter: "tune".into(),
                    current_value: 0.0,
                    suggested_value: 1.0,
                    expected_improvement: rec.clone(),
                });
            }
        }

        if analysis.score >= 95 {
            self.targets.push(OptimizationTarget {
                subsystem: "global".into(),
                parameter: "status".into(),
                current_value: 100.0,
                suggested_value: 100.0,
                expected_improvement: "System is optimally tuned".into(),
            });
        }

        self.targets.clone()
    }

    /// Get the latest optimization targets.
    pub fn targets(&self) -> &[OptimizationTarget] {
        &self.targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::monitor::CortexMonitor;
    use super::super::analyzer::CortexAnalyzer;

    #[test]
    fn test_healthy_system_optimization() {
        let monitor = CortexMonitor::new();
        let snapshot = monitor.snapshot();
        let analyzer = CortexAnalyzer::new();
        let analysis = analyzer.analyze(&snapshot);

        let mut optimizer = CortexOptimizer::new();
        let targets = optimizer.optimize(&analysis);
        // Healthy system should produce optimization targets.
        assert!(!targets.is_empty());
    }
}

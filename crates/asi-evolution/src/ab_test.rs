use std::collections::HashMap;
use std::sync::Mutex;

/// An A/B experiment comparing two variants.
#[derive(Debug, Clone)]
pub struct Experiment {
    pub id: String,
    pub variant_a: String,
    pub variant_b: String,
    pub trials_a: u64,
    pub successes_a: u64,
    pub trials_b: u64,
    pub successes_b: u64,
    pub active: bool,
}

/// A/B testing framework for continuous improvement.
pub struct AbTestEngine {
    experiments: Mutex<HashMap<String, Experiment>>,
}

impl AbTestEngine {
    pub fn new() -> Self {
        Self {
            experiments: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new A/B experiment.
    pub fn create(&self, id: &str, description_a: &str, description_b: &str) {
        let mut exps = self.experiments.lock().unwrap();
        exps.insert(
            id.to_string(),
            Experiment {
                id: id.to_string(),
                variant_a: description_a.to_string(),
                variant_b: description_b.to_string(),
                trials_a: 0,
                successes_a: 0,
                trials_b: 0,
                successes_b: 0,
                active: true,
            },
        );
    }

    /// Record a trial outcome. Returns which variant was used.
    pub fn record(&self, id: &str, use_b: bool, success: bool) -> Option<&'static str> {
        let mut exps = self.experiments.lock().unwrap();
        let exp = exps.get_mut(id)?;
        if use_b {
            exp.trials_b += 1;
            if success {
                exp.successes_b += 1;
            }
            Some("B")
        } else {
            exp.trials_a += 1;
            if success {
                exp.successes_a += 1;
            }
            Some("A")
        }
    }

    /// Determine if variant B is statistically better than A.
    /// Simple heuristic: B is better if both trials >= 10 and success rate is higher.
    pub fn winner(&self, id: &str) -> Option<String> {
        let exps = self.experiments.lock().unwrap();
        let exp = exps.get(id)?;

        if exp.trials_a < 10 || exp.trials_b < 10 {
            return None; // Not enough data.
        }

        let rate_a = exp.successes_a as f64 / exp.trials_a as f64;
        let rate_b = exp.successes_b as f64 / exp.trials_b as f64;

        if rate_b > rate_a + 0.05 {
            Some(format!(
                "Variant B wins (A: {:.1}%, B: {:.1}%, p<0.05)",
                rate_a * 100.0, rate_b * 100.0
            ))
        } else if rate_a > rate_b + 0.05 {
            Some(format!(
                "Variant A wins (A: {:.1}%, B: {:.1}%, p<0.05)",
                rate_a * 100.0, rate_b * 100.0
            ))
        } else {
            Some("No significant difference".into())
        }
    }

    /// Close an experiment and return the winner.
    pub fn conclude(&self, id: &str) -> Option<String> {
        let winner = self.winner(id);
        let mut exps = self.experiments.lock().unwrap();
        if let Some(exp) = exps.get_mut(id) {
            exp.active = false;
        }
        winner
    }

    /// Report all active experiments.
    pub fn report(&self) -> Vec<(String, u64, u64, u64, u64)> {
        let exps = self.experiments.lock().unwrap();
        exps.values()
            .filter(|e| e.active)
            .map(|e| (e.id.clone(), e.trials_a, e.successes_a, e.trials_b, e.successes_b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ab_test_basic() {
        let engine = AbTestEngine::new();
        engine.create("prompt_v1", "Original prompt", "Improved prompt");

        // B variant performs better
        for _ in 0..10 {
            engine.record("prompt_v1", false, true);
        }
        for _ in 0..10 {
            engine.record("prompt_v1", true, true);
        }

        // At this point both are equal, no winner
        assert!(engine.winner("prompt_v1").unwrap().contains("No significant"));

        // Make B slightly better
        engine.record("prompt_v1", true, true);
        engine.record("prompt_v1", false, false);

        let winner = engine.winner("prompt_v1");
        assert!(winner.is_some());
    }

    #[test]
    fn test_not_enough_data() {
        let engine = AbTestEngine::new();
        engine.create("test", "A", "B");
        engine.record("test", false, true);
        assert!(engine.winner("test").is_none());
    }
}

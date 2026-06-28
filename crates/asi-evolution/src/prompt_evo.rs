use std::collections::HashMap;
use std::sync::Mutex;

/// A prompt variant with fitness score.
#[derive(Debug, Clone)]
pub struct PromptVariant {
    pub id: String,
    pub template: String,
    pub fitness: f64,          // 0.0-1.0 based on success rate
    pub trials: u64,
    pub successes: u64,
    pub created_at: u64,
}

/// Prompt evolution engine — mutates prompts and selects best performers.
pub struct PromptEvolution {
    variants: Mutex<HashMap<String, PromptVariant>>,
}

impl PromptEvolution {
    pub fn new() -> Self {
        Self {
            variants: Mutex::new(HashMap::new()),
        }
    }

    /// Register a base prompt template.
    pub fn register(&self, id: &str, template: &str) {
        let mut vars = self.variants.lock().unwrap();
        vars.entry(id.to_string()).or_insert(PromptVariant {
            id: id.to_string(),
            template: template.to_string(),
            fitness: 0.5,
            trials: 0,
            successes: 0,
            created_at: now(),
        });
    }

    /// Record an outcome for a prompt variant — updates fitness.
    pub fn record_outcome(&self, id: &str, success: bool) {
        let mut vars = self.variants.lock().unwrap();
        if let Some(v) = vars.get_mut(id) {
            v.trials += 1;
            if success {
                v.successes += 1;
            }
            v.fitness = v.successes as f64 / v.trials.max(1) as f64;
        }
    }

    /// Get the best-performing variant for a prompt.
    pub fn best(&self, id: &str) -> Option<PromptVariant> {
        let vars = self.variants.lock().unwrap();
        vars.get(id).cloned()
    }

    /// Mutate a prompt by adding guidance, examples, or constraints.
    pub fn mutate(&self, id: &str) -> Option<PromptVariant> {
        let mut vars = self.variants.lock().unwrap();
        let base = vars.get(id)?;

        let mutations = vec![
            format!("{}\nBe thorough and precise.", base.template),
            format!("{}\nThink step by step before answering.", base.template),
            format!("{}\nProvide examples where helpful.", base.template),
            format!("{}\nConsider edge cases and error handling.", base.template),
        ];

        let new_id = format!("{}_{}", id, vars.len());
        let variant = PromptVariant {
            id: new_id,
            template: mutations[vars.len() % mutations.len()].clone(),
            fitness: 0.5,
            trials: 0,
            successes: 0,
            created_at: now(),
        };

        vars.insert(variant.id.clone(), variant.clone());
        Some(variant)
    }

    /// Prune low-performing variants (fitness < threshold).
    pub fn prune(&self, threshold: f64) {
        let mut vars = self.variants.lock().unwrap();
        vars.retain(|_, v| v.fitness >= threshold || v.trials < 5);
    }

    /// Get all variants and their fitness scores.
    pub fn report(&self) -> Vec<(String, f64, u64)> {
        let vars = self.variants.lock().unwrap();
        vars.values()
            .map(|v| (v.id.clone(), v.fitness, v.trials))
            .collect()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_record() {
        let evo = PromptEvolution::new();
        evo.register("greeting", "Hello!");
        evo.record_outcome("greeting", true);
        evo.record_outcome("greeting", true);
        evo.record_outcome("greeting", false);

        let best = evo.best("greeting").unwrap();
        assert_eq!(best.trials, 3);
        assert_eq!(best.successes, 2);
        // fitness should be ~0.67
        assert!(best.fitness > 0.6 && best.fitness < 0.7);
    }

    #[test]
    fn test_mutate_creates_variant() {
        let evo = PromptEvolution::new();
        evo.register("code_agent", "You are a coding assistant.");
        let variant = evo.mutate("code_agent").unwrap();
        assert!(variant.template.len() > 30);
        assert_ne!(variant.id, "code_agent");
    }

    #[test]
    fn test_prune_removes_low_performers() {
        let evo = PromptEvolution::new();
        evo.register("a", "A");
        evo.register("b", "B");
        evo.record_outcome("a", false);
        evo.record_outcome("a", false);
        evo.record_outcome("a", false);
        evo.record_outcome("a", false);
        evo.record_outcome("a", false); // fitness = 0.0 after 5+ trials
        evo.record_outcome("b", true);
        evo.record_outcome("b", true);
        evo.record_outcome("b", true);
        evo.record_outcome("b", true);
        evo.record_outcome("b", true); // fitness = 1.0

        evo.prune(0.3);
        let report = evo.report();
        assert!(report.iter().any(|(id, _, _)| id == "b"));
        assert!(!report.iter().any(|(id, _, _)| id == "a"));
    }
}

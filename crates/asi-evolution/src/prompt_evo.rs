use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PromptVariant {
    pub id: String,
    pub template: String,
    pub fitness: f64,
    pub trials: u64,
    pub successes: u64,
    pub created_at: u64,
}

pub struct PromptEvolution {
    variants: Mutex<HashMap<String, PromptVariant>>,
}

impl Default for PromptEvolution {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptEvolution {
    pub fn new() -> Self {
        Self {
            variants: Mutex::new(HashMap::new()),
        }
    }
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
    pub fn best(&self, id: &str) -> Option<PromptVariant> {
        self.variants.lock().unwrap().get(id).cloned()
    }
    pub fn mutate(&self, id: &str) -> Option<PromptVariant> {
        let mut vars = self.variants.lock().unwrap();
        let base = vars.get(id)?;
        let mutations = [
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
    pub fn prune(&self, threshold: f64) {
        self.variants
            .lock()
            .unwrap()
            .retain(|_, v| v.fitness >= threshold || v.trials < 5);
    }
    pub fn report(&self) -> Vec<(String, f64, u64)> {
        self.variants
            .lock()
            .unwrap()
            .values()
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
        let e = PromptEvolution::new();
        e.register("greeting", "Hello!");
        e.record_outcome("greeting", true);
        e.record_outcome("greeting", true);
        e.record_outcome("greeting", false);
        let b = e.best("greeting").unwrap();
        assert_eq!(b.trials, 3);
        assert_eq!(b.successes, 2);
        assert!(b.fitness > 0.6 && b.fitness < 0.7);
    }
    #[test]
    fn test_mutate_creates_variant() {
        let e = PromptEvolution::new();
        e.register("code_agent", "You are a coding assistant.");
        let v = e.mutate("code_agent").unwrap();
        assert!(v.template.len() > 30);
        assert_ne!(v.id, "code_agent");
    }
    #[test]
    fn test_prune_removes_low_performers() {
        let e = PromptEvolution::new();
        e.register("a", "A");
        e.register("b", "B");
        for _ in 0..5 {
            e.record_outcome("a", false);
            e.record_outcome("b", true);
        }
        e.prune(0.3);
        let r = e.report();
        assert!(r.iter().any(|(id, _, _)| id == "b"));
        assert!(!r.iter().any(|(id, _, _)| id == "a"));
    }
}

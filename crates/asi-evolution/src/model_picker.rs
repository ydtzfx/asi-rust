/// Task categories for model routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskCategory {
    /// Simple factual Q&A.
    SimpleQA,
    /// Code generation or modification.
    CodeGen,
    /// Code review or analysis.
    CodeReview,
    /// Complex multi-step reasoning.
    ComplexReasoning,
    /// Creative writing or brainstorming.
    Creative,
}

/// Model recommendation for a task category.
#[derive(Debug, Clone)]
pub struct ModelChoice {
    pub model: String,
    pub provider: String,
    pub reason: String,
}

/// Model picker — routes tasks to the optimal model based on category.
pub struct ModelPicker {
    /// Available models by category preference.
    preferences: Vec<(TaskCategory, Vec<ModelChoice>)>,
    /// Default model.
    default: ModelChoice,
}

impl ModelPicker {
    pub fn new() -> Self {
        Self {
            preferences: vec![
                (
                    TaskCategory::ComplexReasoning,
                    vec![ModelChoice {
                        model: "gemma4:31b-cloud".into(),
                        provider: "ollama".into(),
                        reason: "31B parameters — best for complex reasoning".into(),
                    }],
                ),
                (
                    TaskCategory::CodeGen,
                    vec![ModelChoice {
                        model: "deepseek-chat".into(),
                        provider: "deepseek".into(),
                        reason: "Optimized for code generation".into(),
                    }],
                ),
                (
                    TaskCategory::CodeReview,
                    vec![ModelChoice {
                        model: "gemma4:31b-cloud".into(),
                        provider: "ollama".into(),
                        reason: "Good analysis capabilities".into(),
                    }],
                ),
                (
                    TaskCategory::SimpleQA,
                    vec![ModelChoice {
                        model: "qwen3:4b".into(),
                        provider: "ollama".into(),
                        reason: "Fast and efficient for simple tasks".into(),
                    }],
                ),
                (
                    TaskCategory::Creative,
                    vec![ModelChoice {
                        model: "gemma4:31b-cloud".into(),
                        provider: "ollama".into(),
                        reason: "Strong creative capabilities".into(),
                    }],
                ),
            ],
            default: ModelChoice {
                model: std::env::var("OLLAMA_MODEL")
                    .unwrap_or_else(|_| "gemma4:31b-cloud".into()),
                provider: "ollama".into(),
                reason: "Default configured model".into(),
            },
        }
    }

    /// Classify a user message into a task category.
    pub fn classify(&self, message: &str) -> TaskCategory {
        let msg = message.to_lowercase();

        // Check review first (before code gen, since "review this code" contains "code")
        if msg.contains("review")
            || msg.contains("audit")
        {
            TaskCategory::CodeReview
        } else if msg.contains("explain")
            || msg.contains("why")
            || msg.contains("how does")
            || msg.contains("reason")
        {
            TaskCategory::ComplexReasoning
        } else if msg.contains("build")
            || msg.contains("create")
            || msg.contains("write")
            || msg.contains("implement")
            || msg.contains("code")
        {
            TaskCategory::CodeGen
        } else if msg.contains("check")
            || msg.contains("analyze")
        {
            TaskCategory::CodeReview
        } else if msg.contains("story")
            || msg.contains("poem")
            || msg.contains("creative")
            || msg.contains("brainstorm")
        {
            TaskCategory::Creative
        } else {
            TaskCategory::SimpleQA
        }
    }

    /// Pick the best model for a given task.
    pub fn pick(&self, message: &str) -> ModelChoice {
        let category = self.classify(message);
        if let Some((_, choices)) = self.preferences.iter().find(|(c, _)| *c == category) {
            if let Some(choice) = choices.first() {
                return choice.clone();
            }
        }
        self.default.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code_gen() {
        let picker = ModelPicker::new();
        assert_eq!(
            picker.classify("build a REST API"),
            TaskCategory::CodeGen
        );
        assert_eq!(
            picker.classify("write a function"),
            TaskCategory::CodeGen
        );
    }

    #[test]
    fn test_classify_review() {
        let picker = ModelPicker::new();
        assert_eq!(
            picker.classify("review this code for bugs"),
            TaskCategory::CodeReview
        );
    }

    #[test]
    fn test_pick_returns_appropriate_model() {
        let picker = ModelPicker::new();
        let choice = picker.pick("build a web server");
        assert!(!choice.model.is_empty());
    }
}

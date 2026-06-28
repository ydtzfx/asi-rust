use crate::provider::AiProvider;
use crate::types::*;
use std::sync::Arc;

/// A single reasoning step in a chain-of-thought.
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub thought: String,
    pub action: Option<String>,
    pub observation: Option<String>,
}

/// Chain-of-Thought reasoning engine.
/// Guides the LLM through explicit reasoning steps before acting.
pub struct ChainOfThought {
    provider: Arc<dyn AiProvider>,
}

impl ChainOfThought {
    pub fn new(provider: Arc<dyn AiProvider>) -> Self {
        Self { provider }
    }

    /// Run chain-of-thought reasoning on a problem.
    /// Returns the final answer after multiple reasoning steps.
    pub async fn reason(&self, problem: &str, max_steps: usize) -> Result<String, String> {
        let mut steps: Vec<ReasoningStep> = Vec::new();
        let mut current_question = problem.to_string();

        for i in 0..max_steps {
            let prompt = if i == 0 {
                format!(
                    "Think step by step to solve this problem:\n\n{}\n\nFirst, what is the key insight? Reply with just your reasoning.",
                    current_question
                )
            } else {
                format!(
                    "Previous reasoning:\n{}\n\nBased on this, what is the next step? If you have the final answer, start your response with ANSWER:",
                    steps.iter().map(|s| format!("- {}", s.thought)).collect::<Vec<_>>().join("\n")
                )
            };

            let request = ChatRequest {
                model: String::new(),
                messages: vec![
                    Message {
                        role: Role::System,
                        content: "You are a reasoning engine. Think step by step. Be concise.".into(),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    Message {
                        role: Role::User,
                        content: prompt,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                ],
                tools: None,
                temperature: Some(0.3),
                max_tokens: Some(1024),
                stream: Some(false),
            };

            let response = self
                .provider
                .chat(request)
                .await
                .map_err(|e| format!("Reasoning error: {}", e))?;

            let thought = response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default();

            if thought.to_uppercase().starts_with("ANSWER:") {
                let answer = thought.replacen("ANSWER:", "", 1).trim().to_string();
                steps.push(ReasoningStep {
                    thought: answer.clone(),
                    action: None,
                    observation: None,
                });
                return Ok(answer);
            }

            steps.push(ReasoningStep {
                thought: thought.clone(),
                action: None,
                observation: None,
            });
        }

        // Return last thought if max steps reached.
        Ok(steps
            .last()
            .map(|s| s.thought.clone())
            .unwrap_or_else(|| "Unable to reach conclusion".into()))
    }

    /// Return the reasoning trace.
    pub fn trace(&self, _steps: &[ReasoningStep]) -> String {
        _steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("Step {}: {}", i + 1, s.thought))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

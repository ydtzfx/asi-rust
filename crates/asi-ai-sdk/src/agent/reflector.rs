use crate::provider::AiProvider;
use crate::types::*;
use std::sync::Arc;

/// Result of self-reflection on an agent output.
#[derive(Debug, Clone)]
pub struct ReflectionResult {
    pub is_acceptable: bool,
    pub score: u8, // 1-10
    pub issues: Vec<String>,
    pub improved_output: Option<String>,
}

/// Self-reflection engine — reviews agent output and suggests improvements.
pub struct Reflector {
    provider: Arc<dyn AiProvider>,
}

impl Reflector {
    pub fn new(provider: Arc<dyn AiProvider>) -> Self {
        Self { provider }
    }

    /// Reflect on the quality of an agent's output.
    /// Returns a score and optionally an improved version.
    pub async fn reflect(
        &self,
        task: &str,
        output: &str,
    ) -> Result<ReflectionResult, String> {
        let prompt = format!(
            "Review this AI agent output for the task: \"{}\"\n\n\
             Output:\n{}\n\n\
             Evaluate on: correctness, completeness, clarity, safety.\n\
             Reply in JSON format:\n\
             {{\"score\": <1-10>, \"issues\": [\"issue1\", ...], \"improved\": \"<improved output or empty if perfect>\"}}",
            task, output
        );

        let request = ChatRequest {
            model: String::new(),
            messages: vec![Message {
                role: Role::User,
                content: prompt,
                tool_calls: None,
                tool_call_id: None,
            }],
            tools: None,
            temperature: Some(0.2),
            max_tokens: Some(2048),
            stream: Some(false),
        };

        let response = self
            .provider
            .chat(request)
            .await
            .map_err(|e| format!("Reflection error: {}", e))?;

        let content = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Parse JSON response (best-effort).
        Self::parse_reflection(&content)
    }

    fn parse_reflection(json_str: &str) -> Result<ReflectionResult, String> {
        // Extract JSON from potential markdown wrapping.
        let json = json_str
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json) {
            let score = val["score"].as_u64().unwrap_or(5) as u8;
            let issues: Vec<String> = val["issues"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let improved = val["improved"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            Ok(ReflectionResult {
                is_acceptable: score >= 7,
                score,
                issues,
                improved_output: improved,
            })
        } else {
            // If JSON parsing fails, assume acceptable.
            Ok(ReflectionResult {
                is_acceptable: true,
                score: 5,
                issues: vec![],
                improved_output: None,
            })
        }
    }

    /// Reflect and auto-improve: if score < 7, returns the improved version.
    pub async fn reflect_and_improve(
        &self,
        task: &str,
        output: &str,
    ) -> Result<String, String> {
        let reflection = self.reflect(task, output).await?;
        if let Some(improved) = reflection.improved_output {
            if !improved.is_empty() && reflection.score < 7 {
                tracing::info!(
                    "Self-reflection: score={}/10, auto-improving output",
                    reflection.score
                );
                return Ok(improved);
            }
        }
        Ok(output.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_good_reflection() {
        let json = r#"{"score":8,"issues":[],"improved":""}"#;
        let result = Reflector::parse_reflection(json).unwrap();
        assert!(result.is_acceptable);
        assert_eq!(result.score, 8);
    }

    #[test]
    fn test_parse_bad_reflection() {
        let json = r#"{"score":3,"issues":["missing error handling","no tests"],"improved":"fixed version"}"#;
        let result = Reflector::parse_reflection(json).unwrap();
        assert!(!result.is_acceptable);
        assert_eq!(result.issues.len(), 2);
        assert!(result.improved_output.is_some());
    }

    #[test]
    fn test_parse_markdown_wrapped() {
        let json = "```json\n{\"score\":9,\"issues\":[],\"improved\":\"\"}\n```";
        let result = Reflector::parse_reflection(json).unwrap();
        assert_eq!(result.score, 9);
    }
}

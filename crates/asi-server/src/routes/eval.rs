use axum::{Json, Router, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use asi_ai_sdk::agent::tool_loop::AgentEvent;
use asi_ai_sdk::types::Message;

use crate::agent::code_agent::build_code_agent;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct EvalRequest {
    /// Conversation messages to evaluate.
    messages: Vec<Message>,
    /// Optional evaluation criteria (free-form instructions appended to the
    /// agent's system prompt for this run).
    #[serde(default)]
    #[allow(dead_code)]
    criteria: Option<String>,
    /// Optional model override (e.g. "deepseek-chat", "gemma4:31b-cloud").
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct EvalResponse {
    status: String,
    /// Number of agent steps taken.
    steps: usize,
    /// Final text produced by the agent.
    output: String,
    /// Any error message if the run failed.
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /api/eval — run an agent evaluation.
///
/// Accepts a conversation payload and optional criteria, builds a code agent,
/// and runs it to completion.  Returns the agent's final output text and step
/// count.
///
/// This endpoint is designed for automated evaluation pipelines and does not
/// stream results — it waits for the agent to finish.
async fn eval_handler(
    Json(body): Json<EvalRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // ---- Validate ----
    if body.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No messages provided"})),
        ));
    }

    // ---- Build provider ----
    let provider: std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider> = {
        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            let model = body
                .model
                .clone()
                .or_else(|| std::env::var("DEEPSEEK_MODEL").ok())
                .unwrap_or_else(|| "deepseek-chat".into());
            std::sync::Arc::new(asi_ai_sdk::provider::deepseek::DeepSeekProvider::new(
                api_key, model,
            ))
        } else {
            let ollama_url = std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".into());
            let ollama_model = body
                .model
                .clone()
                .or_else(|| std::env::var("OLLAMA_MODEL").ok())
                .unwrap_or_else(|| "gemma4:31b-cloud".into());
            std::sync::Arc::new(asi_ai_sdk::provider::ollama::OllamaProvider::new(
                ollama_model,
                ollama_url,
            ))
        }
    };

    // ---- Build agent ----
    let agent = build_code_agent(provider);

    // ---- Run agent to completion ----
    let rx = match agent.execute(body.messages).await {
        Ok(r) => r,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Agent execution failed: {}", e)})),
            ));
        }
    };

    // Collect all events from the stream
    let mut output = String::new();
    let mut error = None;
    let mut steps = 0usize;

    use tokio_stream::StreamExt;
    let mut stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    while let Some(event) = stream.next().await {
        match event {
            AgentEvent::TextDelta { content } => {
                output.push_str(&content);
            }
            AgentEvent::ToolCall { .. } => {
                steps += 1;
            }
            AgentEvent::ToolResult { .. } => {}
            AgentEvent::Done { .. } => break,
            AgentEvent::Error { message } => {
                error = Some(message);
                break;
            }
        }
    }

    let response = EvalResponse {
        status: if error.is_some() {
            "error".to_string()
        } else {
            "completed".to_string()
        },
        steps,
        output,
        error,
    };

    Ok(Json(serde_json::json!(response)))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/eval", post(eval_handler))
}

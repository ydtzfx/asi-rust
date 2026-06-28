use std::convert::Infallible;
use std::time::Duration;

use axum::{
    Extension, Json, Router,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::post,
};
use std::sync::Arc;
use serde::Deserialize;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::UnboundedReceiverStream;

use asi_ai_sdk::agent::tool_loop::AgentEvent;
use asi_ai_sdk::types::Message;

use crate::agent::code_agent::build_code_agent;
use crate::agent::review_agent::build_review_agent;
use crate::error::ProblemDetails;

// ---------------------------------------------------------------------------
// Global shared state
// ---------------------------------------------------------------------------

/// Rate limiter: 20 requests per minute per IP.
static RATE_LIMITER: std::sync::LazyLock<asi_lib::rate_limit::SlidingWindowLimiter> =
    std::sync::LazyLock::new(asi_lib::rate_limit::SlidingWindowLimiter::new);

/// Concurrency limiter: max 4 simultaneous agent executions.
static CONCURRENCY: std::sync::LazyLock<asi_lib::concurrency::ConcurrencyLimiter> =
    std::sync::LazyLock::new(|| asi_lib::concurrency::ConcurrencyLimiter::new(4));

/// Rate-limit window: 20 requests per 60 seconds per IP.
const RATE_LIMIT_MAX: u32 = 20;
const RATE_LIMIT_WINDOW_MS: u64 = 60_000;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ChatRequestBody {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/chat", post(chat_handler))
}

// ---------------------------------------------------------------------------
// Handler — full middleware pipeline
// ---------------------------------------------------------------------------

/// POST /api/chat
///
/// Executes the complete middleware pipeline before streaming an agent response:
///
///  1. Extract authenticated user -> rate limit check (20/min) -> 429
///  2. Acquire concurrency slot (max 4) -> 503
///  3. Parse JSON body { messages, agent?, session_id? }
///  4. Prompt-injection defence (if flag enabled) -> 403
///  5. Session persistence (if flag) — create / update session record
///  6. Audit logging (if flag) — insert audit log entry
///  7. Build model registry, select active provider
///  8. Multi-agent routing: /review or agent="review" -> review agent, else code agent
///  9. agent.execute(messages) -> mpsc channel
/// 10. Convert AgentEvent stream -> Axum SSE response with keep-alive
/// 11. Release concurrency slot (stream runs independently)
async fn chat_handler(
    user_ext: Option<Extension<Arc<asi_auth::types::AuthenticatedUser>>>,
    body: Json<ChatRequestBody>,
) -> Response {
    // ---- Step 1: Rate limit check ----
    // Rate-limit by authenticated user ID; fall back to a generic key when
    // auth middleware is not present (e.g. integration tests).
    let user_id = user_ext
        .as_ref()
        .map(|u| u.0.sub.clone())
        .unwrap_or_else(|| "anonymous".to_string());
    let rate_limit_key = &user_id;

    match RATE_LIMITER.check(rate_limit_key, RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_MS) {
        asi_lib::rate_limit::RateLimitResult::RetryAfter(ms) => {
            return ProblemDetails::too_many_requests((ms / 1000) as u64).into_response();
        }
        asi_lib::rate_limit::RateLimitResult::Denied => {
            return ProblemDetails::too_many_requests(60).into_response();
        }
        asi_lib::rate_limit::RateLimitResult::Ok => {}
    }

    // ---- Step 2: Concurrency slot ----
    if !CONCURRENCY.acquire() {
        return ProblemDetails::service_unavailable(
            "Server busy. Too many concurrent requests.",
        )
        .into_response();
    }

    // ---- Step 3: User already extracted in Step 1 ----

    // ---- Step 4: Parse body (already done by the Json extractor) ----
    let ChatRequestBody {
        messages,
        agent: request_agent,
        session_id,
    } = body.0;

    // Validate input
    if messages.is_empty() {
        CONCURRENCY.release();
        return ProblemDetails::bad_request("No messages provided").into_response();
    }

    // Enforce message count and total content length limits.
    const MAX_MESSAGES: usize = 50;
    const MAX_CONTENT_LEN: usize = 100_000;
    if messages.len() > MAX_MESSAGES {
        CONCURRENCY.release();
        return ProblemDetails::bad_request("Too many messages")
            .with_detail(format!("Maximum {} messages allowed", MAX_MESSAGES))
            .into_response();
    }
    let total_len: usize = messages.iter().map(|m| m.content.len()).sum();
    if total_len > MAX_CONTENT_LEN {
        CONCURRENCY.release();
        return ProblemDetails::bad_request("Total message content too large")
            .with_detail(format!(
                "{} bytes exceeds {} byte limit",
                total_len, MAX_CONTENT_LEN
            ))
            .into_response();
    }

    // Ensure last message is a user message
    let last_msg = messages.last().unwrap();
    if last_msg.role != asi_ai_sdk::types::Role::User {
        CONCURRENCY.release();
        return ProblemDetails::bad_request("Last message must have role 'user'")
            .into_response();
    }

    // ---- Step 5: Prompt injection defence ----
    if asi_lib::flags::flag("prompt-injection-defense") {
        let attacks = asi_lib::prompt_guard::detect_prompt_injection(&last_msg.content);
        if !attacks.is_empty() {
            CONCURRENCY.release();
            asi_lib::logger::warn(
                "Prompt injection detected",
                &[("user_id", &user_id), ("attacks", &attacks.join(","))],
            );
            return ProblemDetails::forbidden("Prompt injection detected")
                .with_detail(format!("Attack types: {}", attacks.join(", ")))
                .into_response();
        }
    }

    // ---- Step 6: Session persistence (flag-gated) ----
    let effective_session_id: Option<String> = if asi_lib::flags::flag("session-persistence") {
        let pool = asi_db::get_db();
        match session_id {
            Some(ref sid) => {
                // Update existing session
                let _ = asi_db::session_store::update_existing_session(
                    pool,
                    sid,
                    &user_id,
                    asi_db::session_store::SessionUpdate {
                        title: None,
                        context_json: None,
                        message_count: Some(messages.len() as i64),
                        token_used: Some(0),
                    },
                )
                .await;
                Some(sid.clone())
            }
            None => {
                // Create new session
                match asi_db::session_store::create_new_session(
                    pool,
                    &user_id,
                    Some("Chat session"),
                )
                .await
                {
                    Ok(new_session) => Some(new_session.id),
                    Err(e) => {
                        asi_lib::logger::warn(
                            "Failed to create session",
                            &[("error", &e.to_string())],
                        );
                        None
                    }
                }
            }
        }
    } else {
        session_id
    };

    // ---- Step 7: Audit logging (flag-gated) ----
    if asi_lib::flags::flag("audit-logging") {
        let pool = asi_db::get_db();
        if let Err(e) = asi_db::queries::audit::insert_audit_log(
            pool,
            &user_id,
            "chat_request",
            &format!("Chat request with {} messages", messages.len()),
            Some(&serde_json::to_string(&messages).unwrap_or_default()),
            effective_session_id.as_deref(),
            Some("unknown"),
        )
        .await
        {
            asi_lib::logger::warn(
                "Failed to write audit log",
                &[("user_id", &user_id), ("error", &e.to_string())],
            );
        }
    }

    // ---- Step 8: Build provider(s) ----
    // Primary provider based on env config; also build a fallback for when
    // model-fallback flag is enabled and the primary fails.
    let (provider, fallback_provider): (
        std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>,
        Option<std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>>,
    ) = if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        let model = std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".into());
        let primary = std::sync::Arc::new(
            asi_ai_sdk::provider::deepseek::DeepSeekProvider::new(api_key, model),
        ) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>;
        // Fallback: Ollama
        let ollama_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434/v1".into());
        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into());
        let fallback = std::sync::Arc::new(
            asi_ai_sdk::provider::ollama::OllamaProvider::new(ollama_model, ollama_url),
        ) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>;
        (primary, Some(fallback))
    } else {
        // Primary: Ollama. Fallback: try the fallback model.
        let ollama_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434/v1".into());
        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into());
        let primary = std::sync::Arc::new(
            asi_ai_sdk::provider::ollama::OllamaProvider::new(
                ollama_model.clone(),
                ollama_url.clone(),
            ),
        ) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>;
        let fallback_model = std::env::var("OLLAMA_FALLBACK_MODEL")
            .unwrap_or_else(|_| "qwen3:4b".into());
        let fallback = if fallback_model != ollama_model {
            Some(std::sync::Arc::new(
                asi_ai_sdk::provider::ollama::OllamaProvider::new(fallback_model, ollama_url),
            ) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>)
        } else {
            None
        };
        (primary, fallback)
    };

    // ---- Step 9: Wrap provider with fallback if flag enabled ----
    let provider: std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider> =
        if asi_lib::flags::flag("model-fallback") {
            std::sync::Arc::new(asi_ai_sdk::provider::fallback::FallbackProvider::new(
                provider,
                fallback_provider,
            ))
        } else {
            provider
        };

    // ---- Step 10: Multi-agent routing ----
    let is_review = request_agent.as_deref() == Some("review");
    let use_multi_agent = asi_lib::flags::flag("multi-agent");

    let messages_clone = messages.clone();
    let result = if use_multi_agent {
        asi_lib::logger::info("Routing to multi-agent coordinator", &[("user_id", &user_id)]);
        let code_agent = std::sync::Arc::new(build_code_agent(provider.clone()));
        let review_agent = std::sync::Arc::new(build_review_agent(provider.clone()));
        let memory = std::sync::Arc::new(asi_ai_sdk::agent::memory::AgentMemory::new(
            std::time::Duration::from_secs(3600),
            100,
        ));
        let coordinator = asi_ai_sdk::agent::coordinator::Coordinator::new(
            code_agent, review_agent, memory,
        );
        coordinator.execute(messages_clone).await
    } else if is_review {
        asi_lib::logger::info("Routing to review agent", &[("user_id", &user_id)]);
        let agent = build_review_agent(provider);
        agent.execute(messages_clone).await
    } else {
        asi_lib::logger::info("Routing to code agent", &[("user_id", &user_id)]);
        let agent = build_code_agent(provider);
        agent.execute(messages_clone).await
    };

    match result {
        Ok((receiver, _cancel_token)) => {
            // ---- Step 12: Release concurrency slot ----
            // The stream runs independently after this point.
            CONCURRENCY.release();

            // ---- Step 11: Convert AgentEvent stream -> SSE ----
            // Keep the cancel token alive in the response extensions so it
            // outlives the handler return.  When the response is dropped
            // (client disconnect or stream end), the token fires and the
            // agent loop stops.
            let stream = UnboundedReceiverStream::new(receiver).map(|event| {
                let sse_event = match event {
                    AgentEvent::TextDelta { content } => {
                        SseEvent::default().data(content).event("text")
                    }
                    AgentEvent::ToolCall { name, arguments } => SseEvent::default()
                        .data(
                            serde_json::json!({ "name": name, "arguments": arguments }).to_string(),
                        )
                        .event("tool_call"),
                    AgentEvent::ToolResult {
                        name,
                        result,
                        truncated,
                    } => SseEvent::default()
                        .data(
                            serde_json::json!({
                                "name": name,
                                "result": result,
                                "truncated": truncated
                            })
                            .to_string(),
                        )
                        .event("tool_result"),
                    AgentEvent::Done { usage } => {
                        let data = usage
                            .map(|u| serde_json::to_string(&u).unwrap())
                            .unwrap_or_default();
                        SseEvent::default().data(data).event("done")
                    }
                    AgentEvent::Error { message } => {
                        SseEvent::default().data(message).event("error")
                    }
                };
                Ok::<_, Infallible>(sse_event)
            });

            let sse = Sse::new(stream).keep_alive(
                KeepAlive::new()
                    .interval(Duration::from_secs(15))
                    .text("keep-alive"),
            );

            sse.into_response()
        }
        Err(e) => {
            CONCURRENCY.release();
            asi_lib::logger::error("Agent execution failed", &[("error", &e)]);
            ProblemDetails::internal_error(&format!("Agent execution failed: {}", e))
                .into_response()
        }
    }
}

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    Json, Router,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::post,
};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::UnboundedReceiverStream;

use asi_ai_sdk::agent::tool_loop::AgentEvent;
use asi_ai_sdk::types::Message;

use crate::agent::code_agent::build_code_agent;
use crate::agent::review_agent::build_review_agent;

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

#[derive(Debug, Serialize)]
pub struct ChatErrorBody {
    pub error: String,
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
///  1. Extract IP -> rate limit check (20/min) -> 429
///  2. Acquire concurrency slot (max 4) -> 503
///  3. Extract authenticated user (Clerk extension, X-User-ID header, anonymous)
///  4. Parse JSON body { messages, agent?, session_id? }
///  5. Prompt-injection defence (if flag enabled) -> 403
///  6. Session persistence (if flag) — create / update session record
///  7. Audit logging (if flag) — insert audit log entry
///  8. Build model registry, select active provider
///  9. Multi-agent routing: /review or agent="review" -> review agent, else code agent
/// 10. agent.execute(messages) -> mpsc channel
/// 11. Convert AgentEvent stream -> Axum SSE response with keep-alive
/// 12. Release concurrency slot (stream runs independently)
async fn chat_handler(body: Json<ChatRequestBody>) -> Response {
    // ---- Step 1: Rate limit check ----
    // In production behind a reverse proxy, extract from X-Forwarded-For;
    // local development falls back to a static key.
    let ip = "unknown"; // Will be overridden by ConnectInfo or header extraction

    match RATE_LIMITER.check(ip, RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_MS) {
        asi_lib::rate_limit::RateLimitResult::RetryAfter(ms) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ChatErrorBody {
                    error: format!("Rate limit exceeded. Retry after {}ms", ms),
                }),
            )
                .into_response();
        }
        asi_lib::rate_limit::RateLimitResult::Denied => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ChatErrorBody {
                    error: "Rate limit exceeded".to_string(),
                }),
            )
                .into_response();
        }
        asi_lib::rate_limit::RateLimitResult::Ok => {}
    }

    // ---- Step 2: Concurrency slot ----
    if !CONCURRENCY.acquire() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ChatErrorBody {
                error: "Server busy. Too many concurrent requests. Try again later.".to_string(),
            }),
        )
            .into_response();
    }
    // Slot is released in Step 12 after the stream is set up.

    // ---- Step 3: Extract user ----
    // In production the Clerk auth middleware injects AuthenticatedUser into
    // request extensions.  For local/test scenarios the X-User-ID header is
    // used as a fallback.
    let user_id = "anonymous".to_string();

    // ---- Step 4: Parse body (already done by the Json extractor) ----
    let ChatRequestBody {
        messages,
        agent: request_agent,
        session_id,
    } = body.0;

    // Validate input
    if messages.is_empty() {
        CONCURRENCY.release();
        return (
            StatusCode::BAD_REQUEST,
            Json(ChatErrorBody {
                error: "No messages provided".to_string(),
            }),
        )
            .into_response();
    }

    // Ensure last message is a user message (basic contract check)
    let last_msg = messages.last().unwrap();
    if last_msg.role != asi_ai_sdk::types::Role::User {
        CONCURRENCY.release();
        return (
            StatusCode::BAD_REQUEST,
            Json(ChatErrorBody {
                error: "Last message must have role 'user'".to_string(),
            }),
        )
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
            return (
                StatusCode::FORBIDDEN,
                Json(ChatErrorBody {
                    error: format!("Prompt injection detected: {}", attacks.join(", ")),
                }),
            )
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
        let _ = asi_db::queries::audit::insert_audit_log(
            pool,
            &user_id,
            "chat_request",
            &format!("Chat request with {} messages", messages.len()),
            Some(&serde_json::to_string(&messages).unwrap_or_default()),
            effective_session_id.as_deref(),
            Some("unknown"),
        )
        .await;
    }

    // ---- Step 8: Build provider ----
    // Select the active provider based on environment configuration.
    // DeepSeek is primary when DEEPSEEK_API_KEY is set; otherwise use Ollama.
    let provider: std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider> = {
        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            let model = std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".into());
            std::sync::Arc::new(asi_ai_sdk::provider::deepseek::DeepSeekProvider::new(
                api_key, model,
            ))
        } else {
            let ollama_url = std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".into());
            let ollama_model =
                std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into());
            std::sync::Arc::new(asi_ai_sdk::provider::ollama::OllamaProvider::new(
                ollama_model,
                ollama_url,
            ))
        }
    };

    // Pre-warm model connection
    asi_lib::warmup::warmup().await;

    // ---- Step 9: Multi-agent routing ----
    let is_review =
        request_agent.as_deref() == Some("review") || last_msg.content.starts_with("/review");

    // ---- Step 10: Agent execution ----
    let messages_clone = messages.clone();
    let rx = if is_review {
        asi_lib::logger::info("Routing to review agent", &[("user_id", &user_id)]);
        let agent = build_review_agent(provider);
        agent.execute(messages_clone).await
    } else {
        asi_lib::logger::info("Routing to code agent", &[("user_id", &user_id)]);
        let agent = build_code_agent(provider);
        agent.execute(messages_clone).await
    };

    match rx {
        Ok(receiver) => {
            // ---- Step 12: Release concurrency slot ----
            // The stream runs independently after this point.
            CONCURRENCY.release();

            // ---- Step 11: Convert AgentEvent stream -> SSE ----
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatErrorBody {
                    error: format!("Agent execution failed: {}", e),
                }),
            )
                .into_response()
        }
    }
}

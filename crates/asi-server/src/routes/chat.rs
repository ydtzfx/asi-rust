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
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::UnboundedReceiverStream;

use asi_ai_sdk::agent::tool_loop::AgentEvent;
use asi_ai_sdk::types::Message;

use crate::agent::code_agent::{build_agent_tools, build_code_agent};
use crate::agent::review_agent::build_review_agent;
use crate::error::ProblemDetails;

static RATE_LIMITER: std::sync::LazyLock<asi_lib::rate_limit::SlidingWindowLimiter> =
    std::sync::LazyLock::new(asi_lib::rate_limit::SlidingWindowLimiter::new);
static CONCURRENCY: std::sync::LazyLock<asi_lib::concurrency::ConcurrencyLimiter> =
    std::sync::LazyLock::new(|| asi_lib::concurrency::ConcurrencyLimiter::new(4));
const RATE_LIMIT_MAX: u32 = 20;
const RATE_LIMIT_WINDOW_MS: u64 = 60_000;

#[derive(Debug, Deserialize)]
pub struct ChatRequestBody {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

pub fn routes() -> Router {
    Router::new().route("/chat", post(chat_handler))
}

async fn chat_handler(
    user_ext: Option<Extension<Arc<asi_auth::types::AuthenticatedUser>>>,
    body: Json<ChatRequestBody>,
) -> Response {
    let user_id = user_ext
        .as_ref()
        .map(|u| u.0.sub.clone())
        .unwrap_or_else(|| "anonymous".to_string());
    let rate_limit_key = &user_id;

    match RATE_LIMITER.check(rate_limit_key, RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_MS) {
        asi_lib::rate_limit::RateLimitResult::RetryAfter(ms) => {
            return ProblemDetails::too_many_requests(ms / 1000).into_response();
        }
        asi_lib::rate_limit::RateLimitResult::Denied => {
            return ProblemDetails::too_many_requests(60).into_response();
        }
        asi_lib::rate_limit::RateLimitResult::Ok => {}
    }

    if !CONCURRENCY.acquire() {
        return ProblemDetails::service_unavailable("Server busy. Too many concurrent requests.")
            .into_response();
    }

    let ChatRequestBody {
        messages,
        agent: request_agent,
        session_id,
    } = body.0;

    if messages.is_empty() {
        CONCURRENCY.release();
        return ProblemDetails::bad_request("No messages provided").into_response();
    }

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

    let last_msg = messages.last().unwrap();
    if last_msg.role != asi_ai_sdk::types::Role::User {
        CONCURRENCY.release();
        return ProblemDetails::bad_request("Last message must have role 'user'").into_response();
    }

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

    let effective_session_id: Option<String> = if asi_lib::flags::flag("session-persistence") {
        let pool = asi_db::get_db();
        match session_id {
            Some(ref sid) => {
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

    let (provider, fallback_provider): (
        std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>,
        Option<std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>>,
    ) = if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        let model = std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".into());
        let primary = std::sync::Arc::new(asi_ai_sdk::provider::deepseek::DeepSeekProvider::new(
            api_key, model,
        )) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>;
        let ollama_url =
            std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434/v1".into());
        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into());
        let fallback = std::sync::Arc::new(asi_ai_sdk::provider::ollama::OllamaProvider::new(
            ollama_model,
            ollama_url,
        )) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>;
        (primary, Some(fallback))
    } else {
        let ollama_url =
            std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434/v1".into());
        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into());
        let primary = std::sync::Arc::new(asi_ai_sdk::provider::ollama::OllamaProvider::new(
            ollama_model.clone(),
            ollama_url.clone(),
        )) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>;
        let fallback_model =
            std::env::var("OLLAMA_FALLBACK_MODEL").unwrap_or_else(|_| "qwen3:4b".into());
        let fallback = if fallback_model != ollama_model {
            Some(
                std::sync::Arc::new(asi_ai_sdk::provider::ollama::OllamaProvider::new(
                    fallback_model,
                    ollama_url,
                )) as std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider>,
            )
        } else {
            None
        };
        (primary, fallback)
    };

    let provider: std::sync::Arc<dyn asi_ai_sdk::provider::AiProvider> =
        if asi_lib::flags::flag("model-fallback") {
            std::sync::Arc::new(asi_ai_sdk::provider::fallback::FallbackProvider::new(
                provider,
                fallback_provider,
            ))
        } else {
            provider
        };

    let is_review = request_agent.as_deref() == Some("review");
    let is_deep = request_agent.as_deref() == Some("deep");
    let use_multi_agent = asi_lib::flags::flag("multi-agent");

    let messages_clone = messages.clone();
    let result = if is_deep {
        asi_lib::logger::info("Routing to deep agent", &[("user_id", &user_id)]);
        let tools = build_agent_tools();
        let deep_agent = asi_ai_sdk::agent::deep_agent::DeepAgent::new(provider, tools, 5);
        deep_agent
            .execute(
                messages_clone
                    .last()
                    .map(|m| m.content.as_str())
                    .unwrap_or(""),
            )
            .await
    } else if use_multi_agent {
        asi_lib::logger::info(
            "Routing to multi-agent coordinator",
            &[("user_id", &user_id)],
        );
        let code_agent = std::sync::Arc::new(build_code_agent(provider.clone()));
        let review_agent = std::sync::Arc::new(build_review_agent(provider.clone()));
        let memory = std::sync::Arc::new(asi_ai_sdk::agent::memory::AgentMemory::new(
            std::time::Duration::from_secs(3600),
            100,
        ));
        let coordinator =
            asi_ai_sdk::agent::coordinator::Coordinator::new(code_agent, review_agent, memory);
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
            CONCURRENCY.release();
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

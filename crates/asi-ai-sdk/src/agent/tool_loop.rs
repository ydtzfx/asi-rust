use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::Duration;

use crate::agent::tool::ToolMap;
use crate::provider::AiProvider;
use crate::types::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// LLM response cache to avoid redundant API calls.
/// Keyed by conversation hash, expires after 5 minutes.
static LLM_CACHE: std::sync::LazyLock<Mutex<asi_lib::cache::Cache<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(asi_lib::cache::Cache::new(Duration::from_secs(300))));

/// Hash a conversation to use as a cache key.
fn conversation_hash(messages: &[Message]) -> String {
    let mut hasher = DefaultHasher::new();
    for msg in messages {
        msg.role.hash(&mut hasher);
        msg.content.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

/// Event emitted during agent execution (for SSE streaming).
#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta {
        content: String,
    },
    ToolCall {
        name: String,
        arguments: String,
    },
    ToolResult {
        name: String,
        result: String,
        truncated: bool,
    },
    Done {
        usage: Option<Usage>,
    },
    Error {
        message: String,
    },
}

/// Maximum result length before truncation (shared with runCommand output limit).
const MAX_RESULT_LEN: usize = 8000;

/// A cancellation token for stopping agent execution when the client disconnects.
#[derive(Clone)]
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// A loop-based agent that calls an LLM with tools, executes tool calls,
/// and feeds results back to the LLM until a text-only response is received.
pub struct ToolLoopAgent {
    pub(crate) provider: Arc<dyn AiProvider>,
    pub(crate) instructions: String,
    pub(crate) tools: ToolMap,
    pub(crate) max_steps: usize,
}

impl ToolLoopAgent {
    pub fn new(
        provider: Arc<dyn AiProvider>,
        instructions: String,
        tools: ToolMap,
        max_steps: usize,
    ) -> Self {
        Self {
            provider,
            instructions,
            tools,
            max_steps,
        }
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }
    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    pub async fn execute(
        &self,
        messages: Vec<Message>,
    ) -> Result<(mpsc::UnboundedReceiver<AgentEvent>, CancelToken), String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancelToken::new();
        let provider = Arc::clone(&self.provider);
        let tool_map = self.tools.clone();
        let tool_definitions: Vec<ToolDefinition> =
            self.tools.values().map(|t| t.definition()).collect();
        let max_steps = self.max_steps;
        let tx_clone = tx.clone();
        let cancel_clone = cancel.clone();
        let mut conversation = vec![Message {
            role: Role::System,
            content: self.instructions.clone(),
            tool_calls: None,
            tool_call_id: None,
        }];
        conversation.extend(messages);

        tokio::spawn(async move {
            run_agent_loop(
                provider,
                tool_map,
                tool_definitions,
                conversation,
                max_steps,
                tx_clone,
                cancel_clone,
            )
            .await;
        });
        Ok((rx, cancel))
    }
}

async fn call_provider_streaming(
    provider: &Arc<dyn AiProvider>,
    conversation: &[Message],
    tools: &Option<Vec<ToolDefinition>>,
    tx: &mpsc::UnboundedSender<AgentEvent>,
) -> Result<(String, Option<Vec<ToolCall>>, Option<Usage>), String> {
    let request = ChatRequest {
        model: String::new(),
        messages: conversation.to_vec(),
        tools: tools.clone(),
        temperature: None,
        max_tokens: Some(4096),
        stream: Some(true),
    };

    let cache_key = conversation_hash(conversation);
    if tools.as_ref().is_none_or(|t| t.is_empty())
        && let Some(cached) = LLM_CACHE.lock().ok().and_then(|c| c.get(&cache_key))
    {
        tracing::info!("LLM cache hit");
        let _ = tx.send(AgentEvent::TextDelta {
            content: cached.clone(),
        });
        return Ok((cached, None, None));
    }

    tracing::info!("Calling provider.chat_stream()…");
    let config = asi_lib::retry::RetryConfig::default();
    let mut stream = asi_lib::retry::retry(&config, || provider.chat_stream(request.clone()))
        .await
        .map_err(|e| format!("Provider error after retries: {}", e))?;

    let mut full_content = String::new();
    let mut tool_calls: Option<Vec<ToolCall>> = None;
    let usage: Option<Usage> = None;

    while let Some(chunk) = stream.as_mut().next().await {
        match chunk {
            Ok(chunk) => {
                for choice in &chunk.choices {
                    if let Some(ref content) = choice.delta.content {
                        full_content.push_str(content);
                        if tx
                            .send(AgentEvent::TextDelta {
                                content: content.clone(),
                            })
                            .is_err()
                        {
                            return Ok((full_content, tool_calls, usage));
                        }
                    }
                    if let Some(ref tc) = choice.delta.tool_calls {
                        tool_calls = Some(tc.clone());
                    }
                }
            }
            Err(e) => return Err(format!("Stream error: {}", e)),
        }
    }

    tracing::info!(
        content_len = full_content.len(),
        has_tool_calls = tool_calls.is_some(),
        "Streaming response complete"
    );

    if tool_calls.is_none()
        && !full_content.is_empty()
        && let Ok(cache) = LLM_CACHE.lock()
    {
        cache.set(&cache_key, full_content.clone());
    }

    Ok((full_content, tool_calls, usage))
}

async fn call_provider_non_streaming(
    provider: &Arc<dyn AiProvider>,
    conversation: &[Message],
    tools: &Option<Vec<ToolDefinition>>,
    tx: &mpsc::UnboundedSender<AgentEvent>,
) -> Result<(String, Option<Vec<ToolCall>>, Option<Usage>), String> {
    let request = ChatRequest {
        model: String::new(),
        messages: conversation.to_vec(),
        tools: tools.clone(),
        temperature: None,
        max_tokens: Some(4096),
        stream: Some(false),
    };

    tracing::info!("Calling provider.chat()…");
    let config = asi_lib::retry::RetryConfig::default();
    let response = asi_lib::retry::retry(&config, || provider.chat(request.clone()))
        .await
        .map_err(|e| format!("Provider error after retries: {}", e))?;

    tracing::info!(
        choices = response.choices.len(),
        has_usage = response.usage.is_some(),
        "Provider response received"
    );

    let choice = response.choices.into_iter().next();
    match choice {
        Some(c) => {
            let content = c.message.content;
            if !content.is_empty()
                && tx
                    .send(AgentEvent::TextDelta {
                        content: content.clone(),
                    })
                    .is_err()
            {
                return Ok((content, c.message.tool_calls, response.usage));
            }
            Ok((content, c.message.tool_calls, response.usage))
        }
        None => Ok((String::new(), None, response.usage)),
    }
}

async fn run_agent_loop(
    provider: Arc<dyn AiProvider>,
    tool_map: ToolMap,
    tool_definitions: Vec<ToolDefinition>,
    mut conversation: Vec<Message>,
    max_steps: usize,
    tx: mpsc::UnboundedSender<AgentEvent>,
    cancel: CancelToken,
) {
    let tools_for_request = if tool_definitions.is_empty() {
        None
    } else {
        Some(tool_definitions)
    };

    for step in 0..max_steps {
        if cancel.is_cancelled() {
            tracing::info!("Agent loop cancelled by caller");
            return;
        }
        tracing::info!(step, msg_count = conversation.len(), "Agent loop iteration");

        let result = if step == 0 {
            call_provider_streaming(&provider, &conversation, &tools_for_request, &tx).await
        } else {
            call_provider_non_streaming(&provider, &conversation, &tools_for_request, &tx).await
        };
        let (assistant_content, assistant_tool_calls, usage) = match result {
            Ok(result) => result,
            Err(e) => {
                let _ = tx.send(AgentEvent::Error { message: e });
                return;
            }
        };

        conversation.push(Message {
            role: Role::Assistant,
            content: assistant_content.clone(),
            tool_calls: assistant_tool_calls.clone(),
            tool_call_id: None,
        });

        match &assistant_tool_calls {
            Some(tool_calls) if !tool_calls.is_empty() => {
                for tc in tool_calls {
                    if cancel.is_cancelled() {
                        return;
                    }
                    if tx
                        .send(AgentEvent::ToolCall {
                            name: tc.function.name.clone(),
                            arguments: tc.function.arguments.clone(),
                        })
                        .is_err()
                    {
                        return;
                    }
                    let result = execute_one_tool(&tool_map, tc, &tx).await;
                    conversation.push(Message {
                        role: Role::Tool,
                        content: result,
                        tool_calls: None,
                        tool_call_id: Some(tc.id.clone()),
                    });
                }
            }
            _ => {
                let _ = tx.send(AgentEvent::Done { usage });
                return;
            }
        }
    }

    let _ = tx.send(AgentEvent::Error {
        message: format!(
            "Agent reached maximum steps ({}) without completing the task.",
            max_steps
        ),
    });
}

async fn execute_one_tool(
    tool_map: &ToolMap,
    tc: &ToolCall,
    tx: &mpsc::UnboundedSender<AgentEvent>,
) -> String {
    let name = &tc.function.name;
    let args_str = &tc.function.arguments;
    let tool = tool_map.get(name);
    let (result, truncated) = match tool {
        Some(t) => {
            let parsed_args: serde_json::Value =
                serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);
            match t.execute(parsed_args).await {
                Ok(output) => {
                    let trunc = output.len() > MAX_RESULT_LEN;
                    let display = if trunc {
                        truncate_utf8_safe(&output, MAX_RESULT_LEN)
                    } else {
                        output
                    };
                    (display, trunc)
                }
                Err(e) => (format!("Tool error: {}", e), false),
            }
        }
        None => (format!("Unknown tool: {}", name), false),
    };
    let _ = tx.send(AgentEvent::ToolResult {
        name: name.clone(),
        result: result.clone(),
        truncated,
    });
    result
}

fn truncate_utf8_safe(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_ascii() {
        assert_eq!(truncate_utf8_safe("hello", 3), "hel");
    }

    #[test]
    fn test_truncate_noop() {
        assert_eq!(truncate_utf8_safe("hi", 10), "hi");
    }

    #[test]
    fn test_truncate_multibyte() {
        let s = "你好世界";
        let result = truncate_utf8_safe(s, 3);
        assert_eq!(result, "你");
        assert!(std::str::from_utf8(result.as_bytes()).is_ok());
    }

    #[test]
    fn test_cancel_token() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
}

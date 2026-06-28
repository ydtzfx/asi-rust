use crate::agent::tool::ToolMap;
use crate::provider::AiProvider;
use crate::types::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

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
///
/// Uses `Arc<dyn AiProvider>` internally so the provider can be shared and
/// moved into a spawned task.
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

    /// Maximum number of steps the agent can take in a single run.
    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    /// The instruction string configured for this agent.
    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    /// Execute the agent loop with streaming output.
    /// Returns a channel receiver for [`AgentEvent`]s and a [`CancelToken`]
    /// that the caller can use to stop execution (e.g. on client disconnect).
    pub async fn execute(
        &self,
        messages: Vec<Message>,
    ) -> Result<(mpsc::UnboundedReceiver<AgentEvent>, CancelToken), String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancelToken::new();

        // Capture everything the spawned task needs by value.
        let provider = Arc::clone(&self.provider);
        let tool_map = self.tools.clone();
        let tool_definitions: Vec<ToolDefinition> =
            self.tools.values().map(|t| t.definition()).collect();
        let max_steps = self.max_steps;
        let tx_clone = tx.clone();
        let cancel_clone = cancel.clone();

        // Build system message as the first message in the conversation.
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

/// Core agent loop — runs inside a spawned Tokio task.
///
/// Algorithm:
///  1. Build `ChatRequest` from the current conversation + tool definitions.
///  2. Call `provider.chat()` (non-streaming — full response at once).
///  3. Send any text content as `TextDelta`.
///  4. If the response contains tool calls:
///     a. Fire `ToolCall` events and add the assistant message.
///     b. Execute each tool, fire `ToolResult` events, add tool messages.
///     c. Loop back to step 1.
///  5. If no tool calls: send `Done` with usage and exit.
///  6. If `max_steps` is reached: send an `Error` and exit.
///  7. If cancelled by the caller: stop immediately.
async fn run_agent_loop(
    provider: Arc<dyn AiProvider>,
    tool_map: ToolMap,
    tool_definitions: Vec<ToolDefinition>,
    mut conversation: Vec<Message>,
    max_steps: usize,
    tx: mpsc::UnboundedSender<AgentEvent>,
    cancel: CancelToken,
) {
    let tools_for_request: Option<Vec<ToolDefinition>> = if tool_definitions.is_empty() {
        None
    } else {
        Some(tool_definitions)
    };

    for _step in 0..max_steps {
        // Check for cancellation at the top of each iteration.
        if cancel.is_cancelled() {
            tracing::info!("Agent loop cancelled by caller");
            return;
        }

        tracing::info!(
            step = _step,
            msg_count = conversation.len(),
            "Agent loop iteration"
        );

        let request = ChatRequest {
            model: String::new(), // provider fills this in
            messages: conversation.clone(),
            tools: tools_for_request.clone(),
            temperature: None,
            max_tokens: None,
            stream: Some(false),
        };

        tracing::info!("Calling provider.chat()…");
        let response = match provider.chat(request).await {
            Ok(r) => {
                tracing::info!(
                    choices = r.choices.len(),
                    has_usage = r.usage.is_some(),
                    "Provider response received"
                );
                r
            }
            Err(e) => {
                tracing::error!(error = %e, "Provider call failed");
                let _ = tx.send(AgentEvent::Error {
                    message: format!("Provider error: {}", e),
                });
                return;
            }
        };

        let choice = match response.choices.first() {
            Some(c) => c,
            None => {
                let _ = tx.send(AgentEvent::Done {
                    usage: response.usage,
                });
                return;
            }
        };

        // Send any text content the assistant produced.
        if !choice.message.content.is_empty() {
            if tx
                .send(AgentEvent::TextDelta {
                    content: choice.message.content.clone(),
                })
                .is_err()
            {
                // Receiver dropped — client disconnected.
                tracing::info!("Agent loop: receiver dropped, stopping");
                return;
            }
        }

        // Check for tool calls.
        match &choice.message.tool_calls {
            Some(tool_calls) if !tool_calls.is_empty() => {
                // Push the assistant message (with tool_calls) into the conversation.
                conversation.push(choice.message.clone());

                // Execute each tool and add results as Role::Tool messages.
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
                // Continue the loop — feed tool results back to the LLM.
            }
            _ => {
                // No tool calls: agent is finished.
                let _ = tx.send(AgentEvent::Done {
                    usage: response.usage,
                });
                return;
            }
        }
    }

    // Exhausted all steps without a terminal response.
    let _ = tx.send(AgentEvent::Error {
        message: format!(
            "Agent reached maximum steps ({}) without completing the task.",
            max_steps
        ),
    });
}

/// Execute a single tool call and return its string result.
///
/// Handles argument parsing, tool lookup, execution, and result truncation.
/// Emits a `ToolResult` event through the channel.
/// Uses UTF-8 safe truncation to avoid panics on multi-byte character boundaries.
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

/// Truncate a string to at most `max_bytes` bytes, ensuring the result
/// ends on a valid UTF-8 character boundary. This prevents panics that
/// would occur when slicing in the middle of a multi-byte character.
fn truncate_utf8_safe(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    // Walk backwards from the cut point to find the nearest char boundary.
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
        // "你好世界" = 12 bytes, 4 chars. Truncate to 3 bytes should give "你" (3 bytes).
        let s = "你好世界";
        let result = truncate_utf8_safe(s, 3);
        assert_eq!(result, "你");
        // Verify no panic and the result is valid UTF-8.
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

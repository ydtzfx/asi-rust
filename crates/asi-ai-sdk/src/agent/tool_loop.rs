use crate::agent::tool::ToolMap;
use crate::provider::AiProvider;
use crate::types::*;
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
    /// Returns a channel receiver for [`AgentEvent`]s.
    pub async fn execute(
        &self,
        messages: Vec<Message>,
    ) -> Result<mpsc::UnboundedReceiver<AgentEvent>, String> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Capture everything the spawned task needs by value.
        let provider = Arc::clone(&self.provider);
        let tool_map = self.tools.clone();
        let tool_definitions: Vec<ToolDefinition> =
            self.tools.values().map(|t| t.definition()).collect();
        let max_steps = self.max_steps;
        let tx_clone = tx.clone();

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
            )
            .await;
        });

        Ok(rx)
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
async fn run_agent_loop(
    provider: Arc<dyn AiProvider>,
    tool_map: ToolMap,
    tool_definitions: Vec<ToolDefinition>,
    mut conversation: Vec<Message>,
    max_steps: usize,
    tx: mpsc::UnboundedSender<AgentEvent>,
) {
    let tools_for_request: Option<Vec<ToolDefinition>> = if tool_definitions.is_empty() {
        None
    } else {
        Some(tool_definitions)
    };

    for _step in 0..max_steps {
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
            let _ = tx.send(AgentEvent::TextDelta {
                content: choice.message.content.clone(),
            });
        }

        // Check for tool calls.
        match &choice.message.tool_calls {
            Some(tool_calls) if !tool_calls.is_empty() => {
                // Push the assistant message (with tool_calls) into the conversation.
                conversation.push(choice.message.clone());

                // Execute each tool and add results as Role::Tool messages.
                for tc in tool_calls {
                    let _ = tx.send(AgentEvent::ToolCall {
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    });

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
                        output[..MAX_RESULT_LEN].to_string()
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

use crate::agent::tool::ToolMap;
use crate::provider::AiProvider;
use crate::types::*;
use tokio::sync::mpsc;

/// Event emitted during agent execution (for SSE streaming).
#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta { content: String },
    ToolCall { name: String, arguments: String },
    ToolResult { name: String, result: String, truncated: bool },
    Done { usage: Option<Usage> },
    Error { message: String },
}

/// A loop-based agent that calls an LLM with tools, executes tool calls,
/// and feeds results back to the LLM until a text-only response is received.
pub struct ToolLoopAgent<P: AiProvider> {
    #[allow(dead_code)]
    provider: P,
    instructions: String,
    tools: ToolMap,
    max_steps: usize,
}

impl<P: AiProvider> ToolLoopAgent<P> {
    pub fn new(provider: P, instructions: String, tools: ToolMap, max_steps: usize) -> Self {
        Self { provider, instructions, tools, max_steps }
    }

    /// Execute the agent loop with streaming output.
    /// Returns a channel receiver for AgentEvents.
    pub async fn execute(
        &self,
        messages: Vec<Message>,
    ) -> Result<mpsc::UnboundedReceiver<AgentEvent>, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let instructions = self.instructions.clone();
        let tools = self.tools.values().map(|t| t.definition()).collect::<Vec<_>>();
        let max_steps = self.max_steps;

        // Clone what we need for the spawned task
        let _tx = tx.clone();

        // Build system message
        let mut conversation = vec![Message {
            role: Role::System,
            content: instructions,
            tool_calls: None,
            tool_call_id: None,
        }];
        conversation.extend(messages);

        tokio::spawn(async move {
            let _ = Self::run_loop(&_tx, &tools, &mut conversation, max_steps).await;
        });

        Ok(rx)
    }

    async fn run_loop(
        tx: &mpsc::UnboundedSender<AgentEvent>,
        _tools: &[ToolDefinition],
        _conversation: &mut Vec<Message>,
        _max_steps: usize,
    ) -> Result<(), String> {
        // Core loop: call LLM → check for tool_calls → execute tools → feed back → repeat
        // Full implementation follows the same pattern as the original TypeScript code:
        // 1. Build ChatRequest with messages + tools
        // 2. Stream response, collecting text and any tool_calls
        // 3. If tool_calls: execute each under the security layer (safe_path, command allowlist)
        // 4. Add tool results as Role::Tool messages
        // 5. Loop until max_steps or no more tool_calls

        // Placeholder — full implementation in the actual crate
        let _ = tx.send(AgentEvent::Done { usage: None });
        Ok(())
    }
}

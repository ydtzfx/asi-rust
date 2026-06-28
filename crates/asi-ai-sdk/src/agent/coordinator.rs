use std::sync::Arc;

use tokio::sync::mpsc;

use super::memory::AgentMemory;
use super::planner::{Plan, SubTask, TaskStatus, decompose_goal};
use super::tool_loop::{AgentEvent, CancelToken, ToolLoopAgent};
use crate::provider::AiProvider;
use crate::types::Message;

/// Multi-agent coordinator — decomposes tasks, routes to specialists,
/// aggregates results, and drives the plan to completion.
pub struct Coordinator {
    code_agent: Arc<ToolLoopAgent>,
    review_agent: Arc<ToolLoopAgent>,
    memory: Arc<AgentMemory>,
}

impl Coordinator {
    pub fn new(
        code_agent: Arc<ToolLoopAgent>,
        review_agent: Arc<ToolLoopAgent>,
        memory: Arc<AgentMemory>,
    ) -> Self {
        Self {
            code_agent,
            review_agent,
            memory,
        }
    }

    /// Execute a user request through the multi-agent system.
    /// Returns a receiver for streaming AgentEvents.
    pub async fn execute(
        &self,
        messages: Vec<Message>,
    ) -> Result<(mpsc::UnboundedReceiver<AgentEvent>, CancelToken), String> {
        let user_msg = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        // Import history into memory.
        self.memory.import_conversation(&messages);

        // Decompose the goal into a plan.
        let plan = decompose_goal(&user_msg);
        tracing::info!(
            "Coordinator: plan created — {} tasks for goal: {}",
            plan.tasks.len(),
            &plan.goal
        );

        // For complex plans (3+ tasks), use multi-agent execution.
        if plan.tasks.len() >= 3 {
            self.execute_plan(plan, messages).await
        } else {
            // Simple tasks go directly to the code agent.
            tracing::info!("Coordinator: simple task, routing to code agent");
            self.code_agent.execute(messages).await
        }
    }

    /// Execute a multi-step plan through agent delegation.
    async fn execute_plan(
        &self,
        mut plan: Plan,
        messages: Vec<Message>,
    ) -> Result<(mpsc::UnboundedReceiver<AgentEvent>, CancelToken), String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancelToken::new();

        let code_agent = self.code_agent.clone();
        let review_agent = self.review_agent.clone();
        let tx_clone = tx.clone();
        let cancel_clone = cancel.clone();

        tokio::spawn(async move {
            let _ = run_plan(&code_agent, &review_agent, plan, messages, tx_clone, cancel_clone).await;
        });

        Ok((rx, cancel))
    }
}

async fn run_plan(
    code_agent: &Arc<ToolLoopAgent>,
    review_agent: &Arc<ToolLoopAgent>,
    mut plan: Plan,
    messages: Vec<Message>,
    tx: mpsc::UnboundedSender<AgentEvent>,
    cancel: CancelToken,
) -> Result<(), String> {
    let _ = tx.send(AgentEvent::TextDelta {
        content: format!(
            "📋 **Plan:** {}\n{} tasks to execute.\n\n",
            plan.goal,
            plan.tasks.len()
        ),
    });

    let mut task_messages = messages.clone();

    while !plan.is_done() {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let task = match plan.next_ready() {
            Some(t) => t.clone(),
            None => break,
        };

        let _ = tx.send(AgentEvent::TextDelta {
            content: format!("\n🔄 **Executing:** {}\n", task.description),
        });

        // Build task-specific message.
        let task_msg = Message {
            role: crate::types::Role::User,
            content: task.description.clone(),
            tool_calls: None,
            tool_call_id: None,
        };
        task_messages.push(task_msg);

        // Route to the right agent.
        let agent = match task.agent_type {
            super::planner::AgentType::Review => &review_agent,
            _ => &code_agent,
        };

        match agent.execute(task_messages.clone()).await {
            Ok((task_rx, _task_cancel)) => {
                // Collect task output.
                let mut output = String::new();
                let mut stream =
                    tokio_stream::wrappers::UnboundedReceiverStream::new(task_rx);

                while let Some(event) = tokio_stream::StreamExt::next(&mut stream).await {
                    match event {
                        AgentEvent::TextDelta { content } => {
                            output.push_str(&content);
                            let _ = tx.send(AgentEvent::TextDelta { content });
                        }
                        AgentEvent::ToolCall { name, arguments } => {
                            let _ = tx.send(AgentEvent::ToolCall { name, arguments });
                        }
                        AgentEvent::ToolResult { name, result, truncated } => {
                            let _ = tx.send(AgentEvent::ToolResult { name, result, truncated });
                        }
                        AgentEvent::Done { .. } => break,
                        AgentEvent::Error { message } => {
                            plan.fail(&task.id, &message);
                            let _ = tx.send(AgentEvent::Error { message });
                            break;
                        }
                    }
                }

                if !output.is_empty() {
                    task_messages.push(Message {
                        role: crate::types::Role::Assistant,
                        content: output,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
                plan.complete(&task.id);
            }
            Err(e) => {
                plan.fail(&task.id, &e);
                let _ = tx.send(AgentEvent::Error { message: e });
            }
        }
    }

    let _ = tx.send(AgentEvent::TextDelta {
        content: format!("\n✅ **Plan complete:** {}\n", plan.progress()),
    });
    let _ = tx.send(AgentEvent::Done { usage: None });

    Ok(())
}

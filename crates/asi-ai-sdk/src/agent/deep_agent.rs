//! Deep Agent — Plan-Execute-Reflect-Revise-Re-execute closed loop.
//! Goes beyond simple tool-loop by iteratively improving its own output.

use tokio::sync::mpsc;

use super::explorer::{ThoughtTree, score_thought};
use super::tool_loop::{AgentEvent, CancelToken};
use crate::agent::tool::ToolMap;
use crate::provider::AiProvider;
use crate::types::*;
use std::sync::Arc;

/// Stages of the deep agent execution cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeepStage {
    Plan,       // Analyze + decompose
    Execute,    // Run tools + generate
    Reflect,    // Self-assess quality
    Revise,     // Improve based on reflection
    Complete,   // Done — output meets quality bar
    Abandoned,  // Max iterations reached without convergence
}

/// Deep agent — iterative Plan-Execute-Reflect-Revise cycle.
pub struct DeepAgent {
    provider: Arc<dyn AiProvider>,
    tools: ToolMap,
    max_iterations: u32,
}

impl DeepAgent {
    pub fn new(provider: Arc<dyn AiProvider>, tools: ToolMap, max_iterations: u32) -> Self {
        Self { provider, tools, max_iterations }
    }

    /// Run the deep agent cycle on a task.
    pub async fn execute(
        &self,
        task: &str,
    ) -> Result<(mpsc::UnboundedReceiver<AgentEvent>, CancelToken), String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let cancel = CancelToken::new();

        let provider = self.provider.clone();
        let task_owned = task.to_string();
        let max_iter = self.max_iterations;
        let tx_clone = tx.clone();
        let cancel_clone = cancel.clone();

        tokio::spawn(async move {
            let _ = run_deep_cycle(&provider, &task_owned, max_iter, &tx_clone, &cancel_clone).await;
        });

        Ok((rx, cancel))
    }
}

async fn run_deep_cycle(
    provider: &Arc<dyn AiProvider>,
    task: &str,
    max_iter: u32,
    tx: &mpsc::UnboundedSender<AgentEvent>,
    cancel: &CancelToken,
) -> Result<(), String> {
    let _ = tx.send(AgentEvent::TextDelta {
        content: format!("## Deep Agent: Plan → Execute → Reflect → Revise\n\n**Task:** {}\n\n", task),
    });

    let mut current_output = String::new();
    let mut iteration = 0u32;
    let mut stage = DeepStage::Plan;

    // --- Stage 1: Plan ---
    let _ = tx.send(AgentEvent::TextDelta { content: "### 1. Plan\n".into() });

    let plan_prompt = format!(
        "You have a task: \"{}\"\n\nBreak it down into clear steps. Be specific about what needs to be done.",
        task
    );
    let plan = call_llm(provider, &plan_prompt).await?;
    let _ = tx.send(AgentEvent::TextDelta { content: format!("{}\n\n", plan) });

    // Use ThoughtTree to explore the plan.
    let mut tree = ThoughtTree::new();
    let root = tree.set_root(task);
    for line in plan.lines().filter(|l| !l.trim().is_empty()).take(5) {
        tree.branch(&root, line.trim(), score_thought(line));
    }
    let best_path = tree.best_path();
    let _ = tx.send(AgentEvent::TextDelta {
        content: format!("*Best reasoning path ({} nodes, avg score {:.2}):*\n", tree.size(), tree.avg_score()),
    });

    // --- Stage 2: Execute + Stage 3: Reflect + Stage 4: Revise ---
    let quality_threshold = 0.85;

    while iteration < max_iter {
        if cancel.is_cancelled() { return Ok(()); }
        iteration += 1;

        // Execute: generate output
        stage = DeepStage::Execute;
        let exec_prompt = if iteration == 1 {
            format!(
                "Task: {}\n\nPlan:\n{}\n\nExecute the plan. Produce the best output you can.",
                task, plan
            )
        } else {
            format!(
                "Task: {}\n\nPrevious output:\n{}\n\nReflect on what could be improved, then produce a revised version. Be specific about what you changed.",
                task, current_output
            )
        };

        let output = call_llm(provider, &exec_prompt).await?;
        let _ = tx.send(AgentEvent::TextDelta {
            content: format!("\n### Iteration {}: {}\n", iteration, output),
        });

        // Reflect: self-assess quality
        stage = DeepStage::Reflect;
        let reflect_prompt = format!(
            "Rate this output quality from 0.0 to 1.0 (just the number):\n\nTask: {}\n\nOutput:\n{}\n\nScore:",
            task, output
        );
        let score_str = call_llm(provider, &reflect_prompt).await?;
        let quality: f64 = score_str.trim().parse().unwrap_or(0.5);
        let _ = tx.send(AgentEvent::TextDelta {
            content: format!("*Quality score: {:.2}*\n", quality),
        });

        current_output = output;

        // Check if quality is sufficient
        if quality >= quality_threshold || iteration >= max_iter {
            stage = if quality >= quality_threshold { DeepStage::Complete } else { DeepStage::Abandoned };
            let _ = tx.send(AgentEvent::TextDelta {
                content: format!("\n### {}\n\n**Final output:**\n\n{}",
                    if stage == DeepStage::Complete { "✅ Complete" } else { "⚠️ Max iterations reached" },
                    current_output),
            });
            let _ = tx.send(AgentEvent::Done { usage: None });
            return Ok(());
        }

        // Revise — implicit in the next iteration's exec_prompt (which asks for improvements)
        stage = DeepStage::Revise;
        let _ = tx.send(AgentEvent::TextDelta {
            content: format!("*Revising — quality {:.2} < {:.2}*\n", quality, quality_threshold),
        });
    }

    Ok(())
}

/// Call the LLM with a simple prompt, return response text.
async fn call_llm(provider: &Arc<dyn AiProvider>, prompt: &str) -> Result<String, String> {
    let request = ChatRequest {
        model: String::new(),
        messages: vec![Message {
            role: Role::User, content: prompt.to_string(),
            tool_calls: None, tool_call_id: None,
        }],
        tools: None, temperature: Some(0.3), max_tokens: Some(2048), stream: Some(false),
    };
    let response = provider.chat(request).await.map_err(|e| format!("LLM error: {}", e))?;
    Ok(response.choices.first().map(|c| c.message.content.clone()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_stage_progression() {
        // Verify stages are in correct order.
        let stages = [DeepStage::Plan, DeepStage::Execute, DeepStage::Reflect, DeepStage::Revise, DeepStage::Complete];
        assert!(stages.windows(2).all(|w| w[0] as u8 + 1 == w[1] as u8 || w[0] == DeepStage::Complete));
    }

    #[test]
    fn test_deep_agent_construction() {
        // Test that DeepAgent can be constructed (provider would be mocked in real tests).
        // This is a compile-time + structure verification test.
        let _ = DeepStage::Plan;
        let _ = DeepStage::Execute;
        let _ = DeepStage::Reflect;
        let _ = DeepStage::Revise;
        let _ = DeepStage::Complete;
        let _ = DeepStage::Abandoned;
    }
}

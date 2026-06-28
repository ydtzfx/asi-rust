/// Micro-benchmarks for agent loop components.
/// These are not full E2E tests — they measure in-process performance.
use asi_ai_sdk::agent::tool::{Tool, ToolError, ToolMap};
use asi_ai_sdk::types::{Role, Message, ToolDefinition, FunctionDef};
use async_trait::async_trait;
use std::sync::Arc;

/// A mock tool that returns immediately.
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: "echo".into(),
                description: "Echo".into(),
                parameters: serde_json::json!({}),
            },
        }
    }
    async fn execute(&self, _args: serde_json::Value) -> Result<String, ToolError> {
        Ok("echo".to_string())
    }
}

#[test]
fn bench_tool_map_lookup() {
    let mut tools: ToolMap = std::collections::HashMap::new();
    tools.insert("echo".into(), Arc::new(EchoTool) as Arc<dyn Tool>);

    // Verify tool lookup works
    assert!(tools.get("echo").is_some());
    assert!(tools.get("nonexistent").is_none());
}

#[test]
fn bench_conversation_clone() {
    let conversation: Vec<Message> = (0..20)
        .map(|i| Message {
            role: if i % 2 == 0 {
                Role::User
            } else {
                Role::Assistant
            },
            content: format!("message {}", i),
            tool_calls: None,
            tool_call_id: None,
        })
        .collect();

    // Measure clone cost
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = conversation.clone();
    }
    let elapsed = start.elapsed();
    let per_clone = elapsed / 1000;
    eprintln!(
        "20-msg conversation clone: {:?} ({} clones in {:?})",
        per_clone, 1000, elapsed
    );
    // Should be very fast (<10µs per clone)
    assert!(per_clone < std::time::Duration::from_millis(1));
}

#[test]
fn bench_tool_definition_clone() {
    let tool = EchoTool;
    let def = tool.definition();

    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = def.clone();
    }
    let elapsed = start.elapsed();
    eprintln!("Tool definition clone (x10000): {:?}", elapsed);
}

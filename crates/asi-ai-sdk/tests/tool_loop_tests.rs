use asi_ai_sdk::agent::tool::{Tool, ToolError};
use asi_ai_sdk::types::{FunctionDef, ToolDefinition};
use async_trait::async_trait;
use serde_json::json;

/// A mock tool for testing — returns a fixed result.
struct MockTool {
    name: String,
    result: String,
}

#[async_trait]
impl Tool for MockTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: self.name.clone(),
                description: "A mock tool for testing".into(),
                parameters: json!({ "type": "object", "properties": {} }),
            },
        }
    }

    async fn execute(&self, _arguments: serde_json::Value) -> Result<String, ToolError> {
        Ok(self.result.clone())
    }
}

#[test]
fn test_mock_tool_definition() {
    let tool = MockTool {
        name: "mock".into(),
        result: "ok".into(),
    };
    let def = tool.definition();
    assert_eq!(def.function.name, "mock");
    assert_eq!(def.def_type, "function");
}

#[test]
fn test_agent_event_types() {
    use asi_ai_sdk::agent::tool_loop::AgentEvent;
    let text = AgentEvent::TextDelta {
        content: "hello".into(),
    };
    let done = AgentEvent::Done { usage: None };
    // Verify they are the expected variants
    assert!(matches!(text, AgentEvent::TextDelta { .. }));
    assert!(matches!(done, AgentEvent::Done { .. }));
}

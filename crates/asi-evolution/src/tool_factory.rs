use asi_ai_sdk::agent::tool::{Tool, ToolError, ToolMap};
use asi_ai_sdk::types::{FunctionDef, ToolDefinition};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// A dynamically created tool from a specification.
pub struct DynamicTool {
    name: String,
    description: String,
    handler: Box<dyn Fn(serde_json::Value) -> String + Send + Sync>,
}

impl DynamicTool {
    pub fn new(
        name: &str,
        description: &str,
        handler: impl Fn(serde_json::Value) -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            handler: Box::new(handler),
        }
    }
}

#[async_trait]
impl Tool for DynamicTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, ToolError> {
        Ok((self.handler)(arguments))
    }
}

/// Factory for creating and registering tools dynamically.
pub struct ToolFactory {
    registry: HashMap<String, Arc<dyn Tool>>,
}

impl ToolFactory {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Create a tool from a JSON specification and register it.
    pub fn create_from_spec(&mut self, spec: &serde_json::Value) -> Result<String, String> {
        let name = spec["name"]
            .as_str()
            .ok_or("Missing 'name' in tool spec")?;
        let description = spec["description"]
            .as_str()
            .unwrap_or("Dynamic tool");

        let handler: Box<dyn Fn(serde_json::Value) -> String + Send + Sync> =
            match spec["handler_type"].as_str().unwrap_or("echo") {
                "http_get" => {
                    let url = spec["url"].as_str().unwrap_or("http://localhost").to_string();
                    Box::new(move |_args: serde_json::Value| -> String {
                        // In production, use reqwest::blocking::get.
                        format!("HTTP GET {} → mock response", url)
                    })
                }
                "shell" => {
                    let cmd = spec["command"]
                        .as_str()
                        .unwrap_or("echo")
                        .to_string();
                    Box::new(move |_args: serde_json::Value| -> String {
                        format!("Executed: {}", cmd)
                    })
                }
                _ => {
                    // Echo tool — returns the arguments as-is.
                    let label = name.to_string();
                    Box::new(move |args: serde_json::Value| -> String {
                        format!("[{}] received: {}", label, args)
                    })
                }
            };

        let tool = Arc::new(DynamicTool::new(name, description, handler));
        self.registry.insert(name.to_string(), tool);
        Ok(name.to_string())
    }

    /// Register all factory tools into a ToolMap for use by an agent.
    pub fn install_into(&self, tool_map: &mut ToolMap) {
        for (name, tool) in &self.registry {
            tool_map.insert(name.clone(), tool.clone());
        }
    }

    /// List registered tools.
    pub fn list(&self) -> Vec<String> {
        self.registry.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_echo_tool() {
        let mut factory = ToolFactory::new();
        let name = factory
            .create_from_spec(&serde_json::json!({
                "name": "greet",
                "description": "Greet the user"
            }))
            .unwrap();
        assert_eq!(name, "greet");
        assert_eq!(factory.list().len(), 1);
    }

    #[test]
    fn test_create_multiple_tools() {
        let mut factory = ToolFactory::new();
        factory
            .create_from_spec(&serde_json::json!({
                "name": "search_docs",
                "description": "Search documentation",
                "handler_type": "http_get",
                "url": "https://docs.example.com/search"
            }))
            .unwrap();
        factory
            .create_from_spec(&serde_json::json!({
                "name": "run_benchmark",
                "description": "Run performance benchmark",
                "handler_type": "shell",
                "command": "cargo bench"
            }))
            .unwrap();
        assert_eq!(factory.list().len(), 2);
    }
}

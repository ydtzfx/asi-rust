use crate::types::ToolDefinition;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool execution failed: {0}")]
    Execution(String),
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
}

/// A callable tool that an agent can invoke.
#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, arguments: serde_json::Value) -> Result<String, ToolError>;
}

/// Wrapper to store tools in a HashMap by name.
pub type ToolMap = std::collections::HashMap<String, Arc<dyn Tool>>;

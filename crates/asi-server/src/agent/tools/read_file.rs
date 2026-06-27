use asi_ai_sdk::agent::tool::{Tool, ToolError};
use asi_ai_sdk::types::{FunctionDef, ToolDefinition};
use async_trait::async_trait;
use serde_json::Value;

/// Tool that reads a file from the local filesystem.
///
/// Uses `asi_lib::safe_path::resolve_safe_path()` for path containment:
/// any path that resolves outside the project root is rejected with
/// an execution error.
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: "readFile".into(),
                description: "Read the contents of a file from the local filesystem. ".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path relative to the project root"
                        }
                    },
                    "required": ["path"]
                }),
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<String, ToolError> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing 'path' argument".into()))?;

        let safe_path = asi_lib::safe_path::resolve_safe_path(path)
            .await
            .map_err(ToolError::Execution)?;

        let content = tokio::fs::read_to_string(&safe_path)
            .await
            .map_err(|e| ToolError::Execution(format!("Failed to read file: {}", e)))?;

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_read_file_success() {
        let tool = ReadFileTool;
        // Cargo.toml always exists at project root
        let result = tool.execute(json!({ "path": "Cargo.toml" })).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.is_empty(), "File content should not be empty");
    }

    #[tokio::test]
    async fn test_read_file_missing_path() {
        let tool = ReadFileTool;
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArgs(_)));
    }

    #[tokio::test]
    async fn test_read_file_non_existent() {
        let tool = ReadFileTool;
        let result = tool
            .execute(json!({ "path": "nonexistent_file_xyz123.tmp" }))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_file_path_traversal_blocked() {
        let tool = ReadFileTool;
        let result = tool.execute(json!({ "path": "../etc/passwd" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Access denied") || err.contains("denied"));
    }

    #[test]
    fn test_read_file_definition() {
        let tool = ReadFileTool;
        let def = tool.definition();
        assert_eq!(def.function.name, "readFile");
        assert_eq!(def.def_type, "function");
    }
}

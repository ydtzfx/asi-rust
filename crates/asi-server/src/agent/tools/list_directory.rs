use asi_ai_sdk::agent::tool::{Tool, ToolError};
use asi_ai_sdk::types::{FunctionDef, ToolDefinition};
use async_trait::async_trait;
use serde_json::Value;

/// Tool that lists entries in a directory on the local filesystem.
///
/// Uses `asi_lib::safe_path::resolve_safe_path()` for path containment.
/// Returns a newline-separated list with `[type] name` format where
/// type is `DIR`, `FILE`, or `SYMLINK`.
pub struct ListDirectoryTool;

#[async_trait]
impl Tool for ListDirectoryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: "listDirectory".into(),
                description: "List entries in a directory on the local filesystem. ".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory path relative to the project root"
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

        let mut entries = tokio::fs::read_dir(&safe_path)
            .await
            .map_err(|e| ToolError::Execution(format!("Failed to read directory: {}", e)))?;

        let mut output: Vec<String> = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ToolError::Execution(format!("Failed to read directory entry: {}", e)))?
        {
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| ToolError::Execution(format!("Failed to read entry type: {}", e)))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let prefix = if file_type.is_dir() {
                "DIR"
            } else if file_type.is_symlink() {
                "SYMLINK"
            } else {
                "FILE"
            };
            output.push(format!("[{}] {}", prefix, name));
        }

        output.sort();
        if output.is_empty() {
            Ok("(empty directory)".into())
        } else {
            Ok(output.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_list_directory_success() {
        let tool = ListDirectoryTool;
        let result = tool.execute(json!({ "path": "." })).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should contain directory entries (at minimum Cargo.toml)
        assert!(!output.is_empty(), "Directory listing should not be empty");
    }

    #[tokio::test]
    async fn test_list_directory_missing_path() {
        let tool = ListDirectoryTool;
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArgs(_)));
    }

    #[tokio::test]
    async fn test_list_directory_non_existent() {
        let tool = ListDirectoryTool;
        let result = tool
            .execute(json!({ "path": "nonexistent_dir_xyz123" }))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_directory_path_traversal_blocked() {
        let tool = ListDirectoryTool;
        let result = tool.execute(json!({ "path": "../etc" })).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Access denied") || err.contains("denied"));
    }

    #[test]
    fn test_list_directory_definition() {
        let tool = ListDirectoryTool;
        let def = tool.definition();
        assert_eq!(def.function.name, "listDirectory");
        assert_eq!(def.def_type, "function");
    }
}

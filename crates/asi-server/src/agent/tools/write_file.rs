use asi_ai_sdk::agent::tool::{Tool, ToolError};
use asi_ai_sdk::types::{FunctionDef, ToolDefinition};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

/// Tool that writes content to a file on the local filesystem.
///
/// Uses `asi_lib::safe_path::resolve_safe_path()` for path containment.
/// Automatically creates parent directories if they do not exist.
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            def_type: "function".into(),
            function: FunctionDef {
                name: "writeFile".into(),
                description: "Write content to a file on the local filesystem. ".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path relative to the project root"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }),
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<String, ToolError> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing 'path' argument".into()))?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing 'content' argument".into()))?;

        let safe_path = asi_lib::safe_path::resolve_safe_path(path)
            .await
            .map_err(ToolError::Execution)?;

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(&safe_path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ToolError::Execution(format!("Failed to create directories: {}", e))
            })?;
        }

        tokio::fs::write(&safe_path, content)
            .await
            .map_err(|e| ToolError::Execution(format!("Failed to write file: {}", e)))?;

        // Re-verify after write to close the TOCTOU window.
        asi_lib::safe_path::verify_path_after_write(&safe_path)
            .await
            .map_err(ToolError::Execution)?;

        Ok(format!(
            "Successfully wrote {} bytes to {}",
            content.len(),
            path
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::fs;

    #[tokio::test]
    async fn test_write_file_success() {
        let tool = WriteFileTool;
        let test_path = "target/_test_write_file.txt";
        let test_content = "hello from test";

        // Clean up if leftover
        let _ = fs::remove_file(
            asi_lib::safe_path::resolve_safe_path(test_path)
                .await
                .unwrap(),
        )
        .await;

        let result = tool
            .execute(json!({ "path": test_path, "content": test_content }))
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Successfully wrote"));

        // Verify file was written
        let abs = asi_lib::safe_path::resolve_safe_path(test_path)
            .await
            .unwrap();
        let written = fs::read_to_string(&abs).await.unwrap();
        assert_eq!(written, test_content);

        // Clean up
        let _ = fs::remove_file(&abs).await;
    }

    #[tokio::test]
    async fn test_write_file_missing_args() {
        let tool = WriteFileTool;
        // Missing path
        let result = tool.execute(json!({ "content": "data" })).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArgs(_)));

        // Missing content
        let result = tool.execute(json!({ "path": "test.txt" })).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArgs(_)));
    }

    #[tokio::test]
    async fn test_write_file_path_traversal_blocked() {
        let tool = WriteFileTool;
        let result = tool
            .execute(json!({ "path": "../outside.txt", "content": "data" }))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Access denied") || err.contains("denied"));
    }

    #[tokio::test]
    async fn test_write_file_creates_parent_dirs() {
        let tool = WriteFileTool;
        let test_path = "target/_test_nested_dir/_test_subdir/test_file.txt";
        let test_content = "nested test";

        // Clean up any leftover
        if let Ok(abs) = asi_lib::safe_path::resolve_safe_path(test_path).await {
            let _ = fs::remove_file(&abs).await;
        }

        let result = tool
            .execute(json!({ "path": test_path, "content": test_content }))
            .await;
        assert!(result.is_ok());

        // Verify file was written
        let abs = asi_lib::safe_path::resolve_safe_path(test_path)
            .await
            .unwrap();
        let written = fs::read_to_string(&abs).await.unwrap();
        assert_eq!(written, test_content);

        // Clean up recursively
        let parent = abs.parent().unwrap();
        let _ = fs::remove_dir_all(parent).await;
    }

    #[test]
    fn test_write_file_definition() {
        let tool = WriteFileTool;
        let def = tool.definition();
        assert_eq!(def.function.name, "writeFile");
        assert_eq!(def.def_type, "function");
    }
}

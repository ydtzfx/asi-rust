use axum::{Json, Router, routing::get};
use serde_json::Value;

use crate::agent::tools::list_directory::ListDirectoryTool;
use crate::agent::tools::read_file::ReadFileTool;
use crate::agent::tools::run_command::RunCommandTool;
use crate::agent::tools::write_file::WriteFileTool;

use asi_ai_sdk::agent::tool::Tool;
use asi_ai_sdk::types::ToolDefinition;

// ---------------------------------------------------------------------------
// Wire all tool definitions
// ---------------------------------------------------------------------------

fn collect_tools() -> Vec<ToolDefinition> {
    vec![
        ReadFileTool.definition(),
        WriteFileTool.definition(),
        ListDirectoryTool.definition(),
        RunCommandTool.definition(),
    ]
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// GET /api/tools — returns the full list of agent tool definitions as JSON.
///
/// Each tool definition follows the OpenAI function-calling schema:
/// ```json
/// {
///   "type": "function",
///   "function": {
///     "name": "...",
///     "description": "...",
///     "parameters": { ... }
///   }
/// }
/// ```
async fn list_tools() -> Json<Value> {
    let defs = collect_tools();
    Json(serde_json::json!(defs))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router {
    Router::new().route("/tools", get(list_tools))
}

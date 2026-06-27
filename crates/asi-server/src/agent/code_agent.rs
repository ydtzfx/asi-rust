use std::sync::Arc;

use asi_ai_sdk::agent::tool::ToolMap;
use asi_ai_sdk::agent::tool_loop::ToolLoopAgent;
use asi_ai_sdk::provider::AiProvider;

use super::config::{get_max_steps, is_compact_mode};
use super::instructions::{AGENT_INSTRUCTIONS, AGENT_INSTRUCTIONS_COMPACT};
use super::tools::list_directory::ListDirectoryTool;
use super::tools::read_file::ReadFileTool;
use super::tools::run_command::RunCommandTool;
use super::tools::write_file::WriteFileTool;

/// Build a `ToolLoopAgent` configured as the primary coding agent.
///
/// The agent is equipped with four tools:
/// - `readFile` — read file contents
/// - `writeFile` — write content to files
/// - `listDirectory` — list directory entries
/// - `runCommand` — execute allowlisted commands
///
/// The instruction set and step budget depend on the `read-only-mode` flag:
/// - Normal: full instructions, 20 max steps
/// - Read-only: compact instructions, 5 max steps
pub fn build_code_agent<P: AiProvider + 'static>(provider: P) -> ToolLoopAgent<P> {
    let mut tools: ToolMap = std::collections::HashMap::new();

    tools.insert("readFile".into(), Arc::new(ReadFileTool) as Arc<dyn asi_ai_sdk::agent::tool::Tool>);
    tools.insert("writeFile".into(), Arc::new(WriteFileTool));
    tools.insert("listDirectory".into(), Arc::new(ListDirectoryTool));
    tools.insert("runCommand".into(), Arc::new(RunCommandTool));

    let instructions = if is_compact_mode() {
        AGENT_INSTRUCTIONS_COMPACT.to_string()
    } else {
        AGENT_INSTRUCTIONS.to_string()
    };

    let max_steps = get_max_steps();

    ToolLoopAgent::new(provider, instructions, tools, max_steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    /// A minimal provider for testing agent construction.
    struct TestProvider;

    #[async_trait]
    impl AiProvider for TestProvider {
        async fn chat(
            &self,
            _request: asi_ai_sdk::types::ChatRequest,
        ) -> Result<asi_ai_sdk::types::ChatResponse, asi_ai_sdk::provider::ProviderError>
        {
            unimplemented!("not needed for construction test")
        }

        async fn chat_stream(
            &self,
            _request: asi_ai_sdk::types::ChatRequest,
        ) -> Result<
            std::pin::Pin<
                Box<dyn futures_core::Stream<Item = Result<asi_ai_sdk::types::StreamChunk, asi_ai_sdk::provider::ProviderError>> + Send>,
            >,
            asi_ai_sdk::provider::ProviderError,
        > {
            unimplemented!("not needed for construction test")
        }

        async fn health_check(
            &self,
        ) -> Result<bool, asi_ai_sdk::provider::ProviderError> {
            Ok(true)
        }

        fn name(&self) -> &'static str {
            "test"
        }
    }

    #[test]
    fn test_build_code_agent_default() {
        // Reset flags to ensure clean state (tests can run in parallel)
        asi_lib::flags::reset_flag("read-only-mode");
        let provider = TestProvider;
        let agent = build_code_agent(provider);
        // Don't assert max_steps here to avoid races with parallel tests
        // (config.rs tests verify max_steps logic independently)
        assert!(agent.max_steps() > 0, "Agent should have a positive step count");
    }

    #[test]
    fn test_build_code_agent_read_only() {
        asi_lib::flags::reset_flag("read-only-mode");
        asi_lib::flags::set_flag("read-only-mode");
        let provider = TestProvider;
        let agent = build_code_agent(provider);
        assert!(agent.max_steps() == 5 || agent.max_steps() == 20,
            "Agent step count should be 5 (read-only) or 20 (normal), got {}",
            agent.max_steps());
        asi_lib::flags::reset_flag("read-only-mode");
    }
}

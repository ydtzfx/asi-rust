use std::sync::Arc;

use asi_ai_sdk::agent::tool::ToolMap;
use asi_ai_sdk::agent::tool_loop::ToolLoopAgent;
use asi_ai_sdk::provider::AiProvider;

use super::instructions::AGENT_INSTRUCTIONS_COMPACT;
use super::tools::list_directory::ListDirectoryTool;
use super::tools::read_file::ReadFileTool;

/// Maximum steps for the review agent (fewer than code agent since review is focused).
const REVIEW_MAX_STEPS: usize = 15;

/// Build a `ToolLoopAgent` configured as a read-only code review agent.
///
/// The agent is equipped with two read-only tools:
/// - `readFile` — read file contents
/// - `listDirectory` — list directory entries
///
/// Uses the compact instruction set and runs for a maximum of 15 steps.
/// This agent cannot modify files or execute commands.
pub fn build_review_agent<P: AiProvider + 'static>(provider: P) -> ToolLoopAgent<P> {
    let mut tools: ToolMap = std::collections::HashMap::new();

    tools.insert("readFile".into(), Arc::new(ReadFileTool) as Arc<dyn asi_ai_sdk::agent::tool::Tool>);
    tools.insert("listDirectory".into(), Arc::new(ListDirectoryTool));

    ToolLoopAgent::new(
        provider,
        AGENT_INSTRUCTIONS_COMPACT.to_string(),
        tools,
        REVIEW_MAX_STEPS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

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
    fn test_build_review_agent() {
        let provider = TestProvider;
        let agent = build_review_agent(provider);
        assert_eq!(agent.max_steps(), 15);
    }
}

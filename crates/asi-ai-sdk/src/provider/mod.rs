use crate::types::*;
use async_trait::async_trait;
use futures_core::Stream;
use std::pin::Pin;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Provider not available: {0}")]
    Unavailable(String),
}

/// Trait for AI model providers (DeepSeek, Ollama, etc.)
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Send a chat completion request (non-streaming).
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;

    /// Send a chat completion request with streaming response.
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>;

    /// Check if the provider is reachable and healthy.
    async fn health_check(&self) -> Result<bool, ProviderError>;

    /// Provider name for logging/metrics.
    fn name(&self) -> &'static str;
}

pub mod deepseek;
pub mod model_registry;
pub mod ollama;

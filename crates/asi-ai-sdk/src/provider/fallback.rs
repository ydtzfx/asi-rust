use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures_core::Stream;

use super::{AiProvider, ProviderError};
use crate::types::*;

/// A provider that tries the primary first and falls back to a secondary
/// provider on failure.  Fallback is only attempted for retryable errors
/// (timeouts, 5xx).  Auth errors (4xx) are returned immediately.
pub struct FallbackProvider {
    primary: Arc<dyn AiProvider>,
    fallback: Option<Arc<dyn AiProvider>>,
}

impl FallbackProvider {
    pub fn new(primary: Arc<dyn AiProvider>, fallback: Option<Arc<dyn AiProvider>>) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl AiProvider for FallbackProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        match self.primary.chat(request.clone()).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                if let Some(ref fb) = self.fallback {
                    if is_retryable(&e) {
                        tracing::warn!(
                            primary_error = %e,
                            fallback = fb.name(),
                            "Primary provider failed, trying fallback"
                        );
                        fb.chat(request).await
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>
    {
        match self.primary.chat_stream(request.clone()).await {
            Ok(stream) => Ok(stream),
            Err(e) => {
                if let Some(ref fb) = self.fallback {
                    if is_retryable(&e) {
                        tracing::warn!(
                            primary_error = %e,
                            fallback = fb.name(),
                            "Primary provider streaming failed, trying fallback"
                        );
                        fb.chat_stream(request).await
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        self.primary.health_check().await
    }

    fn name(&self) -> &'static str {
        "fallback"
    }
}

/// Returns true for errors where retrying with a different provider makes sense.
fn is_retryable(e: &ProviderError) -> bool {
    match e {
        ProviderError::Http(_) => true,          // timeout, connection refused
        ProviderError::Api { status, .. } => *status >= 500, // server errors
        ProviderError::Unavailable(_) => true,    // explicitly unavailable
        ProviderError::Parse(_) => false,         // bad response — won't help
    }
}

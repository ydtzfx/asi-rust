use super::{AiProvider, ProviderError};
use crate::types::*;
use async_trait::async_trait;
use futures_core::Stream;
use reqwest::Client;
use std::pin::Pin;
use tokio_stream::StreamExt;

pub struct DeepSeekProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            base_url: "https://api.deepseek.com/v1".to_string(),
        }
    }
}

#[async_trait]
impl AiProvider for DeepSeekProvider {
    async fn chat(&self, mut request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        request.model = self.model.clone();
        request.stream = Some(false);

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                status: status.as_u16(),
                message: text,
            });
        }

        resp.json()
            .await
            .map_err(|e| ProviderError::Parse(e.to_string()))
    }

    async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>,
        ProviderError,
    > {
        request.model = self.model.clone();
        request.stream = Some(true);

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::Http(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Api {
                status: status.as_u16(),
                message: text,
            });
        }

        let stream = resp
            .bytes_stream()
            .filter_map(|result| match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    text.lines()
                        .filter(|line| line.starts_with("data: "))
                        .filter(|line| *line != "data: [DONE]")
                        .map(|line| {
                            let json = &line[6..];
                            serde_json::from_str::<StreamChunk>(json)
                                .map_err(|e| ProviderError::Parse(e.to_string()))
                        })
                        .next()
                }
                Err(e) => Some(Err(ProviderError::Http(e.to_string()))),
            });

        Ok(Box::pin(stream)
            as Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "deepseek"
    }
}

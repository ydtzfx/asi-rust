use super::{AiProvider, ProviderError};
use crate::types::*;
use async_trait::async_trait;
use futures_core::Stream;
use reqwest::Client;
use std::pin::Pin;
use tokio_stream::StreamExt;

pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(model: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            model,
            base_url,
        }
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn chat(&self, mut request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        request.model = self.model.clone();
        request.stream = Some(false);

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
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
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>
    {
        request.model = self.model.clone();
        request.stream = Some(true);

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
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

        let stream = resp.bytes_stream().filter_map(|result| match result {
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
            as Pin<
                Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>,
            >)
    }

    async fn health_check(&self) -> Result<bool, ProviderError> {
        match self.client.get(&self.base_url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn name(&self) -> &'static str {
        "ollama"
    }
}

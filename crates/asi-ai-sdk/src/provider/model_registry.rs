use super::AiProvider;
use crate::provider::deepseek::DeepSeekProvider;
use crate::provider::ollama::OllamaProvider;

/// Available models in the registry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub available: bool,
}

pub struct ModelRegistry {
    providers: Vec<Box<dyn AiProvider>>,
    active_index: usize,
}

impl ModelRegistry {
    pub fn new(providers: Vec<Box<dyn AiProvider>>) -> Self {
        Self {
            providers,
            active_index: 0,
        }
    }

    pub fn active(&self) -> &dyn AiProvider {
        self.providers[self.active_index].as_ref()
    }

    pub fn switch_active(&mut self, index: usize) -> Result<(), String> {
        if index < self.providers.len() {
            self.active_index = index;
            Ok(())
        } else {
            Err(format!("Invalid provider index: {}", index))
        }
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.providers
            .iter()
            .map(|p| ModelInfo {
                id: p.name().to_string(),
                name: p.name().to_string(),
                provider: p.name().to_string(),
                available: true,
            })
            .collect()
    }
}

/// Build the default registry from environment config.
pub fn build_default_registry() -> ModelRegistry {
    let mut providers: Vec<Box<dyn AiProvider>> = Vec::new();

    // DeepSeek is primary
    if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
        let model = std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".into());
        providers.push(Box::new(DeepSeekProvider::new(api_key, model)));
    }

    // Ollama is fallback
    let ollama_url = std::env::var("OLLAMA_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434/v1".into());
    let ollama_model =
        std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into());
    providers.push(Box::new(OllamaProvider::new(ollama_model, ollama_url)));

    ModelRegistry::new(providers)
}

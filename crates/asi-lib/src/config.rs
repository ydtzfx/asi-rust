use std::env;

pub struct Config {
    pub database_url: String,
    pub deepseek_api_key: Option<String>,
    pub deepseek_model: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub ollama_fallback_model: String,
    pub clerk_secret_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "asi.db".into()),
            deepseek_api_key: env::var("DEEPSEEK_API_KEY").ok(),
            deepseek_model: env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".into()),
            ollama_base_url: env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".into()),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "gemma4:31b-cloud".into()),
            ollama_fallback_model: env::var("OLLAMA_FALLBACK_MODEL")
                .unwrap_or_else(|_| "qwen3:4b".into()),
            clerk_secret_key: env::var("CLERK_SECRET_KEY").expect("CLERK_SECRET_KEY is required"),
        }
    }
}

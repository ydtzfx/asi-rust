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

    /// Validate configuration and log a summary for operator visibility.
    /// Returns warnings for non-fatal issues.
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Log configuration summary
        crate::logger::info(
            "Configuration loaded",
            &[
                ("database_url", &self.database_url),
                (
                    "ai_provider",
                    &if self.deepseek_api_key.is_some() {
                        "deepseek"
                    } else {
                        "ollama"
                    },
                ),
                ("ollama_model", &self.ollama_model),
            ],
        );

        // Warn if DATABASE_URL is using the default relative path
        if self.database_url == "asi.db" {
            let warning = "DATABASE_URL is using default 'asi.db' (relative path). \
                Set an absolute path for production deployments.";
            crate::logger::warn("Config", &[("warning", &warning)]);
            warnings.push(warning.to_string());
        }

        // Warn if EVOLVE_SECRET is not set
        if env::var("EVOLVE_SECRET").unwrap_or_default().is_empty() {
            let warning = "EVOLVE_SECRET is not set — /api/evolve endpoint will return 503.";
            crate::logger::warn("Config", &[("warning", &warning)]);
            warnings.push(warning.to_string());
        }

        warnings
    }
}

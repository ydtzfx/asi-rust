// Integration tests for AI provider construction and basic properties.
use asi_ai_sdk::provider::AiProvider;

#[test]
fn test_deepseek_provider_name() {
    let provider = asi_ai_sdk::provider::deepseek::DeepSeekProvider::new(
        "test-key".into(),
        "deepseek-chat".into(),
    );
    assert_eq!(provider.name(), "deepseek");
}

#[test]
fn test_ollama_provider_name() {
    let provider = asi_ai_sdk::provider::ollama::OllamaProvider::new(
        "gemma4:31b-cloud".into(),
        "http://localhost:11434/v1".into(),
    );
    assert_eq!(provider.name(), "ollama");
}

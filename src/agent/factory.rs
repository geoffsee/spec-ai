//! Provider Factory
//!
//! Creates model provider instances based on configuration.

use crate::agent::model::{ModelProvider, ProviderKind};
use crate::agent::providers::MockProvider;
use crate::config::ModelConfig;
use anyhow::{anyhow, Context, Result};
use std::sync::Arc;

/// Create a model provider from configuration
pub fn create_provider(config: &ModelConfig) -> Result<Arc<dyn ModelProvider>> {
    let provider_kind = ProviderKind::from_str(&config.provider)
        .ok_or_else(|| anyhow!("Unknown provider: {}", config.provider))?;

    match provider_kind {
        ProviderKind::Mock => {
            // Create mock provider with optional custom responses
            let provider = if let Some(ref model_name) = config.model_name {
                MockProvider::default().with_model_name(model_name.clone())
            } else {
                MockProvider::default()
            };
            Ok(Arc::new(provider))
        }

        #[cfg(feature = "openai")]
        ProviderKind::OpenAI => {
            // TODO: Implement OpenAI provider
            Err(anyhow!("OpenAI provider not yet implemented"))
        }

        #[cfg(feature = "anthropic")]
        ProviderKind::Anthropic => {
            // TODO: Implement Anthropic provider
            Err(anyhow!("Anthropic provider not yet implemented"))
        }

        #[cfg(feature = "ollama")]
        ProviderKind::Ollama => {
            // TODO: Implement Ollama provider
            Err(anyhow!("Ollama provider not yet implemented"))
        }
    }
}

/// Load API key from environment variable or file
pub fn load_api_key_from_env(env_var: &str) -> Result<String> {
    std::env::var(env_var).context(format!("Environment variable {} not set", env_var))
}

/// Load API key from file
pub fn load_api_key_from_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .context(format!("Failed to read API key from file: {}", path))
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModelConfig;

    #[test]
    fn test_create_mock_provider() {
        let config = ModelConfig {
            provider: "mock".to_string(),
            model_name: Some("test-model".to_string()),
            api_key_source: None,
            temperature: 0.8,
        };

        let provider = create_provider(&config).unwrap();
        assert_eq!(provider.kind(), ProviderKind::Mock);
    }

    #[test]
    fn test_create_unknown_provider() {
        let config = ModelConfig {
            provider: "unknown-provider".to_string(),
            model_name: None,
            api_key_source: None,
            temperature: 0.7,
        };

        let result = create_provider(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_api_key_from_env() {
        unsafe {
            std::env::set_var("TEST_API_KEY", "env-key-value");
        }
        let key = load_api_key_from_env("TEST_API_KEY").unwrap();
        assert_eq!(key, "env-key-value");
        unsafe {
            std::env::remove_var("TEST_API_KEY");
        }
    }

    #[test]
    fn test_load_api_key_env_var_missing() {
        let result = load_api_key_from_env("NONEXISTENT_VAR");
        assert!(result.is_err());
    }
}

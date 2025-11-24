//! Ollama Model Provider
//!
//! Integration with Ollama for running local LLMs.
//! Supports any model available through your local Ollama instance.

use crate::agent::model::{
    parse_thinking_tokens, GenerationConfig, ModelProvider, ModelResponse, ProviderKind,
    ProviderMetadata, TokenUsage,
};
use anyhow::{anyhow, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Message in an Ollama chat conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Ollama chat API request
#[derive(Debug, Clone, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Options for Ollama API requests
#[derive(Debug, Clone, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// Ollama chat API response
#[derive(Debug, Clone, Deserialize)]
struct OllamaChatResponse {
    #[serde(default)]
    message: MessageResponse,
    #[serde(default)]
    done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<u32>,
}

/// Message content in Ollama response
#[derive(Debug, Clone, Default, Deserialize)]
struct MessageResponse {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: String,
}

/// Ollama provider for local LLM instances
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    /// HTTP client for API requests
    client: reqwest::Client,
    /// Base URL for the Ollama API
    base_url: String,
    /// Default model to use
    model: String,
    /// Optional system message for all requests
    system_message: Option<String>,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the default configuration
    ///
    /// This will use http://localhost:11434 as the base URL and
    /// "llama2" as the default model.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: DEFAULT_OLLAMA_URL.to_string(),
            model: "llama2".to_string(),
            system_message: None,
        }
    }

    /// Create a new Ollama provider with a custom base URL
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            model: "llama2".to_string(),
            system_message: None,
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set a system message to be included in all requests
    pub fn with_system_message(mut self, message: impl Into<String>) -> Self {
        self.system_message = Some(message.into());
        self
    }

    /// Build the request for the Ollama chat API
    fn build_request(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        stream: bool,
    ) -> OllamaChatRequest {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system_msg) = &self.system_message {
            messages.push(Message {
                role: "system".to_string(),
                content: system_msg.clone(),
            });
        }

        // Add user prompt
        messages.push(Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        // Build options from config
        let options = if config.temperature.is_some()
            || config.max_tokens.is_some()
            || config.top_p.is_some()
            || config.stop_sequences.is_some()
        {
            Some(OllamaOptions {
                temperature: config.temperature,
                num_predict: config.max_tokens,
                top_p: config.top_p,
                stop: config.stop_sequences.clone(),
            })
        } else {
            None
        };

        OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: if stream { Some(true) } else { Some(false) },
            options,
        }
    }

    /// Get the chat endpoint URL
    fn chat_url(&self) -> String {
        format!("{}/api/chat", self.base_url)
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<ModelResponse> {
        let request = self.build_request(prompt, config, false);

        // Make the API call
        let response = self
            .client
            .post(self.chat_url())
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Ollama API request failed: {}", e))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama API error ({}): {}", status, error_text));
        }

        // Parse the response
        let api_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        // Extract text content
        let raw_content = api_response.message.content;

        // Parse thinking tokens if present
        let (reasoning, content) = parse_thinking_tokens(&raw_content);

        // Calculate token usage from eval counts
        let usage = if api_response.prompt_eval_count.is_some() || api_response.eval_count.is_some()
        {
            let prompt_tokens = api_response.prompt_eval_count.unwrap_or(0);
            let completion_tokens = api_response.eval_count.unwrap_or(0);
            Some(TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            })
        } else {
            None
        };

        Ok(ModelResponse {
            content,
            model: self.model.clone(),
            usage,
            finish_reason: if api_response.done {
                Some("stop".to_string())
            } else {
                None
            },
            tool_calls: None,
            reasoning,
        })
    }

    async fn stream(
        &self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let request = self.build_request(prompt, config, true);

        // Make the streaming API call
        let response = self
            .client
            .post(self.chat_url())
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Ollama streaming API request failed: {}", e))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama streaming API error ({}): {}",
                status,
                error_text
            ));
        }

        // Convert the response into a stream
        let byte_stream = response.bytes_stream();

        let stream = stream! {
            use futures::StreamExt;

            let mut buffer = String::new();
            let mut line_buffer = String::new();
            let mut in_think_block = false;
            let mut think_ended = false;

            let mut stream = byte_stream;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk);
                        line_buffer.push_str(&chunk_str);

                        // Process complete lines (Ollama sends newline-delimited JSON)
                        while let Some(newline_pos) = line_buffer.find('\n') {
                            let line = line_buffer[..newline_pos].trim().to_string();
                            line_buffer = line_buffer[newline_pos + 1..].to_string();

                            // Skip empty lines
                            if line.is_empty() {
                                continue;
                            }

                            // Parse JSON line
                            if let Ok(chunk_response) = serde_json::from_str::<OllamaChatResponse>(&line) {
                                let text = chunk_response.message.content;
                                if !text.is_empty() {
                                    buffer.push_str(&text);

                                    // Check if we're entering a think block
                                    if buffer.contains("<think>") && !in_think_block {
                                        in_think_block = true;
                                    }

                                    // Check if we're exiting a think block
                                    if buffer.contains("</think>") && in_think_block {
                                        in_think_block = false;
                                        think_ended = true;
                                        // Clear buffer up to and including </think>
                                        if let Some(idx) = buffer.find("</think>") {
                                            buffer = buffer[idx + "</think>".len()..].to_string();
                                        }
                                    }

                                    // Only yield content if we're not in a think block
                                    if !in_think_block && (think_ended || !buffer.contains("<think>")) {
                                        let output = buffer.clone();
                                        buffer.clear();
                                        if !output.is_empty() {
                                            yield Ok(output);
                                        }
                                    }
                                }

                                // Check if streaming is done
                                if chunk_response.done {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(anyhow!("Stream error: {}", e));
                        break;
                    }
                }
            }

            // Yield any remaining buffered content
            if !buffer.is_empty() && !in_think_block {
                yield Ok(buffer);
            }
        };

        Ok(Box::pin(stream))
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Ollama".to_string(),
            supported_models: vec![
                "llama2".to_string(),
                "llama3".to_string(),
                "mistral".to_string(),
                "mixtral".to_string(),
                "codellama".to_string(),
                "phi".to_string(),
                "neural-chat".to_string(),
                "starling-lm".to_string(),
                "vicuna".to_string(),
                "gemma".to_string(),
            ],
            supports_streaming: true,
        }
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.base_url, DEFAULT_OLLAMA_URL);
        assert_eq!(provider.model, "llama2");
        assert!(provider.system_message.is_none());
    }

    #[test]
    fn test_ollama_provider_with_base_url() {
        let provider = OllamaProvider::with_base_url("http://custom:11434");
        assert_eq!(provider.base_url, "http://custom:11434");
    }

    #[test]
    fn test_ollama_provider_with_model() {
        let provider = OllamaProvider::new().with_model("mistral");
        assert_eq!(provider.model, "mistral");
    }

    #[test]
    fn test_ollama_provider_with_system_message() {
        let provider = OllamaProvider::new().with_system_message("You are a helpful assistant.");
        assert_eq!(
            provider.system_message,
            Some("You are a helpful assistant.".to_string())
        );
    }

    #[test]
    fn test_ollama_provider_metadata() {
        let provider = OllamaProvider::new();
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Ollama");
        assert!(metadata.supports_streaming);
        assert!(metadata.supported_models.contains(&"llama2".to_string()));
        assert!(metadata.supported_models.contains(&"mistral".to_string()));
    }

    #[test]
    fn test_ollama_provider_kind() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.kind(), ProviderKind::Ollama);
    }

    #[test]
    fn test_build_request() {
        let provider = OllamaProvider::new().with_system_message("System prompt");
        let config = GenerationConfig {
            temperature: Some(0.8),
            max_tokens: Some(1024),
            ..Default::default()
        };

        let request = provider.build_request("Hello", &config, false);

        assert_eq!(request.model, "llama2");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[0].content, "System prompt");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[1].content, "Hello");
        assert_eq!(request.stream, Some(false));
        assert!(request.options.is_some());
        let options = request.options.unwrap();
        assert_eq!(options.temperature, Some(0.8));
        assert_eq!(options.num_predict, Some(1024));
    }

    #[test]
    fn test_build_request_streaming() {
        let provider = OllamaProvider::new();
        let config = GenerationConfig::default();

        let request = provider.build_request("Hello", &config, true);

        assert_eq!(request.stream, Some(true));
    }

    #[test]
    fn test_chat_url() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.chat_url(), "http://localhost:11434/api/chat");

        let custom_provider = OllamaProvider::with_base_url("http://custom:8080");
        assert_eq!(custom_provider.chat_url(), "http://custom:8080/api/chat");
    }
}

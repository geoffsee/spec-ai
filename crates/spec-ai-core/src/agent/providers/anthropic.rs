//! Anthropic Model Provider
//!
//! Integration with Anthropic's API (Claude models).
//! Supports Claude 3 family models including Opus, Sonnet, and Haiku.

use crate::agent::model::{
    parse_thinking_tokens, GenerationConfig, ModelProvider, ModelResponse, ProviderKind,
    ProviderMetadata, TokenUsage, ToolCall,
};
use anyhow::{anyhow, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Message in an Anthropic conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Tool definition for Anthropic function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Anthropic API request
#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Content block in Anthropic response
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

/// Anthropic API response
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
    usage: Usage,
}

/// Token usage in Anthropic response
#[derive(Debug, Clone, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Streaming event from Anthropic
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageInfo },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: Delta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDelta, usage: Usage },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct MessageInfo {
    id: String,
    model: String,
    usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum Delta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct MessageDelta {
    stop_reason: Option<String>,
}

/// Anthropic provider for Claude models
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    /// HTTP client for API requests
    client: reqwest::Client,
    /// API key for authentication
    api_key: String,
    /// Default model to use
    model: String,
    /// Optional system message for all requests
    system_message: Option<String>,
    /// Optional tools available for function calling
    tools: Option<Vec<Tool>>,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the default configuration
    ///
    /// This will use the ANTHROPIC_API_KEY environment variable for authentication
    /// and default to the "claude-3-5-sonnet-20241022" model.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: "claude-3-5-sonnet-20241022".to_string(),
            system_message: None,
            tools: None,
        })
    }

    /// Create a new Anthropic provider with a custom API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            system_message: None,
            tools: None,
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

    /// Set tools available for function calling
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = if tools.is_empty() { None } else { Some(tools) };
        self
    }

    /// Build the request for the Anthropic API
    fn build_request(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        stream: bool,
    ) -> AnthropicRequest {
        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        AnthropicRequest {
            model: self.model.clone(),
            messages,
            max_tokens: config.max_tokens.unwrap_or(2048),
            system: self.system_message.clone(),
            temperature: config.temperature,
            top_p: config.top_p,
            stop_sequences: config.stop_sequences.clone(),
            tools: self.tools.clone(),
            stream: if stream { Some(true) } else { None },
        }
    }

    /// Parse SSE (Server-Sent Events) line
    fn parse_sse_line(line: &str) -> Option<StreamEvent> {
        if let Some(data) = line.strip_prefix("data: ") {
            serde_json::from_str(data).ok()
        } else {
            None
        }
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create default Anthropic provider")
    }
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<ModelResponse> {
        let request = self.build_request(prompt, config, false);

        // Make the API call
        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Anthropic API request failed: {}", e))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic API error ({}): {}", status, error_text));
        }

        // Parse the response
        let api_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Anthropic response: {}", e))?;

        // Extract text content and tool calls
        let mut raw_content = String::new();
        let mut tool_calls = Vec::new();

        for block in api_response.content {
            match block {
                ContentBlock::Text { text } => {
                    raw_content.push_str(&text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id,
                        function_name: name,
                        arguments: input,
                    });
                }
            }
        }

        // Parse thinking tokens if present
        let (reasoning, content) = parse_thinking_tokens(&raw_content);

        let usage = TokenUsage {
            prompt_tokens: api_response.usage.input_tokens,
            completion_tokens: api_response.usage.output_tokens,
            total_tokens: api_response.usage.input_tokens + api_response.usage.output_tokens,
        };

        Ok(ModelResponse {
            content,
            model: api_response.model,
            usage: Some(usage),
            finish_reason: api_response.stop_reason,
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
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
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Anthropic streaming API request failed: {}", e))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Anthropic streaming API error ({}): {}",
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

                        // Process complete lines
                        while let Some(newline_pos) = line_buffer.find('\n') {
                            let line = line_buffer[..newline_pos].trim().to_string();
                            line_buffer = line_buffer[newline_pos + 1..].to_string();

                            // Parse SSE line
                            if let Some(event) = Self::parse_sse_line(&line) {
                                match event {
                                    StreamEvent::ContentBlockDelta { delta, .. } => {
                                        if let Delta::TextDelta { text } = delta {
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
                                    }
                                    StreamEvent::MessageStop => {
                                        // Yield any remaining buffered content
                                        if !buffer.is_empty() && !in_think_block {
                                            yield Ok(buffer.clone());
                                            buffer.clear();
                                        }
                                    }
                                    _ => {
                                        // Ignore other event types
                                    }
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
            name: "Anthropic".to_string(),
            supported_models: vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
            supports_streaming: true,
        }
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_creation() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        let provider = AnthropicProvider::new().unwrap();
        assert_eq!(provider.model, "claude-3-5-sonnet-20241022");
        assert!(provider.system_message.is_none());
    }

    #[test]
    fn test_anthropic_provider_with_api_key() {
        let provider = AnthropicProvider::with_api_key("custom-key");
        assert_eq!(provider.api_key, "custom-key");
    }

    #[test]
    fn test_anthropic_provider_with_model() {
        let provider =
            AnthropicProvider::with_api_key("test-key").with_model("claude-3-opus-20240229");
        assert_eq!(provider.model, "claude-3-opus-20240229");
    }

    #[test]
    fn test_anthropic_provider_with_system_message() {
        let provider = AnthropicProvider::with_api_key("test-key")
            .with_system_message("You are a helpful assistant.");
        assert_eq!(
            provider.system_message,
            Some("You are a helpful assistant.".to_string())
        );
    }

    #[test]
    fn test_anthropic_provider_metadata() {
        let provider = AnthropicProvider::with_api_key("test-key");
        let metadata = provider.metadata();

        assert_eq!(metadata.name, "Anthropic");
        assert!(metadata.supports_streaming);
        assert!(metadata
            .supported_models
            .contains(&"claude-3-5-sonnet-20241022".to_string()));
        assert!(metadata
            .supported_models
            .contains(&"claude-3-opus-20240229".to_string()));
    }

    #[test]
    fn test_anthropic_provider_kind() {
        let provider = AnthropicProvider::with_api_key("test-key");
        assert_eq!(provider.kind(), ProviderKind::Anthropic);
    }

    #[test]
    fn test_build_request() {
        let provider =
            AnthropicProvider::with_api_key("test-key").with_system_message("System prompt");
        let config = GenerationConfig {
            temperature: Some(0.8),
            max_tokens: Some(1024),
            ..Default::default()
        };

        let request = provider.build_request("Hello", &config, false);

        assert_eq!(request.model, "claude-3-5-sonnet-20241022");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert_eq!(request.messages[0].content, "Hello");
        assert_eq!(request.system, Some("System prompt".to_string()));
        assert_eq!(request.temperature, Some(0.8));
        assert_eq!(request.max_tokens, 1024);
        assert_eq!(request.stream, None);
    }

    #[test]
    fn test_build_request_streaming() {
        let provider = AnthropicProvider::with_api_key("test-key");
        let config = GenerationConfig::default();

        let request = provider.build_request("Hello", &config, true);

        assert_eq!(request.stream, Some(true));
    }
}

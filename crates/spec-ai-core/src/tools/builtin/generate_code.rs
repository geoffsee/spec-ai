use crate::agent::model::{GenerationConfig, ModelProvider};
use crate::tools::{Tool, ToolResult};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

/// Generate code using a dedicated code model (configured via `model.code_model`).
pub struct GenerateCodeTool {
    provider: Arc<dyn ModelProvider>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::providers::MockProvider;
    use serde_json::json;

    #[tokio::test]
    async fn generate_code_returns_content() {
        let provider = Arc::new(MockProvider::new("fn main() {}"));
        let tool = GenerateCodeTool::new(provider);

        let args = json!({
            "prompt": "write rust main"
        });

        let result = tool.execute(args).await.unwrap();
        assert!(result.success);

        let payload: serde_json::Value = serde_json::from_str(&result.output).unwrap();
        assert_eq!(payload["content"], "fn main() {}");
        assert_eq!(payload["model"], "mock-model");
    }
}

impl GenerateCodeTool {
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self { provider }
    }
}

#[derive(Debug, Deserialize)]
struct GenerateCodeArgs {
    prompt: String,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    temperature: Option<f32>,
}

#[async_trait]
impl Tool for GenerateCodeTool {
    fn name(&self) -> &str {
        "generate_code"
    }

    fn description(&self) -> &str {
        "Generate code or code reviews using the configured code model."
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Instruction or request for the code model"
                },
                "max_tokens": {
                    "type": "integer",
                    "description": "Optional max tokens to generate"
                },
                "temperature": {
                    "type": "number",
                    "description": "Optional temperature override (0.0 - 2.0)"
                }
            },
            "required": ["prompt"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: GenerateCodeArgs =
            serde_json::from_value(args).context("parsing generate_code arguments")?;

        let prompt = args.prompt.trim();
        if prompt.is_empty() {
            return Err(anyhow!("prompt cannot be empty"));
        }

        let generation_config = GenerationConfig {
            temperature: args.temperature.map(|t| t.clamp(0.0, 2.0)),
            max_tokens: args.max_tokens,
            stop_sequences: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };

        let response = self
            .provider
            .generate(prompt, &generation_config)
            .await
            .context("calling code model")?;

        let output = serde_json::json!({
            "model": response.model,
            "content": response.content,
            "usage": response.usage,
            "finish_reason": response.finish_reason
        });

        Ok(ToolResult::success(
            serde_json::to_string(&output).context("serializing code model response")?,
        ))
    }
}

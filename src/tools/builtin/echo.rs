use crate::tools::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

/// A simple echo tool that returns its input
pub struct EchoTool;

#[derive(Debug, Deserialize)]
struct EchoArgs {
    message: String,
}

impl EchoTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the provided message"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo back"
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let echo_args: EchoArgs = serde_json::from_value(args)
            .context("Failed to parse echo arguments")?;

        Ok(ToolResult::success(echo_args.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_tool_basic() {
        let tool = EchoTool::new();

        assert_eq!(tool.name(), "echo");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_echo_tool_parameters() {
        let tool = EchoTool::new();
        let params = tool.parameters();

        assert!(params.is_object());
        assert!(params["properties"]["message"].is_object());
        assert_eq!(params["required"][0], "message");
    }

    #[tokio::test]
    async fn test_echo_tool_execute() {
        let tool = EchoTool::new();
        let args = serde_json::json!({
            "message": "Hello, world!"
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "Hello, world!");
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_echo_tool_execute_empty_message() {
        let tool = EchoTool::new();
        let args = serde_json::json!({
            "message": ""
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "");
    }

    #[tokio::test]
    async fn test_echo_tool_execute_missing_argument() {
        let tool = EchoTool::new();
        let args = serde_json::json!({});

        let result = tool.execute(args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_echo_tool_execute_invalid_type() {
        let tool = EchoTool::new();
        let args = serde_json::json!({
            "message": 123
        });

        let result = tool.execute(args).await;

        assert!(result.is_err());
    }
}

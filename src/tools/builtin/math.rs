use crate::tools::{Tool, ToolResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

/// A basic math calculator tool
pub struct MathTool;

#[derive(Debug, Deserialize)]
struct MathArgs {
    operation: String,
    a: f64,
    b: f64,
}

impl MathTool {
    pub fn new() -> Self {
        Self
    }

    fn evaluate(&self, operation: &str, a: f64, b: f64) -> Result<f64> {
        match operation {
            "add" | "+" => Ok(a + b),
            "subtract" | "-" => Ok(a - b),
            "multiply" | "*" => Ok(a * b),
            "divide" | "/" => {
                if b == 0.0 {
                    anyhow::bail!("Division by zero");
                }
                Ok(a / b)
            }
            "power" | "**" => Ok(a.powf(b)),
            "modulo" | "%" => {
                if b == 0.0 {
                    anyhow::bail!("Modulo by zero");
                }
                Ok(a % b)
            }
            _ => anyhow::bail!("Unsupported operation: {}", operation),
        }
    }
}

impl Default for MathTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for MathTool {
    fn name(&self) -> &str {
        "math"
    }

    fn description(&self) -> &str {
        "Performs basic mathematical operations: add, subtract, multiply, divide, power, modulo"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "The operation to perform: add, subtract, multiply, divide, power, modulo (or +, -, *, /, **, %)",
                    "enum": ["add", "subtract", "multiply", "divide", "power", "modulo", "+", "-", "*", "/", "**", "%"]
                },
                "a": {
                    "type": "number",
                    "description": "The first operand"
                },
                "b": {
                    "type": "number",
                    "description": "The second operand"
                }
            },
            "required": ["operation", "a", "b"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let math_args: MathArgs = serde_json::from_value(args)
            .context("Failed to parse math arguments")?;

        match self.evaluate(&math_args.operation, math_args.a, math_args.b) {
            Ok(result) => Ok(ToolResult::success(result.to_string())),
            Err(e) => Ok(ToolResult::failure(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_math_tool_basic() {
        let tool = MathTool::new();

        assert_eq!(tool.name(), "math");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_math_tool_parameters() {
        let tool = MathTool::new();
        let params = tool.parameters();

        assert!(params.is_object());
        assert!(params["properties"]["operation"].is_object());
        assert!(params["properties"]["a"].is_object());
        assert!(params["properties"]["b"].is_object());
    }

    #[tokio::test]
    async fn test_math_tool_add() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "add",
            "a": 5.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "8");
    }

    #[tokio::test]
    async fn test_math_tool_add_symbol() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "+",
            "a": 10.5,
            "b": 2.5
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "13");
    }

    #[tokio::test]
    async fn test_math_tool_subtract() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "subtract",
            "a": 10.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "7");
    }

    #[tokio::test]
    async fn test_math_tool_multiply() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "multiply",
            "a": 4.0,
            "b": 5.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "20");
    }

    #[tokio::test]
    async fn test_math_tool_divide() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "divide",
            "a": 15.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "5");
    }

    #[tokio::test]
    async fn test_math_tool_divide_by_zero() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "divide",
            "a": 10.0,
            "b": 0.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Division by zero"));
    }

    #[tokio::test]
    async fn test_math_tool_power() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "power",
            "a": 2.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "8");
    }

    #[tokio::test]
    async fn test_math_tool_modulo() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "modulo",
            "a": 10.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "1");
    }

    #[tokio::test]
    async fn test_math_tool_modulo_by_zero() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "modulo",
            "a": 10.0,
            "b": 0.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_math_tool_invalid_operation() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "invalid",
            "a": 10.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_math_tool_missing_arguments() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "add"
        });

        let result = tool.execute(args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_math_tool_negative_numbers() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "add",
            "a": -5.0,
            "b": 3.0
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "-2");
    }

    #[tokio::test]
    async fn test_math_tool_decimal_numbers() {
        let tool = MathTool::new();
        let args = serde_json::json!({
            "operation": "multiply",
            "a": 2.5,
            "b": 4.2
        });

        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        let output: f64 = result.output.parse().unwrap();
        assert!((output - 10.5).abs() < 0.0001);
    }
}

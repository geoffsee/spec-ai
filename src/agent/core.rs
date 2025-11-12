//! Agent Core Execution Loop
//!
//! The heart of the agent system - orchestrates reasoning, memory, and model interaction.

use crate::agent::model::{GenerationConfig, ModelProvider};
use crate::config::AgentProfile;
use crate::persistence::Persistence;
use crate::policy::{PolicyDecision, PolicyEngine};
use crate::tools::{ToolRegistry, ToolResult};
use crate::types::{Message, MessageRole};
use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Output from an agent execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// The response text
    pub response: String,
    /// Token usage information
    pub token_usage: Option<crate::agent::model::TokenUsage>,
    /// Tool calls made (future enhancement)
    pub tool_calls: Vec<String>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Core agent execution engine
pub struct AgentCore {
    /// Agent profile with configuration
    profile: AgentProfile,
    /// Model provider
    provider: Arc<dyn ModelProvider>,
    /// Persistence layer
    persistence: Persistence,
    /// Current session ID
    session_id: String,
    /// Conversation history (in-memory cache)
    conversation_history: Vec<Message>,
    /// Tool registry for executing tools
    tool_registry: Arc<ToolRegistry>,
    /// Policy engine for permission checks
    policy_engine: Arc<PolicyEngine>,
}

impl AgentCore {
    /// Create a new agent core
    pub fn new(
        profile: AgentProfile,
        provider: Arc<dyn ModelProvider>,
        persistence: Persistence,
        session_id: String,
        tool_registry: Arc<ToolRegistry>,
        policy_engine: Arc<PolicyEngine>,
    ) -> Self {
        Self {
            profile,
            provider,
            persistence,
            session_id,
            conversation_history: Vec::new(),
            tool_registry,
            policy_engine,
        }
    }

    /// Set a new session ID and clear conversation history
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = session_id;
        self.conversation_history.clear();
        self
    }

    /// Execute a single interaction step
    pub async fn run_step(&mut self, input: &str) -> Result<AgentOutput> {
        // Step 1: Recall relevant memories
        let recalled_messages = self.recall_memories(input).await?;

        // Step 2: Build prompt with context
        let mut prompt = self.build_prompt(input, &recalled_messages)?;

        // Step 3: Store user message
        self.store_message(MessageRole::User, input).await?;

        // Step 4: Agent loop with tool execution
        let mut tool_calls = Vec::new();
        let mut final_response = String::new();
        let mut token_usage = None;
        let mut finish_reason = None;

        // Allow up to 5 iterations to handle tool calls
        for _iteration in 0..5 {
            // Generate response using model
            let generation_config = self.build_generation_config();
            let response = self
                .provider
                .generate(&prompt, &generation_config)
                .await
                .context("Failed to generate response from model")?;

            token_usage = response.usage;
            finish_reason = response.finish_reason.clone();
            final_response = response.content.clone();

            // Check for tool calls in the response
            if let Some(tool_call) = self.parse_tool_call(&response.content) {
                let tool_name = tool_call.0;
                let tool_args = tool_call.1;

                // Check if tool is allowed
                if !self.is_tool_allowed(&tool_name) {
                    let error_msg = format!("Tool '{}' is not allowed by agent policy", tool_name);
                    prompt.push_str(&format!("\n\nTOOL_ERROR: {}\n\nPlease continue without using this tool.", error_msg));
                    tool_calls.push(format!("{} (denied)", tool_name));
                    continue;
                }

                // Execute tool
                match self.execute_tool(&tool_name, &tool_args).await {
                    Ok(result) => {
                        tool_calls.push(tool_name.clone());

                        // Add tool result to prompt for next iteration
                        prompt.push_str(&format!(
                            "\n\nTOOL_RESULT from {}:\n{}\n\nBased on this result, please continue.",
                            tool_name, result.output
                        ));

                        // If the model response contains only the tool call, continue loop
                        if response.content.trim().starts_with("TOOL_CALL:") {
                            continue;
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Error executing tool '{}': {}", tool_name, e);
                        prompt.push_str(&format!("\n\nTOOL_ERROR: {}\n\nPlease continue without this tool.", error_msg));
                        tool_calls.push(format!("{} (error)", tool_name));
                        continue;
                    }
                }
            }

            // No tool call found or response includes final answer, break
            break;
        }

        // Step 5: Store assistant response
        self.store_message(MessageRole::Assistant, &final_response).await?;

        // Step 6: Update conversation history
        self.conversation_history.push(Message {
            id: 0, // Will be set by DB
            session_id: self.session_id.clone(),
            role: MessageRole::User,
            content: input.to_string(),
            created_at: Utc::now(),
        });

        self.conversation_history.push(Message {
            id: 0,
            session_id: self.session_id.clone(),
            role: MessageRole::Assistant,
            content: final_response.clone(),
            created_at: Utc::now(),
        });

        Ok(AgentOutput {
            response: final_response,
            token_usage,
            tool_calls,
            finish_reason,
        })
    }

    /// Build generation configuration from profile
    fn build_generation_config(&self) -> GenerationConfig {
        GenerationConfig {
            temperature: self.profile.temperature,
            max_tokens: self.profile.max_context_tokens.map(|t| t as u32),
            stop_sequences: None,
            top_p: Some(self.profile.top_p),
            frequency_penalty: None,
            presence_penalty: None,
        }
    }

    /// Recall relevant memories for the given input
    async fn recall_memories(&self, _query: &str) -> Result<Vec<Message>> {
        // For now, use simple recency-based recall
        // TODO: Implement vector similarity search when embeddings are available
        let limit = self.profile.memory_k as i64;

        let messages = self
            .persistence
            .list_messages(&self.session_id, limit)?;

        Ok(messages)
    }

    /// Build the prompt from system prompt, context, and user input
    fn build_prompt(&self, input: &str, context_messages: &[Message]) -> Result<String> {
        let mut prompt = String::new();

        // Add system prompt if configured
        if let Some(ref system_prompt) = self.profile.prompt {
            prompt.push_str("System: ");
            prompt.push_str(system_prompt);
            prompt.push_str("\n\n");
        }

        // Add conversation context
        if !context_messages.is_empty() {
            prompt.push_str("Previous conversation:\n");
            for msg in context_messages {
                prompt.push_str(&format!("{}: {}\n", msg.role.as_str(), msg.content));
            }
            prompt.push_str("\n");
        }

        // Add current user input
        prompt.push_str(&format!("user: {}\nassistant:", input));

        Ok(prompt)
    }

    /// Store a message in persistence
    async fn store_message(&self, role: MessageRole, content: &str) -> Result<i64> {
        self.persistence
            .insert_message(&self.session_id, role, content)
            .context("Failed to store message")
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the agent profile
    pub fn profile(&self) -> &AgentProfile {
        &self.profile
    }

    /// Get conversation history
    pub fn conversation_history(&self) -> &[Message] {
        &self.conversation_history
    }

    /// Load conversation history from persistence
    pub fn load_history(&mut self, limit: i64) -> Result<()> {
        self.conversation_history = self
            .persistence
            .list_messages(&self.session_id, limit)?;
        Ok(())
    }

    /// Parse tool call from model response
    /// Expected format:
    /// TOOL_CALL: tool_name
    /// ARGS: {"arg1": "value1"}
    fn parse_tool_call(&self, response: &str) -> Option<(String, Value)> {
        let re = Regex::new(r"TOOL_CALL:\s*(\w+)\s*\nARGS:\s*(\{.*?\})").ok()?;
        let captures = re.captures(response)?;

        let tool_name = captures.get(1)?.as_str().to_string();
        let args_str = captures.get(2)?.as_str();
        let args: Value = serde_json::from_str(args_str).ok()?;

        Some((tool_name, args))
    }

    /// Check if a tool is allowed by the agent profile and policy engine
    fn is_tool_allowed(&self, tool_name: &str) -> bool {
        // First check profile-level permissions (backward compatibility)
        if !self.profile.is_tool_allowed(tool_name) {
            return false;
        }

        // Then check policy engine
        let agent_name = "agent"; // Could be enhanced to use profile name
        let decision = self.policy_engine.check(agent_name, "tool_call", tool_name);

        matches!(decision, PolicyDecision::Allow)
    }

    /// Execute a tool and log the result
    async fn execute_tool(&self, tool_name: &str, args: &Value) -> Result<ToolResult> {
        // Execute the tool
        let result = self
            .tool_registry
            .execute(tool_name, args.clone())
            .await?;

        // Log to persistence
        let result_json = serde_json::json!({
            "output": result.output,
            "success": result.success,
            "error": result.error,
        });

        let error_str = result.error.as_deref();
        self.persistence
            .log_tool(tool_name, args, &result_json, result.success, error_str)
            .context("Failed to log tool execution")?;

        Ok(result)
    }

    /// Get the tool registry
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }

    /// Get the policy engine
    pub fn policy_engine(&self) -> &PolicyEngine {
        &self.policy_engine
    }

    /// Set a new policy engine (useful for reloading policies)
    pub fn set_policy_engine(&mut self, policy_engine: Arc<PolicyEngine>) {
        self.policy_engine = policy_engine;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::providers::MockProvider;
    use crate::config::AgentProfile;
    use tempfile::tempdir;

    fn create_test_agent(session_id: &str) -> (AgentCore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.duckdb");
        let persistence = Persistence::new(&db_path).unwrap();

        let profile = AgentProfile {
            prompt: Some("You are a helpful assistant.".to_string()),
            style: None,
            temperature: Some(0.7),
            model_provider: None,
            model_name: None,
            allowed_tools: None,
            denied_tools: None,
            memory_k: 5,
            top_p: 0.9,
            max_context_tokens: Some(2048),
        };

        let provider = Arc::new(MockProvider::new("This is a test response."));
        let tool_registry = Arc::new(crate::tools::ToolRegistry::new());
        let policy_engine = Arc::new(PolicyEngine::new());

        (
            AgentCore::new(
                profile,
                provider,
                persistence,
                session_id.to_string(),
                tool_registry,
                policy_engine,
            ),
            dir,
        )
    }

    #[tokio::test]
    async fn test_agent_core_run_step() {
        let (mut agent, _dir) = create_test_agent("test-session-1");

        let output = agent.run_step("Hello, how are you?").await.unwrap();

        assert!(!output.response.is_empty());
        assert!(output.token_usage.is_some());
        assert_eq!(output.tool_calls.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_core_conversation_history() {
        let (mut agent, _dir) = create_test_agent("test-session-2");

        agent.run_step("First message").await.unwrap();
        agent.run_step("Second message").await.unwrap();

        let history = agent.conversation_history();
        assert_eq!(history.len(), 4); // 2 user + 2 assistant
        assert_eq!(history[0].role, MessageRole::User);
        assert_eq!(history[1].role, MessageRole::Assistant);
    }

    #[tokio::test]
    async fn test_agent_core_session_switch() {
        let (mut agent, _dir) = create_test_agent("session-1");

        agent.run_step("Message in session 1").await.unwrap();
        assert_eq!(agent.session_id(), "session-1");

        agent = agent.with_session("session-2".to_string());
        assert_eq!(agent.session_id(), "session-2");
        assert_eq!(agent.conversation_history().len(), 0);
    }

    #[tokio::test]
    async fn test_agent_core_build_prompt() {
        let (agent, _dir) = create_test_agent("test-session-3");

        let context = vec![
            Message {
                id: 1,
                session_id: "test-session-3".to_string(),
                role: MessageRole::User,
                content: "Previous question".to_string(),
                created_at: Utc::now(),
            },
            Message {
                id: 2,
                session_id: "test-session-3".to_string(),
                role: MessageRole::Assistant,
                content: "Previous answer".to_string(),
                created_at: Utc::now(),
            },
        ];

        let prompt = agent
            .build_prompt("Current question", &context)
            .unwrap();

        assert!(prompt.contains("You are a helpful assistant"));
        assert!(prompt.contains("Previous conversation"));
        assert!(prompt.contains("user: Previous question"));
        assert!(prompt.contains("assistant: Previous answer"));
        assert!(prompt.contains("user: Current question"));
    }

    #[tokio::test]
    async fn test_agent_core_persistence() {
        let (mut agent, _dir) = create_test_agent("persist-test");

        agent.run_step("Test message").await.unwrap();

        // Load messages from DB
        let messages = agent
            .persistence
            .list_messages("persist-test", 100)
            .unwrap();

        assert_eq!(messages.len(), 2); // user + assistant
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[0].content, "Test message");
    }

    #[tokio::test]
    async fn test_agent_tool_call_parsing() {
        let (agent, _dir) = create_test_agent("tool-parse-test");

        // Valid tool call
        let response = "TOOL_CALL: echo\nARGS: {\"message\": \"hello\"}";
        let parsed = agent.parse_tool_call(response);
        assert!(parsed.is_some());
        let (name, args) = parsed.unwrap();
        assert_eq!(name, "echo");
        assert_eq!(args["message"], "hello");

        // No tool call
        let response = "Just a regular response";
        let parsed = agent.parse_tool_call(response);
        assert!(parsed.is_none());

        // Malformed tool call
        let response = "TOOL_CALL: echo\nARGS: invalid json";
        let parsed = agent.parse_tool_call(response);
        assert!(parsed.is_none());
    }

    #[tokio::test]
    async fn test_agent_tool_permission_allowed() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.duckdb");
        let persistence = Persistence::new(&db_path).unwrap();

        let mut profile = AgentProfile {
            prompt: Some("Test".to_string()),
            style: None,
            temperature: Some(0.7),
            model_provider: None,
            model_name: None,
            allowed_tools: Some(vec!["echo".to_string()]),
            denied_tools: None,
            memory_k: 5,
            top_p: 0.9,
            max_context_tokens: Some(2048),
        };

        let provider = Arc::new(MockProvider::new("Test"));
        let tool_registry = Arc::new(crate::tools::ToolRegistry::new());

        // Create policy engine with permissive rule for testing
        let mut policy_engine = PolicyEngine::new();
        policy_engine.add_rule(crate::policy::PolicyRule {
            agent: "*".to_string(),
            action: "tool_call".to_string(),
            resource: "*".to_string(),
            effect: crate::policy::PolicyEffect::Allow,
        });
        let policy_engine = Arc::new(policy_engine);

        let agent = AgentCore::new(
            profile.clone(),
            provider.clone(),
            persistence.clone(),
            "test-session".to_string(),
            tool_registry.clone(),
            policy_engine.clone(),
        );

        assert!(agent.is_tool_allowed("echo"));
        assert!(!agent.is_tool_allowed("math"));

        // Test with denied list
        profile.allowed_tools = None;
        profile.denied_tools = Some(vec!["math".to_string()]);

        let agent = AgentCore::new(
            profile,
            provider,
            persistence,
            "test-session-2".to_string(),
            tool_registry,
            policy_engine,
        );

        assert!(agent.is_tool_allowed("echo"));
        assert!(!agent.is_tool_allowed("math"));
    }

    #[tokio::test]
    async fn test_agent_tool_execution_with_logging() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.duckdb");
        let persistence = Persistence::new(&db_path).unwrap();

        let profile = AgentProfile {
            prompt: Some("Test".to_string()),
            style: None,
            temperature: Some(0.7),
            model_provider: None,
            model_name: None,
            allowed_tools: Some(vec!["echo".to_string()]),
            denied_tools: None,
            memory_k: 5,
            top_p: 0.9,
            max_context_tokens: Some(2048),
        };

        let provider = Arc::new(MockProvider::new("Test"));

        // Create tool registry and register echo tool
        let mut tool_registry = crate::tools::ToolRegistry::new();
        tool_registry.register(Arc::new(crate::tools::builtin::EchoTool::new()));

        let policy_engine = Arc::new(PolicyEngine::new());

        let agent = AgentCore::new(
            profile,
            provider,
            persistence.clone(),
            "tool-exec-test".to_string(),
            Arc::new(tool_registry),
            policy_engine,
        );

        // Execute tool directly
        let args = serde_json::json!({"message": "test message"});
        let result = agent.execute_tool("echo", &args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.output, "test message");

        // Verify tool execution was logged (we can't easily check DB here without more setup)
    }

    #[tokio::test]
    async fn test_agent_tool_registry_access() {
        let (agent, _dir) = create_test_agent("registry-test");

        let registry = agent.tool_registry();
        assert!(registry.is_empty());
    }
}

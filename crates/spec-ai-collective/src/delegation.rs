//! Task delegation and routing between agents.
//!
//! This module provides infrastructure for routing tasks to capable agents
//! and managing the task delegation lifecycle.

use crate::capability::CapabilityTracker;
use crate::types::{CollectiveError, Domain, InstanceId, Result, TaskId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Priority level for a delegated task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Status of a delegated task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is pending assignment
    Pending,
    /// Task has been delegated to an agent
    Delegated { to: InstanceId },
    /// Task is being executed
    InProgress { by: InstanceId },
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed { reason: String },
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    TimedOut,
}

/// A task that can be delegated to other agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedTask {
    /// Unique task identifier
    pub task_id: TaskId,

    /// Parent task if this is a subtask
    pub parent_task_id: Option<TaskId>,

    /// Type of task (e.g., "code_review", "data_analysis")
    pub task_type: String,

    /// Human-readable description
    pub description: String,

    /// Capabilities required to execute this task
    pub required_capabilities: Vec<Domain>,

    /// Task payload (context, parameters, etc.)
    pub payload: serde_json::Value,

    /// Task priority
    #[serde(default)]
    pub priority: TaskPriority,

    /// Optional deadline
    pub deadline: Option<DateTime<Utc>>,

    /// Chain of delegation (who delegated to whom)
    #[serde(default)]
    pub delegation_chain: Vec<InstanceId>,

    /// Current status
    #[serde(default = "default_status")]
    pub status: TaskStatus,

    /// When the task was created
    pub created_at: DateTime<Utc>,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Current retry count
    #[serde(default)]
    pub retry_count: u32,
}

fn default_status() -> TaskStatus {
    TaskStatus::Pending
}

fn default_max_retries() -> u32 {
    3
}

impl DelegatedTask {
    /// Create a new delegated task.
    pub fn new(
        task_type: impl Into<String>,
        description: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            task_id: uuid::Uuid::new_v4().to_string(),
            parent_task_id: None,
            task_type: task_type.into(),
            description: description.into(),
            required_capabilities: Vec::new(),
            payload,
            priority: TaskPriority::Normal,
            deadline: None,
            delegation_chain: Vec::new(),
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            max_retries: 3,
            retry_count: 0,
        }
    }

    /// Set required capabilities.
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.required_capabilities = capabilities;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline.
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Check if task has expired.
    pub fn is_expired(&self) -> bool {
        self.deadline.map(|d| Utc::now() > d).unwrap_or(false)
    }

    /// Check if task can be retried.
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Record a delegation.
    pub fn record_delegation(&mut self, from: InstanceId, to: InstanceId) {
        self.delegation_chain.push(from);
        self.status = TaskStatus::Delegated { to };
    }
}

/// Result of task execution by a delegate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// The task ID
    pub task_id: TaskId,

    /// The agent that executed the task
    pub executor_id: InstanceId,

    /// Final status
    pub status: TaskStatus,

    /// Result data if successful
    pub result: Option<serde_json::Value>,

    /// Execution metrics
    pub metrics: ExecutionMetrics,

    /// Learnings from execution (strategies that worked)
    #[serde(default)]
    pub learnings: Vec<String>,
}

/// Metrics from task execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Time to complete in milliseconds
    pub duration_ms: u64,

    /// Number of tool calls made
    pub tool_calls: u32,

    /// Number of model invocations
    pub model_calls: u32,

    /// Confidence in the result
    pub confidence: f32,

    /// Tokens used
    pub tokens_used: Option<u64>,
}

/// Routing decision for task delegation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Target instance ID
    pub target_instance: InstanceId,

    /// Confidence in this routing decision
    pub confidence: f32,

    /// Reasoning for this decision
    pub reasoning: String,

    /// Fallback instances if primary fails
    pub fallback_instances: Vec<InstanceId>,
}

/// Manages task delegation to capable agents.
#[derive(Debug)]
pub struct DelegationManager {
    /// This agent's instance ID
    instance_id: InstanceId,

    /// Pending tasks waiting for delegation
    pending_tasks: HashMap<TaskId, DelegatedTask>,

    /// Tasks delegated to other agents
    delegated_tasks: HashMap<TaskId, DelegatedTask>,

    /// Tasks received from other agents
    received_tasks: HashMap<TaskId, DelegatedTask>,

    /// Completed task results
    completed_tasks: HashMap<TaskId, TaskResult>,

    /// Minimum capability score for delegation
    min_capability_score: f32,

    /// Default timeout for tasks without deadline
    default_timeout: Duration,
}

impl DelegationManager {
    /// Create a new delegation manager.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            pending_tasks: HashMap::new(),
            delegated_tasks: HashMap::new(),
            received_tasks: HashMap::new(),
            completed_tasks: HashMap::new(),
            min_capability_score: 0.3,
            default_timeout: Duration::minutes(30),
        }
    }

    /// Get this agent's instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Set minimum capability score for delegation.
    pub fn set_min_capability_score(&mut self, score: f32) {
        self.min_capability_score = score;
    }

    /// Add a task for delegation.
    pub fn add_task(&mut self, task: DelegatedTask) {
        self.pending_tasks.insert(task.task_id.clone(), task);
    }

    /// Get routing decision for a task.
    pub fn get_routing_decision(
        &self,
        task: &DelegatedTask,
        tracker: &CapabilityTracker,
    ) -> Result<RoutingDecision> {
        let agents =
            tracker.get_capable_agents(&task.required_capabilities, self.min_capability_score);

        if agents.is_empty() {
            return Err(CollectiveError::NoCapableAgent(task.task_type.clone()));
        }

        // Get primary and fallbacks
        let primary = &agents[0];
        let fallbacks: Vec<InstanceId> = agents
            .iter()
            .skip(1)
            .take(3)
            .map(|a| a.instance_id.clone())
            .collect();

        let reasoning = if primary.is_self {
            format!("Self is best candidate with score {:.2}", primary.score)
        } else {
            format!(
                "Agent {} has best capability score {:.2}",
                primary.instance_id, primary.score
            )
        };

        Ok(RoutingDecision {
            target_instance: primary.instance_id.clone(),
            confidence: primary.score,
            reasoning,
            fallback_instances: fallbacks,
        })
    }

    /// Mark a task as delegated.
    pub fn mark_delegated(&mut self, task_id: &str, to: InstanceId) -> Result<()> {
        if let Some(mut task) = self.pending_tasks.remove(task_id) {
            task.record_delegation(self.instance_id.clone(), to.clone());
            self.delegated_tasks.insert(task_id.to_string(), task);
            Ok(())
        } else {
            Err(CollectiveError::DelegationFailed(format!(
                "Task not found: {}",
                task_id
            )))
        }
    }

    /// Accept a task delegated to this agent.
    pub fn accept_task(&mut self, mut task: DelegatedTask) -> Result<()> {
        task.status = TaskStatus::InProgress {
            by: self.instance_id.clone(),
        };
        self.received_tasks.insert(task.task_id.clone(), task);
        Ok(())
    }

    /// Report task completion.
    pub fn report_completion(&mut self, result: TaskResult) {
        let task_id = result.task_id.clone();

        // Update task status
        if let Some(task) = self.received_tasks.get_mut(&task_id) {
            task.status = result.status.clone();
        }

        // Store result
        self.completed_tasks.insert(task_id, result);
    }

    /// Get pending tasks.
    pub fn pending_tasks(&self) -> &HashMap<TaskId, DelegatedTask> {
        &self.pending_tasks
    }

    /// Get delegated tasks.
    pub fn delegated_tasks(&self) -> &HashMap<TaskId, DelegatedTask> {
        &self.delegated_tasks
    }

    /// Get received tasks.
    pub fn received_tasks(&self) -> &HashMap<TaskId, DelegatedTask> {
        &self.received_tasks
    }

    /// Get a completed task result.
    pub fn get_result(&self, task_id: &str) -> Option<&TaskResult> {
        self.completed_tasks.get(task_id)
    }

    /// Handle delegation failure with retry logic.
    pub fn handle_failure(&mut self, task_id: &str, reason: &str) -> Result<Option<InstanceId>> {
        if let Some(task) = self.delegated_tasks.get_mut(task_id) {
            task.retry_count += 1;

            if task.can_retry() {
                // Move back to pending for retry
                if let Some(mut task) = self.delegated_tasks.remove(task_id) {
                    task.status = TaskStatus::Pending;
                    self.pending_tasks.insert(task_id.to_string(), task);
                }
                // Would return next fallback, but we don't have access to routing here
                Ok(None)
            } else {
                task.status = TaskStatus::Failed {
                    reason: reason.to_string(),
                };
                Err(CollectiveError::DelegationFailed(format!(
                    "Task {} failed after {} retries: {}",
                    task_id, task.retry_count, reason
                )))
            }
        } else {
            Err(CollectiveError::DelegationFailed(format!(
                "Task not found: {}",
                task_id
            )))
        }
    }

    /// Clean up expired tasks.
    pub fn cleanup_expired(&mut self) -> Vec<TaskId> {
        let mut expired = Vec::new();

        // Check pending tasks
        self.pending_tasks.retain(|id, task| {
            if task.is_expired() {
                expired.push(id.clone());
                false
            } else {
                true
            }
        });

        // Check delegated tasks
        for (id, task) in &mut self.delegated_tasks {
            if task.is_expired() {
                task.status = TaskStatus::TimedOut;
                expired.push(id.clone());
            }
        }

        expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = DelegatedTask::new(
            "code_review",
            "Review the auth module",
            serde_json::json!({"file": "auth.rs"}),
        )
        .with_capabilities(vec!["code_review".to_string(), "rust".to_string()])
        .with_priority(TaskPriority::High);

        assert_eq!(task.task_type, "code_review");
        assert_eq!(task.required_capabilities.len(), 2);
        assert_eq!(task.priority, TaskPriority::High);
        assert!(!task.is_expired());
    }

    #[test]
    fn test_delegation_manager() {
        let mut manager = DelegationManager::new("agent-1".to_string());

        let task = DelegatedTask::new("data_analysis", "Analyze sales data", serde_json::json!({}));

        let task_id = task.task_id.clone();
        manager.add_task(task);

        assert!(manager.pending_tasks().contains_key(&task_id));

        manager
            .mark_delegated(&task_id, "agent-2".to_string())
            .unwrap();

        assert!(!manager.pending_tasks().contains_key(&task_id));
        assert!(manager.delegated_tasks().contains_key(&task_id));
    }
}

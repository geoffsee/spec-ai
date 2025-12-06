//! Multi-agent workflow orchestration.
//!
//! This module provides infrastructure for coordinating complex
//! multi-agent workflows with sequential, parallel, and consensus stages.

use crate::types::{CollectiveError, Domain, ExecutionId, InstanceId, Result, WorkflowId};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of workflow stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StageType {
    /// Single agent executes the stage
    Sequential,
    /// Multiple agents execute the same task in parallel
    Parallel { min_agents: usize },
    /// Split work, process in parallel, combine results
    MapReduce { chunks: usize },
    /// Require agreement from multiple agents
    Consensus { min_agreement: f32 },
    /// Branch based on previous stage result
    ConditionalBranch { condition: String },
}

/// State of a workflow stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    /// Stage is waiting for dependencies
    Pending,
    /// Stage is ready to execute
    Ready,
    /// Stage is currently executing
    Running,
    /// Stage completed successfully
    Completed,
    /// Stage failed
    Failed { reason: String },
    /// Stage was skipped (conditional branch)
    Skipped,
}

/// A stage in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStage {
    /// Unique stage identifier within the workflow
    pub stage_id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what this stage does
    pub description: String,

    /// Type of stage
    pub stage_type: StageType,

    /// Capabilities required to execute this stage
    pub required_capabilities: Vec<Domain>,

    /// Stage IDs that must complete before this stage can start
    pub dependencies: Vec<String>,

    /// Timeout for this stage
    pub timeout: Duration,

    /// Payload/configuration for this stage
    pub config: serde_json::Value,
}

impl WorkflowStage {
    /// Create a new sequential stage.
    pub fn sequential(
        stage_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            stage_id: stage_id.into(),
            name: name.into(),
            description: description.into(),
            stage_type: StageType::Sequential,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            timeout: Duration::minutes(30),
            config: serde_json::json!({}),
        }
    }

    /// Create a parallel stage.
    pub fn parallel(
        stage_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        min_agents: usize,
    ) -> Self {
        Self {
            stage_id: stage_id.into(),
            name: name.into(),
            description: description.into(),
            stage_type: StageType::Parallel { min_agents },
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            timeout: Duration::minutes(30),
            config: serde_json::json!({}),
        }
    }

    /// Create a map-reduce stage.
    pub fn map_reduce(
        stage_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        chunks: usize,
    ) -> Self {
        Self {
            stage_id: stage_id.into(),
            name: name.into(),
            description: description.into(),
            stage_type: StageType::MapReduce { chunks },
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            timeout: Duration::minutes(60),
            config: serde_json::json!({}),
        }
    }

    /// Create a consensus stage.
    pub fn consensus(
        stage_id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        min_agreement: f32,
    ) -> Self {
        Self {
            stage_id: stage_id.into(),
            name: name.into(),
            description: description.into(),
            stage_type: StageType::Consensus { min_agreement },
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            timeout: Duration::hours(1),
            config: serde_json::json!({}),
        }
    }

    /// Set required capabilities.
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.required_capabilities = capabilities;
        self
    }

    /// Set dependencies.
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set configuration.
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }
}

/// State of a workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    /// Workflow is defined but not started
    Draft,
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed { reason: String },
    /// Workflow was cancelled
    Cancelled,
    /// Workflow is paused
    Paused,
}

/// A workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow identifier
    pub workflow_id: WorkflowId,

    /// Human-readable name
    pub name: String,

    /// Description of the workflow
    pub description: String,

    /// Stages in this workflow
    pub stages: Vec<WorkflowStage>,

    /// Current state
    pub state: WorkflowState,

    /// The agent that created this workflow
    pub created_by: InstanceId,

    /// When the workflow was created
    pub created_at: DateTime<Utc>,

    /// Input data for the workflow
    pub input: serde_json::Value,
}

impl Workflow {
    /// Create a new workflow.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        created_by: InstanceId,
    ) -> Self {
        Self {
            workflow_id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            stages: Vec::new(),
            state: WorkflowState::Draft,
            created_by,
            created_at: Utc::now(),
            input: serde_json::json!({}),
        }
    }

    /// Add a stage to the workflow.
    pub fn add_stage(mut self, stage: WorkflowStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Set input data.
    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }

    /// Validate the workflow (check for cycles, missing dependencies, etc.).
    pub fn validate(&self) -> Result<()> {
        let stage_ids: std::collections::HashSet<_> =
            self.stages.iter().map(|s| s.stage_id.as_str()).collect();

        // Check all dependencies exist
        for stage in &self.stages {
            for dep in &stage.dependencies {
                if !stage_ids.contains(dep.as_str()) {
                    return Err(CollectiveError::WorkflowExecutionFailed(format!(
                        "Stage {} depends on unknown stage {}",
                        stage.stage_id, dep
                    )));
                }
            }
        }

        // Check for cycles (simple check)
        // TODO: Implement proper cycle detection
        for stage in &self.stages {
            if stage.dependencies.contains(&stage.stage_id) {
                return Err(CollectiveError::WorkflowExecutionFailed(format!(
                    "Stage {} has a self-dependency",
                    stage.stage_id
                )));
            }
        }

        Ok(())
    }
}

/// Tracks the execution state of a stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageExecution {
    /// Stage ID
    pub stage_id: String,

    /// Current state
    pub state: StageState,

    /// Agents assigned to this stage
    pub assigned_agents: Vec<InstanceId>,

    /// Results from each agent
    pub results: HashMap<InstanceId, serde_json::Value>,

    /// When the stage started
    pub started_at: Option<DateTime<Utc>>,

    /// When the stage completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Error message if failed
    pub error: Option<String>,
}

impl StageExecution {
    /// Create a new stage execution.
    pub fn new(stage_id: String) -> Self {
        Self {
            stage_id,
            state: StageState::Pending,
            assigned_agents: Vec::new(),
            results: HashMap::new(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}

/// Tracks the execution of a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// The workflow being executed
    pub workflow_id: WorkflowId,

    /// Unique execution ID
    pub execution_id: ExecutionId,

    /// Stage execution states
    pub stages: HashMap<String, StageExecution>,

    /// Combined results from all stages
    pub results: HashMap<String, serde_json::Value>,

    /// When execution started
    pub started_at: DateTime<Utc>,

    /// When execution completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Final state
    pub state: WorkflowState,
}

impl WorkflowExecution {
    /// Create a new workflow execution.
    pub fn new(workflow: &Workflow) -> Self {
        let mut stages = HashMap::new();
        for stage in &workflow.stages {
            stages.insert(
                stage.stage_id.clone(),
                StageExecution::new(stage.stage_id.clone()),
            );
        }

        Self {
            workflow_id: workflow.workflow_id.clone(),
            execution_id: uuid::Uuid::new_v4().to_string(),
            stages,
            results: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
            state: WorkflowState::Running,
        }
    }

    /// Get stages that are ready to execute.
    pub fn ready_stages<'a>(&self, workflow: &'a Workflow) -> Vec<&'a str> {
        let mut ready = Vec::new();

        for stage in &workflow.stages {
            if let Some(execution) = self.stages.get(&stage.stage_id) {
                if execution.state != StageState::Pending {
                    continue;
                }

                // Check if all dependencies are completed
                let deps_completed = stage.dependencies.iter().all(|dep| {
                    self.stages
                        .get(dep)
                        .map(|s| s.state == StageState::Completed)
                        .unwrap_or(false)
                });

                if deps_completed {
                    ready.push(stage.stage_id.as_str());
                }
            }
        }

        ready
    }

    /// Check if the workflow is complete.
    pub fn is_complete(&self) -> bool {
        self.stages
            .values()
            .all(|s| matches!(s.state, StageState::Completed | StageState::Skipped))
    }

    /// Check if the workflow has failed.
    pub fn has_failed(&self) -> bool {
        self.stages
            .values()
            .any(|s| matches!(s.state, StageState::Failed { .. }))
    }
}

/// Orchestrates workflow execution.
#[derive(Debug)]
pub struct WorkflowEngine {
    /// This agent's instance ID
    instance_id: InstanceId,

    /// Workflow definitions
    workflows: HashMap<WorkflowId, Workflow>,

    /// Active workflow executions
    executions: HashMap<ExecutionId, WorkflowExecution>,

    /// Maximum concurrent workflows
    max_concurrent: usize,
}

impl WorkflowEngine {
    /// Create a new workflow engine.
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            instance_id,
            workflows: HashMap::new(),
            executions: HashMap::new(),
            max_concurrent: 5,
        }
    }

    /// Get this agent's instance ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Set maximum concurrent workflows.
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max;
    }

    /// Register a workflow definition.
    pub fn register_workflow(&mut self, workflow: Workflow) -> Result<WorkflowId> {
        workflow.validate()?;
        let workflow_id = workflow.workflow_id.clone();
        self.workflows.insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }

    /// Get a workflow definition.
    pub fn get_workflow(&self, workflow_id: &str) -> Option<&Workflow> {
        self.workflows.get(workflow_id)
    }

    /// Start executing a workflow.
    pub fn start_execution(&mut self, workflow_id: &str) -> Result<ExecutionId> {
        if self.executions.len() >= self.max_concurrent {
            return Err(CollectiveError::WorkflowExecutionFailed(
                "Maximum concurrent workflows reached".to_string(),
            ));
        }

        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(workflow_id.to_string()))?;

        let execution = WorkflowExecution::new(workflow);
        let execution_id = execution.execution_id.clone();
        self.executions.insert(execution_id.clone(), execution);

        Ok(execution_id)
    }

    /// Get an execution.
    pub fn get_execution(&self, execution_id: &str) -> Option<&WorkflowExecution> {
        self.executions.get(execution_id)
    }

    /// Get a mutable execution.
    pub fn get_execution_mut(&mut self, execution_id: &str) -> Option<&mut WorkflowExecution> {
        self.executions.get_mut(execution_id)
    }

    /// Mark a stage as started.
    pub fn start_stage(
        &mut self,
        execution_id: &str,
        stage_id: &str,
        agents: Vec<InstanceId>,
    ) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(execution_id.to_string()))?;

        if let Some(stage) = execution.stages.get_mut(stage_id) {
            stage.state = StageState::Running;
            stage.assigned_agents = agents;
            stage.started_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Record a stage result from an agent.
    pub fn record_stage_result(
        &mut self,
        execution_id: &str,
        stage_id: &str,
        agent_id: InstanceId,
        result: serde_json::Value,
    ) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(execution_id.to_string()))?;

        if let Some(stage) = execution.stages.get_mut(stage_id) {
            stage.results.insert(agent_id, result);
        }

        Ok(())
    }

    /// Mark a stage as completed.
    pub fn complete_stage(
        &mut self,
        execution_id: &str,
        stage_id: &str,
        final_result: serde_json::Value,
    ) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(execution_id.to_string()))?;

        if let Some(stage) = execution.stages.get_mut(stage_id) {
            stage.state = StageState::Completed;
            stage.completed_at = Some(Utc::now());
        }

        execution.results.insert(stage_id.to_string(), final_result);

        // Check if workflow is complete
        if execution.is_complete() {
            execution.state = WorkflowState::Completed;
            execution.completed_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Mark a stage as failed.
    pub fn fail_stage(&mut self, execution_id: &str, stage_id: &str, reason: String) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(execution_id.to_string()))?;

        if let Some(stage) = execution.stages.get_mut(stage_id) {
            stage.state = StageState::Failed {
                reason: reason.clone(),
            };
            stage.error = Some(reason.clone());
            stage.completed_at = Some(Utc::now());
        }

        execution.state = WorkflowState::Failed { reason };
        execution.completed_at = Some(Utc::now());

        Ok(())
    }

    /// Get stages ready for execution.
    pub fn get_ready_stages(&self, execution_id: &str) -> Result<Vec<String>> {
        let execution = self
            .executions
            .get(execution_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(execution_id.to_string()))?;

        let workflow = self
            .workflows
            .get(&execution.workflow_id)
            .ok_or_else(|| CollectiveError::WorkflowNotFound(execution.workflow_id.clone()))?;

        Ok(execution
            .ready_stages(workflow)
            .into_iter()
            .map(String::from)
            .collect())
    }

    /// Get active executions.
    pub fn active_executions(&self) -> Vec<&WorkflowExecution> {
        self.executions
            .values()
            .filter(|e| e.state == WorkflowState::Running)
            .collect()
    }

    /// Clean up completed executions.
    pub fn cleanup_completed(&mut self, max_age: Duration) -> usize {
        let cutoff = Utc::now() - max_age;
        let before = self.executions.len();

        self.executions
            .retain(|_, e| e.completed_at.map(|t| t > cutoff).unwrap_or(true));

        before - self.executions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new("test-workflow", "A test workflow", "agent-1".to_string())
            .add_stage(WorkflowStage::sequential(
                "stage-1",
                "First Stage",
                "Do first thing",
            ))
            .add_stage(
                WorkflowStage::parallel("stage-2", "Second Stage", "Do in parallel", 2)
                    .with_dependencies(vec!["stage-1".to_string()]),
            );

        assert_eq!(workflow.stages.len(), 2);
        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_workflow_execution() {
        let mut engine = WorkflowEngine::new("agent-1".to_string());

        let workflow = Workflow::new("test", "Test", "agent-1".to_string())
            .add_stage(WorkflowStage::sequential("s1", "Stage 1", "First"))
            .add_stage(
                WorkflowStage::sequential("s2", "Stage 2", "Second")
                    .with_dependencies(vec!["s1".to_string()]),
            );

        let workflow_id = engine.register_workflow(workflow).unwrap();
        let execution_id = engine.start_execution(&workflow_id).unwrap();

        // Check ready stages
        let ready = engine.get_ready_stages(&execution_id).unwrap();
        assert_eq!(ready, vec!["s1"]);

        // Start and complete first stage
        engine
            .start_stage(&execution_id, "s1", vec!["agent-1".to_string()])
            .unwrap();
        engine
            .complete_stage(&execution_id, "s1", serde_json::json!({"done": true}))
            .unwrap();

        // Now s2 should be ready
        let ready = engine.get_ready_stages(&execution_id).unwrap();
        assert_eq!(ready, vec!["s2"]);
    }
}

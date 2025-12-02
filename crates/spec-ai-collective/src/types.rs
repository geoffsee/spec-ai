//! Common types used across the collective intelligence system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for an agent instance in the mesh.
pub type InstanceId = String;

/// Unique identifier for a task.
pub type TaskId = String;

/// Unique identifier for a strategy.
pub type StrategyId = String;

/// Unique identifier for a proposal.
pub type ProposalId = String;

/// Unique identifier for a workflow.
pub type WorkflowId = String;

/// Unique identifier for a workflow execution.
pub type ExecutionId = String;

/// Domain identifier for capabilities (e.g., "code_review", "data_analysis").
pub type Domain = String;

/// Message types for collective intelligence communication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CollectiveMessageType {
    /// Update capability information
    CapabilityUpdate,
    /// Share a learned strategy
    LearningShare,
    /// Submit a proposal for collective decision
    ProposalSubmit,
    /// Vote on a proposal
    ProposalVote,
    /// Assign a workflow stage to an agent
    WorkflowAssignment,
    /// Report completion of a workflow stage
    WorkflowStageResult,
    /// Request capability information from peers
    CapabilityQuery,
    /// Response to capability query
    CapabilityResponse,
}

/// Timestamp wrapper for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl Default for Timestamp {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

/// Error types for collective intelligence operations.
#[derive(Debug, thiserror::Error)]
pub enum CollectiveError {
    #[error("No capable agent found for task: {0}")]
    NoCapableAgent(String),

    #[error("Task delegation failed: {0}")]
    DelegationFailed(String),

    #[error("Strategy not found: {0}")]
    StrategyNotFound(String),

    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),

    #[error("Proposal expired: {0}")]
    ProposalExpired(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Workflow execution failed: {0}")]
    WorkflowExecutionFailed(String),

    #[error("Quorum not reached: required {required}, got {actual}")]
    QuorumNotReached { required: f32, actual: f32 },

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, CollectiveError>;

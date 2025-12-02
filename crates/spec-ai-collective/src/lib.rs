//! Emergent collective intelligence for spec-ai agents.
//!
//! This crate provides the infrastructure for agents to work together as a
//! collective intelligence system, enabling:
//!
//! - **Capability Tracking**: Agents track their proficiency in different domains
//! - **Task Delegation**: Agents route tasks to peers with matching capabilities
//! - **Inter-Agent Learning**: Agents share successful strategies with each other
//! - **Collective Decision-Making**: Agents vote on proposals with expertise-weighted voting
//! - **Workflow Orchestration**: Coordinate complex multi-agent workflows
//! - **Emergent Specialization**: Agents develop and leverage expertise over time
//!
//! # Architecture
//!
//! The collective intelligence system is built on top of the existing spec-ai
//! infrastructure:
//!
//! - Uses the mesh communication layer for inter-agent messaging
//! - Leverages the knowledge graph for storing capabilities and strategies
//! - Integrates with graph sync for distributed learning
//!
//! # Usage
//!
//! ```ignore
//! use spec_ai_collective::{
//!     CapabilityTracker, DelegationManager, LearningFabric,
//!     ConsensusCoordinator, WorkflowEngine, SpecializationEngine,
//! };
//!
//! // Track agent capabilities
//! let tracker = CapabilityTracker::new(instance_id);
//! tracker.record_task_outcome(domain, outcome).await?;
//!
//! // Delegate tasks to capable peers
//! let delegator = DelegationManager::new(tracker.clone(), mesh_client);
//! delegator.delegate_task(task).await?;
//!
//! // Share learnings across the mesh
//! let fabric = LearningFabric::new(graph_store);
//! fabric.share_learning(strategy).await?;
//! ```

pub mod capability;
pub mod consensus;
pub mod delegation;
pub mod learning;
pub mod orchestration;
pub mod specialization;
pub mod types;

// Re-export main types for convenience
pub use capability::{Capability, CapabilityTracker, ExpertiseProfile, LearningEvent, TaskOutcome};
pub use consensus::{ConsensusCoordinator, Proposal, ProposalStatus, ProposalType, Vote, VoteDecision};
pub use delegation::{DelegatedTask, DelegationManager, ExecutionMetrics, RoutingDecision, TaskPriority, TaskResult, TaskStatus};
pub use learning::{LearningFabric, Strategy, StrategyMatch};
pub use orchestration::{StageState, StageType, Workflow, WorkflowEngine, WorkflowExecution, WorkflowStage, WorkflowState};
pub use specialization::{Specialist, SpecializationEngine, SpecializationStatus};

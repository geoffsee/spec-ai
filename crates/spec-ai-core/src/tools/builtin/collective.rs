//! Collective intelligence tools for multi-agent coordination.
//!
//! These tools enable agents to:
//! - Delegate tasks to capable peers
//! - Share and query learned strategies
//! - Participate in collective decision-making
//! - Coordinate multi-agent workflows

use crate::mesh::{MeshClient, MessageType};
use crate::tools::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// Capability & Delegation Tools
// ============================================================================

/// Tool for delegating a task to a capable peer agent
pub struct DelegateTaskTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl DelegateTaskTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DelegateTaskArgs {
    task_type: String,
    description: String,
    required_capabilities: Vec<String>,
    payload: Value,
    #[serde(default)]
    priority: Option<String>,
    target_instance: Option<String>,
}

#[async_trait]
impl Tool for DelegateTaskTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "Delegate a task to another agent in the mesh based on required capabilities. \
         If target_instance is not specified, the system will route to the best available agent."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_type": {
                    "type": "string",
                    "description": "Type of task (e.g., 'code_review', 'data_analysis', 'testing')"
                },
                "description": {
                    "type": "string",
                    "description": "Human-readable description of the task"
                },
                "required_capabilities": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of capabilities required to perform this task"
                },
                "payload": {
                    "type": "object",
                    "description": "Task-specific data and context"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "normal", "high", "critical"],
                    "description": "Task priority level"
                },
                "target_instance": {
                    "type": "string",
                    "description": "Specific instance to delegate to. If omitted, routes to best capable agent."
                }
            },
            "required": ["task_type", "description", "required_capabilities", "payload"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: DelegateTaskArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured. Cannot delegate tasks.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        // Create task delegation payload
        let task_id = uuid::Uuid::new_v4().to_string();
        let delegation_payload = json!({
            "task_id": task_id,
            "task_type": args.task_type,
            "description": args.description,
            "required_capabilities": args.required_capabilities,
            "payload": args.payload,
            "priority": args.priority.unwrap_or_else(|| "normal".to_string()),
            "delegator": self.instance_id,
        });

        // Send task delegation message
        let response = client
            .send_message(
                self.instance_id.clone(),
                args.target_instance,
                MessageType::TaskDelegation,
                delegation_payload,
                Some(task_id.clone()),
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Task delegated successfully.\n\
             Task ID: {}\n\
             Status: {}\n\
             Delivered to: {:?}",
            task_id, response.status, response.delivered_to
        )))
    }
}

/// Tool for querying capabilities of agents in the mesh
pub struct QueryCapabilitiesTool {
    mesh_url: Option<String>,
}

impl QueryCapabilitiesTool {
    pub fn new(mesh_url: Option<String>) -> Self {
        Self { mesh_url }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct QueryCapabilitiesArgs {
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    min_proficiency: Option<f32>,
}

#[async_trait]
impl Tool for QueryCapabilitiesTool {
    fn name(&self) -> &str {
        "query_capabilities"
    }

    fn description(&self) -> &str {
        "Query the capabilities and expertise of agents in the mesh. \
         Can filter by domain and minimum proficiency level."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "domain": {
                    "type": "string",
                    "description": "Filter by specific domain (e.g., 'code_review', 'data_analysis')"
                },
                "min_proficiency": {
                    "type": "number",
                    "description": "Minimum proficiency level (0.0 to 1.0)"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: QueryCapabilitiesArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        // Get all instances
        let instances = client.list_instances().await?;

        // Format capability information
        let mut result = String::from("## Agent Capabilities\n\n");

        for instance in instances.instances {
            result.push_str(&format!("### {}\n", instance.instance_id));
            result.push_str(&format!("- Host: {}:{}\n", instance.hostname, instance.port));
            result.push_str(&format!("- Leader: {}\n", instance.is_leader));
            result.push_str(&format!("- Capabilities: {:?}\n", instance.capabilities));
            result.push_str(&format!("- Profiles: {:?}\n\n", instance.agent_profiles));
        }

        if let Some(domain) = args.domain {
            result.push_str(&format!("\n*Filtered by domain: {}*\n", domain));
        }
        if let Some(min) = args.min_proficiency {
            result.push_str(&format!("*Minimum proficiency: {:.2}*\n", min));
        }

        Ok(ToolResult::success(result))
    }
}

/// Tool for broadcasting capability updates to the mesh
pub struct ShareCapabilitiesTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl ShareCapabilitiesTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ShareCapabilitiesArgs {
    capabilities: Vec<CapabilityInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CapabilityInfo {
    domain: String,
    proficiency: f32,
    experience_count: u64,
    success_rate: f32,
}

#[async_trait]
impl Tool for ShareCapabilitiesTool {
    fn name(&self) -> &str {
        "share_capabilities"
    }

    fn description(&self) -> &str {
        "Broadcast this agent's capability profile to other agents in the mesh."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "capabilities": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "domain": {"type": "string"},
                            "proficiency": {"type": "number"},
                            "experience_count": {"type": "integer"},
                            "success_rate": {"type": "number"}
                        },
                        "required": ["domain", "proficiency"]
                    },
                    "description": "List of capabilities to share"
                }
            },
            "required": ["capabilities"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: ShareCapabilitiesArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        let payload = json!({
            "instance_id": self.instance_id,
            "capabilities": args.capabilities,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Broadcast to all agents
        let response = client
            .send_message(
                self.instance_id.clone(),
                None, // Broadcast
                MessageType::CapabilityUpdate,
                payload,
                None,
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Capabilities shared with {} agents.",
            response.delivered_to.len()
        )))
    }
}

// ============================================================================
// Learning & Strategy Tools
// ============================================================================

/// Tool for sharing a learned strategy with the mesh
pub struct ShareStrategyTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl ShareStrategyTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ShareStrategyArgs {
    task_type: String,
    description: String,
    approach: Vec<String>,
    success_rate: f32,
    #[serde(default)]
    tags: Vec<String>,
}

#[async_trait]
impl Tool for ShareStrategyTool {
    fn name(&self) -> &str {
        "share_strategy"
    }

    fn description(&self) -> &str {
        "Share a successful strategy with other agents in the mesh. \
         Strategies include task type, description, approach steps, and success metrics."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_type": {
                    "type": "string",
                    "description": "Type of task this strategy applies to"
                },
                "description": {
                    "type": "string",
                    "description": "Brief description of the strategy"
                },
                "approach": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Step-by-step approach"
                },
                "success_rate": {
                    "type": "number",
                    "description": "Success rate of this strategy (0.0 to 1.0)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags for categorization"
                }
            },
            "required": ["task_type", "description", "approach", "success_rate"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: ShareStrategyArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        let strategy_id = uuid::Uuid::new_v4().to_string();
        let payload = json!({
            "strategy_id": strategy_id,
            "task_type": args.task_type,
            "description": args.description,
            "approach": args.approach,
            "success_rate": args.success_rate,
            "tags": args.tags,
            "created_by": self.instance_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let response = client
            .send_message(
                self.instance_id.clone(),
                None, // Broadcast
                MessageType::LearningShare,
                payload,
                None,
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Strategy '{}' shared with {} agents.\nStrategy ID: {}",
            args.description,
            response.delivered_to.len(),
            strategy_id
        )))
    }
}

// ============================================================================
// Consensus & Voting Tools
// ============================================================================

/// Tool for submitting a proposal for collective decision
pub struct SubmitProposalTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl SubmitProposalTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SubmitProposalArgs {
    title: String,
    description: String,
    proposal_type: String,
    content: Value,
    #[serde(default = "default_quorum")]
    required_quorum: f32,
    #[serde(default = "default_duration_hours")]
    duration_hours: u64,
}

fn default_quorum() -> f32 {
    0.5
}

fn default_duration_hours() -> u64 {
    24
}

#[async_trait]
impl Tool for SubmitProposalTool {
    fn name(&self) -> &str {
        "submit_proposal"
    }

    fn description(&self) -> &str {
        "Submit a proposal for collective decision-making. Other agents can vote on the proposal."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Title of the proposal"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description of what is being proposed"
                },
                "proposal_type": {
                    "type": "string",
                    "enum": ["strategy_adoption", "policy_change", "resource_allocation", "conflict_resolution"],
                    "description": "Type of proposal"
                },
                "content": {
                    "type": "object",
                    "description": "Proposal-specific content and details"
                },
                "required_quorum": {
                    "type": "number",
                    "description": "Required quorum (0.0 to 1.0). Default: 0.5"
                },
                "duration_hours": {
                    "type": "integer",
                    "description": "Voting duration in hours. Default: 24"
                }
            },
            "required": ["title", "description", "proposal_type", "content"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: SubmitProposalArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        let proposal_id = uuid::Uuid::new_v4().to_string();
        let deadline = chrono::Utc::now() + chrono::Duration::hours(args.duration_hours as i64);

        let payload = json!({
            "proposal_id": proposal_id,
            "proposer_id": self.instance_id,
            "title": args.title,
            "description": args.description,
            "proposal_type": args.proposal_type,
            "content": args.content,
            "required_quorum": args.required_quorum,
            "deadline": deadline.to_rfc3339(),
        });

        let response = client
            .send_message(
                self.instance_id.clone(),
                None, // Broadcast
                MessageType::ProposalSubmit,
                payload,
                Some(proposal_id.clone()),
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Proposal submitted successfully.\n\
             Proposal ID: {}\n\
             Title: {}\n\
             Deadline: {}\n\
             Broadcast to: {} agents",
            proposal_id,
            args.title,
            deadline.format("%Y-%m-%d %H:%M UTC"),
            response.delivered_to.len()
        )))
    }
}

/// Tool for casting a vote on a proposal
pub struct CastVoteTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl CastVoteTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CastVoteArgs {
    proposal_id: String,
    decision: String,
    #[serde(default)]
    rationale: Option<String>,
}

#[async_trait]
impl Tool for CastVoteTool {
    fn name(&self) -> &str {
        "cast_vote"
    }

    fn description(&self) -> &str {
        "Cast a vote on an open proposal. Votes are weighted by expertise in relevant domains."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "proposal_id": {
                    "type": "string",
                    "description": "ID of the proposal to vote on"
                },
                "decision": {
                    "type": "string",
                    "enum": ["approve", "reject", "abstain"],
                    "description": "Vote decision"
                },
                "rationale": {
                    "type": "string",
                    "description": "Optional explanation for the vote"
                }
            },
            "required": ["proposal_id", "decision"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: CastVoteArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        let payload = json!({
            "proposal_id": args.proposal_id,
            "voter_id": self.instance_id,
            "decision": args.decision,
            "rationale": args.rationale,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let response = client
            .send_message(
                self.instance_id.clone(),
                None, // Broadcast to ensure proposer receives vote
                MessageType::ProposalVote,
                payload,
                Some(args.proposal_id.clone()),
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Vote cast successfully.\n\
             Proposal: {}\n\
             Decision: {}\n\
             Broadcast to: {} agents",
            args.proposal_id,
            args.decision,
            response.delivered_to.len()
        )))
    }
}

// ============================================================================
// Workflow Tools
// ============================================================================

/// Tool for creating and starting a multi-agent workflow
pub struct CreateWorkflowTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl CreateWorkflowTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateWorkflowArgs {
    name: String,
    description: String,
    stages: Vec<WorkflowStageArg>,
    #[serde(default)]
    input: Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowStageArg {
    stage_id: String,
    name: String,
    description: String,
    stage_type: String,
    required_capabilities: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    config: Value,
}

#[async_trait]
impl Tool for CreateWorkflowTool {
    fn name(&self) -> &str {
        "create_workflow"
    }

    fn description(&self) -> &str {
        "Create a multi-agent workflow with defined stages. \
         Stages can be sequential, parallel, map-reduce, or require consensus."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the workflow"
                },
                "description": {
                    "type": "string",
                    "description": "Description of what the workflow accomplishes"
                },
                "stages": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "stage_id": {"type": "string"},
                            "name": {"type": "string"},
                            "description": {"type": "string"},
                            "stage_type": {
                                "type": "string",
                                "enum": ["sequential", "parallel", "map_reduce", "consensus"]
                            },
                            "required_capabilities": {
                                "type": "array",
                                "items": {"type": "string"}
                            },
                            "dependencies": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Stage IDs that must complete before this stage"
                            },
                            "config": {
                                "type": "object",
                                "description": "Stage-specific configuration"
                            }
                        },
                        "required": ["stage_id", "name", "description", "stage_type", "required_capabilities"]
                    },
                    "description": "Workflow stages"
                },
                "input": {
                    "type": "object",
                    "description": "Initial input data for the workflow"
                }
            },
            "required": ["name", "description", "stages"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: CreateWorkflowArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        let workflow_id = uuid::Uuid::new_v4().to_string();
        let execution_id = uuid::Uuid::new_v4().to_string();

        let payload = json!({
            "workflow_id": workflow_id,
            "execution_id": execution_id,
            "name": args.name,
            "description": args.description,
            "stages": args.stages,
            "input": args.input,
            "orchestrator": self.instance_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        // Broadcast workflow to all agents
        let response = client
            .send_message(
                self.instance_id.clone(),
                None, // Broadcast
                MessageType::WorkflowAssignment,
                payload,
                Some(execution_id.clone()),
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Workflow created and started.\n\
             Workflow ID: {}\n\
             Execution ID: {}\n\
             Name: {}\n\
             Stages: {}\n\
             Broadcast to: {} agents",
            workflow_id,
            execution_id,
            args.name,
            args.stages.len(),
            response.delivered_to.len()
        )))
    }
}

/// Tool for reporting workflow stage completion
pub struct ReportStageResultTool {
    instance_id: String,
    mesh_url: Option<String>,
}

impl ReportStageResultTool {
    pub fn new(instance_id: String, mesh_url: Option<String>) -> Self {
        Self {
            instance_id,
            mesh_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ReportStageResultArgs {
    execution_id: String,
    stage_id: String,
    status: String,
    result: Value,
    #[serde(default)]
    learnings: Vec<String>,
}

#[async_trait]
impl Tool for ReportStageResultTool {
    fn name(&self) -> &str {
        "report_stage_result"
    }

    fn description(&self) -> &str {
        "Report the completion of a workflow stage. Includes status, result, and any learnings."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "execution_id": {
                    "type": "string",
                    "description": "Workflow execution ID"
                },
                "stage_id": {
                    "type": "string",
                    "description": "Stage that was completed"
                },
                "status": {
                    "type": "string",
                    "enum": ["completed", "failed", "skipped"],
                    "description": "Completion status"
                },
                "result": {
                    "type": "object",
                    "description": "Stage output/result"
                },
                "learnings": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Strategies or insights learned during execution"
                }
            },
            "required": ["execution_id", "stage_id", "status", "result"]
        })
    }

    async fn execute(&self, args: Value) -> Result<ToolResult> {
        let args: ReportStageResultArgs = serde_json::from_value(args)?;

        let Some(ref mesh_url) = self.mesh_url else {
            return Ok(ToolResult::failure(
                "Mesh communication not configured.",
            ));
        };

        let parts: Vec<&str> = mesh_url.split(':').collect();
        if parts.len() != 2 {
            return Ok(ToolResult::failure(format!(
                "Invalid mesh URL: {}",
                mesh_url
            )));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()?;
        let client = MeshClient::new(host, port);

        let payload = json!({
            "execution_id": args.execution_id,
            "stage_id": args.stage_id,
            "executor_id": self.instance_id,
            "status": args.status,
            "result": args.result,
            "learnings": args.learnings,
            "completed_at": chrono::Utc::now().to_rfc3339(),
        });

        let response = client
            .send_message(
                self.instance_id.clone(),
                None, // Broadcast to orchestrator
                MessageType::WorkflowStageResult,
                payload,
                Some(args.execution_id.clone()),
            )
            .await?;

        Ok(ToolResult::success(format!(
            "Stage result reported.\n\
             Execution: {}\n\
             Stage: {}\n\
             Status: {}\n\
             Broadcast to: {} agents",
            args.execution_id,
            args.stage_id,
            args.status,
            response.delivered_to.len()
        )))
    }
}

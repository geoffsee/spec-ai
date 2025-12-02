//! Shared mesh protocol types and client helpers.
use anyhow::Result;
use chrono::{DateTime, Utc};
use hostname::get as get_hostname;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::{NoContext, Timestamp, Uuid};

/// Agent instance information in the mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshInstance {
    pub instance_id: String,
    pub hostname: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub is_leader: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub agent_profiles: Vec<String>,
}

/// Request to register a new instance
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub instance_id: String,
    pub hostname: String,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub agent_profiles: Vec<String>,
}

/// Response from registration
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub instance_id: String,
    pub is_leader: bool,
    pub leader_id: Option<String>,
    pub peers: Vec<MeshInstance>,
}

/// List of registered instances
#[derive(Debug, Serialize, Deserialize)]
pub struct InstancesResponse {
    pub instances: Vec<MeshInstance>,
    pub leader_id: Option<String>,
}

/// Heartbeat request
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub status: String,
    pub metrics: Option<HashMap<String, serde_json::Value>>,
}

/// Heartbeat response
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub acknowledged: bool,
    pub leader_id: Option<String>,
    pub should_sync: bool,
}

/// Message types for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Query,
    Response,
    Notification,
    TaskDelegation,
    TaskResult,
    GraphSync,
    // Collective Intelligence message types
    CapabilityUpdate,    // Share capability/expertise profile updates
    CapabilityQuery,     // Request capability information from peers
    LearningShare,       // Share a learned strategy with the mesh
    ProposalSubmit,      // Submit a proposal for collective decision
    ProposalVote,        // Cast a vote on a proposal
    WorkflowAssignment,  // Assign a workflow stage to an agent
    WorkflowStageResult, // Report completion of a workflow stage
    Custom(String),
}

impl MessageType {
    pub fn as_str(&self) -> String {
        match self {
            MessageType::Query => "query".to_string(),
            MessageType::Response => "response".to_string(),
            MessageType::Notification => "notification".to_string(),
            MessageType::TaskDelegation => "task_delegation".to_string(),
            MessageType::TaskResult => "task_result".to_string(),
            MessageType::GraphSync => "graph_sync".to_string(),
            MessageType::CapabilityUpdate => "capability_update".to_string(),
            MessageType::CapabilityQuery => "capability_query".to_string(),
            MessageType::LearningShare => "learning_share".to_string(),
            MessageType::ProposalSubmit => "proposal_submit".to_string(),
            MessageType::ProposalVote => "proposal_vote".to_string(),
            MessageType::WorkflowAssignment => "workflow_assignment".to_string(),
            MessageType::WorkflowStageResult => "workflow_stage_result".to_string(),
            MessageType::Custom(s) => s.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "query" => MessageType::Query,
            "response" => MessageType::Response,
            "notification" => MessageType::Notification,
            "task_delegation" => MessageType::TaskDelegation,
            "task_result" => MessageType::TaskResult,
            "graph_sync" => MessageType::GraphSync,
            "capability_update" => MessageType::CapabilityUpdate,
            "capability_query" => MessageType::CapabilityQuery,
            "learning_share" => MessageType::LearningShare,
            "proposal_submit" => MessageType::ProposalSubmit,
            "proposal_vote" => MessageType::ProposalVote,
            "workflow_assignment" => MessageType::WorkflowAssignment,
            "workflow_stage_result" => MessageType::WorkflowStageResult,
            custom => MessageType::Custom(custom.to_string()),
        }
    }
}

/// Inter-agent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: String,
    pub source_instance: String,
    pub target_instance: Option<String>,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Message send request
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub target_instance: Option<String>,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
}

/// Message send response
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub status: String,
    pub delivered_to: Vec<String>,
}

/// Pending messages response
#[derive(Debug, Serialize, Deserialize)]
pub struct PendingMessagesResponse {
    pub messages: Vec<AgentMessage>,
}

/// Client-side mesh operations
#[derive(Clone)]
pub struct MeshClient {
    base_url: String,
    client: Client,
}

impl MeshClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            base_url: format!("http://{}:{}", host, port),
            client: Client::new(),
        }
    }

    /// Generate a unique instance ID
    pub fn generate_instance_id() -> String {
        let hostname = get_hostname()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        format!("{}-{}", hostname, uuid)
    }

    /// Register this instance with a mesh registry
    pub async fn register(
        &self,
        instance_id: String,
        hostname: String,
        port: u16,
        capabilities: Vec<String>,
        agent_profiles: Vec<String>,
    ) -> Result<RegisterResponse> {
        let request = RegisterRequest {
            instance_id,
            hostname,
            port,
            capabilities,
            agent_profiles,
        };

        let response = self
            .client
            .post(format!("{}/registry/register", self.base_url))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Registration failed: {}", response.status())
        }
    }

    /// Send heartbeat to registry
    pub async fn heartbeat(
        &self,
        instance_id: &str,
        metrics: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<HeartbeatResponse> {
        let request = HeartbeatRequest {
            status: "healthy".to_string(),
            metrics,
        };

        let response = self
            .client
            .post(format!(
                "{}/registry/heartbeat/{}",
                self.base_url, instance_id
            ))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Heartbeat failed: {}", response.status())
        }
    }

    /// List all instances in the mesh
    pub async fn list_instances(&self) -> Result<InstancesResponse> {
        let response = self
            .client
            .get(format!("{}/registry/agents", self.base_url))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to list instances: {}", response.status())
        }
    }

    /// Deregister from the mesh
    pub async fn deregister(&self, instance_id: &str) -> Result<()> {
        let response = self
            .client
            .delete(format!(
                "{}/registry/deregister/{}",
                self.base_url, instance_id
            ))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("Deregistration failed: {}", response.status())
        }
    }

    /// Send a message to another instance
    pub async fn send_message(
        &self,
        source_instance: String,
        target_instance: Option<String>,
        message_type: MessageType,
        payload: serde_json::Value,
        correlation_id: Option<String>,
    ) -> Result<SendMessageResponse> {
        let request = SendMessageRequest {
            target_instance,
            message_type,
            payload,
            correlation_id,
        };

        let response = self
            .client
            .post(format!(
                "{}/messages/send/{}",
                self.base_url, source_instance
            ))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Send message failed: {}", response.status())
        }
    }

    /// Get pending messages for an instance
    pub async fn get_messages(&self, instance_id: &str) -> Result<PendingMessagesResponse> {
        let response = self
            .client
            .get(format!("{}/messages/{}", self.base_url, instance_id))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to get messages: {}", response.status())
        }
    }

    /// Acknowledge delivered messages
    pub async fn acknowledge_messages(
        &self,
        instance_id: &str,
        message_ids: Vec<String>,
    ) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/messages/{}/ack", self.base_url, instance_id))
            .json(&json!({ "message_ids": message_ids }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("Failed to acknowledge messages: {}", response.status())
        }
    }
}

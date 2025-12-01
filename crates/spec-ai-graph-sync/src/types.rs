//! Sync-specific types for graph synchronization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A changelog entry tracking mutations to graph entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub id: i64,
    pub session_id: String,
    pub instance_id: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub operation: String,
    pub vector_clock: String,
    pub data: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A graph node record with sync metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedNodeRecord {
    pub id: i64,
    pub session_id: String,
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub embedding_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub vector_clock: String,
    pub last_modified_by: Option<String>,
    pub is_deleted: bool,
    pub sync_enabled: bool,
}

/// A graph edge record with sync metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedEdgeRecord {
    pub id: i64,
    pub session_id: String,
    pub source_id: i64,
    pub target_id: i64,
    pub edge_type: String,
    pub predicate: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub weight: f32,
    pub temporal_start: Option<DateTime<Utc>>,
    pub temporal_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub vector_clock: String,
    pub last_modified_by: Option<String>,
    pub is_deleted: bool,
    pub sync_enabled: bool,
}

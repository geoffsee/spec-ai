//! Persistence trait for graph synchronization.
//!
//! This module defines the trait that must be implemented by any storage backend
//! that wants to use the sync engine.

use anyhow::Result;
use spec_ai_knowledge_graph::{EdgeType, NodeType};

use crate::types::{ChangelogEntry, SyncedEdgeRecord, SyncedNodeRecord};

/// Trait for persistence operations required by the sync engine.
///
/// This trait abstracts the database operations needed for graph synchronization,
/// allowing the sync engine to be used with any storage backend.
pub trait SyncPersistence: Send + Sync {
    /// Get the instance ID for this persistence instance.
    fn instance_id(&self) -> &str;

    // ========== Sync State Operations ==========

    /// Get the vector clock for the sync state of a specific graph.
    fn graph_sync_state_get(
        &self,
        instance_id: &str,
        session_id: &str,
        graph_name: &str,
    ) -> Result<Option<String>>;

    /// Update the vector clock for the sync state of a specific graph.
    fn graph_sync_state_update(
        &self,
        instance_id: &str,
        session_id: &str,
        graph_name: &str,
        vector_clock: &str,
    ) -> Result<()>;

    // ========== Node Count ==========

    /// Count nodes in a session.
    fn count_graph_nodes(&self, session_id: &str) -> Result<i64>;

    // ========== Changelog Operations ==========

    /// Append an entry to the changelog.
    fn graph_changelog_append(
        &self,
        session_id: &str,
        instance_id: &str,
        entity_type: &str,
        entity_id: i64,
        operation: &str,
        vector_clock: &str,
        data: Option<&str>,
    ) -> Result<i64>;

    /// Get changelog entries since a timestamp.
    fn graph_changelog_get_since(
        &self,
        session_id: &str,
        since_timestamp: &str,
    ) -> Result<Vec<ChangelogEntry>>;

    // ========== Synced Node Operations ==========

    /// Get a node with its sync metadata.
    fn graph_get_node_with_sync(&self, node_id: i64) -> Result<Option<SyncedNodeRecord>>;

    /// List nodes with sync metadata.
    fn graph_list_nodes_with_sync(
        &self,
        session_id: &str,
        sync_enabled_only: bool,
        include_deleted: bool,
    ) -> Result<Vec<SyncedNodeRecord>>;

    /// Update sync metadata for a node.
    fn graph_update_node_sync_metadata(
        &self,
        node_id: i64,
        vector_clock: &str,
        last_modified_by: &str,
        sync_enabled: bool,
    ) -> Result<()>;

    /// Mark a node as deleted.
    fn graph_mark_node_deleted(
        &self,
        node_id: i64,
        vector_clock: &str,
        deleted_by: &str,
    ) -> Result<()>;

    // ========== Synced Edge Operations ==========

    /// Get an edge with its sync metadata.
    fn graph_get_edge_with_sync(&self, edge_id: i64) -> Result<Option<SyncedEdgeRecord>>;

    /// List edges with sync metadata.
    fn graph_list_edges_with_sync(
        &self,
        session_id: &str,
        sync_enabled_only: bool,
        include_deleted: bool,
    ) -> Result<Vec<SyncedEdgeRecord>>;

    /// Update sync metadata for an edge.
    fn graph_update_edge_sync_metadata(
        &self,
        edge_id: i64,
        vector_clock: &str,
        last_modified_by: &str,
        sync_enabled: bool,
    ) -> Result<()>;

    /// Mark an edge as deleted.
    fn graph_mark_edge_deleted(
        &self,
        edge_id: i64,
        vector_clock: &str,
        deleted_by: &str,
    ) -> Result<()>;

    // ========== Node/Edge Insert and Update ==========

    /// Insert a new graph node.
    fn insert_graph_node(
        &self,
        session_id: &str,
        node_type: NodeType,
        label: &str,
        properties: &serde_json::Value,
        embedding_id: Option<i64>,
    ) -> Result<i64>;

    /// Update node properties.
    fn update_graph_node(&self, node_id: i64, properties: &serde_json::Value) -> Result<()>;

    /// Insert a new graph edge.
    fn insert_graph_edge(
        &self,
        session_id: &str,
        source_id: i64,
        target_id: i64,
        edge_type: EdgeType,
        predicate: Option<&str>,
        properties: Option<&serde_json::Value>,
        weight: f32,
    ) -> Result<i64>;
}

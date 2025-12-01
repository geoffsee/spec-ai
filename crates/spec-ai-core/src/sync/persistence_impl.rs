//! Implementation of `SyncPersistence` for `spec-ai-config::Persistence`.
//!
//! This module provides a wrapper type that implements the `SyncPersistence` trait,
//! allowing the sync engine to work with the persistence layer.

use anyhow::Result;
use spec_ai_graph_sync::{ChangelogEntry, SyncPersistence, SyncedEdgeRecord, SyncedNodeRecord};
use spec_ai_knowledge_graph::{EdgeType, NodeType};

use crate::persistence::Persistence;

/// Wrapper around `Persistence` that implements `SyncPersistence`.
///
/// This wrapper is necessary due to Rust's orphan rules - we cannot implement
/// a foreign trait (`SyncPersistence`) for a foreign type (`Persistence`) directly.
pub struct SyncPersistenceAdapter {
    persistence: Persistence,
}

impl SyncPersistenceAdapter {
    /// Create a new sync persistence adapter.
    pub fn new(persistence: Persistence) -> Self {
        Self { persistence }
    }

    /// Get a reference to the underlying persistence.
    pub fn persistence(&self) -> &Persistence {
        &self.persistence
    }

    /// Get a mutable reference to the underlying persistence.
    pub fn persistence_mut(&mut self) -> &mut Persistence {
        &mut self.persistence
    }

    /// Consume the adapter and return the underlying persistence.
    pub fn into_persistence(self) -> Persistence {
        self.persistence
    }
}

impl SyncPersistence for SyncPersistenceAdapter {
    fn instance_id(&self) -> &str {
        self.persistence.instance_id()
    }

    fn graph_sync_state_get(
        &self,
        instance_id: &str,
        session_id: &str,
        graph_name: &str,
    ) -> Result<Option<String>> {
        self.persistence
            .graph_sync_state_get(instance_id, session_id, graph_name)
    }

    fn graph_sync_state_update(
        &self,
        instance_id: &str,
        session_id: &str,
        graph_name: &str,
        vector_clock: &str,
    ) -> Result<()> {
        self.persistence
            .graph_sync_state_update(instance_id, session_id, graph_name, vector_clock)
    }

    fn count_graph_nodes(&self, session_id: &str) -> Result<i64> {
        self.persistence.count_graph_nodes(session_id)
    }

    fn graph_changelog_append(
        &self,
        session_id: &str,
        instance_id: &str,
        entity_type: &str,
        entity_id: i64,
        operation: &str,
        vector_clock: &str,
        data: Option<&str>,
    ) -> Result<i64> {
        self.persistence.graph_changelog_append(
            session_id,
            instance_id,
            entity_type,
            entity_id,
            operation,
            vector_clock,
            data,
        )
    }

    fn graph_changelog_get_since(
        &self,
        session_id: &str,
        since_timestamp: &str,
    ) -> Result<Vec<ChangelogEntry>> {
        let entries = self
            .persistence
            .graph_changelog_get_since(session_id, since_timestamp)?;
        Ok(entries
            .into_iter()
            .map(|e| ChangelogEntry {
                id: e.id,
                session_id: e.session_id,
                instance_id: e.instance_id,
                entity_type: e.entity_type,
                entity_id: e.entity_id,
                operation: e.operation,
                vector_clock: e.vector_clock,
                data: e.data,
                created_at: e.created_at,
            })
            .collect())
    }

    fn graph_get_node_with_sync(&self, node_id: i64) -> Result<Option<SyncedNodeRecord>> {
        let record = self.persistence.graph_get_node_with_sync(node_id)?;
        Ok(record.map(|r| SyncedNodeRecord {
            id: r.id,
            session_id: r.session_id,
            node_type: r.node_type,
            label: r.label,
            properties: r.properties,
            embedding_id: r.embedding_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            vector_clock: r.vector_clock,
            last_modified_by: r.last_modified_by,
            is_deleted: r.is_deleted,
            sync_enabled: r.sync_enabled,
        }))
    }

    fn graph_list_nodes_with_sync(
        &self,
        session_id: &str,
        sync_enabled_only: bool,
        include_deleted: bool,
    ) -> Result<Vec<SyncedNodeRecord>> {
        let records = self
            .persistence
            .graph_list_nodes_with_sync(session_id, sync_enabled_only, include_deleted)?;
        Ok(records
            .into_iter()
            .map(|r| SyncedNodeRecord {
                id: r.id,
                session_id: r.session_id,
                node_type: r.node_type,
                label: r.label,
                properties: r.properties,
                embedding_id: r.embedding_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                vector_clock: r.vector_clock,
                last_modified_by: r.last_modified_by,
                is_deleted: r.is_deleted,
                sync_enabled: r.sync_enabled,
            })
            .collect())
    }

    fn graph_update_node_sync_metadata(
        &self,
        node_id: i64,
        vector_clock: &str,
        last_modified_by: &str,
        sync_enabled: bool,
    ) -> Result<()> {
        self.persistence.graph_update_node_sync_metadata(
            node_id,
            vector_clock,
            last_modified_by,
            sync_enabled,
        )
    }

    fn graph_mark_node_deleted(
        &self,
        node_id: i64,
        vector_clock: &str,
        deleted_by: &str,
    ) -> Result<()> {
        self.persistence
            .graph_mark_node_deleted(node_id, vector_clock, deleted_by)
    }

    fn graph_get_edge_with_sync(&self, edge_id: i64) -> Result<Option<SyncedEdgeRecord>> {
        let record = self.persistence.graph_get_edge_with_sync(edge_id)?;
        Ok(record.map(|r| SyncedEdgeRecord {
            id: r.id,
            session_id: r.session_id,
            source_id: r.source_id,
            target_id: r.target_id,
            edge_type: r.edge_type,
            predicate: r.predicate,
            properties: r.properties,
            weight: r.weight,
            temporal_start: r.temporal_start,
            temporal_end: r.temporal_end,
            created_at: r.created_at,
            vector_clock: r.vector_clock,
            last_modified_by: r.last_modified_by,
            is_deleted: r.is_deleted,
            sync_enabled: r.sync_enabled,
        }))
    }

    fn graph_list_edges_with_sync(
        &self,
        session_id: &str,
        sync_enabled_only: bool,
        include_deleted: bool,
    ) -> Result<Vec<SyncedEdgeRecord>> {
        let records = self
            .persistence
            .graph_list_edges_with_sync(session_id, sync_enabled_only, include_deleted)?;
        Ok(records
            .into_iter()
            .map(|r| SyncedEdgeRecord {
                id: r.id,
                session_id: r.session_id,
                source_id: r.source_id,
                target_id: r.target_id,
                edge_type: r.edge_type,
                predicate: r.predicate,
                properties: r.properties,
                weight: r.weight,
                temporal_start: r.temporal_start,
                temporal_end: r.temporal_end,
                created_at: r.created_at,
                vector_clock: r.vector_clock,
                last_modified_by: r.last_modified_by,
                is_deleted: r.is_deleted,
                sync_enabled: r.sync_enabled,
            })
            .collect())
    }

    fn graph_update_edge_sync_metadata(
        &self,
        edge_id: i64,
        vector_clock: &str,
        last_modified_by: &str,
        sync_enabled: bool,
    ) -> Result<()> {
        self.persistence.graph_update_edge_sync_metadata(
            edge_id,
            vector_clock,
            last_modified_by,
            sync_enabled,
        )
    }

    fn graph_mark_edge_deleted(
        &self,
        edge_id: i64,
        vector_clock: &str,
        deleted_by: &str,
    ) -> Result<()> {
        self.persistence
            .graph_mark_edge_deleted(edge_id, vector_clock, deleted_by)
    }

    fn insert_graph_node(
        &self,
        session_id: &str,
        node_type: NodeType,
        label: &str,
        properties: &serde_json::Value,
        embedding_id: Option<i64>,
    ) -> Result<i64> {
        self.persistence
            .insert_graph_node(session_id, node_type, label, properties, embedding_id)
    }

    fn update_graph_node(&self, node_id: i64, properties: &serde_json::Value) -> Result<()> {
        self.persistence.update_graph_node(node_id, properties)
    }

    fn insert_graph_edge(
        &self,
        session_id: &str,
        source_id: i64,
        target_id: i64,
        edge_type: EdgeType,
        predicate: Option<&str>,
        properties: Option<&serde_json::Value>,
        weight: f32,
    ) -> Result<i64> {
        self.persistence.insert_graph_edge(
            session_id,
            source_id,
            target_id,
            edge_type,
            predicate,
            properties,
            weight,
        )
    }
}

//! Graph synchronization module.
//!
//! This module re-exports types from the `spec-ai-graph-sync` crate and provides
//! the implementation of `SyncPersistence` for `spec-ai-config::Persistence`.

mod persistence_impl;

// Re-export the adapter
pub use persistence_impl::SyncPersistenceAdapter;

// Re-export everything from spec-ai-graph-sync
pub use spec_ai_graph_sync::{
    ClockOrder, ConflictResolution, ConflictResolver, GraphSyncPayload, SyncAck, SyncConflict,
    SyncEngine, SyncFullRequest, SyncIncrementalRequest, SyncPersistence, SyncResponse, SyncStats,
    SyncType, SyncedEdge, SyncedNode, Tombstone, VectorClock,
};

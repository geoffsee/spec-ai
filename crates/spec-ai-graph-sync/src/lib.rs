//! Knowledge graph synchronization engine for spec-ai.
//!
//! This crate provides a distributed synchronization engine for knowledge graphs
//! using vector clocks for causal ordering and conflict detection.
//!
//! # Features
//!
//! - **Adaptive Sync Strategy**: Automatically decides between full and incremental sync
//!   based on the amount of changes.
//! - **Vector Clock Based Ordering**: Uses vector clocks to track causal relationships
//!   and detect concurrent modifications.
//! - **Conflict Resolution**: Configurable conflict resolution strategies for handling
//!   concurrent updates.
//! - **Tombstone Support**: Proper handling of deleted entities across distributed instances.
//!
//! # Usage
//!
//! To use this crate, implement the [`SyncPersistence`] trait for your storage backend,
//! then create a [`SyncEngine`] instance:
//!
//! ```ignore
//! use spec_ai_graph_sync::{SyncEngine, SyncPersistence};
//!
//! // Implement SyncPersistence for your storage
//! struct MyStorage { /* ... */ }
//! impl SyncPersistence for MyStorage { /* ... */ }
//!
//! // Create the sync engine
//! let storage = MyStorage::new();
//! let engine = SyncEngine::new(storage, "instance-1".to_string());
//!
//! // Perform sync operations
//! let payload = engine.sync_full("session-1", "default").await?;
//! ```

pub mod engine;
pub mod persistence;
pub mod protocol;
pub mod resolver;
pub mod types;

// Re-export main types for convenience
pub use engine::{SyncEngine, SyncStats};
pub use persistence::SyncPersistence;
pub use protocol::{
    GraphSyncPayload, SyncAck, SyncConflict, SyncFullRequest, SyncIncrementalRequest,
    SyncResponse, SyncType, SyncedEdge, SyncedNode, Tombstone,
};
pub use resolver::{ConflictResolution, ConflictResolver, ConflictRecord, ConflictType};
pub use types::{ChangelogEntry, SyncedEdgeRecord, SyncedNodeRecord};

// Re-export vector clock types from knowledge-graph crate
pub use spec_ai_knowledge_graph::{ClockOrder, VectorClock};

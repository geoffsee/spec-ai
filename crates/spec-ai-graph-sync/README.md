# spec-ai-graph-sync

Knowledge graph synchronization engine for spec-ai.

## Overview

This crate provides a distributed synchronization engine for knowledge graphs using vector clocks for causal ordering and conflict detection.

- **Adaptive Sync Strategy**: Automatically decides between full and incremental sync based on change volume
- **Vector Clock Ordering**: Tracks causal relationships and detects concurrent modifications
- **Conflict Resolution**: Configurable strategies for handling concurrent updates
- **Tombstone Support**: Proper handling of deleted entities across distributed instances

## Architecture

```
spec-ai-graph-sync
├── engine.rs      # Main sync engine implementation
├── persistence.rs # Storage backend trait
├── protocol.rs    # Sync protocol messages and payloads
├── resolver.rs    # Conflict resolution strategies
└── types.rs       # Sync-specific data types
```

## Usage

Implement the `SyncPersistence` trait for your storage backend, then create a `SyncEngine` instance:

```rust
use spec_ai_graph_sync::{SyncEngine, SyncPersistence};

// Implement SyncPersistence for your storage
struct MyStorage { /* ... */ }
impl SyncPersistence for MyStorage { /* ... */ }

// Create the sync engine
let storage = MyStorage::new();
let engine = SyncEngine::new(storage, "instance-1".to_string());

// Perform sync operations
let payload = engine.sync_full("session-1", "default").await?;
```

## Key Types

- `SyncEngine` - Main synchronization engine
- `SyncPersistence` - Trait for storage backends
- `GraphSyncPayload` - Full sync data payload
- `ConflictResolver` - Conflict resolution interface
- `VectorClock` - Causal ordering primitive (re-exported from knowledge-graph)

## Dependencies

- `spec-ai-knowledge-graph` - Graph types and vector clock implementation
- `uuid` - Unique identifiers for sync sessions
- `chrono` - Timestamps for change tracking

## Usage

This is an internal crate primarily used by:
- `spec-ai-core` - For distributed knowledge graph synchronization

For end-user documentation, see the main [spec-ai README](../../README.md).

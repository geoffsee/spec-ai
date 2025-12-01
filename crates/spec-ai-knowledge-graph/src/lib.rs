pub mod graph_store;
pub mod types;
pub mod vector_clock;

pub use graph_store::{
    ChangelogEntry, GraphSyncConfig, KnowledgeGraphStore, SyncStateRecord, SyncedEdgeRecord,
    SyncedNodeRecord,
};
pub use types::{
    EdgeType, GraphEdge, GraphNode, GraphPath, GraphQuery, GraphQueryResult, GraphQueryReturnType,
    NodeType, TraversalDirection,
};
pub use vector_clock::{ClockOrder, VectorClock};

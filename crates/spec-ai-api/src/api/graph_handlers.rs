/// Graph API handlers for direct knowledge graph access
///
/// These endpoints expose the knowledge graph as a generic key-value store
/// with nodes and edges. Clients interpret the data in domain-specific ways.
use crate::api::handlers::AppState;
use crate::api::models::ErrorResponse;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use spec_ai_core::bootstrap_self::plugin::{BootstrapMode, PluginContext};
use spec_ai_core::bootstrap_self::plugins::universal_code::UniversalCodePlugin;
use spec_ai_core::bootstrap_self::plugin::BootstrapPlugin;
use spec_ai_knowledge_graph::{EdgeType, NodeType};
use std::convert::Infallible;
use std::path::PathBuf;
use std::time::Duration;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for listing nodes
#[derive(Debug, Deserialize)]
pub struct ListNodesQuery {
    /// Session ID to scope the query
    pub session_id: String,
    /// Optional node type filter
    pub node_type: Option<String>,
    /// Maximum number of nodes to return
    pub limit: Option<usize>,
}

/// Query parameters for listing edges
#[derive(Debug, Deserialize)]
pub struct ListEdgesQuery {
    /// Session ID to scope the query
    pub session_id: String,
    /// Optional source node ID filter
    pub source_id: Option<i64>,
    /// Optional target node ID filter
    pub target_id: Option<i64>,
}

/// Request to create a new node
#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    /// Session ID for the node
    pub session_id: String,
    /// Node type (entity, concept, fact, message, tool_result, event, goal)
    pub node_type: String,
    /// Human-readable label
    pub label: String,
    /// Arbitrary properties as JSON
    #[serde(default)]
    pub properties: JsonValue,
}

/// Request to update a node's properties
#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    /// New properties (replaces existing)
    pub properties: JsonValue,
}

/// Request to create a new edge
#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    /// Session ID for the edge
    pub session_id: String,
    /// Source node ID
    pub source_id: i64,
    /// Target node ID
    pub target_id: i64,
    /// Edge type
    pub edge_type: String,
    /// Optional predicate/relationship name
    pub predicate: Option<String>,
    /// Optional properties
    pub properties: Option<JsonValue>,
    /// Edge weight (0.0 to 1.0)
    #[serde(default = "default_weight")]
    pub weight: f32,
}

fn default_weight() -> f32 {
    1.0
}

/// Response containing a single node
#[derive(Debug, Serialize)]
pub struct NodeResponse {
    pub id: i64,
    pub session_id: String,
    pub node_type: String,
    pub label: String,
    pub properties: JsonValue,
    pub created_at: String,
    pub updated_at: String,
}

/// Response containing multiple nodes
#[derive(Debug, Serialize)]
pub struct NodesListResponse {
    pub nodes: Vec<NodeResponse>,
    pub count: usize,
}

/// Response containing a single edge
#[derive(Debug, Serialize)]
pub struct EdgeResponse {
    pub id: i64,
    pub session_id: String,
    pub source_id: i64,
    pub target_id: i64,
    pub edge_type: String,
    pub predicate: Option<String>,
    pub properties: Option<JsonValue>,
    pub weight: f32,
    pub created_at: String,
}

/// Response containing multiple edges
#[derive(Debug, Serialize)]
pub struct EdgesListResponse {
    pub edges: Vec<EdgeResponse>,
    pub count: usize,
}

/// Query parameters for changelog stream
#[derive(Debug, Deserialize)]
pub struct ChangelogStreamQuery {
    /// Session ID to watch
    pub session_id: String,
    /// Optional: only return changes after this timestamp (ISO 8601)
    pub since: Option<String>,
}

/// A changelog event sent via SSE
#[derive(Debug, Serialize)]
pub struct ChangelogEvent {
    pub entity_type: String,
    pub entity_id: i64,
    pub operation: String,
    pub timestamp: String,
    pub data: Option<JsonValue>,
}

// ============================================================================
// Node Handlers
// ============================================================================

/// List nodes with optional filtering
pub async fn list_nodes(
    State(state): State<AppState>,
    Query(query): Query<ListNodesQuery>,
) -> Response {
    let node_type = query.node_type.map(|s| NodeType::from_str(&s));
    let limit = query.limit.map(|l| l as i64);

    match state
        .persistence
        .list_graph_nodes(&query.session_id, node_type, limit)
    {
        Ok(nodes) => {
            let response_nodes: Vec<NodeResponse> = nodes
                .into_iter()
                .map(|n| NodeResponse {
                    id: n.id,
                    session_id: n.session_id,
                    node_type: n.node_type.as_str().to_string(),
                    label: n.label,
                    properties: n.properties,
                    created_at: n.created_at.to_rfc3339(),
                    updated_at: n.updated_at.to_rfc3339(),
                })
                .collect();

            let count = response_nodes.len();
            Json(NodesListResponse {
                nodes: response_nodes,
                count,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Get a single node by ID
pub async fn get_node(State(state): State<AppState>, Path(node_id): Path<i64>) -> Response {
    match state.persistence.get_graph_node(node_id) {
        Ok(Some(node)) => Json(NodeResponse {
            id: node.id,
            session_id: node.session_id,
            node_type: node.node_type.as_str().to_string(),
            label: node.label,
            properties: node.properties,
            created_at: node.created_at.to_rfc3339(),
            updated_at: node.updated_at.to_rfc3339(),
        })
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Node not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Create a new node
pub async fn create_node(
    State(state): State<AppState>,
    Json(request): Json<CreateNodeRequest>,
) -> Response {
    let node_type = NodeType::from_str(&request.node_type);

    match state.persistence.insert_graph_node(
        &request.session_id,
        node_type,
        &request.label,
        &request.properties,
        None,
    ) {
        Ok(node_id) => {
            // Fetch the created node to return it
            match state.persistence.get_graph_node(node_id) {
                Ok(Some(node)) => (
                    StatusCode::CREATED,
                    Json(NodeResponse {
                        id: node.id,
                        session_id: node.session_id,
                        node_type: node.node_type.as_str().to_string(),
                        label: node.label,
                        properties: node.properties,
                        created_at: node.created_at.to_rfc3339(),
                        updated_at: node.updated_at.to_rfc3339(),
                    }),
                )
                    .into_response(),
                _ => (
                    StatusCode::CREATED,
                    Json(serde_json::json!({ "id": node_id })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Update a node's properties
pub async fn update_node(
    State(state): State<AppState>,
    Path(node_id): Path<i64>,
    Json(request): Json<UpdateNodeRequest>,
) -> Response {
    // First check if node exists
    match state.persistence.get_graph_node(node_id) {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("not_found", "Node not found")),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("database_error", e.to_string())),
            )
                .into_response()
        }
        Ok(Some(_)) => {}
    }

    match state
        .persistence
        .update_graph_node(node_id, &request.properties)
    {
        Ok(()) => {
            // Fetch updated node
            match state.persistence.get_graph_node(node_id) {
                Ok(Some(node)) => Json(NodeResponse {
                    id: node.id,
                    session_id: node.session_id,
                    node_type: node.node_type.as_str().to_string(),
                    label: node.label,
                    properties: node.properties,
                    created_at: node.created_at.to_rfc3339(),
                    updated_at: node.updated_at.to_rfc3339(),
                })
                .into_response(),
                _ => StatusCode::NO_CONTENT.into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Delete a node
pub async fn delete_node(State(state): State<AppState>, Path(node_id): Path<i64>) -> Response {
    match state.persistence.delete_graph_node(node_id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Edge Handlers
// ============================================================================

/// List edges with optional filtering
pub async fn list_edges(
    State(state): State<AppState>,
    Query(query): Query<ListEdgesQuery>,
) -> Response {
    match state.persistence.list_graph_edges(
        &query.session_id,
        query.source_id,
        query.target_id,
    ) {
        Ok(edges) => {
            let response_edges: Vec<EdgeResponse> = edges
                .into_iter()
                .map(|e| EdgeResponse {
                    id: e.id,
                    session_id: e.session_id,
                    source_id: e.source_id,
                    target_id: e.target_id,
                    edge_type: e.edge_type.as_str(),
                    predicate: e.predicate,
                    properties: e.properties,
                    weight: e.weight,
                    created_at: e.created_at.to_rfc3339(),
                })
                .collect();

            let count = response_edges.len();
            Json(EdgesListResponse {
                edges: response_edges,
                count,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Get a single edge by ID
pub async fn get_edge(State(state): State<AppState>, Path(edge_id): Path<i64>) -> Response {
    match state.persistence.get_graph_edge(edge_id) {
        Ok(Some(edge)) => Json(EdgeResponse {
            id: edge.id,
            session_id: edge.session_id,
            source_id: edge.source_id,
            target_id: edge.target_id,
            edge_type: edge.edge_type.as_str(),
            predicate: edge.predicate,
            properties: edge.properties,
            weight: edge.weight,
            created_at: edge.created_at.to_rfc3339(),
        })
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Edge not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Create a new edge
pub async fn create_edge(
    State(state): State<AppState>,
    Json(request): Json<CreateEdgeRequest>,
) -> Response {
    let edge_type = EdgeType::from_str(&request.edge_type);

    match state.persistence.insert_graph_edge(
        &request.session_id,
        request.source_id,
        request.target_id,
        edge_type,
        request.predicate.as_deref(),
        request.properties.as_ref(),
        request.weight,
    ) {
        Ok(edge_id) => {
            // Fetch the created edge to return it
            match state.persistence.get_graph_edge(edge_id) {
                Ok(Some(edge)) => (
                    StatusCode::CREATED,
                    Json(EdgeResponse {
                        id: edge.id,
                        session_id: edge.session_id,
                        source_id: edge.source_id,
                        target_id: edge.target_id,
                        edge_type: edge.edge_type.as_str(),
                        predicate: edge.predicate,
                        properties: edge.properties,
                        weight: edge.weight,
                        created_at: edge.created_at.to_rfc3339(),
                    }),
                )
                    .into_response(),
                _ => (
                    StatusCode::CREATED,
                    Json(serde_json::json!({ "id": edge_id })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Delete an edge
pub async fn delete_edge(State(state): State<AppState>, Path(edge_id): Path<i64>) -> Response {
    match state.persistence.delete_graph_edge(edge_id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("database_error", e.to_string())),
        )
            .into_response(),
    }
}

// ============================================================================
// Changelog Stream (SSE)
// ============================================================================

/// Stream changelog events via Server-Sent Events
pub async fn stream_changelog(
    State(state): State<AppState>,
    Query(query): Query<ChangelogStreamQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = query.session_id;
    let since = query.since.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let stream = async_stream::stream! {
        let mut last_timestamp = since;
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            // Poll for new changelog entries
            match state.persistence.graph_changelog_get_since(&session_id, &last_timestamp) {
                Ok(entries) => {
                    for entry in entries {
                        let timestamp_str = entry.created_at.to_rfc3339();
                        let event = ChangelogEvent {
                            entity_type: entry.entity_type.clone(),
                            entity_id: entry.entity_id,
                            operation: entry.operation.clone(),
                            timestamp: timestamp_str.clone(),
                            data: entry.data.and_then(|s| serde_json::from_str(&s).ok()),
                        };

                        // Update last timestamp for next poll
                        last_timestamp = timestamp_str;

                        if let Ok(json) = serde_json::to_string(&event) {
                            yield Ok(Event::default().data(json));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Changelog poll error: {}", e);
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    )
}

// ============================================================================
// Bootstrap Handler
// ============================================================================

/// Request to bootstrap a knowledge graph from a directory
#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    /// Session ID for the graph (optional, defaults to "visionos-dashboard")
    pub session_id: Option<String>,
}

/// Response from bootstrap operation
#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    pub session_id: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub root_node_id: Option<i64>,
}

/// Bootstrap a knowledge graph from the server's current working directory
pub async fn bootstrap_graph(
    State(state): State<AppState>,
    Json(request): Json<BootstrapRequest>,
) -> Response {
    let session_id = request.session_id.unwrap_or_else(|| "visionos-dashboard".to_string());

    // Get current working directory
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("cwd_error", format!("Failed to get current directory: {}", e))),
            )
                .into_response()
        }
    };

    tracing::info!("Bootstrapping knowledge graph from: {:?}", cwd);

    // Create plugin context
    let context = PluginContext {
        persistence: &state.persistence,
        session_id: &session_id,
        repo_root: &cwd,
        mode: BootstrapMode::Fresh,
    };

    // Run the universal code plugin
    let plugin = UniversalCodePlugin;

    if !plugin.should_activate(&cwd) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "not_a_repository",
                "Current directory does not appear to be a code repository",
            )),
        )
            .into_response();
    }

    match plugin.run(context) {
        Ok(outcome) => {
            tracing::info!(
                "Bootstrap complete: {} nodes, {} edges created",
                outcome.nodes_created,
                outcome.edges_created
            );
            (
                StatusCode::CREATED,
                Json(BootstrapResponse {
                    session_id,
                    nodes_created: outcome.nodes_created,
                    edges_created: outcome.edges_created,
                    root_node_id: outcome.root_node_id,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Bootstrap failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("bootstrap_error", e.to_string())),
            )
                .into_response()
        }
    }
}

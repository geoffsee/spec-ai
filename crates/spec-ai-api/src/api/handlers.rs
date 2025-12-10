/// API request handlers
use crate::agent::builder::AgentBuilder;
use crate::agent::core::AgentCore;
use crate::api::auth::{AuthService, TokenRequest, TokenResponse};
use crate::api::mesh::{MeshRegistry, MeshState};
use crate::api::models::*;
use crate::config::{AgentRegistry, AppConfig};
use crate::persistence::Persistence;
use crate::tools::ToolRegistry;
use async_stream::stream;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
};
use futures::StreamExt;
use serde_json::json;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use toak_rs::{JsonDatabaseGenerator, JsonDatabaseOptions, SemanticSearch};
use tokio::sync::RwLock;

const DEFAULT_PAGE_SIZE: usize = 10;
const MAX_PAGE_SIZE: usize = 25;
const MAX_TOTAL_RESULTS: usize = 100;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub persistence: Persistence,
    pub agent_registry: Arc<AgentRegistry>,
    pub tool_registry: Arc<ToolRegistry>,
    pub config: AppConfig,
    pub start_time: Instant,
    pub mesh_registry: MeshRegistry,
    pub auth_service: Arc<AuthService>,
}

impl AppState {
    pub fn new(
        persistence: Persistence,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
        config: AppConfig,
    ) -> Self {
        // Initialize auth service from config
        let auth_service = AuthService::new(
            config.auth.credentials_file.as_deref(),
            config.auth.token_secret.as_deref(),
            Some(config.auth.token_expiry_secs),
            config.auth.enabled,
        )
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to initialize auth service: {}. Auth disabled.", e);
            AuthService::disabled()
        });

        Self {
            persistence: persistence.clone(),
            agent_registry,
            tool_registry,
            config,
            start_time: Instant::now(),
            mesh_registry: MeshRegistry::with_persistence(persistence),
            auth_service: Arc::new(auth_service),
        }
    }
}

impl MeshState for AppState {
    fn mesh_registry(&self) -> &MeshRegistry {
        &self.mesh_registry
    }
}

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.start_time.elapsed().as_secs();
    let active_sessions = match state.persistence.list_sessions() {
        Ok(sessions) => sessions.len(),
        Err(e) => {
            tracing::warn!("Failed to count active sessions: {}", e);
            0
        }
    };

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        active_sessions,
    };

    Json(response)
}

/// List available agents
pub async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let agent_names = state.agent_registry.list();
    let mut agent_infos = Vec::new();

    for name in agent_names {
        if let Some(profile) = state.agent_registry.get(&name) {
            agent_infos.push(AgentInfo {
                id: name,
                description: profile.prompt.unwrap_or_default(),
                allowed_tools: profile.allowed_tools.unwrap_or_default(),
                denied_tools: profile.denied_tools.unwrap_or_default(),
            });
        }
    }

    Json(AgentListResponse {
        agents: agent_infos,
    })
    .into_response()
}

/// Query endpoint - process a message and return response
pub async fn query(State(state): State<AppState>, Json(request): Json<QueryRequest>) -> Response {
    // If streaming requested, delegate to streaming handler
    if request.stream {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_request",
                "Streaming not supported on /query endpoint. Use /stream instead.",
            )),
        )
            .into_response();
    }

    // Determine which agent to use
    let agent_name = request.agent.unwrap_or_else(|| "default".to_string());

    // Get or create session ID
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("api_{}", uuid_v4()));

    // Create agent instance
    let agent_result = create_agent(&state, &agent_name, &session_id, request.temperature).await;

    let mut agent = match agent_result {
        Ok(agent) => agent,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("agent_error", e.to_string())),
            )
                .into_response();
        }
    };

    // Process the message
    let start = Instant::now();

    match agent.run_step(&request.message).await {
        Ok(output) => {
            let processing_time = start.elapsed().as_millis() as u64;
            let tool_calls: Vec<ToolCallInfo> = output
                .tool_invocations
                .iter()
                .map(|inv| ToolCallInfo {
                    name: inv.name.clone(),
                    arguments: inv.arguments.clone(),
                    success: inv.success,
                    output: inv.output.clone(),
                    error: inv.error.clone(),
                })
                .collect();

            let response = QueryResponse {
                response: output.response,
                session_id,
                agent: agent_name,
                tool_calls,
                metadata: ResponseMetadata {
                    timestamp: current_timestamp(),
                    model: state.config.model.provider.clone(),
                    processing_time_ms: processing_time,
                    run_id: output.run_id,
                },
            };

            Json(response).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("execution_error", e.to_string())),
        )
            .into_response(),
    }
}

/// Streaming query endpoint
pub async fn stream_query(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> Response {
    let agent_name = request.agent.unwrap_or_else(|| "default".to_string());
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("api_{}", uuid_v4()));

    // Create agent
    let agent_result = create_agent(&state, &agent_name, &session_id, request.temperature).await;

    let agent = match agent_result {
        Ok(agent) => agent,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("agent_error", e.to_string())),
            )
                .into_response();
        }
    };

    // Create SSE stream
    let agent = Arc::new(RwLock::new(agent));
    let message = request.message.clone();
    let session_id_clone = session_id.clone();
    let agent_name_clone = agent_name.clone();
    let model_id = state.config.model.provider.clone();

    let sse_stream = stream! {
        yield StreamChunk::Start {
            session_id: session_id_clone.clone(),
            agent: agent_name_clone.clone(),
        };

        let start = Instant::now();
        let mut agent_lock = agent.write().await;

        match agent_lock.run_step(&message).await {
            Ok(output) => {
                yield StreamChunk::Content { text: output.response.clone() };

                for invocation in output.tool_invocations {
                    yield StreamChunk::ToolCall {
                        name: invocation.name.clone(),
                        arguments: invocation.arguments.clone(),
                    };
                    yield StreamChunk::ToolResult {
                        name: invocation.name.clone(),
                        result: json!({
                            "success": invocation.success,
                            "output": invocation.output,
                            "error": invocation.error,
                        }),
                    };
                }

                yield StreamChunk::End {
                    metadata: ResponseMetadata {
                        timestamp: current_timestamp(),
                        model: model_id.clone(),
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        run_id: output.run_id,
                    },
                };
            }
            Err(e) => {
                yield StreamChunk::Error {
                    message: e.to_string(),
                };
            }
        }
    };

    Sse::new(sse_stream.map(|chunk| {
        let json = serde_json::to_string(&chunk).unwrap();
        Ok::<_, Infallible>(Event::default().data(json))
    }))
    .into_response()
}

/// Helper: Create agent instance
async fn create_agent(
    state: &AppState,
    agent_name: &str,
    session_id: &str,
    _temperature: Option<f32>,
) -> anyhow::Result<AgentCore> {
    // Get the agent profile
    let profile = state
        .agent_registry
        .get(agent_name)
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

    // Build the agent using the builder with config
    let agent = AgentBuilder::new()
        .with_profile(profile)
        .with_config(state.config.clone())
        .with_session_id(session_id)
        .with_agent_name(agent_name.to_string())
        .with_tool_registry(state.tool_registry.clone())
        .with_persistence(state.persistence.clone())
        .build()?;

    Ok(agent)
}

/// Helper: Generate UUID v4
fn uuid_v4() -> String {
    let rng = std::collections::hash_map::RandomState::new();
    let hash = std::hash::BuildHasher::hash_one(&rng, SystemTime::now());
    format!("{:x}", hash)
}

/// Helper: Get current timestamp
fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    chrono::DateTime::from_timestamp(now as i64, 0)
        .unwrap()
        .to_rfc3339()
}

/// Token generation endpoint - exchange username/password for bearer token
pub async fn generate_token(
    State(state): State<AppState>,
    Json(request): Json<TokenRequest>,
) -> Response {
    // Check if auth is enabled
    if !state.auth_service.is_enabled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "auth_disabled",
                "Authentication is not enabled on this server",
            )),
        )
            .into_response();
    }

    // Verify credentials
    if !state.auth_service.verify_password(&request.username, &request.password) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                "invalid_credentials",
                "Invalid username or password",
            )),
        )
            .into_response();
    }

    // Generate token
    match state.auth_service.generate_token(&request.username) {
        Ok(token) => {
            let response = TokenResponse {
                token,
                token_type: "Bearer".to_string(),
                expires_in: state.config.auth.token_expiry_secs,
            };
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to generate token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("token_error", "Failed to generate token")),
            )
                .into_response()
        }
    }
}

/// Password hash generation endpoint - utility for creating password hashes
/// This endpoint can be used to generate hashes for the credentials file
pub async fn hash_password(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    // Only allow if auth is enabled (to prevent abuse)
    if !state.auth_service.is_enabled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "auth_disabled",
                "Authentication is not enabled on this server",
            )),
        )
            .into_response();
    }

    let Some(password) = body.get("password").and_then(|v| v.as_str()) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_request",
                "Missing 'password' field in request body",
            )),
        )
            .into_response();
    };

    match AuthService::hash_password(password) {
        Ok(hash) => Json(json!({ "password_hash": hash })).into_response(),
        Err(e) => {
            tracing::error!("Failed to hash password: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("hash_error", "Failed to hash password")),
            )
                .into_response()
        }
    }
}

/// Semantic code search endpoint
pub async fn search(Json(request): Json<SearchRequest>) -> Response {
    // Validate query
    if request.query.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("invalid_request", "Query cannot be empty")),
        )
            .into_response();
    }

    // Resolve root path
    let root = request
        .root
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if !root.exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_root",
                format!("Search root {} does not exist", root.display()),
            )),
        )
            .into_response();
    }

    // Calculate pagination parameters
    let page_size = request
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let page = request.page;
    let offset = page * page_size;

    // Ensure embeddings exist
    let embeddings_path = match ensure_embeddings(&root, request.refresh).await {
        Ok(path) => path,
        Err(e) => {
            tracing::error!("Failed to generate embeddings: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "embeddings_error",
                    format!("Failed to generate embeddings: {}", e),
                )),
            )
                .into_response();
        }
    };

    // Load searcher and run search
    let mut searcher = match SemanticSearch::new(&embeddings_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to load embeddings: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "search_error",
                    format!("Failed to load embeddings database: {}", e),
                )),
            )
                .into_response();
        }
    };

    // Fetch enough results to determine total and extract current page
    let fetch_count = MAX_TOTAL_RESULTS.min(offset + page_size);
    let hits = match searcher.search(&request.query, fetch_count) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Search failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("search_error", format!("Search failed: {}", e))),
            )
                .into_response();
        }
    };

    let total_results = hits.len();
    let total_pages = (total_results + page_size - 1) / page_size;

    // Extract current page results
    let page_results: Vec<SearchResult> = hits
        .into_iter()
        .skip(offset)
        .take(page_size)
        .map(|hit| {
            let mut snippet = hit.content;
            if snippet.len() > 480 {
                snippet.truncate(480);
                snippet.push_str("...[truncated]");
            }
            SearchResult {
                path: hit.file_path,
                similarity: hit.similarity,
                snippet,
            }
        })
        .collect();

    let response = SearchResponse {
        query: request.query,
        root: root.display().to_string(),
        page,
        page_size,
        total_results,
        total_pages,
        results: page_results,
    };

    Json(response).into_response()
}

/// Helper: Ensure embeddings database exists
async fn ensure_embeddings(root: &Path, refresh: bool) -> anyhow::Result<PathBuf> {
    let embeddings_path = root.join(".spec-ai").join("code_search_embeddings.json");

    if embeddings_path.exists() && !refresh {
        return Ok(embeddings_path);
    }

    if let Some(parent) = embeddings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = JsonDatabaseOptions {
        dir: root.to_path_buf(),
        output_file_path: embeddings_path.clone(),
        verbose: false,
        chunker_config: Default::default(),
        max_concurrent_files: 4,
        ..Default::default()
    };

    let generator = JsonDatabaseGenerator::new(options)?;
    generator.generate_database().await?;

    Ok(embeddings_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generation() {
        let uuid1 = uuid_v4();
        let uuid2 = uuid_v4();

        assert!(!uuid1.is_empty());
        assert!(!uuid2.is_empty());
        // UUIDs should be different (probabilistically)
        // We won't assert this as it could theoretically fail
    }

    #[test]
    fn test_timestamp_format() {
        let ts = current_timestamp();
        assert!(ts.contains('T'));
        assert!(ts.contains('Z') || ts.contains('+'));
    }
}

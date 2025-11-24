pub mod handlers;
pub mod mesh;
pub mod middleware;
pub mod models;
/// REST API and WebSocket server for programmatic agent access
///
/// This module provides:
/// - REST endpoints for agent interaction
/// - WebSocket streaming for real-time responses
/// - API key authentication
/// - JSON request/response format
pub mod server;
pub mod sync_handlers;
pub use spec_ai_core::sync;

pub use models::{ErrorResponse, QueryRequest, QueryResponse, StreamChunk};
pub use server::{ApiConfig, ApiServer};

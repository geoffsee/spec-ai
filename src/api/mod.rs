/// REST API and WebSocket server for programmatic agent access
///
/// This module provides:
/// - REST endpoints for agent interaction
/// - WebSocket streaming for real-time responses
/// - API key authentication
/// - JSON request/response format

#[cfg(feature = "api")]
pub mod server;
#[cfg(feature = "api")]
pub mod handlers;
#[cfg(feature = "api")]
pub mod middleware;
#[cfg(feature = "api")]
pub mod models;

#[cfg(feature = "api")]
pub use server::{ApiServer, ApiConfig};
#[cfg(feature = "api")]
pub use models::{QueryRequest, QueryResponse, StreamChunk, ErrorResponse};

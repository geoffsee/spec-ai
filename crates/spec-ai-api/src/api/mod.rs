pub mod auth;
pub mod graph_handlers;
pub mod handlers;
pub mod mesh;
pub mod middleware;
pub mod models;
/// REST API and WebSocket server for programmatic agent access
///
/// This module provides:
/// - REST endpoints for agent interaction
/// - WebSocket streaming for real-time responses
/// - Bearer token authentication with optional user credentials
/// - Mandatory TLS with self-signed certificates
/// - JSON request/response format
pub mod server;
pub mod sync_handlers;
pub mod tls;
pub use spec_ai_core::sync;

pub use auth::{AuthService, TokenRequest, TokenResponse};
pub use models::{ErrorResponse, QueryRequest, QueryResponse, StreamChunk};
pub use server::{ApiConfig, ApiServer};
pub use tls::{CertificateInfo, TlsConfig};

/// HTTP server implementation with mandatory TLS
use crate::api::graph_handlers::{
    bootstrap_graph, create_edge, create_node, delete_edge, delete_node, get_edge, get_node,
    list_edges, list_nodes, stream_changelog, update_node,
};
use crate::api::handlers::{
    generate_token, hash_password, health_check, list_agents, query, search, stream_query,
    AppState,
};
use crate::api::mesh::{
    acknowledge_messages, deregister_instance, get_messages, heartbeat, list_instances,
    register_instance, send_message, MeshClient,
};
use crate::api::middleware::auth_middleware;
use crate::api::sync_handlers::{
    bulk_toggle_sync, configure_sync, get_sync_status, handle_sync_apply, handle_sync_request,
    list_conflicts, list_sync_configs, toggle_sync,
};
use crate::api::tls::TlsConfig;
use crate::config::{AgentRegistry, AppConfig};
use crate::persistence::Persistence;
use crate::sync::{start_sync_coordinator, SyncCoordinatorConfig};
use crate::tools::ToolRegistry;
use anyhow::{Context, Result};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Install the rustls crypto provider (call once at startup)
fn install_crypto_provider() {
    // Install aws-lc-rs as the default crypto provider for rustls
    // This is required because rustls 0.23+ doesn't auto-select a provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

/// API server configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Optional API key for authentication (legacy, prefer token auth)
    pub api_key: Option<String>,
    /// Enable CORS
    pub enable_cors: bool,
    /// Path to TLS certificate file (PEM format)
    /// If not provided, a self-signed certificate is generated
    pub tls_cert_path: Option<PathBuf>,
    /// Path to TLS private key file (PEM format)
    pub tls_key_path: Option<PathBuf>,
    /// Additional Subject Alternative Names for generated certificate
    pub tls_san: Vec<String>,
    /// Certificate validity in days (for generated certs)
    pub tls_validity_days: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            api_key: None,
            enable_cors: true,
            tls_cert_path: None,
            tls_key_path: None,
            tls_san: Vec::new(),
            tls_validity_days: 365,
        }
    }
}

impl ApiConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_cors(mut self, enable: bool) -> Self {
        self.enable_cors = enable;
        self
    }

    pub fn with_tls_cert(mut self, cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        self.tls_cert_path = Some(cert_path.into());
        self.tls_key_path = Some(key_path.into());
        self
    }

    pub fn with_tls_san(mut self, san: Vec<String>) -> Self {
        self.tls_san = san;
        self
    }

    pub fn with_tls_validity(mut self, days: u32) -> Self {
        self.tls_validity_days = days;
        self
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// API server with mandatory TLS
pub struct ApiServer {
    config: ApiConfig,
    state: AppState,
    tls_config: TlsConfig,
}

impl ApiServer {
    /// Create a new API server with TLS
    ///
    /// If no certificate is provided in config, a self-signed certificate is generated.
    pub fn new(
        config: ApiConfig,
        persistence: Persistence,
        agent_registry: Arc<AgentRegistry>,
        tool_registry: Arc<ToolRegistry>,
        app_config: AppConfig,
    ) -> Result<Self> {
        // Install crypto provider for rustls (idempotent, safe to call multiple times)
        install_crypto_provider();

        let state = AppState::new(persistence, agent_registry, tool_registry, app_config);

        // Initialize TLS - either load from files or generate self-signed
        let tls_config = if let (Some(cert_path), Some(key_path)) =
            (&config.tls_cert_path, &config.tls_key_path)
        {
            TlsConfig::load_from_files(cert_path, key_path)
                .context("Failed to load TLS certificate")?
        } else {
            let tls = TlsConfig::generate(
                &config.host,
                &config.tls_san,
                Some(config.tls_validity_days),
            )
            .context("Failed to generate TLS certificate")?;

            // Save generated cert for potential reuse
            let cert_dir = dirs_next::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".spec-ai")
                .join("tls");
            let cert_path = cert_dir.join("server.crt");
            let key_path = cert_dir.join("server.key");

            if let Err(e) = tls.save_to_files(&cert_path, &key_path) {
                tracing::warn!("Could not save generated TLS certificate: {}", e);
            } else {
                tracing::info!(
                    "Saved TLS certificate to {} (fingerprint: {})",
                    cert_path.display(),
                    tls.fingerprint
                );
            }

            tls
        };

        tracing::info!(
            "TLS initialized with certificate fingerprint: {}",
            tls_config.fingerprint
        );

        Ok(Self {
            config,
            state,
            tls_config,
        })
    }

    /// Get the mesh registry for self-registration
    pub fn mesh_registry(&self) -> &crate::api::mesh::MeshRegistry {
        &self.state.mesh_registry
    }

    /// Get the TLS configuration (for certificate info)
    pub fn tls_config(&self) -> &TlsConfig {
        &self.tls_config
    }

    /// Get the certificate fingerprint
    pub fn certificate_fingerprint(&self) -> &str {
        &self.tls_config.fingerprint
    }

    /// Build the router with all routes
    fn build_router(&self) -> Router {
        // Create certificate info for the endpoint
        let cert_info = self.tls_config.get_certificate_info(&self.config.host);

        // Public routes that don't require authentication
        let public_routes = Router::new()
            // Health endpoint is always public
            .route("/health", get(health_check))
            // Certificate info endpoint - clients can use this to get/verify the fingerprint
            .route(
                "/cert",
                get(move || async move { Json(cert_info.clone()) }),
            )
            // Auth endpoints are public (needed to get tokens)
            .route("/auth/token", post(generate_token))
            .route("/auth/hash", post(hash_password));

        // Protected routes that require authentication when enabled
        let protected_routes = Router::new()
            // Info endpoints
            .route("/agents", get(list_agents))
            // Query endpoints
            .route("/query", post(query))
            .route("/stream", post(stream_query))
            // Search endpoint
            .route("/api/search", post(search))
            // Mesh registry endpoints
            .route("/registry/register", post(register_instance::<AppState>))
            .route("/registry/agents", get(list_instances::<AppState>))
            .route(
                "/registry/heartbeat/{instance_id}",
                post(heartbeat::<AppState>),
            )
            .route(
                "/registry/deregister/{instance_id}",
                delete(deregister_instance::<AppState>),
            )
            // Message routing endpoints
            .route(
                "/messages/send/{source_instance}",
                post(send_message::<AppState>),
            )
            .route("/messages/{instance_id}", get(get_messages::<AppState>))
            .route(
                "/messages/ack/{instance_id}",
                post(acknowledge_messages::<AppState>),
            )
            // Graph sync endpoints
            .route("/sync/request", post(handle_sync_request))
            .route("/sync/apply", post(handle_sync_apply))
            .route("/sync/status/{session_id}/{graph_name}", get(get_sync_status))
            .route("/sync/enable/{session_id}/{graph_name}", post(toggle_sync))
            .route("/sync/configs/{session_id}", get(list_sync_configs))
            .route("/sync/bulk/{session_id}", post(bulk_toggle_sync))
            .route(
                "/sync/configure/{session_id}/{graph_name}",
                post(configure_sync),
            )
            .route("/sync/conflicts", get(list_conflicts))
            // Graph CRUD endpoints
            .route("/graph/nodes", get(list_nodes))
            .route("/graph/nodes", post(create_node))
            .route("/graph/nodes/{node_id}", get(get_node))
            .route("/graph/nodes/{node_id}", put(update_node))
            .route("/graph/nodes/{node_id}", delete(delete_node))
            .route("/graph/edges", get(list_edges))
            .route("/graph/edges", post(create_edge))
            .route("/graph/edges/{edge_id}", get(get_edge))
            .route("/graph/edges/{edge_id}", delete(delete_edge))
            .route("/graph/stream", get(stream_changelog))
            // Bootstrap endpoint
            .route("/bootstrap", post(bootstrap_graph))
            // Apply auth middleware to protected routes
            .layer(middleware::from_fn_with_state(
                self.state.auth_service.clone(),
                auth_middleware,
            ));

        // Merge public and protected routes
        let mut router = Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .with_state(self.state.clone());

        // Add CORS if enabled
        if self.config.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            router = router.layer(cors);
        }

        // Add tracing
        router = router.layer(TraceLayer::new_for_http());

        router
    }

    /// Run the server with TLS
    pub async fn run(self) -> Result<()> {
        // Start sync coordinator if sync is enabled
        if self.state.config.sync.enabled {
            self.start_sync_coordinator_background();
        }

        let app = self.build_router();
        let bind_addr: SocketAddr = self.config.bind_address().parse()
            .context("Invalid bind address")?;

        // Build rustls config
        let rustls_config = RustlsConfig::from_der(
            vec![self.tls_config.certificate.clone()],
            self.tls_config.private_key.clone(),
        )
        .await
        .context("Failed to create TLS config")?;

        tracing::info!(
            "Starting HTTPS server on {} (fingerprint: {})",
            bind_addr,
            self.tls_config.fingerprint
        );

        axum_server::bind_rustls(bind_addr, rustls_config)
            .serve(app.into_make_service())
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }

    /// Start the sync coordinator as a background task
    fn start_sync_coordinator_background(&self) {
        let persistence = Arc::new(self.state.persistence.clone());
        let mesh_registry = Arc::new(self.state.mesh_registry.clone());
        let mesh_client = Arc::new(MeshClient::new("localhost", self.config.port));
        let sync_config = SyncCoordinatorConfig::from(&self.state.config.sync);

        // Apply configured namespaces
        for ns in &self.state.config.sync.namespaces {
            if let Err(e) =
                self.state
                    .persistence
                    .graph_set_sync_enabled(&ns.session_id, &ns.graph_name, true)
            {
                tracing::warn!(
                    "Failed to enable sync for {}/{}: {}",
                    ns.session_id,
                    ns.graph_name,
                    e
                );
            }
        }

        // Spawn the sync coordinator
        tokio::spawn(async move {
            let _handle =
                start_sync_coordinator(persistence, mesh_registry, mesh_client, sync_config).await;
            // The coordinator runs indefinitely
        });

        tracing::info!(
            "Started sync coordinator with {} configured namespaces",
            self.state.config.sync.namespaces.len()
        );
    }

    /// Run the server with TLS and graceful shutdown
    pub async fn run_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<()> {
        // Start sync coordinator if sync is enabled
        if self.state.config.sync.enabled {
            self.start_sync_coordinator_background();
        }

        let app = self.build_router();
        let bind_addr: SocketAddr = self.config.bind_address().parse()
            .context("Invalid bind address")?;

        // Build rustls config
        let rustls_config = RustlsConfig::from_der(
            vec![self.tls_config.certificate.clone()],
            self.tls_config.private_key.clone(),
        )
        .await
        .context("Failed to create TLS config")?;

        tracing::info!(
            "Starting HTTPS server on {} (fingerprint: {})",
            bind_addr,
            self.tls_config.fingerprint
        );

        // Create handle for graceful shutdown
        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();

        // Spawn shutdown listener
        tokio::spawn(async move {
            shutdown_signal.await;
            handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        });

        axum_server::bind_rustls(bind_addr, rustls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert!(config.api_key.is_none());
        assert!(config.enable_cors);
    }

    #[test]
    fn test_api_config_builder() {
        let config = ApiConfig::new()
            .with_host("0.0.0.0")
            .with_port(8080)
            .with_api_key("secret123")
            .with_cors(false);

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.api_key, Some("secret123".to_string()));
        assert!(!config.enable_cors);
    }

    #[test]
    fn test_bind_address() {
        let config = ApiConfig::new().with_host("localhost").with_port(5000);

        assert_eq!(config.bind_address(), "localhost:5000");
    }
}

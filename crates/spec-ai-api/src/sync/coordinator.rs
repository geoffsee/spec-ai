/// Background sync coordinator for automatic graph synchronization
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, warn};

use crate::api::mesh::{MeshClient, MeshRegistry};
use crate::persistence::Persistence;
use spec_ai_core::sync::{GraphSyncPayload, SyncEngine, SyncPersistenceAdapter};

/// Configuration for the sync coordinator
#[derive(Debug, Clone)]
pub struct SyncCoordinatorConfig {
    /// How often to check for sync opportunities (in seconds)
    pub sync_interval_secs: u64,
    /// Maximum number of concurrent sync operations
    pub max_concurrent_syncs: usize,
    /// Retry interval for failed syncs (in seconds)
    pub retry_interval_secs: u64,
    /// Maximum number of retry attempts
    pub max_retries: usize,
}

impl Default for SyncCoordinatorConfig {
    fn default() -> Self {
        Self {
            sync_interval_secs: 60,   // Check every minute
            max_concurrent_syncs: 3,  // Up to 3 concurrent syncs
            retry_interval_secs: 300, // Retry after 5 minutes
            max_retries: 3,           // Max 3 retry attempts
        }
    }
}

impl From<&spec_ai_config::config::SyncConfig> for SyncCoordinatorConfig {
    fn from(config: &spec_ai_config::config::SyncConfig) -> Self {
        Self {
            sync_interval_secs: config.interval_secs,
            max_concurrent_syncs: config.max_concurrent_syncs,
            retry_interval_secs: config.retry_interval_secs,
            max_retries: config.max_retries,
        }
    }
}

/// Background sync coordinator
#[derive(Clone)]
pub struct SyncCoordinator {
    persistence: Arc<Persistence>,
    mesh_registry: Arc<MeshRegistry>,
    mesh_client: Arc<MeshClient>,
    config: SyncCoordinatorConfig,
    instance_id: String,
}

impl SyncCoordinator {
    /// Create a new sync coordinator
    pub fn new(
        persistence: Arc<Persistence>,
        mesh_registry: Arc<MeshRegistry>,
        mesh_client: Arc<MeshClient>,
        config: SyncCoordinatorConfig,
    ) -> Self {
        let instance_id = persistence.instance_id().to_string();
        Self {
            persistence,
            mesh_registry,
            mesh_client,
            config,
            instance_id,
        }
    }

    /// Start the background sync coordinator
    pub async fn start(self: Arc<Self>) {
        info!(
            "Starting sync coordinator with interval {} seconds",
            self.config.sync_interval_secs
        );

        let mut interval = time::interval(Duration::from_secs(self.config.sync_interval_secs));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            if let Err(e) = self.run_sync_cycle().await {
                error!("Sync cycle failed: {}", e);
            }
        }
    }

    /// Run a single sync cycle
    async fn run_sync_cycle(&self) -> Result<()> {
        debug!("Starting sync cycle");

        // Get all sessions with sync-enabled graphs
        let sessions = self.get_sync_enabled_sessions()?;

        if sessions.is_empty() {
            debug!("No sync-enabled graphs found");
            return Ok(());
        }

        // Get active peers from the mesh
        let peers = self.mesh_registry.list().await;

        if peers.is_empty() {
            debug!("No active peers found in mesh");
            return Ok(());
        }

        // Create a semaphore to limit concurrent syncs
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.max_concurrent_syncs,
        ));
        let mut sync_tasks = Vec::new();

        for (session_id, graph_name) in sessions {
            // Check if we should sync this graph
            if !self.should_sync(&session_id, &graph_name)? {
                continue;
            }

            // Find peers that might have this graph
            for peer in &peers {
                if peer.instance_id == self.instance_id {
                    continue; // Skip self
                }

                let permit = semaphore.clone().acquire_owned().await?;
                let self_clone = self.clone();
                let session_id = session_id.clone();
                let graph_name = graph_name.clone();
                let peer_id = peer.instance_id.clone();
                let peer_url = format!("http://{}:{}", peer.hostname, peer.port);

                // Spawn sync task
                let task = tokio::spawn(async move {
                    let _permit = permit; // Hold permit until task completes

                    match self_clone
                        .sync_with_peer(&session_id, &graph_name, &peer_id, &peer_url)
                        .await
                    {
                        Ok(_) => {
                            info!(
                                "Successfully synced {}/{} with peer {}",
                                session_id, graph_name, peer_id
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to sync {}/{} with peer {}: {}",
                                session_id, graph_name, peer_id, e
                            );
                        }
                    }
                });

                sync_tasks.push(task);
            }
        }

        // Wait for all sync tasks to complete
        for task in sync_tasks {
            let _ = task.await;
        }

        debug!("Sync cycle completed");
        Ok(())
    }

    /// Get all sessions with sync-enabled graphs
    fn get_sync_enabled_sessions(&self) -> Result<Vec<(String, String)>> {
        self.persistence.graph_list_sync_enabled()
    }

    /// Check if we should sync this graph now
    fn should_sync(&self, session_id: &str, graph_name: &str) -> Result<bool> {
        // Check if sync is enabled
        let sync_enabled = self
            .persistence
            .graph_get_sync_enabled(session_id, graph_name)?;
        if !sync_enabled {
            return Ok(false);
        }

        // Check if there are pending changes
        let since = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::seconds(
                self.config.sync_interval_secs as i64,
            ))
            .unwrap()
            .to_rfc3339();

        let changes = self
            .persistence
            .graph_changelog_get_since(session_id, &since)?;

        Ok(!changes.is_empty())
    }

    /// Sync with a specific peer
    async fn sync_with_peer(
        &self,
        session_id: &str,
        graph_name: &str,
        peer_id: &str,
        peer_url: &str,
    ) -> Result<()> {
        debug!(
            "Syncing {}/{} with peer {} at {}",
            session_id, graph_name, peer_id, peer_url
        );

        // Create sync engine using the adapter
        let adapter = SyncPersistenceAdapter::new((*self.persistence).clone());
        let sync_engine = SyncEngine::new(adapter, self.instance_id.clone());

        // Get our current vector clock
        let our_vc = self
            .persistence
            .graph_sync_state_get(&self.instance_id, session_id, graph_name)?
            .unwrap_or_else(|| "{}".to_string());

        // Send sync request to peer
        let sync_request = serde_json::json!({
            "session_id": session_id,
            "graph_name": graph_name,
            "requesting_instance": self.instance_id,
            "vector_clock": our_vc,
        });

        // Make HTTP request to peer's sync endpoint
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/sync/request", peer_url))
            .json(&sync_request)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Sync request failed: {}", error_text));
        }

        // Parse sync response
        let sync_response: serde_json::Value = response.json().await?;

        if let Some(payload) = sync_response.get("payload") {
            let sync_payload: GraphSyncPayload = serde_json::from_value(payload.clone())?;

            // Apply the sync payload
            let stats = sync_engine.apply_sync(&sync_payload, graph_name).await?;

            info!(
                "Applied sync from peer {}: {} nodes, {} edges, {} conflicts",
                peer_id, stats.nodes_applied, stats.edges_applied, stats.conflicts_detected
            );
        }

        Ok(())
    }

    /// Handle cleanup on shutdown
    pub async fn shutdown(&self) {
        info!("Shutting down sync coordinator");
        // Any cleanup tasks would go here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn get_sync_enabled_sessions_returns_empty_when_no_sync_enabled() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("sync.duckdb");
        let persistence = Arc::new(Persistence::new(&db_path).expect("persistence"));
        let mesh_registry = Arc::new(MeshRegistry::new());
        let mesh_client = Arc::new(MeshClient::new("localhost", 0));
        let coordinator = SyncCoordinator::new(
            persistence,
            mesh_registry,
            mesh_client,
            SyncCoordinatorConfig::default(),
        );

        let sessions = coordinator.get_sync_enabled_sessions().expect("sessions");
        assert!(
            sessions.is_empty(),
            "Expected no sync-enabled sessions initially"
        );
    }

    #[test]
    fn get_sync_enabled_sessions_returns_enabled_graphs() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("sync.duckdb");
        let persistence = Arc::new(Persistence::new(&db_path).expect("persistence"));

        // Enable sync for a graph (this will create the metadata entry)
        persistence
            .graph_set_sync_enabled("test-session", "test-graph", true)
            .expect("enable sync");

        let mesh_registry = Arc::new(MeshRegistry::new());
        let mesh_client = Arc::new(MeshClient::new("localhost", 0));
        let coordinator = SyncCoordinator::new(
            persistence,
            mesh_registry,
            mesh_client,
            SyncCoordinatorConfig::default(),
        );

        let sessions = coordinator.get_sync_enabled_sessions().expect("sessions");
        assert_eq!(
            sessions,
            vec![("test-session".to_string(), "test-graph".to_string())]
        );
    }

    #[test]
    fn get_sync_enabled_sessions_excludes_disabled_graphs() {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("sync.duckdb");
        let persistence = Arc::new(Persistence::new(&db_path).expect("persistence"));

        // Create graph with sync enabled
        persistence
            .graph_set_sync_enabled("session-a", "enabled", true)
            .expect("enable sync");

        // Create graph with sync disabled
        persistence
            .graph_set_sync_enabled("session-b", "disabled", false)
            .expect("create disabled graph");

        let mesh_registry = Arc::new(MeshRegistry::new());
        let mesh_client = Arc::new(MeshClient::new("localhost", 0));
        let coordinator = SyncCoordinator::new(
            persistence,
            mesh_registry,
            mesh_client,
            SyncCoordinatorConfig::default(),
        );

        let sessions = coordinator.get_sync_enabled_sessions().expect("sessions");
        assert_eq!(
            sessions,
            vec![("session-a".to_string(), "enabled".to_string())]
        );
    }
}

/// Start the sync coordinator as a background task
pub async fn start_sync_coordinator(
    persistence: Arc<Persistence>,
    mesh_registry: Arc<MeshRegistry>,
    mesh_client: Arc<MeshClient>,
    config: SyncCoordinatorConfig,
) -> tokio::task::JoinHandle<()> {
    let coordinator = Arc::new(SyncCoordinator::new(
        persistence,
        mesh_registry,
        mesh_client,
        config,
    ));

    tokio::spawn(async move {
        coordinator.start().await;
    })
}

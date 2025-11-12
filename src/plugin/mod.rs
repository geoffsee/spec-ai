/// Plugin system for extending agent capabilities
///
/// This module provides a flexible plugin architecture that allows:
/// - Dynamic registration of new model providers
/// - Custom tool implementations
/// - Extension of agent capabilities
/// - Plugin lifecycle management
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: Option<String>,
    /// Plugin capabilities/tags
    pub capabilities: Vec<String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            author: None,
            capabilities: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Add capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
}

/// Plugin lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is registered but not initialized
    Registered,
    /// Plugin is initialized and ready
    Active,
    /// Plugin encountered an error
    Error,
    /// Plugin has been shutdown
    Shutdown,
}

/// Core plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    ///
    /// Called once when the plugin is loaded. Use this to:
    /// - Validate configuration
    /// - Initialize resources
    /// - Register capabilities
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the plugin
    ///
    /// Called when the plugin is being unloaded. Use this to:
    /// - Clean up resources
    /// - Save state
    /// - Disconnect from services
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Health check for the plugin
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// Plugin registration entry
struct PluginEntry {
    plugin: Box<dyn Plugin>,
    state: PluginState,
}

/// Plugin registry for managing all plugins
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, PluginEntry>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new plugin
    ///
    /// # Arguments
    /// * `plugin` - The plugin instance to register
    ///
    /// # Returns
    /// * `Ok(())` if registration succeeds
    /// * `Err` if a plugin with the same ID already exists
    pub async fn register(&self, plugin: Box<dyn Plugin>) -> Result<()> {
        let id = plugin.metadata().id.clone();
        let mut plugins = self.plugins.write().await;

        if plugins.contains_key(&id) {
            anyhow::bail!("Plugin with id '{}' already registered", id);
        }

        plugins.insert(
            id,
            PluginEntry {
                plugin,
                state: PluginState::Registered,
            },
        );

        Ok(())
    }

    /// Initialize a specific plugin by ID
    pub async fn init_plugin(&self, id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        let entry = plugins
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", id))?;

        if entry.state != PluginState::Registered {
            anyhow::bail!("Plugin '{}' is not in Registered state", id);
        }

        match entry.plugin.init().await {
            Ok(()) => {
                entry.state = PluginState::Active;
                Ok(())
            }
            Err(e) => {
                entry.state = PluginState::Error;
                Err(e)
            }
        }
    }

    /// Initialize all registered plugins
    pub async fn init_all(&self) -> Result<Vec<String>> {
        let plugin_ids: Vec<String> = {
            let plugins = self.plugins.read().await;
            plugins
                .iter()
                .filter(|(_, entry)| entry.state == PluginState::Registered)
                .map(|(id, _)| id.clone())
                .collect()
        };

        let mut failed = Vec::new();

        for id in &plugin_ids {
            if let Err(e) = self.init_plugin(id).await {
                tracing::error!("Failed to initialize plugin '{}': {}", id, e);
                failed.push(id.clone());
            }
        }

        Ok(failed)
    }

    /// Shutdown a specific plugin by ID
    pub async fn shutdown_plugin(&self, id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        let entry = plugins
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", id))?;

        if entry.state != PluginState::Active {
            anyhow::bail!("Plugin '{}' is not active", id);
        }

        entry.plugin.shutdown().await?;
        entry.state = PluginState::Shutdown;

        Ok(())
    }

    /// Shutdown all active plugins
    pub async fn shutdown_all(&self) -> Result<()> {
        let plugin_ids: Vec<String> = {
            let plugins = self.plugins.read().await;
            plugins
                .iter()
                .filter(|(_, entry)| entry.state == PluginState::Active)
                .map(|(id, _)| id.clone())
                .collect()
        };

        for id in &plugin_ids {
            if let Err(e) = self.shutdown_plugin(id).await {
                tracing::error!("Failed to shutdown plugin '{}': {}", id, e);
            }
        }

        Ok(())
    }

    /// Unregister a plugin by ID
    pub async fn unregister(&self, id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        let entry = plugins
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", id))?;

        if entry.state == PluginState::Active {
            anyhow::bail!("Cannot unregister active plugin '{}'. Shutdown first.", id);
        }

        plugins.remove(id);
        Ok(())
    }

    /// Get plugin metadata by ID
    pub async fn get_metadata(&self, id: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().await;
        plugins.get(id).map(|entry| entry.plugin.metadata().clone())
    }

    /// Get plugin state by ID
    pub async fn get_state(&self, id: &str) -> Option<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.get(id).map(|entry| entry.state)
    }

    /// List all plugin IDs
    pub async fn list_plugin_ids(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// List all plugin metadata
    pub async fn list_plugins(&self) -> Vec<(PluginMetadata, PluginState)> {
        let plugins = self.plugins.read().await;
        plugins
            .values()
            .map(|entry| (entry.plugin.metadata().clone(), entry.state))
            .collect()
    }

    /// Check if a plugin is registered
    pub async fn has_plugin(&self, id: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(id)
    }

    /// Get count of registered plugins
    pub async fn count(&self) -> usize {
        let plugins = self.plugins.read().await;
        plugins.len()
    }

    /// Run health check on all active plugins
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let plugins = self.plugins.read().await;
        let mut results = HashMap::new();

        for (id, entry) in plugins.iter() {
            if entry.state == PluginState::Active {
                let healthy = entry.plugin.health_check().await.unwrap_or(false);
                results.insert(id.clone(), healthy);
            }
        }

        results
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test plugin implementation
    struct TestPlugin {
        metadata: PluginMetadata,
        init_called: bool,
        shutdown_called: bool,
    }

    impl TestPlugin {
        fn new(id: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(id, format!("Test Plugin {}", id), "1.0.0"),
                init_called: false,
                shutdown_called: false,
            }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn init(&mut self) -> Result<()> {
            self.init_called = true;
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<()> {
            self.shutdown_called = true;
            Ok(())
        }
    }

    // Failing plugin for error tests
    struct FailingPlugin {
        metadata: PluginMetadata,
    }

    impl FailingPlugin {
        fn new() -> Self {
            Self {
                metadata: PluginMetadata::new("failing", "Failing Plugin", "1.0.0"),
            }
        }
    }

    #[async_trait]
    impl Plugin for FailingPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn init(&mut self) -> Result<()> {
            anyhow::bail!("Intentional failure")
        }
    }

    #[tokio::test]
    async fn test_plugin_metadata() {
        let meta = PluginMetadata::new("test", "Test Plugin", "1.0.0")
            .with_description("A test plugin")
            .with_author("Test Author")
            .with_capability("testing");

        assert_eq!(meta.id, "test");
        assert_eq!(meta.name, "Test Plugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.description, "A test plugin");
        assert_eq!(meta.author, Some("Test Author".to_string()));
        assert_eq!(meta.capabilities, vec!["testing"]);
    }

    #[tokio::test]
    async fn test_register_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).await.unwrap();

        assert!(registry.has_plugin("test1").await);
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_plugin() {
        let registry = PluginRegistry::new();
        let plugin1 = Box::new(TestPlugin::new("test1"));
        let plugin2 = Box::new(TestPlugin::new("test1"));

        registry.register(plugin1).await.unwrap();
        let result = registry.register(plugin2).await;

        assert!(result.is_err());
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_init_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).await.unwrap();
        registry.init_plugin("test1").await.unwrap();

        let state = registry.get_state("test1").await;
        assert_eq!(state, Some(PluginState::Active));
    }

    #[tokio::test]
    async fn test_init_all_plugins() {
        let registry = PluginRegistry::new();

        registry
            .register(Box::new(TestPlugin::new("test1")))
            .await
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("test2")))
            .await
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("test3")))
            .await
            .unwrap();

        let failed = registry.init_all().await.unwrap();

        assert!(failed.is_empty());
        assert_eq!(registry.get_state("test1").await, Some(PluginState::Active));
        assert_eq!(registry.get_state("test2").await, Some(PluginState::Active));
        assert_eq!(registry.get_state("test3").await, Some(PluginState::Active));
    }

    #[tokio::test]
    async fn test_init_plugin_failure() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(FailingPlugin::new());

        registry.register(plugin).await.unwrap();
        let result = registry.init_plugin("failing").await;

        assert!(result.is_err());
        assert_eq!(
            registry.get_state("failing").await,
            Some(PluginState::Error)
        );
    }

    #[tokio::test]
    async fn test_shutdown_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).await.unwrap();
        registry.init_plugin("test1").await.unwrap();
        registry.shutdown_plugin("test1").await.unwrap();

        let state = registry.get_state("test1").await;
        assert_eq!(state, Some(PluginState::Shutdown));
    }

    #[tokio::test]
    async fn test_shutdown_all_plugins() {
        let registry = PluginRegistry::new();

        registry
            .register(Box::new(TestPlugin::new("test1")))
            .await
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("test2")))
            .await
            .unwrap();

        registry.init_all().await.unwrap();
        registry.shutdown_all().await.unwrap();

        assert_eq!(
            registry.get_state("test1").await,
            Some(PluginState::Shutdown)
        );
        assert_eq!(
            registry.get_state("test2").await,
            Some(PluginState::Shutdown)
        );
    }

    #[tokio::test]
    async fn test_unregister_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).await.unwrap();
        registry.unregister("test1").await.unwrap();

        assert!(!registry.has_plugin("test1").await);
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_cannot_unregister_active_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).await.unwrap();
        registry.init_plugin("test1").await.unwrap();

        let result = registry.unregister("test1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_plugins() {
        let registry = PluginRegistry::new();

        registry
            .register(Box::new(TestPlugin::new("test1")))
            .await
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("test2")))
            .await
            .unwrap();

        let plugins = registry.list_plugins().await;
        assert_eq!(plugins.len(), 2);

        let ids = registry.list_plugin_ids().await;
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"test1".to_string()));
        assert!(ids.contains(&"test2".to_string()));
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin::new("test1"));

        registry.register(plugin).await.unwrap();

        let metadata = registry.get_metadata("test1").await;
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().id, "test1");
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let registry = PluginRegistry::new();

        registry
            .register(Box::new(TestPlugin::new("test1")))
            .await
            .unwrap();
        registry
            .register(Box::new(TestPlugin::new("test2")))
            .await
            .unwrap();

        registry.init_all().await.unwrap();

        let results = registry.health_check_all().await;
        assert_eq!(results.len(), 2);
        assert_eq!(results.get("test1"), Some(&true));
        assert_eq!(results.get("test2"), Some(&true));
    }
}

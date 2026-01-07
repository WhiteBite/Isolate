//! Plugin System for Isolate
//!
//! Provides plugin loading, management, and service checking functionality.
//!
//! ## Architecture
//!
//! The plugin system supports three levels:
//!
//! - **Level 1 (Declarative)**: JSON manifests for simple plugins (service-checker, hostlist-provider)
//! - **Level 2 (UI)**: Svelte components for dashboard widgets and settings panels
//! - **Level 3 (Scripts)**: Lua scripts for custom logic with sandboxed execution
//!
//! ## Modules
//!
//! - `manifest`: Plugin manifest schema and types
//! - `js_loader`: Plugin discovery and manifest loading
//! - `lua_runtime`: Sandboxed Lua 5.4 runtime for script execution
//! - `script_executor`: High-level script execution with timeout and permissions

pub mod hostlist_registry;
pub mod js_loader;
pub mod lua_runtime;
pub mod manifest;
pub mod script_executor;
pub mod strategy_registry;

#[cfg(test)]
mod integration_tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

pub use hostlist_registry::{
    create_hostlist_registry, HostlistRegistry, HostlistRegistryError, RegisteredHostlist,
    RegistryStats,
};
pub use js_loader::{
    get_all_plugins, get_all_plugins_async, get_all_services, get_all_services_async,
    load_manifest, load_manifest_async, scan_plugins, scan_plugins_async, PluginLoaderError,
};
pub use manifest::{
    HostlistDefinition, HostlistFormat, LoadedPluginInfo, PluginContributes, PluginManifest,
    PluginType, ServiceDefinition, ServiceEndpoint,
    // Strategy types
    PluginStrategyDefinition, PluginStrategyConfig, StrategyFamily, StrategyPorts, StrategyProfile,
};
pub use strategy_registry::{
    StrategyRegistry, RegisteredStrategy, StrategySource, RegistryError as StrategyRegistryError,
};

// Re-export Lua runtime and script executor for Level 3 plugins
pub use lua_runtime::{CheckResult, LuaRuntime, PluginStorage, ScriptPermissions};
pub use script_executor::{
    create_script_executor, ScriptError, ScriptExecutor, SharedScriptExecutor,
};

// Re-export checker functionality from core
pub use crate::core::checker::{check_plugin_endpoints, ServiceStatus};

/// Legacy checker module compatibility
pub mod checker {
    use super::*;
    use crate::core::checker;

    /// Legacy endpoint check result (for backward compatibility)
    pub type EndpointCheckResult = checker::CheckResult;

    /// Legacy service status (for backward compatibility)
    pub type ServiceStatus = checker::ServiceStatus;

    /// Check a single endpoint (legacy API)
    pub async fn check_endpoint(endpoint: &ServiceEndpoint) -> EndpointCheckResult {
        let ep = checker::Endpoint {
            url: endpoint.url.clone(),
            name: endpoint.name.clone(),
            method: checker::HttpMethod::from_str_loose(&endpoint.method),
            expected_status: Vec::new(),
            timeout_ms: 5000,
        };
        
        let checker_instance = checker::EndpointChecker::new();
        checker_instance.check(&ep).await
    }

    /// Check all endpoints for a service (legacy API)
    pub async fn check_service_endpoints(endpoints: &[ServiceEndpoint]) -> ServiceStatus {
        checker::check_plugin_endpoints(endpoints).await
    }
}

// ============================================================================
// Plugin Manager
// ============================================================================

/// Plugin state
#[derive(Debug, Clone)]
pub struct PluginState {
    /// Plugin info
    pub info: LoadedPluginInfo,
    /// Whether the plugin is currently loaded
    pub loaded: bool,
}

/// Plugin Manager - centralized plugin management
pub struct PluginManager {
    /// Plugins directory path
    plugins_dir: PathBuf,
    /// Loaded plugins by ID
    plugins: RwLock<HashMap<String, PluginState>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// Get the plugins directory path
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Initialize and load all plugins from the plugins directory
    pub async fn init(&self) -> Result<usize, PluginLoaderError> {
        info!(plugins_dir = %self.plugins_dir.display(), "Initializing plugin manager");

        // Ensure plugins directory exists
        if !tokio::fs::try_exists(&self.plugins_dir).await.unwrap_or(false) {
            tokio::fs::create_dir_all(&self.plugins_dir).await?;
        }

        // Scan and load all plugins
        let loaded = self.reload_all().await?;
        
        info!(count = loaded, "Plugin manager initialized");
        Ok(loaded)
    }

    /// Reload all plugins from disk
    pub async fn reload_all(&self) -> Result<usize, PluginLoaderError> {
        let plugin_infos = get_all_plugins_async(&self.plugins_dir).await;
        let count = plugin_infos.len();

        let mut plugins = self.plugins.write().await;
        plugins.clear();

        for info in plugin_infos {
            let id = info.manifest.id.clone();
            let loaded = info.enabled && info.error.is_none();
            
            if loaded {
                debug!(plugin_id = %id, "Loaded plugin");
            } else if let Some(ref error) = info.error {
                warn!(plugin_id = %id, error = %error, "Failed to load plugin");
            }

            plugins.insert(
                id,
                PluginState {
                    info,
                    loaded,
                },
            );
        }

        Ok(count)
    }

    /// Load a specific plugin by ID
    pub async fn load(&self, plugin_id: &str) -> Result<(), PluginLoaderError> {
        let mut plugins = self.plugins.write().await;

        if let Some(state) = plugins.get_mut(plugin_id) {
            if state.loaded {
                debug!(plugin_id = %plugin_id, "Plugin already loaded");
                return Ok(());
            }

            // Try to reload the manifest
            let plugin_path = PathBuf::from(&state.info.path);
            match load_manifest_async(&plugin_path).await {
                Ok(manifest) => {
                    state.info.manifest = manifest;
                    state.info.enabled = true;
                    state.info.error = None;
                    state.loaded = true;
                    info!(plugin_id = %plugin_id, "Plugin loaded");
                    Ok(())
                }
                Err(e) => {
                    state.info.error = Some(e.to_string());
                    state.loaded = false;
                    Err(e)
                }
            }
        } else {
            Err(PluginLoaderError::NotFound(plugin_id.to_string()))
        }
    }

    /// Unload a specific plugin by ID
    pub async fn unload(&self, plugin_id: &str) -> Result<(), PluginLoaderError> {
        let mut plugins = self.plugins.write().await;

        if let Some(state) = plugins.get_mut(plugin_id) {
            if !state.loaded {
                debug!(plugin_id = %plugin_id, "Plugin already unloaded");
                return Ok(());
            }

            state.loaded = false;
            state.info.enabled = false;
            info!(plugin_id = %plugin_id, "Plugin unloaded");
            Ok(())
        } else {
            Err(PluginLoaderError::NotFound(plugin_id.to_string()))
        }
    }

    /// Get all plugins
    pub async fn list(&self) -> Vec<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Get all loaded plugins
    pub async fn list_loaded(&self) -> Vec<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.values().filter(|p| p.loaded).cloned().collect()
    }

    /// Get a plugin by ID
    pub async fn get(&self, plugin_id: &str) -> Option<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).cloned()
    }

    /// Check if a plugin is loaded
    pub async fn is_loaded(&self, plugin_id: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| p.loaded).unwrap_or(false)
    }

    /// Get all service definitions from loaded service-checker plugins
    pub async fn get_services(&self) -> Vec<ServiceDefinition> {
        let plugins = self.plugins.read().await;
        plugins
            .values()
            .filter(|p| p.loaded)
            .filter(|p| p.info.manifest.plugin_type == PluginType::ServiceChecker)
            .filter_map(|p| p.info.manifest.service.clone())
            .collect()
    }

    /// Get plugins by type
    pub async fn get_by_type(&self, plugin_type: PluginType) -> Vec<PluginState> {
        let plugins = self.plugins.read().await;
        plugins
            .values()
            .filter(|p| p.loaded && p.info.manifest.plugin_type == plugin_type)
            .cloned()
            .collect()
    }

    /// Get plugin count
    pub async fn count(&self) -> usize {
        let plugins = self.plugins.read().await;
        plugins.len()
    }

    /// Get loaded plugin count
    pub async fn loaded_count(&self) -> usize {
        let plugins = self.plugins.read().await;
        plugins.values().filter(|p| p.loaded).count()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        // Default to current directory's plugins folder
        Self::new("plugins")
    }
}

/// Create a shared plugin manager
pub fn create_plugin_manager(plugins_dir: impl Into<PathBuf>) -> Arc<PluginManager> {
    Arc::new(PluginManager::new(plugins_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_plugin(plugins_dir: &Path, id: &str, plugin_type: &str) {
        let plugin_dir = plugins_dir.join(id);
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        let manifest = format!(
            r#"{{
                "id": "{}",
                "name": "Test Plugin {}",
                "version": "1.0.0",
                "author": "Test",
                "type": "{}"
            }}"#,
            id, id, plugin_type
        );

        tokio::fs::write(plugin_dir.join("plugin.json"), manifest)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_plugin_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");

        let manager = PluginManager::new(&plugins_dir);
        let count = manager.init().await.unwrap();

        assert_eq!(count, 0);
        assert!(plugins_dir.exists());
    }

    #[tokio::test]
    async fn test_plugin_manager_load_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        // Create test plugins
        create_test_plugin(plugins_dir, "plugin-a", "service-checker").await;
        create_test_plugin(plugins_dir, "plugin-b", "strategy-provider").await;

        let manager = PluginManager::new(plugins_dir);
        let count = manager.init().await.unwrap();

        assert_eq!(count, 2);
        assert_eq!(manager.loaded_count().await, 2);
    }

    #[tokio::test]
    async fn test_plugin_manager_list() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "test-plugin", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let plugins = manager.list().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].info.manifest.id, "test-plugin");
        assert!(plugins[0].loaded);
    }

    #[tokio::test]
    async fn test_plugin_manager_unload_load() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "test-plugin", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        // Unload
        manager.unload("test-plugin").await.unwrap();
        assert!(!manager.is_loaded("test-plugin").await);
        assert_eq!(manager.loaded_count().await, 0);

        // Load again
        manager.load("test-plugin").await.unwrap();
        assert!(manager.is_loaded("test-plugin").await);
        assert_eq!(manager.loaded_count().await, 1);
    }

    #[tokio::test]
    async fn test_plugin_manager_get_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "checker-1", "service-checker").await;
        create_test_plugin(plugins_dir, "checker-2", "service-checker").await;
        create_test_plugin(plugins_dir, "strategy-1", "strategy-provider").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let checkers = manager.get_by_type(PluginType::ServiceChecker).await;
        assert_eq!(checkers.len(), 2);

        let strategies = manager.get_by_type(PluginType::StrategyProvider).await;
        assert_eq!(strategies.len(), 1);
    }

    #[tokio::test]
    async fn test_plugin_manager_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path());
        manager.init().await.unwrap();

        let result = manager.load("nonexistent").await;
        assert!(matches!(result, Err(PluginLoaderError::NotFound(_))));
    }
}

//! Strategy Registry for Plugin System
//!
//! Manages strategies contributed by plugins (Level 1).
//! Integrates with existing strategy_loader.rs for YAML strategies.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::manifest::{
    PluginStrategyConfig, PluginStrategyDefinition, StrategyFamily,
};
use super::{PluginManager, PluginType};

// ============================================================================
// Registry Entry
// ============================================================================

/// Source of a strategy
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StrategySource {
    /// Built-in YAML strategy from configs/strategies/
    Builtin,
    /// Strategy from a plugin
    Plugin { plugin_id: String },
    /// User-created custom strategy
    Custom,
}

/// Registered strategy with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisteredStrategy {
    /// Strategy definition
    pub definition: PluginStrategyDefinition,
    /// Source of the strategy
    pub source: StrategySource,
    /// Whether the strategy is enabled
    pub enabled: bool,
}

// ============================================================================
// Strategy Registry
// ============================================================================

/// Strategy Registry - centralized management of all strategies
pub struct StrategyRegistry {
    /// Registered strategies by ID
    strategies: RwLock<HashMap<String, RegisteredStrategy>>,
    /// Plugin manager reference for loading plugin strategies
    plugin_manager: Option<Arc<PluginManager>>,
}

impl StrategyRegistry {
    /// Create a new strategy registry
    pub fn new() -> Self {
        Self {
            strategies: RwLock::new(HashMap::new()),
            plugin_manager: None,
        }
    }

    /// Create a new strategy registry with plugin manager
    pub fn with_plugin_manager(plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            strategies: RwLock::new(HashMap::new()),
            plugin_manager: Some(plugin_manager),
        }
    }

    /// Set the plugin manager
    pub fn set_plugin_manager(&mut self, plugin_manager: Arc<PluginManager>) {
        self.plugin_manager = Some(plugin_manager);
    }

    // ========================================================================
    // Registration Methods
    // ========================================================================

    /// Register a strategy
    pub async fn register(&self, strategy: PluginStrategyDefinition, source: StrategySource) -> Result<(), RegistryError> {
        let id = strategy.id.clone();
        
        let mut strategies = self.strategies.write().await;
        
        if strategies.contains_key(&id) {
            return Err(RegistryError::AlreadyExists(id));
        }

        debug!(strategy_id = %id, source = ?source, "Registering strategy");

        strategies.insert(
            id.clone(),
            RegisteredStrategy {
                definition: strategy,
                source,
                enabled: true,
            },
        );

        Ok(())
    }

    /// Register multiple strategies from a plugin
    pub async fn register_from_plugin(
        &self,
        plugin_id: &str,
        strategies: Vec<PluginStrategyDefinition>,
    ) -> Result<usize, RegistryError> {
        let mut count = 0;
        
        for mut strategy in strategies {
            // Set source plugin
            strategy.source_plugin = Some(plugin_id.to_string());
            
            // Prefix ID with plugin ID to avoid conflicts
            let original_id = strategy.id.clone();
            if !strategy.id.starts_with(&format!("{}:", plugin_id)) {
                strategy.id = format!("{}:{}", plugin_id, strategy.id);
            }

            match self.register(strategy, StrategySource::Plugin { plugin_id: plugin_id.to_string() }).await {
                Ok(()) => {
                    count += 1;
                    debug!(plugin_id = %plugin_id, strategy_id = %original_id, "Registered plugin strategy");
                }
                Err(RegistryError::AlreadyExists(id)) => {
                    warn!(strategy_id = %id, "Strategy already registered, skipping");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to register strategy");
                }
            }
        }

        info!(plugin_id = %plugin_id, count, "Registered strategies from plugin");
        Ok(count)
    }

    /// Unregister a strategy by ID
    pub async fn unregister(&self, strategy_id: &str) -> Result<RegisteredStrategy, RegistryError> {
        let mut strategies = self.strategies.write().await;
        
        strategies
            .remove(strategy_id)
            .ok_or_else(|| RegistryError::NotFound(strategy_id.to_string()))
    }

    /// Unregister all strategies from a plugin
    pub async fn unregister_plugin(&self, plugin_id: &str) -> usize {
        let mut strategies = self.strategies.write().await;
        
        let to_remove: Vec<String> = strategies
            .iter()
            .filter(|(_, s)| matches!(&s.source, StrategySource::Plugin { plugin_id: pid } if pid == plugin_id))
            .map(|(id, _)| id.clone())
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            strategies.remove(&id);
        }

        info!(plugin_id = %plugin_id, count, "Unregistered plugin strategies");
        count
    }

    // ========================================================================
    // Query Methods
    // ========================================================================

    /// Get a strategy by ID
    pub async fn get(&self, strategy_id: &str) -> Option<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies.get(strategy_id).cloned()
    }

    /// List all registered strategies
    pub async fn list(&self) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies.values().cloned().collect()
    }

    /// List enabled strategies only
    pub async fn list_enabled(&self) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies
            .values()
            .filter(|s| s.enabled)
            .cloned()
            .collect()
    }

    /// Get strategies by service ID
    ///
    /// Returns strategies that target the specified service.
    pub async fn get_by_service(&self, service_id: &str) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies
            .values()
            .filter(|s| {
                s.enabled && s.definition.target_services.iter().any(|sid| sid == service_id)
            })
            .cloned()
            .collect()
    }

    /// Get strategies by family
    pub async fn get_by_family(&self, family: StrategyFamily) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies
            .values()
            .filter(|s| s.enabled && s.definition.family == family)
            .cloned()
            .collect()
    }

    /// Get strategies from plugins only
    pub async fn get_plugin_strategies(&self) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies
            .values()
            .filter(|s| matches!(s.source, StrategySource::Plugin { .. }))
            .cloned()
            .collect()
    }

    /// Get strategies from a specific plugin
    pub async fn get_strategies_from_plugin(&self, plugin_id: &str) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies
            .values()
            .filter(|s| matches!(&s.source, StrategySource::Plugin { plugin_id: pid } if pid == plugin_id))
            .cloned()
            .collect()
    }

    /// Get builtin strategies only
    pub async fn get_builtin_strategies(&self) -> Vec<RegisteredStrategy> {
        let strategies = self.strategies.read().await;
        strategies
            .values()
            .filter(|s| matches!(s.source, StrategySource::Builtin))
            .cloned()
            .collect()
    }

    // ========================================================================
    // State Management
    // ========================================================================

    /// Enable a strategy
    pub async fn enable(&self, strategy_id: &str) -> Result<(), RegistryError> {
        let mut strategies = self.strategies.write().await;
        
        if let Some(strategy) = strategies.get_mut(strategy_id) {
            strategy.enabled = true;
            Ok(())
        } else {
            Err(RegistryError::NotFound(strategy_id.to_string()))
        }
    }

    /// Disable a strategy
    pub async fn disable(&self, strategy_id: &str) -> Result<(), RegistryError> {
        let mut strategies = self.strategies.write().await;
        
        if let Some(strategy) = strategies.get_mut(strategy_id) {
            strategy.enabled = false;
            Ok(())
        } else {
            Err(RegistryError::NotFound(strategy_id.to_string()))
        }
    }

    /// Get strategy count
    pub async fn count(&self) -> usize {
        let strategies = self.strategies.read().await;
        strategies.len()
    }

    /// Get enabled strategy count
    pub async fn enabled_count(&self) -> usize {
        let strategies = self.strategies.read().await;
        strategies.values().filter(|s| s.enabled).count()
    }

    /// Clear all strategies
    pub async fn clear(&self) {
        let mut strategies = self.strategies.write().await;
        strategies.clear();
    }

    // ========================================================================
    // Plugin Integration
    // ========================================================================

    /// Load strategies from all loaded plugins
    pub async fn load_from_plugins(&self) -> Result<usize, RegistryError> {
        let plugin_manager = self.plugin_manager.as_ref()
            .ok_or(RegistryError::NoPluginManager)?;

        let plugins = plugin_manager.get_by_type(PluginType::StrategyProvider).await;
        let mut total_count = 0;

        for plugin_state in plugins {
            let plugin_id = &plugin_state.info.manifest.id;
            
            // Get strategies from contributes
            let strategies = plugin_state.info.manifest.contributes.strategies.clone();
            
            if !strategies.is_empty() {
                // Convert StrategyDefinition to PluginStrategyDefinition
                let plugin_strategies: Vec<PluginStrategyDefinition> = strategies
                    .into_iter()
                    .map(|s| convert_legacy_strategy(&s, plugin_id))
                    .collect();

                match self.register_from_plugin(plugin_id, plugin_strategies).await {
                    Ok(count) => total_count += count,
                    Err(e) => warn!(plugin_id = %plugin_id, error = %e, "Failed to load strategies from plugin"),
                }
            }
        }

        info!(total_count, "Loaded strategies from plugins");
        Ok(total_count)
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert legacy StrategyDefinition to PluginStrategyDefinition
fn convert_legacy_strategy(
    legacy: &super::manifest::StrategyDefinition,
    plugin_id: &str,
) -> PluginStrategyDefinition {
    PluginStrategyDefinition {
        id: legacy.id.clone(),
        name: legacy.name.clone(),
        description: None,
        family: StrategyFamily::from(legacy.family.as_str()),
        engine: "winws".to_string(),
        target_services: Vec::new(),
        priority: 0,
        config: PluginStrategyConfig {
            args: Vec::new(),
            hostlist: None,
            ports: None,
            profiles: None,
        },
        author: None,
        label: None,
        source_plugin: Some(plugin_id.to_string()),
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Strategy already exists: {0}")]
    AlreadyExists(String),

    #[error("Strategy not found: {0}")]
    NotFound(String),

    #[error("No plugin manager configured")]
    NoPluginManager,

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_strategy(id: &str, name: &str) -> PluginStrategyDefinition {
        PluginStrategyDefinition {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Test strategy".to_string()),
            family: StrategyFamily::Zapret,
            engine: "winws".to_string(),
            target_services: vec!["youtube".to_string(), "discord".to_string()],
            priority: 10,
            config: PluginStrategyConfig::default(),
            author: Some("Test".to_string()),
            label: Some("test".to_string()),
            source_plugin: None,
        }
    }

    #[tokio::test]
    async fn test_register_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = create_test_strategy("test-1", "Test Strategy 1");

        registry.register(strategy, StrategySource::Builtin).await.unwrap();

        assert_eq!(registry.count().await, 1);
        
        let retrieved = registry.get("test-1").await.unwrap();
        assert_eq!(retrieved.definition.name, "Test Strategy 1");
        assert!(matches!(retrieved.source, StrategySource::Builtin));
    }

    #[tokio::test]
    async fn test_register_duplicate() {
        let registry = StrategyRegistry::new();
        let strategy1 = create_test_strategy("test-1", "Test Strategy 1");
        let strategy2 = create_test_strategy("test-1", "Test Strategy 1 Duplicate");

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();
        let result = registry.register(strategy2, StrategySource::Builtin).await;

        assert!(matches!(result, Err(RegistryError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_unregister_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = create_test_strategy("test-1", "Test Strategy 1");

        registry.register(strategy, StrategySource::Builtin).await.unwrap();
        assert_eq!(registry.count().await, 1);

        let removed = registry.unregister("test-1").await.unwrap();
        assert_eq!(removed.definition.id, "test-1");
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_get_by_service() {
        let registry = StrategyRegistry::new();

        let mut strategy1 = create_test_strategy("yt-1", "YouTube Strategy");
        strategy1.target_services = vec!["youtube".to_string()];

        let mut strategy2 = create_test_strategy("discord-1", "Discord Strategy");
        strategy2.target_services = vec!["discord".to_string()];

        let mut strategy3 = create_test_strategy("general-1", "General Strategy");
        strategy3.target_services = vec!["youtube".to_string(), "discord".to_string()];

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();
        registry.register(strategy2, StrategySource::Builtin).await.unwrap();
        registry.register(strategy3, StrategySource::Builtin).await.unwrap();

        let youtube_strategies = registry.get_by_service("youtube").await;
        assert_eq!(youtube_strategies.len(), 2);

        let discord_strategies = registry.get_by_service("discord").await;
        assert_eq!(discord_strategies.len(), 2);

        let telegram_strategies = registry.get_by_service("telegram").await;
        assert_eq!(telegram_strategies.len(), 0);
    }

    #[tokio::test]
    async fn test_get_by_family() {
        let registry = StrategyRegistry::new();

        let mut strategy1 = create_test_strategy("zapret-1", "Zapret Strategy");
        strategy1.family = StrategyFamily::Zapret;

        let mut strategy2 = create_test_strategy("vless-1", "VLESS Strategy");
        strategy2.family = StrategyFamily::Vless;

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();
        registry.register(strategy2, StrategySource::Builtin).await.unwrap();

        let zapret = registry.get_by_family(StrategyFamily::Zapret).await;
        assert_eq!(zapret.len(), 1);
        assert_eq!(zapret[0].definition.id, "zapret-1");

        let vless = registry.get_by_family(StrategyFamily::Vless).await;
        assert_eq!(vless.len(), 1);
        assert_eq!(vless[0].definition.id, "vless-1");
    }

    #[tokio::test]
    async fn test_register_from_plugin() {
        let registry = StrategyRegistry::new();

        let strategies = vec![
            create_test_strategy("strat-1", "Strategy 1"),
            create_test_strategy("strat-2", "Strategy 2"),
        ];

        let count = registry.register_from_plugin("test-plugin", strategies).await.unwrap();
        assert_eq!(count, 2);

        // Check that IDs are prefixed
        assert!(registry.get("test-plugin:strat-1").await.is_some());
        assert!(registry.get("test-plugin:strat-2").await.is_some());

        // Check plugin strategies
        let plugin_strategies = registry.get_plugin_strategies().await;
        assert_eq!(plugin_strategies.len(), 2);
    }

    #[tokio::test]
    async fn test_unregister_plugin() {
        let registry = StrategyRegistry::new();

        let strategies = vec![
            create_test_strategy("strat-1", "Strategy 1"),
            create_test_strategy("strat-2", "Strategy 2"),
        ];

        registry.register_from_plugin("test-plugin", strategies).await.unwrap();
        registry.register(create_test_strategy("builtin-1", "Builtin"), StrategySource::Builtin).await.unwrap();

        assert_eq!(registry.count().await, 3);

        let removed = registry.unregister_plugin("test-plugin").await;
        assert_eq!(removed, 2);
        assert_eq!(registry.count().await, 1);

        // Builtin should remain
        assert!(registry.get("builtin-1").await.is_some());
    }

    #[tokio::test]
    async fn test_enable_disable() {
        let registry = StrategyRegistry::new();
        let strategy = create_test_strategy("test-1", "Test Strategy");

        registry.register(strategy, StrategySource::Builtin).await.unwrap();

        // Initially enabled
        assert!(registry.get("test-1").await.unwrap().enabled);
        assert_eq!(registry.enabled_count().await, 1);

        // Disable
        registry.disable("test-1").await.unwrap();
        assert!(!registry.get("test-1").await.unwrap().enabled);
        assert_eq!(registry.enabled_count().await, 0);

        // Enable again
        registry.enable("test-1").await.unwrap();
        assert!(registry.get("test-1").await.unwrap().enabled);
        assert_eq!(registry.enabled_count().await, 1);
    }

    #[tokio::test]
    async fn test_list_enabled() {
        let registry = StrategyRegistry::new();

        registry.register(create_test_strategy("s1", "S1"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("s2", "S2"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("s3", "S3"), StrategySource::Builtin).await.unwrap();

        registry.disable("s2").await.unwrap();

        let all = registry.list().await;
        assert_eq!(all.len(), 3);

        let enabled = registry.list_enabled().await;
        assert_eq!(enabled.len(), 2);
    }

    #[tokio::test]
    async fn test_strategy_family_from_str() {
        assert_eq!(StrategyFamily::from("zapret"), StrategyFamily::Zapret);
        assert_eq!(StrategyFamily::from("ZAPRET"), StrategyFamily::Zapret);
        assert_eq!(StrategyFamily::from("vless"), StrategyFamily::Vless);
        assert_eq!(StrategyFamily::from("shadowsocks"), StrategyFamily::Shadowsocks);
        assert_eq!(StrategyFamily::from("unknown"), StrategyFamily::Custom);
    }
}

//! Plugin-related Tauri commands

use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tracing::{info, warn};

use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::paths::get_app_data_dir;
use crate::state::AppState;
use crate::plugins::js_loader;
use crate::plugins::manifest::{LoadedPluginInfo, PluginManifest, ServiceDefinition};
use crate::plugins::checker::{check_service_endpoints, ServiceStatus as PluginServiceStatus};

// ============================================================================
// Plugin Commands (JS Plugins)
// ============================================================================

/// Plugin state stored in plugins_state.json (mirrors scripts.rs structure)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct PluginsStateConfig {
    #[serde(default)]
    plugins: HashMap<String, PluginStateEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PluginStateEntry {
    enabled: bool,
}

/// Load saved plugin states from plugins_state.json
async fn load_saved_plugin_states() -> HashMap<String, bool> {
    let config_path = get_app_data_dir().join("plugins_state.json");
    
    if !tokio::fs::try_exists(&config_path).await.unwrap_or(false) {
        return HashMap::new();
    }
    
    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => {
            match serde_json::from_str::<PluginsStateConfig>(&content) {
                Ok(config) => {
                    config.plugins
                        .into_iter()
                        .map(|(id, state)| (id, state.enabled))
                        .collect()
                }
                Err(e) => {
                    warn!(error = %e, "Failed to parse plugins_state.json");
                    HashMap::new()
                }
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to read plugins_state.json");
            HashMap::new()
        }
    }
}

/// Get plugins directory path
#[tauri::command]
pub async fn get_plugins_dir(state: State<'_, Arc<AppState>>) -> Result<String, IsolateError> {
    Ok(state.plugins_dir.display().to_string())
}

/// Scan for plugins and return their paths
#[tauri::command]
pub async fn scan_plugin_directories(state: State<'_, Arc<AppState>>) -> Result<Vec<String>, IsolateError> {
    info!("Scanning for plugins");
    
    js_loader::scan_plugins_async(&state.plugins_dir)
        .await
        .map(|paths| paths.iter().map(|p| p.display().to_string()).collect())
        .io_context("Failed to scan plugins")
}

/// Load plugin manifest (plugin.json)
#[tauri::command]
pub async fn load_plugin_manifest(path: String) -> Result<PluginManifest, IsolateError> {
    info!(path = %path, "Loading plugin manifest");
    let plugin_dir = std::path::PathBuf::from(&path);
    
    js_loader::load_manifest_async(&plugin_dir)
        .await
        .config_context("Failed to load plugin manifest")
}

/// Get all plugins with their info
/// 
/// Returns plugins with their enabled state synchronized from plugins_state.json.
/// This ensures frontend receives the correct enabled state without needing
/// a separate call to get_all_plugin_states.
#[tauri::command]
pub async fn get_all_plugins_cmd(state: State<'_, Arc<AppState>>) -> Result<Vec<LoadedPluginInfo>, IsolateError> {
    info!("Getting all plugins");
    
    // Load plugins from disk
    let mut plugins = js_loader::get_all_plugins_async(&state.plugins_dir).await;
    
    // Load saved states
    let saved_states = load_saved_plugin_states().await;
    
    // Apply saved states to plugins
    for plugin in &mut plugins {
        if let Some(&saved_enabled) = saved_states.get(&plugin.manifest.id) {
            plugin.enabled = saved_enabled;
        }
        // If no saved state, keep the default (true for valid plugins, false for errored)
    }
    
    info!(count = plugins.len(), saved_states = saved_states.len(), "Plugins loaded with saved states");
    Ok(plugins)
}

/// Get all services from service-checker plugins
#[tauri::command]
pub async fn get_plugin_services(state: State<'_, Arc<AppState>>) -> Result<Vec<ServiceDefinition>, IsolateError> {
    info!("Getting plugin services");
    Ok(crate::plugins::get_all_services_async(&state.plugins_dir).await)
}

/// Check a specific service from plugins by ID
#[tauri::command]
pub async fn check_plugin_service(service_id: String, state: State<'_, Arc<AppState>>) -> Result<PluginServiceStatus, IsolateError> {
    info!(service_id = %service_id, "Checking plugin service");
    let services = crate::plugins::get_all_services_async(&state.plugins_dir).await;
    
    let service = services.iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| IsolateError::StrategyNotFound(format!("Service not found: {}", service_id)))?;
    
    let mut status = check_service_endpoints(&service.endpoints).await;
    status.service_id = service.id.clone();
    Ok(status)
}

/// Check all services from plugins
#[tauri::command]
pub async fn check_all_plugin_services(state: State<'_, Arc<AppState>>) -> Result<Vec<PluginServiceStatus>, IsolateError> {
    info!("Checking all plugin services");
    let services = crate::plugins::get_all_services_async(&state.plugins_dir).await;
    
    let mut results = Vec::new();
    for service in services {
        let mut status = check_service_endpoints(&service.endpoints).await;
        status.service_id = service.id.clone();
        results.push(status);
    }
    
    Ok(results)
}

// ============================================================================
// Strategy Registry Commands (Plugin Strategies)
// ============================================================================

use crate::plugins::{
    RegisteredStrategy, StrategyFamily as PluginStrategyFamily,
};
use crate::plugins::sandbox::{SandboxViolation, VIOLATION_STORE};

/// Get all strategies from plugins
#[tauri::command]
pub async fn get_plugin_strategies(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<RegisteredStrategy>, IsolateError> {
    info!("Getting plugin strategies");
    
    let strategies = state.strategy_registry.get_plugin_strategies().await;
    Ok(strategies)
}

/// Get all registered strategies (builtin + plugins)
#[tauri::command]
pub async fn get_all_registered_strategies(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<RegisteredStrategy>, IsolateError> {
    info!("Getting all registered strategies");
    
    let strategies = state.strategy_registry.list_enabled().await;
    Ok(strategies)
}

/// Get strategies for a specific service
#[tauri::command]
pub async fn get_strategies_for_service(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<Vec<RegisteredStrategy>, IsolateError> {
    info!(service_id = %service_id, "Getting strategies for service");
    
    let strategies = state.strategy_registry.get_by_service(&service_id).await;
    Ok(strategies)
}

/// Get strategies by family (zapret, vless, etc.)
#[tauri::command]
pub async fn get_strategies_by_family(
    state: State<'_, Arc<AppState>>,
    family: String,
) -> Result<Vec<RegisteredStrategy>, IsolateError> {
    info!(family = %family, "Getting strategies by family");
    
    let family = match family.to_lowercase().as_str() {
        "zapret" => PluginStrategyFamily::Zapret,
        "vless" => PluginStrategyFamily::Vless,
        "shadowsocks" => PluginStrategyFamily::Shadowsocks,
        _ => PluginStrategyFamily::Custom,
    };
    
    let strategies = state.strategy_registry.get_by_family(family).await;
    Ok(strategies)
}

/// Get a specific strategy by ID
#[tauri::command]
pub async fn get_registered_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<Option<RegisteredStrategy>, IsolateError> {
    info!(strategy_id = %strategy_id, "Getting registered strategy");
    
    let strategy = state.strategy_registry.get(&strategy_id).await;
    Ok(strategy)
}

/// Enable a strategy
#[tauri::command]
pub async fn enable_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<(), IsolateError> {
    info!(strategy_id = %strategy_id, "Enabling strategy");
    
    state
        .strategy_registry
        .enable(&strategy_id)
        .await
        .strategy_context("Failed to enable strategy")
}

/// Disable a strategy
#[tauri::command]
pub async fn disable_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<(), IsolateError> {
    info!(strategy_id = %strategy_id, "Disabling strategy");
    
    state
        .strategy_registry
        .disable(&strategy_id)
        .await
        .strategy_context("Failed to disable strategy")
}

/// Strategy registry statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct StrategyRegistryStats {
    pub total: usize,
    pub enabled: usize,
    pub plugin_strategies: usize,
    pub builtin_strategies: usize,
}

/// Get strategy registry statistics
#[tauri::command]
pub async fn get_strategy_registry_stats(
    state: State<'_, Arc<AppState>>,
) -> Result<StrategyRegistryStats, IsolateError> {
    info!("Getting strategy registry stats");
    
    let total = state.strategy_registry.count().await;
    let enabled = state.strategy_registry.enabled_count().await;
    let plugin_strategies = state.strategy_registry.get_plugin_strategies().await.len();
    let builtin_strategies = state.strategy_registry.get_builtin_strategies().await.len();
    
    Ok(StrategyRegistryStats {
        total,
        enabled,
        plugin_strategies,
        builtin_strategies,
    })
}

/// Reload strategies from plugins
#[tauri::command]
pub async fn reload_plugin_strategies(
    state: State<'_, Arc<AppState>>,
) -> Result<usize, IsolateError> {
    info!("Reloading plugin strategies");
    
    state
        .strategy_registry
        .load_from_plugins()
        .await
        .strategy_context("Failed to reload strategies")
}

// ============================================================================
// Plugin Hot Reload Commands
// ============================================================================

/// Result of reloading all plugins
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReloadPluginsResult {
    pub plugins_loaded: usize,
    pub hostlists_loaded: usize,
    pub strategies_loaded: usize,
    pub services_loaded: usize,
}

/// Reload all plugins (hot reload)
///
/// Rescans the plugins directory and reloads all plugin manifests.
/// This allows adding/removing/updating plugins without restarting the app.
#[tauri::command]
pub async fn reload_plugins(
    state: State<'_, Arc<AppState>>,
) -> Result<ReloadPluginsResult, IsolateError> {
    info!("Hot reloading all plugins");
    
    // 1. Reload plugin manager
    let plugin_count = state
        .plugin_manager
        .reload_all()
        .await
        .config_context("Failed to reload plugins")?;
    
    // 2. Reload hostlists from plugins
    let hostlist_count = reload_hostlists_internal(&state).await.unwrap_or(0);
    
    // 3. Reload strategies from plugins
    let strategy_count = reload_strategies_internal(&state).await.unwrap_or(0);
    
    // 4. Reload services from plugins
    let service_count = reload_services_internal(&state).await.unwrap_or(0);
    
    info!(
        plugins = plugin_count,
        hostlists = hostlist_count,
        strategies = strategy_count,
        services = service_count,
        "Plugins hot reloaded"
    );
    
    Ok(ReloadPluginsResult {
        plugins_loaded: plugin_count,
        hostlists_loaded: hostlist_count,
        strategies_loaded: strategy_count,
        services_loaded: service_count,
    })
}

/// Reload a single plugin by ID
///
/// Reloads a specific plugin's manifest and updates registries.
#[tauri::command]
pub async fn reload_plugin(
    state: State<'_, Arc<AppState>>,
    plugin_id: String,
) -> Result<bool, IsolateError> {
    info!(plugin_id = %plugin_id, "Hot reloading single plugin");
    
    // Check if plugin exists
    let plugin = state.plugin_manager.get(&plugin_id).await;
    if plugin.is_none() {
        return Err(IsolateError::StrategyNotFound(format!("Plugin not found: {}", plugin_id)));
    }
    
    // Unload and reload the plugin
    let _ = state.plugin_manager.unload(&plugin_id).await;
    
    state
        .plugin_manager
        .load(&plugin_id)
        .await
        .config_context("Failed to reload plugin")?;
    
    // Reload associated registries
    let _ = reload_hostlists_internal(&state).await;
    let _ = reload_strategies_internal(&state).await;
    let _ = reload_services_internal(&state).await;
    
    info!(plugin_id = %plugin_id, "Plugin hot reloaded");
    Ok(true)
}

// Internal helper to reload hostlists
async fn reload_hostlists_internal(state: &State<'_, Arc<AppState>>) -> Result<usize, String> {
    use crate::plugins::{get_all_plugins_async, PluginType};
    
    let plugins = get_all_plugins_async(&state.plugins_dir).await;
    let mut loaded_count = 0;
    
    for plugin_info in plugins {
        if !plugin_info.enabled || plugin_info.error.is_some() {
            continue;
        }
        
        let manifest = &plugin_info.manifest;
        let plugin_path = std::path::PathBuf::from(&plugin_info.path);
        
        // Load hostlists from contributes.hostlists
        for hostlist_def in &manifest.contributes.hostlists {
            if state.hostlist_registry
                .register(&manifest.id, plugin_path.clone(), hostlist_def.clone())
                .await
                .is_ok()
            {
                loaded_count += 1;
            }
        }
        
        // Legacy hostlist field
        if manifest.plugin_type == PluginType::HostlistProvider {
            if let Some(ref hostlist_def) = manifest.hostlist {
                if state.hostlist_registry
                    .register(&manifest.id, plugin_path.clone(), hostlist_def.clone())
                    .await
                    .is_ok()
                {
                    loaded_count += 1;
                }
            }
        }
    }
    
    Ok(loaded_count)
}

// Internal helper to reload strategies
async fn reload_strategies_internal(state: &State<'_, Arc<AppState>>) -> Result<usize, String> {
    use crate::plugins::{
        get_all_plugins_async, PluginType, PluginStrategyDefinition, PluginStrategyConfig,
        StrategyFamily, StrategySource,
    };
    
    let plugins = get_all_plugins_async(&state.plugins_dir).await;
    let mut loaded_count = 0;
    
    for plugin_info in plugins {
        if !plugin_info.enabled || plugin_info.error.is_some() {
            continue;
        }
        
        let manifest = &plugin_info.manifest;
        
        if manifest.plugin_type != PluginType::StrategyProvider {
            continue;
        }
        
        // Load strategies from contributes.strategies
        for strategy_def in &manifest.contributes.strategies {
            let plugin_strategy = PluginStrategyDefinition {
                id: strategy_def.id.clone(),
                name: strategy_def.name.clone(),
                description: None,
                family: StrategyFamily::from(strategy_def.family.as_str()),
                engine: "winws".to_string(),
                target_services: Vec::new(),
                priority: 0,
                config: PluginStrategyConfig::default(),
                author: None,
                label: None,
                source_plugin: Some(manifest.id.clone()),
            };
            
            if state.strategy_registry
                .register(
                    plugin_strategy,
                    StrategySource::Plugin { plugin_id: manifest.id.clone() },
                )
                .await
                .is_ok()
            {
                loaded_count += 1;
            }
        }
        
        // Legacy strategy field
        if let Some(ref strategy_def) = manifest.strategy {
            let plugin_strategy = PluginStrategyDefinition {
                id: strategy_def.id.clone(),
                name: strategy_def.name.clone(),
                description: None,
                family: StrategyFamily::from(strategy_def.family.as_str()),
                engine: "winws".to_string(),
                target_services: Vec::new(),
                priority: 0,
                config: PluginStrategyConfig::default(),
                author: None,
                label: None,
                source_plugin: Some(manifest.id.clone()),
            };
            
            if state.strategy_registry
                .register(
                    plugin_strategy,
                    StrategySource::Plugin { plugin_id: manifest.id.clone() },
                )
                .await
                .is_ok()
            {
                loaded_count += 1;
            }
        }
    }
    
    Ok(loaded_count)
}

// Internal helper to reload services
async fn reload_services_internal(state: &State<'_, Arc<AppState>>) -> Result<usize, String> {
    // Reload services from plugins
    match state.service_registry.load_from_plugins(&state.plugins_dir).await {
        Ok(count) => Ok(count),
        Err(e) => {
            warn!(error = %e, "Failed to reload services from plugins");
            Ok(0)
        }
    }
}

// ============================================================================
// Plugin Sandbox Violations Commands
// ============================================================================

/// Get sandbox violations for a specific plugin
/// 
/// Returns the list of security violations recorded for the plugin,
/// sorted by timestamp (newest first). Maximum 100 violations per plugin.
#[tauri::command]
pub async fn get_plugin_violations(plugin_id: String) -> Result<Vec<SandboxViolation>, IsolateError> {
    info!(plugin_id = %plugin_id, "Getting plugin violations");
    Ok(VIOLATION_STORE.get_violations(&plugin_id))
}

/// Get violation count for a specific plugin
#[tauri::command]
pub async fn get_plugin_violation_count(plugin_id: String) -> Result<usize, IsolateError> {
    Ok(VIOLATION_STORE.count(&plugin_id))
}

/// Clear violations for a specific plugin
#[tauri::command]
pub async fn clear_plugin_violations(plugin_id: String) -> Result<(), IsolateError> {
    info!(plugin_id = %plugin_id, "Clearing plugin violations");
    VIOLATION_STORE.clear(&plugin_id);
    Ok(())
}

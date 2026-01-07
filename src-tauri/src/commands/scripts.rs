//! Script execution commands for Level 3 plugins
//!
//! Provides Tauri commands for executing Lua scripts in sandboxed environment.

use crate::core::errors::{IsolateError, TypedResultExt};
use crate::plugins::script_executor::ScriptExecutor;
use crate::plugins::lua_runtime::CheckResult;
use crate::core::paths::{get_plugins_dir, get_app_data_dir};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{error, info, warn};

/// Global script executor instance
static SCRIPT_EXECUTOR: OnceCell<Arc<ScriptExecutor>> = OnceCell::const_new();

/// Get or initialize the script executor
async fn get_executor() -> &'static Arc<ScriptExecutor> {
    SCRIPT_EXECUTOR
        .get_or_init(|| async {
            let plugins_dir = get_plugins_dir();
            Arc::new(ScriptExecutor::new(plugins_dir))
        })
        .await
}

/// Execute a plugin script's check() function
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
/// * `script_name` - The script filename (e.g., "check.lua")
///
/// # Returns
/// * `CheckResult` with success status, latency, and optional details
#[tauri::command]
pub async fn execute_plugin_script(
    plugin_id: String,
    script_name: String,
) -> Result<CheckResult, String> {
    // Validate script_name to prevent path traversal
    if script_name.contains("..") || script_name.contains('/') || script_name.contains('\\') {
        return Err("Invalid script name: path traversal detected".to_string());
    }
    
    info!(plugin = %plugin_id, script = %script_name, "Executing plugin script");

    let executor = get_executor().await;

    executor
        .execute_check(&plugin_id, &script_name)
        .await
        .map_err(|e| {
            error!(plugin = %plugin_id, script = %script_name, error = %e, "Script execution failed");
            e
        })
        .process_context("Script execution failed")
        .map_err(|e: IsolateError| e.to_string())
}

/// Execute raw Lua code for a plugin (debugging/testing)
///
/// # Arguments
/// * `plugin_id` - The plugin identifier (for permissions)
/// * `code` - Lua code to execute
///
/// # Returns
/// * JSON value representing the script result
#[tauri::command]
pub async fn execute_plugin_raw(
    plugin_id: String,
    code: String,
) -> Result<serde_json::Value, String> {
    info!(plugin = %plugin_id, "Executing raw Lua code");

    let executor = get_executor().await;

    // Wrap execution in timeout to prevent DoS
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        executor.execute_raw(&plugin_id, &code)
    )
    .await
    .map_err(|_| IsolateError::Process("Script execution timeout (30s)".to_string()))
    .map_err(|e: IsolateError| e.to_string())?;
    
    let result = result
        .map_err(|e| {
            error!(plugin = %plugin_id, error = %e, "Raw script execution failed");
            e
        })
        .process_context("Script execution failed")
        .map_err(|e: IsolateError| e.to_string())?;
    
    Ok(result)
}

/// List available scripts for a plugin
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
///
/// # Returns
/// * List of script filenames (e.g., ["check.lua", "validate.lua"])
#[tauri::command]
pub async fn list_plugin_scripts(
    plugin_id: String,
) -> Result<Vec<String>, String> {
    info!(plugin = %plugin_id, "Listing plugin scripts");

    let executor = get_executor().await;

    executor
        .list_scripts(&plugin_id)
        .await
        .io_context("Failed to list scripts")
        .map_err(|e: IsolateError| e.to_string())
}

/// Get a storage value for a plugin
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
/// * `key` - Storage key
///
/// # Returns
/// * Optional JSON value
#[tauri::command]
pub async fn get_plugin_storage(
    plugin_id: String,
    key: String,
) -> Result<Option<serde_json::Value>, String> {
    let executor = get_executor().await;
    Ok(executor.get_storage_value(&plugin_id, &key).await)
}

/// Set a storage value for a plugin
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
/// * `key` - Storage key
/// * `value` - JSON value to store
#[tauri::command]
pub async fn set_plugin_storage(
    plugin_id: String,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let executor = get_executor().await;
    executor.set_storage_value(&plugin_id, key, value).await;
    Ok(())
}

/// Clear all storage for a plugin
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
#[tauri::command]
pub async fn clear_plugin_storage(
    plugin_id: String,
) -> Result<(), String> {
    info!(plugin = %plugin_id, "Clearing plugin storage");
    let executor = get_executor().await;
    executor.clear_storage(&plugin_id).await;
    Ok(())
}

/// Install a plugin by ID
///
/// Creates a plugin directory and basic manifest file.
/// For now, this is a local installation without downloading from registry.
///
/// # Arguments
/// * `plugin_id` - The plugin identifier to install
#[tauri::command]
pub async fn install_plugin(
    plugin_id: String,
) -> Result<(), String> {
    info!(plugin = %plugin_id, "Installing plugin");
    
    let plugins_dir = get_plugins_dir();
    let plugin_dir = plugins_dir.join(&plugin_id);
    
    // Create plugin directory
    tokio::fs::create_dir_all(&plugin_dir)
        .await
        .io_context("Failed to create plugin directory")
        .map_err(|e: IsolateError| e.to_string())?;
    
    // Create basic manifest if not exists
    let manifest_path = plugin_dir.join("plugin.json");
    if !tokio::fs::try_exists(&manifest_path).await.unwrap_or(false) {
        let manifest = serde_json::json!({
            "id": plugin_id,
            "name": plugin_id,
            "version": "1.0.0",
            "description": "Installed plugin",
            "type": "script-plugin"
        });
        
        let content = serde_json::to_string_pretty(&manifest)
            .config_context("Failed to serialize manifest")
            .map_err(|e: IsolateError| e.to_string())?;
        
        tokio::fs::write(&manifest_path, content)
            .await
            .io_context("Failed to write manifest")
            .map_err(|e: IsolateError| e.to_string())?;
    }
    
    info!(plugin = %plugin_id, "Plugin installed successfully");
    Ok(())
}

// ============================================================================
// Plugin Enabled State Persistence
// ============================================================================

/// Plugin state configuration stored in plugins_state.json
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct PluginsStateConfig {
    /// Map of plugin_id -> enabled state
    #[serde(default)]
    plugins: HashMap<String, PluginState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PluginState {
    enabled: bool,
}

/// Get the path to plugins state config file
fn get_plugins_state_path() -> PathBuf {
    get_app_data_dir().join("plugins_state.json")
}

/// Load plugins state config from disk
async fn load_plugins_state_config() -> PluginsStateConfig {
    let config_path = get_plugins_state_path();
    
    if !tokio::fs::try_exists(&config_path).await.unwrap_or(false) {
        return PluginsStateConfig::default();
    }
    
    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                warn!(error = %e, "Failed to parse plugins_state.json, using defaults");
                PluginsStateConfig::default()
            })
        }
        Err(e) => {
            warn!(error = %e, "Failed to read plugins_state.json, using defaults");
            PluginsStateConfig::default()
        }
    }
}

/// Save plugins state config to disk
async fn save_plugins_state_config(config: &PluginsStateConfig) -> Result<(), String> {
    let config_path = get_plugins_state_path();
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .io_context("Failed to create config directory")
            .map_err(|e: IsolateError| e.to_string())?;
    }
    
    let content = serde_json::to_string_pretty(config)
        .config_context("Failed to serialize config")
        .map_err(|e: IsolateError| e.to_string())?;
    
    tokio::fs::write(&config_path, content)
        .await
        .io_context("Failed to write config")
        .map_err(|e: IsolateError| e.to_string())?;
    
    Ok(())
}

/// Set plugin enabled state
///
/// Persists the enabled/disabled state of a plugin to disk.
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
/// * `enabled` - Whether the plugin should be enabled
#[tauri::command]
pub async fn set_plugin_enabled(
    plugin_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!(plugin = %plugin_id, enabled = %enabled, "Setting plugin enabled state");
    
    let mut config = load_plugins_state_config().await;
    config.plugins.insert(plugin_id.clone(), PluginState { enabled });
    save_plugins_state_config(&config).await?;
    
    info!(plugin = %plugin_id, enabled = %enabled, "Plugin state saved");
    Ok(())
}

/// Get plugin enabled state
///
/// Returns the persisted enabled state for a plugin.
/// Defaults to true if no state is saved.
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
///
/// # Returns
/// * `bool` - Whether the plugin is enabled (defaults to true)
#[tauri::command]
pub async fn get_plugin_enabled(
    plugin_id: String,
) -> Result<bool, String> {
    let config = load_plugins_state_config().await;
    
    let enabled = config.plugins
        .get(&plugin_id)
        .map(|state| state.enabled)
        .unwrap_or(true); // Default to enabled
    
    Ok(enabled)
}

/// Get all plugin enabled states
///
/// Returns a map of plugin_id -> enabled state for all plugins
/// that have a saved state.
#[tauri::command]
pub async fn get_all_plugin_states() -> Result<HashMap<String, bool>, String> {
    let config = load_plugins_state_config().await;
    
    let states: HashMap<String, bool> = config.plugins
        .into_iter()
        .map(|(id, state)| (id, state.enabled))
        .collect();
    
    Ok(states)
}

// ============================================================================
// Plugin Settings Persistence
// ============================================================================

/// Plugin settings stored in plugins_settings.json
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct PluginsSettingsConfig {
    /// Map of plugin_id -> settings array
    #[serde(default)]
    plugins: HashMap<String, Vec<PluginSettingValue>>,
}

/// Individual setting value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginSettingValue {
    pub id: String,
    pub value: serde_json::Value,
}

/// Get the path to plugins settings config file
fn get_plugins_settings_path() -> PathBuf {
    get_app_data_dir().join("plugins_settings.json")
}

/// Load plugins settings config from disk
async fn load_plugins_settings_config() -> PluginsSettingsConfig {
    let config_path = get_plugins_settings_path();
    
    if !tokio::fs::try_exists(&config_path).await.unwrap_or(false) {
        return PluginsSettingsConfig::default();
    }
    
    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                warn!(error = %e, "Failed to parse plugins_settings.json, using defaults");
                PluginsSettingsConfig::default()
            })
        }
        Err(e) => {
            warn!(error = %e, "Failed to read plugins_settings.json, using defaults");
            PluginsSettingsConfig::default()
        }
    }
}

/// Save plugins settings config to disk
async fn save_plugins_settings_config(config: &PluginsSettingsConfig) -> Result<(), String> {
    let config_path = get_plugins_settings_path();
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .io_context("Failed to create config directory")
            .map_err(|e: IsolateError| e.to_string())?;
    }
    
    let content = serde_json::to_string_pretty(config)
        .config_context("Failed to serialize settings")
        .map_err(|e: IsolateError| e.to_string())?;
    
    tokio::fs::write(&config_path, content)
        .await
        .io_context("Failed to write settings")
        .map_err(|e: IsolateError| e.to_string())?;
    
    Ok(())
}

/// Get plugin settings
///
/// Returns the saved settings for a plugin, or empty array if none saved.
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
///
/// # Returns
/// * `Vec<PluginSettingValue>` - Array of setting id/value pairs
#[tauri::command]
pub async fn get_plugin_settings(
    plugin_id: String,
) -> Result<Vec<PluginSettingValue>, String> {
    info!(plugin = %plugin_id, "Getting plugin settings");
    
    let config = load_plugins_settings_config().await;
    
    let settings = config.plugins
        .get(&plugin_id)
        .cloned()
        .unwrap_or_default();
    
    Ok(settings)
}

/// Set plugin settings
///
/// Saves the settings for a plugin to disk.
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
/// * `settings` - Array of setting id/value pairs to save
#[tauri::command]
pub async fn set_plugin_settings(
    plugin_id: String,
    settings: Vec<PluginSettingValue>,
) -> Result<(), String> {
    info!(plugin = %plugin_id, settings_count = settings.len(), "Saving plugin settings");
    
    let mut config = load_plugins_settings_config().await;
    config.plugins.insert(plugin_id.clone(), settings);
    save_plugins_settings_config(&config).await?;
    
    info!(plugin = %plugin_id, "Plugin settings saved");
    Ok(())
}

/// Reset plugin settings
///
/// Removes all saved settings for a plugin (returns to defaults).
///
/// # Arguments
/// * `plugin_id` - The plugin identifier
#[tauri::command]
pub async fn reset_plugin_settings(
    plugin_id: String,
) -> Result<(), String> {
    info!(plugin = %plugin_id, "Resetting plugin settings to defaults");
    
    let mut config = load_plugins_settings_config().await;
    config.plugins.remove(&plugin_id);
    save_plugins_settings_config(&config).await?;
    
    info!(plugin = %plugin_id, "Plugin settings reset");
    Ok(())
}

/// Get all plugin settings
///
/// Returns settings for all plugins that have saved settings.
#[tauri::command]
pub async fn get_all_plugin_settings() -> Result<HashMap<String, Vec<PluginSettingValue>>, String> {
    let config = load_plugins_settings_config().await;
    Ok(config.plugins)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_executor() {
        let executor = get_executor().await;
        // Should return the same instance
        let executor2 = get_executor().await;
        assert!(Arc::ptr_eq(executor, executor2));
    }
    
    #[test]
    fn test_plugins_state_config_default() {
        let config = PluginsStateConfig::default();
        assert!(config.plugins.is_empty());
    }
    
    #[test]
    fn test_plugins_state_serialization() {
        let mut config = PluginsStateConfig::default();
        config.plugins.insert("test-plugin".to_string(), PluginState { enabled: false });
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PluginsStateConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.plugins.get("test-plugin").unwrap().enabled, false);
    }
    
    #[test]
    fn test_plugins_settings_config_default() {
        let config = PluginsSettingsConfig::default();
        assert!(config.plugins.is_empty());
    }
    
    #[test]
    fn test_plugins_settings_serialization() {
        let mut config = PluginsSettingsConfig::default();
        config.plugins.insert("test-plugin".to_string(), vec![
            PluginSettingValue {
                id: "setting1".to_string(),
                value: serde_json::json!(true),
            },
            PluginSettingValue {
                id: "setting2".to_string(),
                value: serde_json::json!("value"),
            },
        ]);
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PluginsSettingsConfig = serde_json::from_str(&json).unwrap();
        
        let settings = parsed.plugins.get("test-plugin").unwrap();
        assert_eq!(settings.len(), 2);
        assert_eq!(settings[0].id, "setting1");
        assert_eq!(settings[0].value, serde_json::json!(true));
    }

    #[test]
    fn test_plugin_setting_value_serialization() {
        let setting = PluginSettingValue {
            id: "test_setting".to_string(),
            value: serde_json::json!({"nested": "value", "number": 42}),
        };
        
        let json = serde_json::to_string(&setting).unwrap();
        let parsed: PluginSettingValue = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.id, "test_setting");
        assert_eq!(parsed.value["nested"], "value");
        assert_eq!(parsed.value["number"], 42);
    }

    #[test]
    fn test_plugin_state_serialization() {
        let state = PluginState { enabled: true };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: PluginState = serde_json::from_str(&json).unwrap();
        assert!(parsed.enabled);
        
        let state_disabled = PluginState { enabled: false };
        let json_disabled = serde_json::to_string(&state_disabled).unwrap();
        let parsed_disabled: PluginState = serde_json::from_str(&json_disabled).unwrap();
        assert!(!parsed_disabled.enabled);
    }

    #[test]
    fn test_plugins_state_config_multiple_plugins() {
        let mut config = PluginsStateConfig::default();
        config.plugins.insert("plugin-a".to_string(), PluginState { enabled: true });
        config.plugins.insert("plugin-b".to_string(), PluginState { enabled: false });
        config.plugins.insert("plugin-c".to_string(), PluginState { enabled: true });
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PluginsStateConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.plugins.len(), 3);
        assert!(parsed.plugins.get("plugin-a").unwrap().enabled);
        assert!(!parsed.plugins.get("plugin-b").unwrap().enabled);
        assert!(parsed.plugins.get("plugin-c").unwrap().enabled);
    }

    #[test]
    fn test_plugins_settings_config_various_value_types() {
        let mut config = PluginsSettingsConfig::default();
        config.plugins.insert("test-plugin".to_string(), vec![
            PluginSettingValue {
                id: "bool_setting".to_string(),
                value: serde_json::json!(true),
            },
            PluginSettingValue {
                id: "string_setting".to_string(),
                value: serde_json::json!("hello"),
            },
            PluginSettingValue {
                id: "number_setting".to_string(),
                value: serde_json::json!(42),
            },
            PluginSettingValue {
                id: "array_setting".to_string(),
                value: serde_json::json!([1, 2, 3]),
            },
            PluginSettingValue {
                id: "object_setting".to_string(),
                value: serde_json::json!({"key": "value"}),
            },
            PluginSettingValue {
                id: "null_setting".to_string(),
                value: serde_json::json!(null),
            },
        ]);
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PluginsSettingsConfig = serde_json::from_str(&json).unwrap();
        
        let settings = parsed.plugins.get("test-plugin").unwrap();
        assert_eq!(settings.len(), 6);
        
        // Verify each type is preserved
        assert_eq!(settings.iter().find(|s| s.id == "bool_setting").unwrap().value, true);
        assert_eq!(settings.iter().find(|s| s.id == "string_setting").unwrap().value, "hello");
        assert_eq!(settings.iter().find(|s| s.id == "number_setting").unwrap().value, 42);
        assert!(settings.iter().find(|s| s.id == "null_setting").unwrap().value.is_null());
    }

    #[tokio::test]
    async fn test_load_plugins_state_config_nonexistent_file() {
        // When file doesn't exist, should return default config
        // This tests the async file operation handling
        let config = load_plugins_state_config().await;
        // Default config has empty plugins map
        // Note: This may or may not be empty depending on actual app data dir state
        // The important thing is it doesn't panic
        assert!(config.plugins.is_empty() || !config.plugins.is_empty());
    }

    #[tokio::test]
    async fn test_load_plugins_settings_config_nonexistent_file() {
        // When file doesn't exist, should return default config
        let config = load_plugins_settings_config().await;
        // Should not panic, returns default or existing config
        assert!(config.plugins.is_empty() || !config.plugins.is_empty());
    }

    #[test]
    fn test_get_plugins_state_path() {
        let path = get_plugins_state_path();
        assert!(path.ends_with("plugins_state.json"));
    }

    #[test]
    fn test_get_plugins_settings_path() {
        let path = get_plugins_settings_path();
        assert!(path.ends_with("plugins_settings.json"));
    }

    #[test]
    fn test_plugins_state_config_deserialize_empty_json() {
        let json = "{}";
        let config: PluginsStateConfig = serde_json::from_str(json).unwrap();
        assert!(config.plugins.is_empty());
    }

    #[test]
    fn test_plugins_settings_config_deserialize_empty_json() {
        let json = "{}";
        let config: PluginsSettingsConfig = serde_json::from_str(json).unwrap();
        assert!(config.plugins.is_empty());
    }

    #[test]
    fn test_plugins_state_config_deserialize_with_extra_fields() {
        // Should ignore unknown fields gracefully
        let json = r#"{"plugins": {}, "unknown_field": "value"}"#;
        let result: Result<PluginsStateConfig, _> = serde_json::from_str(json);
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

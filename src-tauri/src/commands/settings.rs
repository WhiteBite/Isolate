//! Settings management commands

use tauri::AppHandle;
use tracing::info;

use crate::commands::state_guard::get_state_or_error;
use crate::commands::validation::validate_not_empty;
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::models::{Settings, ServiceWithState, WinDivertMode, ProxyConfig, GameFilterMode};
use crate::core::storage::RoutingRule;

/// Get user settings
#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Getting user settings");
    
    state
        .storage
        .get_settings()
        .await
        .storage_context("Failed to get settings")
}

/// Save user settings
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    settings: Settings,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Saving user settings");
    
    state
        .storage
        .save_settings(&settings)
        .await
        .storage_context("Failed to save settings")
}

/// Get services with their enabled/disabled state
#[tauri::command]
pub async fn get_services_settings(
    app: AppHandle,
) -> Result<Vec<ServiceWithState>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Loading services with settings");
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .config_context("Failed to load services")?;
    
    let mut services_with_state = Vec::new();
    
    for service in services_map.into_values() {
        let enabled = state
            .storage
            .get_service_enabled(&service.id)
            .await
            .unwrap_or(service.enabled_by_default);
        
        services_with_state.push(ServiceWithState {
            id: service.id,
            name: service.name,
            enabled,
            critical: service.critical,
        });
    }
    
    Ok(services_with_state)
}

/// Toggle a service's enabled state
#[tauri::command]
pub async fn toggle_service(
    app: AppHandle,
    service_id: String,
    enabled: bool,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(service_id = %service_id, enabled, "Toggling service");
    
    state
        .storage
        .set_service_enabled(&service_id, enabled)
        .await
        .storage_context("Failed to toggle service")
}

/// Get a setting by key
#[tauri::command]
pub async fn get_setting(
    app: AppHandle,
    key: String,
) -> Result<serde_json::Value, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(key = %key, "Getting setting");
    
    let value: Option<serde_json::Value> = state
        .storage
        .get_setting(&key)
        .await
        .storage_context("Failed to get setting")?;
    
    Ok(value.unwrap_or(serde_json::Value::Null))
}

/// Set a setting by key
#[tauri::command]
pub async fn set_setting(
    app: AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    validate_not_empty(&key, "Setting key")?;
    
    info!(key = %key, "Setting setting");
    
    state
        .storage
        .set_setting(&key, &value)
        .await
        .storage_context("Failed to set setting")
}

/// Get WinDivert operation mode
/// 
/// Returns the current WinDivert mode: "normal", "autottl", or "autohostlist"
#[tauri::command]
pub async fn get_windivert_mode(app: AppHandle) -> Result<String, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Getting WinDivert mode");
    
    let settings = state
        .storage
        .get_settings()
        .await
        .storage_context("Failed to get settings")?;
    
    Ok(settings.windivert_mode.to_string())
}

/// Set WinDivert operation mode
/// 
/// # Arguments
/// * `mode` - One of: "normal", "autottl", "autohostlist"
/// 
/// # Effects
/// - normal: Standard winws operation with fixed parameters
/// - autottl: Adds --autottl flag to winws for automatic TTL detection
/// - autohostlist: Adds --autohostlist flag for automatic hostlist management
#[tauri::command]
pub async fn set_windivert_mode(
    app: AppHandle,
    mode: String,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    let windivert_mode = WinDivertMode::from_str(&mode);
    
    info!(mode = %windivert_mode, "Setting WinDivert mode");
    
    // Get current settings
    let mut settings = state
        .storage
        .get_settings()
        .await
        .storage_context("Failed to get settings")?;
    
    // Update WinDivert mode
    settings.windivert_mode = windivert_mode;
    
    // Save settings
    state
        .storage
        .save_settings(&settings)
        .await
        .storage_context("Failed to save settings")?;
    
    Ok(())
}

/// Get Game Filter mode
/// 
/// Returns the current game filter mode: "normal" or "gaming"
#[tauri::command]
pub async fn get_game_filter_mode(app: AppHandle) -> Result<String, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Getting Game Filter mode");
    
    let settings = state
        .storage
        .get_settings()
        .await
        .storage_context("Failed to get settings")?;
    
    Ok(settings.game_filter_mode.to_string())
}

/// Set Game Filter mode
/// 
/// # Arguments
/// * `mode` - One of: "normal", "gaming"
/// 
/// # Effects
/// - normal: Intercepts standard web ports (80, 443)
/// - gaming: Intercepts extended port range (1024-65535) for game traffic
/// 
/// Note: Changes take effect on next strategy apply. If a strategy is currently
/// running, it needs to be restarted for the new mode to take effect.
#[tauri::command]
pub async fn set_game_filter_mode(
    app: AppHandle,
    mode: String,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    let game_filter_mode = GameFilterMode::from_str(&mode);
    
    info!(mode = %game_filter_mode, "Setting Game Filter mode");
    
    // Get current settings
    let mut settings = state
        .storage
        .get_settings()
        .await
        .storage_context("Failed to get settings")?;
    
    // Update Game Filter mode
    settings.game_filter_mode = game_filter_mode;
    
    // Save settings
    state
        .storage
        .save_settings(&settings)
        .await
        .storage_context("Failed to save settings")?;
    
    Ok(())
}

// ============================================================================
// Config Export/Import Commands
// ============================================================================

/// Configuration export structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportedConfig {
    /// Config format version
    pub version: String,
    /// Export timestamp
    pub exported_at: String,
    /// Application settings
    pub settings: Settings,
    /// Proxy configurations (passwords excluded for security)
    pub proxies: Vec<ProxyConfig>,
    /// Routing rules
    pub routing_rules: Vec<RoutingRule>,
}

/// Export all configuration to JSON string
/// 
/// Exports settings, proxies (without passwords), and routing rules.
/// Passwords are excluded for security reasons.
#[tauri::command]
pub async fn export_config(app: AppHandle) -> Result<String, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Exporting configuration");
    
    // Get settings
    let settings = state
        .storage
        .get_settings()
        .await
        .storage_context("Failed to get settings")?;
    
    // Get proxies (strip passwords for security)
    let mut proxies = state
        .storage
        .get_all_proxies()
        .await
        .storage_context("Failed to get proxies")?;
    
    // Remove sensitive data from proxies
    for proxy in &mut proxies {
        proxy.password = None;
    }
    
    // Get routing rules
    let routing_rules = state
        .storage
        .get_routing_rules()
        .await
        .storage_context("Failed to get routing rules")?;
    
    let config = ExportedConfig {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        settings,
        proxies,
        routing_rules,
    };
    
    let json = serde_json::to_string_pretty(&config)
        .config_context("Failed to serialize config")?;
    
    info!(
        proxies_count = config.proxies.len(),
        rules_count = config.routing_rules.len(),
        "Configuration exported successfully"
    );
    
    Ok(json)
}

/// Import configuration from JSON string
/// 
/// Imports settings, proxies, and routing rules from a previously exported config.
/// Existing data will be merged (proxies and rules are added, settings are replaced).
#[tauri::command]
pub async fn import_config(
    app: AppHandle,
    config_json: String,
) -> Result<ImportResult, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    validate_not_empty(&config_json, "Config JSON")?;
    
    info!("Importing configuration");
    
    // Parse config
    let config: ExportedConfig = serde_json::from_str(&config_json)
        .config_context("Invalid config JSON")?;
    
    // Validate version
    if !config.version.starts_with("1.") {
        return Err(IsolateError::Validation(format!("Unsupported config version: {}", config.version)));
    }
    
    let mut result = ImportResult {
        settings_imported: false,
        proxies_imported: 0,
        proxies_skipped: 0,
        routing_rules_imported: 0,
        routing_rules_skipped: 0,
    };
    
    // Import settings
    if let Err(e) = state.storage.save_settings(&config.settings).await {
        info!(error = %e, "Failed to import settings, skipping");
    } else {
        result.settings_imported = true;
    }
    
    // Import proxies (skip duplicates by ID)
    for proxy in config.proxies {
        // Check if proxy already exists
        match state.storage.get_proxy(&proxy.id).await {
            Ok(Some(_)) => {
                // Proxy exists, skip
                result.proxies_skipped += 1;
            }
            Ok(None) => {
                // Proxy doesn't exist, import it
                if let Err(e) = state.storage.save_proxy(&proxy).await {
                    info!(id = %proxy.id, error = %e, "Failed to import proxy");
                    result.proxies_skipped += 1;
                } else {
                    result.proxies_imported += 1;
                }
            }
            Err(e) => {
                info!(id = %proxy.id, error = %e, "Error checking proxy existence");
                result.proxies_skipped += 1;
            }
        }
    }
    
    // Import routing rules (skip duplicates by ID)
    for rule in config.routing_rules {
        // Try to add the rule (will fail if ID exists due to unique constraint)
        match state.storage.add_routing_rule(&rule).await {
            Ok(_) => {
                result.routing_rules_imported += 1;
            }
            Err(e) => {
                info!(id = %rule.id, error = %e, "Failed to import routing rule (may already exist)");
                result.routing_rules_skipped += 1;
            }
        }
    }
    
    info!(
        settings = result.settings_imported,
        proxies_imported = result.proxies_imported,
        proxies_skipped = result.proxies_skipped,
        rules_imported = result.routing_rules_imported,
        rules_skipped = result.routing_rules_skipped,
        "Configuration import completed"
    );
    
    Ok(result)
}

/// Result of config import operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub settings_imported: bool,
    pub proxies_imported: u32,
    pub proxies_skipped: u32,
    pub routing_rules_imported: u32,
    pub routing_rules_skipped: u32,
}

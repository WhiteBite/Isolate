//! VLESS proxy commands
//!
//! Commands for managing VLESS proxy configurations and sing-box instances.

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::commands::validation::validate_not_empty;
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::models::VlessConfig;
use crate::state::AppState;

/// Storage key for VLESS configs
const VLESS_CONFIGS_KEY: &str = "vless_configs";

// ============================================================================
// VLESS Config Management Commands
// ============================================================================

/// Import VLESS config from vless:// URL
#[tauri::command]
pub async fn import_vless(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<VlessConfig, String> {
    validate_not_empty(&url, "VLESS URL").map_err(|e| e.to_string())?;
    
    if !url.starts_with("vless://") {
        return Err("URL must start with vless://".to_string());
    }
    
    info!("Importing VLESS config from URL");

    // Parse the URL
    let config = VlessConfig::from_url(&url).map_err(|e| e.to_string())?;

    // Load existing configs
    let mut configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .await
        .storage_context("Failed to load configs")
        .map_err(|e: IsolateError| e.to_string())?
        .unwrap_or_default();

    // Check for duplicate UUID
    if configs.iter().any(|c| c.uuid == config.uuid && c.server == config.server) {
        return Err("Config with this UUID and server already exists".to_string());
    }

    // Add new config
    configs.push(config.clone());

    // Save
    state
        .storage
        .set_setting(VLESS_CONFIGS_KEY, &configs)
        .await
        .storage_context("Failed to save config")
        .map_err(|e: IsolateError| e.to_string())?;

    info!(id = %config.id, name = %config.name, "VLESS config imported");
    Ok(config)
}


/// Get all saved VLESS configs
#[tauri::command]
pub async fn get_vless_configs(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<VlessConfig>, String> {
    info!("Loading VLESS configs");

    let configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .await
        .storage_context("Failed to load configs")
        .map_err(|e: IsolateError| e.to_string())?
        .unwrap_or_default();

    Ok(configs)
}

/// Delete VLESS config by ID
#[tauri::command]
pub async fn delete_vless_config(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    info!(id = %id, "Deleting VLESS config");

    // Load existing configs
    let mut configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .await
        .storage_context("Failed to load configs")
        .map_err(|e: IsolateError| e.to_string())?
        .unwrap_or_default();

    // Find and remove
    let initial_len = configs.len();
    configs.retain(|c| c.id != id);

    if configs.len() == initial_len {
        return Err("Config not found".to_string());
    }

    // Save
    state
        .storage
        .set_setting(VLESS_CONFIGS_KEY, &configs)
        .await
        .storage_context("Failed to save configs")
        .map_err(|e: IsolateError| e.to_string())?;

    info!(id = %id, "VLESS config deleted");
    Ok(())
}

/// Toggle VLESS config active state
#[tauri::command]
pub async fn toggle_vless_config(
    state: State<'_, Arc<AppState>>,
    id: String,
    active: bool,
) -> Result<(), String> {
    info!(id = %id, active, "Toggling VLESS config");

    // Load existing configs
    let mut configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .await
        .storage_context("Failed to load configs")
        .map_err(|e: IsolateError| e.to_string())?
        .unwrap_or_default();

    // Find and update
    let mut found = false;
    for config in &mut configs {
        if config.id == id {
            config.active = active;
            found = true;
        } else if active {
            // Deactivate other configs when activating one
            config.active = false;
        }
    }

    if !found {
        return Err("Config not found".to_string());
    }

    // Save
    state
        .storage
        .set_setting(VLESS_CONFIGS_KEY, &configs)
        .await
        .storage_context("Failed to save configs")
        .map_err(|e: IsolateError| e.to_string())?;

    Ok(())
}


// ============================================================================
// VLESS Proxy Control Commands
// ============================================================================

/// Start VLESS proxy for a specific config
///
/// Starts sing-box with the given VLESS configuration.
/// Returns the SOCKS port for the proxy.
#[tauri::command]
pub async fn start_vless_proxy(
    state: State<'_, Arc<AppState>>,
    config_id: String,
    socks_port: Option<u16>,
) -> Result<crate::core::singbox_manager::SingboxInstance, String> {
    // Rate limit: max 1 call per 5 seconds
    crate::commands::rate_limiter::check_rate_limit("start_vless_proxy", 5)
        .map_err(|e| e.to_string())?;
    
    info!(config_id = %config_id, "Starting VLESS proxy");

    // Load the config
    let configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .await
        .storage_context("Failed to load configs")
        .map_err(|e: IsolateError| e.to_string())?
        .unwrap_or_default();

    let config = configs
        .iter()
        .find(|c| c.id == config_id)
        .ok_or_else(|| format!("Config '{}' not found", config_id))?;

    // Convert to vless_engine config
    let vless_config = crate::core::vless_engine::VlessConfig::new(
        config.server.clone(),
        config.port,
        config.uuid.clone(),
    )
    .with_name(&config.name)
    .with_sni(config.sni.clone().unwrap_or_else(|| config.server.clone()));

    // Get manager and allocate port
    let manager = crate::core::singbox_manager::get_manager();
    let port = match socks_port {
        Some(p) => p,
        None => manager.allocate_port(1080).await,
    };

    // Start the proxy
    let instance = manager
        .start(&vless_config, port)
        .await
        .process_context("Failed to start VLESS proxy")
        .map_err(|e: IsolateError| e.to_string())?;

    info!(
        config_id = %config_id,
        socks_port = instance.socks_port,
        "VLESS proxy started"
    );

    Ok(instance)
}

/// Stop VLESS proxy for a specific config
#[tauri::command]
pub async fn stop_vless_proxy(config_id: String) -> Result<(), String> {
    info!(config_id = %config_id, "Stopping VLESS proxy");

    let manager = crate::core::singbox_manager::get_manager();

    manager
        .stop(&config_id)
        .await
        .process_context("Failed to stop VLESS proxy")
        .map_err(|e: IsolateError| e.to_string())?;

    info!(config_id = %config_id, "VLESS proxy stopped");
    Ok(())
}

/// Stop all running VLESS proxies
#[tauri::command]
pub async fn stop_all_vless_proxies() -> Result<(), String> {
    info!("Stopping all VLESS proxies");

    let manager = crate::core::singbox_manager::get_manager();

    manager
        .stop_all()
        .await
        .process_context("Failed to stop VLESS proxies")
        .map_err(|e: IsolateError| e.to_string())?;

    info!("All VLESS proxies stopped");
    Ok(())
}


/// Get status of a specific VLESS proxy
#[tauri::command]
pub async fn get_vless_status(
    config_id: String,
) -> Result<Option<crate::core::singbox_manager::SingboxInstance>, String> {
    let manager = crate::core::singbox_manager::get_manager();
    Ok(manager.get_status(&config_id).await)
}

/// Get status of all running VLESS proxies
#[tauri::command]
pub async fn get_all_vless_status() -> Result<Vec<crate::core::singbox_manager::SingboxInstance>, String> {
    let manager = crate::core::singbox_manager::get_manager();
    Ok(manager.list_instances().await)
}

/// Perform health check on a running VLESS proxy
#[tauri::command]
pub async fn health_check_vless(config_id: String) -> Result<bool, String> {
    info!(config_id = %config_id, "Performing VLESS health check");

    let manager = crate::core::singbox_manager::get_manager();

    manager
        .health_check(&config_id)
        .await
        .network_context("Health check failed")
        .map_err(|e: IsolateError| e.to_string())
}

/// Test VLESS proxy connectivity
///
/// Makes a test request through the proxy to verify it's working.
#[tauri::command]
pub async fn test_vless_connectivity(
    config_id: String,
    test_url: Option<String>,
) -> Result<u32, String> {
    info!(config_id = %config_id, "Testing VLESS connectivity");

    let manager = crate::core::singbox_manager::get_manager();

    // Get the SOCKS port for this config
    let socks_port = manager
        .get_socks_port(&config_id)
        .await
        .ok_or_else(|| format!("Config '{}' is not running", config_id))?;

    let url = test_url.unwrap_or_else(|| "https://www.google.com".to_string());

    crate::core::vless_engine::test_proxy_connectivity(socks_port, &url)
        .await
        .network_context("Connectivity test failed")
        .map_err(|e: IsolateError| e.to_string())
}

/// Check if sing-box binary is available
#[tauri::command]
pub async fn is_singbox_available() -> Result<bool, String> {
    Ok(crate::core::singbox_manager::is_singbox_available())
}

/// Get sing-box version
#[tauri::command]
pub async fn get_singbox_version() -> Result<String, String> {
    crate::core::singbox_manager::get_singbox_version()
        .await
        .process_context("Failed to get sing-box version")
        .map_err(|e: IsolateError| e.to_string())
}

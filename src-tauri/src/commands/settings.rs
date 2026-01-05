//! Settings management commands

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::models::{Settings, ServiceWithState};
use crate::state::AppState;

/// Get user settings
#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    info!("Getting user settings");
    
    state
        .storage
        .get_settings()
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))
}

/// Save user settings
#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    info!("Saving user settings");
    
    state
        .storage
        .save_settings(&settings)
        .await
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Get services with their enabled/disabled state
#[tauri::command]
pub async fn get_services_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ServiceWithState>, String> {
    info!("Loading services with settings");
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
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
    state: State<'_, Arc<AppState>>,
    service_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!(service_id = %service_id, enabled, "Toggling service");
    
    state
        .storage
        .set_service_enabled(&service_id, enabled)
        .await
        .map_err(|e| format!("Failed to toggle service: {}", e))
}

/// Get a setting by key
#[tauri::command]
pub async fn get_setting(
    state: State<'_, Arc<AppState>>,
    key: String,
) -> Result<serde_json::Value, String> {
    info!(key = %key, "Getting setting");
    
    let value: Option<serde_json::Value> = state
        .storage
        .get_setting(&key)
        .await
        .map_err(|e| format!("Failed to get setting: {}", e))?;
    
    Ok(value.unwrap_or(serde_json::Value::Null))
}

/// Set a setting by key
#[tauri::command]
pub async fn set_setting(
    state: State<'_, Arc<AppState>>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    info!(key = %key, "Setting setting");
    
    state
        .storage
        .set_setting(&key, &value)
        .await
        .map_err(|e| format!("Failed to set setting: {}", e))
}

//! Service Health History Tauri commands

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::errors::IsolateError;
use crate::core::storage::{ServiceHealthRecord, ServiceHealthStats};
use crate::state::AppState;

/// Get service health history for a specific service
#[tauri::command]
pub async fn get_service_health_history(
    state: State<'_, Arc<AppState>>,
    service_id: String,
    hours: Option<u32>,
) -> Result<Vec<ServiceHealthRecord>, IsolateError> {
    let hours = hours.unwrap_or(24);
    info!(service_id = %service_id, hours = hours, "Getting service health history");
    
    state
        .storage
        .get_service_history(&service_id, hours)
        .await
}

/// Get service health statistics for a specific service
#[tauri::command]
pub async fn get_service_health_stats(
    state: State<'_, Arc<AppState>>,
    service_id: String,
    hours: Option<u32>,
) -> Result<ServiceHealthStats, IsolateError> {
    let hours = hours.unwrap_or(24);
    info!(service_id = %service_id, hours = hours, "Getting service health stats");
    
    state
        .storage
        .get_service_health_stats(&service_id, hours)
        .await
}

/// Get health history for all services
#[tauri::command]
pub async fn get_all_services_health_history(
    state: State<'_, Arc<AppState>>,
    hours: Option<u32>,
) -> Result<std::collections::HashMap<String, Vec<ServiceHealthRecord>>, IsolateError> {
    let hours = hours.unwrap_or(24);
    info!(hours = hours, "Getting all services health history");
    
    state
        .storage
        .get_all_services_history(hours)
        .await
}

/// Cleanup old health history records (older than 7 days)
#[tauri::command]
pub async fn cleanup_health_history(
    state: State<'_, Arc<AppState>>,
) -> Result<u64, IsolateError> {
    info!("Cleaning up old health history records");
    
    state
        .storage
        .cleanup_old_health_history()
        .await
}

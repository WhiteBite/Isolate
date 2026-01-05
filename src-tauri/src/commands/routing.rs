//! Routing commands (domain and app routing)

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::app_routing::InstalledApp;
use crate::core::models::{AppRoute, DomainRoute};
use crate::state::AppState;

// ============================================================================
// Domain Routing Commands
// ============================================================================

/// Get all domain routes
#[tauri::command]
pub async fn get_domain_routes(state: State<'_, Arc<AppState>>) -> Result<Vec<DomainRoute>, String> {
    info!("Getting domain routes");
    state
        .domain_router
        .get_routes()
        .await
        .map_err(|e| format!("Failed to get domain routes: {}", e))
}

/// Add a domain route
#[tauri::command]
pub async fn add_domain_route(
    state: State<'_, Arc<AppState>>,
    domain: String,
    proxy_id: String,
) -> Result<(), String> {
    info!(domain = %domain, proxy_id = %proxy_id, "Adding domain route");
    state
        .domain_router
        .add_route(&domain, &proxy_id)
        .await
        .map_err(|e| format!("Failed to add domain route: {}", e))
}

/// Remove a domain route
#[tauri::command]
pub async fn remove_domain_route(
    state: State<'_, Arc<AppState>>,
    domain: String,
) -> Result<(), String> {
    info!(domain = %domain, "Removing domain route");
    state
        .domain_router
        .remove_route(&domain)
        .await
        .map_err(|e| format!("Failed to remove domain route: {}", e))
}

// ============================================================================
// App Routing Commands
// ============================================================================

/// Get all app routes
#[tauri::command]
pub async fn get_app_routes(state: State<'_, Arc<AppState>>) -> Result<Vec<AppRoute>, String> {
    info!("Getting app routes");
    state
        .app_router
        .get_routes()
        .await
        .map_err(|e| format!("Failed to get app routes: {}", e))
}

/// Add an app route
#[tauri::command]
pub async fn add_app_route(
    state: State<'_, Arc<AppState>>,
    app_name: String,
    app_path: String,
    proxy_id: String,
) -> Result<(), String> {
    info!(app_name = %app_name, proxy_id = %proxy_id, "Adding app route");
    state
        .app_router
        .add_route(&app_name, &app_path, &proxy_id)
        .await
        .map_err(|e| format!("Failed to add app route: {}", e))
}

/// Remove an app route
#[tauri::command]
pub async fn remove_app_route(
    state: State<'_, Arc<AppState>>,
    app_path: String,
) -> Result<(), String> {
    info!(app_path = %app_path, "Removing app route");
    state
        .app_router
        .remove_route(&app_path)
        .await
        .map_err(|e| format!("Failed to remove app route: {}", e))
}

/// Get list of installed applications (Windows)
#[tauri::command]
pub async fn get_installed_apps(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<InstalledApp>, String> {
    info!("Getting installed apps");
    state
        .app_router
        .get_installed_apps()
        .await
        .map_err(|e| format!("Failed to get installed apps: {}", e))
}

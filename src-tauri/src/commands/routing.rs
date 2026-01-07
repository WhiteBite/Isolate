//! Routing commands (domain and app routing)

use std::sync::Arc;
use tauri::State;
use tracing::info;

use super::validation::validate_domain;
use crate::core::app_routing::InstalledApp;
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::models::{AppRoute, DomainRoute};
use crate::core::storage::RoutingRule;
use crate::state::AppState;

// ============================================================================
// High-Level Routing Rules Commands
// ============================================================================

/// Get all routing rules
#[tauri::command]
pub async fn get_routing_rules(state: State<'_, Arc<AppState>>) -> Result<Vec<RoutingRule>, String> {
    info!("Getting routing rules");
    state
        .storage
        .get_routing_rules()
        .await
        .storage_context("Failed to get routing rules")
        .map_err(|e: IsolateError| e.to_string())
}

/// Add a new routing rule
#[tauri::command]
pub async fn add_routing_rule(
    state: State<'_, Arc<AppState>>,
    rule: RoutingRule,
) -> Result<RoutingRule, String> {
    info!(id = %rule.id, name = %rule.name, "Adding routing rule");
    state
        .storage
        .add_routing_rule(&rule)
        .await
        .storage_context("Failed to add routing rule")
        .map_err(|e: IsolateError| e.to_string())?;
    Ok(rule)
}

/// Update an existing routing rule
#[tauri::command]
pub async fn update_routing_rule(
    state: State<'_, Arc<AppState>>,
    rule: RoutingRule,
) -> Result<(), String> {
    info!(id = %rule.id, name = %rule.name, "Updating routing rule");
    state
        .storage
        .update_routing_rule(&rule)
        .await
        .storage_context("Failed to update routing rule")
        .map_err(|e: IsolateError| e.to_string())
}

/// Delete a routing rule
#[tauri::command]
pub async fn delete_routing_rule(
    state: State<'_, Arc<AppState>>,
    rule_id: String,
) -> Result<(), String> {
    info!(rule_id = %rule_id, "Deleting routing rule");
    state
        .storage
        .delete_routing_rule(&rule_id)
        .await
        .storage_context("Failed to delete routing rule")
        .map_err(|e: IsolateError| e.to_string())
}

/// Reorder routing rules
#[tauri::command]
pub async fn reorder_routing_rules(
    state: State<'_, Arc<AppState>>,
    rule_ids: Vec<String>,
) -> Result<(), String> {
    info!(count = rule_ids.len(), "Reordering routing rules");
    state
        .storage
        .reorder_routing_rules(&rule_ids)
        .await
        .storage_context("Failed to reorder routing rules")
        .map_err(|e: IsolateError| e.to_string())
}

/// Toggle routing rule enabled state
#[tauri::command]
pub async fn toggle_routing_rule(
    state: State<'_, Arc<AppState>>,
    rule_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!(rule_id = %rule_id, enabled, "Toggling routing rule");
    state
        .storage
        .toggle_routing_rule(&rule_id, enabled)
        .await
        .storage_context("Failed to toggle routing rule")
        .map_err(|e: IsolateError| e.to_string())
}

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
        .storage_context("Failed to get domain routes")
        .map_err(|e: IsolateError| e.to_string())
}

/// Add a domain route
#[tauri::command]
pub async fn add_domain_route(
    state: State<'_, Arc<AppState>>,
    domain: String,
    proxy_id: String,
) -> Result<(), String> {
    // Validate domain format
    validate_domain(&domain).map_err(|e| e.to_string())?;
    
    info!(domain = %domain, proxy_id = %proxy_id, "Adding domain route");
    state
        .domain_router
        .add_route(&domain, &proxy_id)
        .await
        .storage_context("Failed to add domain route")
        .map_err(|e: IsolateError| e.to_string())
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
        .storage_context("Failed to remove domain route")
        .map_err(|e: IsolateError| e.to_string())
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
        .storage_context("Failed to get app routes")
        .map_err(|e: IsolateError| e.to_string())
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
        .storage_context("Failed to add app route")
        .map_err(|e: IsolateError| e.to_string())
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
        .storage_context("Failed to remove app route")
        .map_err(|e: IsolateError| e.to_string())
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
        .storage_context("Failed to get installed apps")
        .map_err(|e: IsolateError| e.to_string())
}

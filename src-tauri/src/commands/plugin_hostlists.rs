//! Plugin Hostlist Registry Commands
//!
//! Tauri commands for managing hostlists from the plugin system.
//! These commands work with the HostlistRegistry in AppState.

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::plugins::{HostlistRegistry, RegisteredHostlist, RegistryStats};

/// Get all registered hostlists from plugins
#[tauri::command]
pub async fn get_plugin_hostlists(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
) -> Result<Vec<RegisteredHostlist>, String> {
    info!("Getting all plugin hostlists");
    
    let hostlists = hostlist_registry.list().await;
    
    info!(count = hostlists.len(), "Retrieved plugin hostlists");
    
    Ok(hostlists)
}

/// Get a specific hostlist by ID
#[tauri::command]
pub async fn get_plugin_hostlist(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    id: String,
) -> Result<RegisteredHostlist, String> {
    info!(id = %id, "Getting plugin hostlist");
    
    hostlist_registry
        .get(&id)
        .await
        .ok_or_else(|| format!("Hostlist '{}' not found", id))
}

/// Get domains from a specific hostlist
#[tauri::command]
pub async fn get_plugin_hostlist_domains(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    id: String,
) -> Result<Vec<String>, String> {
    info!(id = %id, "Getting domains from plugin hostlist");
    
    hostlist_registry
        .get_domains(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Merge multiple hostlists into a single domain list
#[tauri::command]
pub async fn merge_plugin_hostlists(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    ids: Vec<String>,
) -> Result<Vec<String>, String> {
    info!(ids = ?ids, "Merging plugin hostlists");
    
    let id_refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
    
    hostlist_registry
        .merge_hostlists(&id_refs)
        .await
        .map_err(|e| e.to_string())
}

/// Get all domains from all enabled hostlists
#[tauri::command]
pub async fn get_all_plugin_domains(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
) -> Result<Vec<String>, String> {
    info!("Getting all domains from enabled plugin hostlists");
    
    let domains = hostlist_registry.merge_all().await;
    
    info!(count = domains.len(), "Retrieved all plugin domains");
    
    Ok(domains)
}

/// Check if a domain matches any registered hostlist
#[tauri::command]
pub async fn check_domain_in_hostlists(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    domain: String,
) -> Result<bool, String> {
    let matches = hostlist_registry.domain_matches_any(&domain).await;
    Ok(matches)
}

/// Find which hostlists match a specific domain
#[tauri::command]
pub async fn find_matching_hostlists(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    domain: String,
) -> Result<Vec<String>, String> {
    info!(domain = %domain, "Finding matching hostlists");
    
    let matching = hostlist_registry.find_matching_hostlists(&domain).await;
    
    Ok(matching)
}

/// Get hostlists by category
#[tauri::command]
pub async fn get_hostlists_by_category(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    category: String,
) -> Result<Vec<RegisteredHostlist>, String> {
    info!(category = %category, "Getting hostlists by category");
    
    let hostlists = hostlist_registry.list_by_category(&category).await;
    
    Ok(hostlists)
}

/// Get hostlists by plugin ID
#[tauri::command]
pub async fn get_hostlists_by_plugin(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    plugin_id: String,
) -> Result<Vec<RegisteredHostlist>, String> {
    info!(plugin_id = %plugin_id, "Getting hostlists by plugin");
    
    let hostlists = hostlist_registry.list_by_plugin(&plugin_id).await;
    
    Ok(hostlists)
}

/// Enable or disable a hostlist
#[tauri::command]
pub async fn set_hostlist_enabled(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    info!(id = %id, enabled, "Setting hostlist enabled state");
    
    hostlist_registry
        .set_enabled(&id, enabled)
        .await
        .map_err(|e| e.to_string())
}

/// Reload a hostlist (re-fetch domains from file/URL)
#[tauri::command]
pub async fn reload_plugin_hostlist(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
    id: String,
) -> Result<(), String> {
    info!(id = %id, "Reloading plugin hostlist");
    
    hostlist_registry
        .reload(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Get hostlist registry statistics
#[tauri::command]
pub async fn get_hostlist_registry_stats(
    hostlist_registry: State<'_, Arc<HostlistRegistry>>,
) -> Result<RegistryStats, String> {
    let stats = hostlist_registry.stats().await;
    Ok(stats)
}

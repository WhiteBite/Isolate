//! Hostlist management commands
//!
//! Commands for managing domain hostlists (Discord, YouTube, etc.)
//! These commands work directly with the hostlists module without State.

use tracing::info;

use crate::core::hostlists::{self, Hostlist};

/// Get all available hostlists
#[tauri::command]
pub async fn get_hostlists() -> Result<Vec<Hostlist>, String> {
    info!("Loading all hostlists");

    hostlists::get_all_hostlists()
        .await
        .map_err(|e| format!("Failed to load hostlists: {}", e))
}

/// Get a specific hostlist by ID
#[tauri::command]
pub async fn get_hostlist(id: String) -> Result<Hostlist, String> {
    info!(id = %id, "Loading hostlist");

    hostlists::load_hostlist(&id)
        .await
        .map_err(|e| format!("Failed to load hostlist: {}", e))
}

/// Add domain to a hostlist
#[tauri::command]
pub async fn add_hostlist_domain(hostlist_id: String, domain: String) -> Result<(), String> {
    info!(hostlist_id = %hostlist_id, domain = %domain, "Adding domain to hostlist");

    hostlists::add_domain(&hostlist_id, &domain)
        .await
        .map_err(|e| format!("Failed to add domain: {}", e))
}

/// Remove domain from a hostlist
#[tauri::command]
pub async fn remove_hostlist_domain(hostlist_id: String, domain: String) -> Result<(), String> {
    info!(hostlist_id = %hostlist_id, domain = %domain, "Removing domain from hostlist");

    hostlists::remove_domain(&hostlist_id, &domain)
        .await
        .map_err(|e| format!("Failed to remove domain: {}", e))
}

/// Create a new hostlist
#[tauri::command]
pub async fn create_hostlist(id: String, name: String) -> Result<Hostlist, String> {
    info!(id = %id, name = %name, "Creating new hostlist");

    hostlists::create_hostlist(&id, &name)
        .await
        .map_err(|e| format!("Failed to create hostlist: {}", e))
}

/// Delete a hostlist
#[tauri::command]
pub async fn delete_hostlist(id: String) -> Result<(), String> {
    info!(id = %id, "Deleting hostlist");

    hostlists::delete_hostlist(&id)
        .await
        .map_err(|e| format!("Failed to delete hostlist: {}", e))
}

/// Update hostlist from remote URL
#[tauri::command]
pub async fn update_hostlist_from_url(id: String, url: String) -> Result<Hostlist, String> {
    info!(id = %id, url = %url, "Updating hostlist from URL");

    hostlists::update_hostlist(&id, &url)
        .await
        .map_err(|e| format!("Failed to update hostlist: {}", e))?;

    // Return updated hostlist
    hostlists::load_hostlist(&id)
        .await
        .map_err(|e| format!("Failed to load updated hostlist: {}", e))
}

/// Save hostlist with new domains
#[tauri::command]
pub async fn save_hostlist(hostlist: Hostlist) -> Result<(), String> {
    info!(id = %hostlist.id, domain_count = hostlist.domains.len(), "Saving hostlist");

    hostlists::save_hostlist(&hostlist)
        .await
        .map_err(|e| format!("Failed to save hostlist: {}", e))
}

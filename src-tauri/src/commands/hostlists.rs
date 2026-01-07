//! Hostlist management commands
//!
//! Commands for managing domain hostlists (Discord, YouTube, etc.)
//! These commands work directly with the hostlists module without State.

use tracing::info;

use crate::commands::validation::{validate_not_empty, validate_domain};
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::hostlists::{self, Hostlist};
use crate::core::hostlist_updater::{
    self, HostlistInfo, UpdateCheckResult, UpdateResult, HostlistUpdater
};

/// Get all available hostlists
#[tauri::command]
pub async fn get_hostlists() -> Result<Vec<Hostlist>, IsolateError> {
    info!("Loading all hostlists");

    hostlists::get_all_hostlists()
        .await
        .io_context("Failed to load hostlists")
}

/// Get a specific hostlist by ID
#[tauri::command]
pub async fn get_hostlist(id: String) -> Result<Hostlist, IsolateError> {
    validate_not_empty(&id, "Hostlist ID")?;
    
    info!(id = %id, "Loading hostlist");

    hostlists::load_hostlist(&id)
        .await
        .io_context("Failed to load hostlist")
}

/// Add domain to a hostlist
#[tauri::command]
pub async fn add_hostlist_domain(hostlist_id: String, domain: String) -> Result<(), IsolateError> {
    validate_not_empty(&hostlist_id, "Hostlist ID")?;
    validate_domain(&domain)?;
    
    info!(hostlist_id = %hostlist_id, domain = %domain, "Adding domain to hostlist");

    hostlists::add_domain(&hostlist_id, &domain)
        .await
        .io_context("Failed to add domain")
}

/// Remove domain from a hostlist
#[tauri::command]
pub async fn remove_hostlist_domain(hostlist_id: String, domain: String) -> Result<(), IsolateError> {
    validate_not_empty(&hostlist_id, "Hostlist ID")?;
    validate_domain(&domain)?;
    
    info!(hostlist_id = %hostlist_id, domain = %domain, "Removing domain from hostlist");

    hostlists::remove_domain(&hostlist_id, &domain)
        .await
        .io_context("Failed to remove domain")
}

/// Create a new hostlist
#[tauri::command]
pub async fn create_hostlist(id: String, name: String) -> Result<Hostlist, IsolateError> {
    validate_not_empty(&id, "Hostlist ID")?;
    validate_not_empty(&name, "Hostlist name")?;
    
    info!(id = %id, name = %name, "Creating new hostlist");

    hostlists::create_hostlist(&id, &name)
        .await
        .io_context("Failed to create hostlist")
}

/// Delete a hostlist
#[tauri::command]
pub async fn delete_hostlist(id: String) -> Result<(), IsolateError> {
    validate_not_empty(&id, "Hostlist ID")?;
    
    info!(id = %id, "Deleting hostlist");

    hostlists::delete_hostlist(&id)
        .await
        .io_context("Failed to delete hostlist")
}

/// Update hostlist from remote URL
#[tauri::command]
pub async fn update_hostlist_from_url(id: String, url: String) -> Result<Hostlist, IsolateError> {
    info!(id = %id, url = %url, "Updating hostlist from URL");

    hostlists::update_hostlist(&id, &url)
        .await
        .network_context("Failed to update hostlist")?;

    // Return updated hostlist
    hostlists::load_hostlist(&id)
        .await
        .io_context("Failed to load updated hostlist")
}

/// Save hostlist with new domains
#[tauri::command]
pub async fn save_hostlist(hostlist: Hostlist) -> Result<(), IsolateError> {
    info!(id = %hostlist.id, domain_count = hostlist.domains.len(), "Saving hostlist");

    hostlists::save_hostlist(&hostlist)
        .await
        .io_context("Failed to save hostlist")
}


// ============================================================================
// Hostlist Updater Commands
// ============================================================================

/// Get information about all hostlists (for UI display)
#[tauri::command]
pub async fn get_hostlist_info() -> Result<Vec<HostlistInfo>, IsolateError> {
    info!("Getting hostlist info");

    hostlist_updater::get_hostlist_info()
        .await
        .io_context("Failed to get hostlist info")
}

/// Check for available hostlist updates
/// 
/// Returns a list of hostlists with update availability status.
/// Uses ETag/Last-Modified headers for efficient checking.
#[tauri::command]
pub async fn check_hostlist_updates() -> Result<Vec<UpdateCheckResult>, IsolateError> {
    info!("Checking for hostlist updates");

    hostlist_updater::check_hostlist_updates()
        .await
        .network_context("Failed to check for updates")
}

/// Update all hostlists from their configured sources
/// 
/// Downloads and updates all hostlists that have remote sources configured.
/// Creates backups of existing files before updating.
#[tauri::command]
pub async fn update_hostlists() -> Result<UpdateResult, IsolateError> {
    info!("Updating all hostlists");

    hostlist_updater::update_all_hostlists()
        .await
        .network_context("Failed to update hostlists")
}

/// Update a single hostlist by ID
#[tauri::command]
pub async fn update_single_hostlist(id: String) -> Result<(), IsolateError> {
    validate_not_empty(&id, "Hostlist ID")?;
    
    info!(id = %id, "Updating single hostlist");

    let mut updater = HostlistUpdater::new()
        .await
        .network_context("Failed to create updater")?;
    
    updater.update_hostlist(&id)
        .await
        .network_context("Failed to update hostlist")
}

/// Restore hostlist from backup
#[tauri::command]
pub async fn restore_hostlist_backup(id: String) -> Result<(), IsolateError> {
    validate_not_empty(&id, "Hostlist ID")?;
    
    info!(id = %id, "Restoring hostlist from backup");

    let updater = HostlistUpdater::new()
        .await
        .io_context("Failed to create updater")?;
    
    updater.restore_from_backup(&id)
        .await
        .io_context("Failed to restore backup")
}

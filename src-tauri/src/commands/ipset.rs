//! IP Set management commands
//!
//! Commands for managing IP address lists (ipset) for DPI bypass.
//! Supports downloading from remote sources, auto-updates, and mode switching.
//!
//! ## Modes
//! - `any`: Allow all traffic (ipset filtering disabled)
//! - `none`: Block all traffic that would match ipset rules
//! - `loaded`: Use loaded ipset for filtering

use tracing::info;

use crate::commands::validation::validate_public_url;
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::ipset_manager::{
    self, IpsetMode, IpsetStats,
};
use crate::core::ipset_updater::{
    self, IpsetInfo, IpsetUpdateResult, IpsetUpdater, IpsetValidationResult,
};

/// Get information about the current ipset
/// 
/// Returns metadata including IP count, last update time, and source URL.
#[tauri::command]
pub async fn get_ipset_info() -> Result<IpsetInfo, IsolateError> {
    info!("Getting ipset info");

    ipset_updater::get_ipset_info()
        .await
        .io_context("Failed to get ipset info")
}

/// Update ipset from a specific source URL
/// 
/// Downloads IP addresses from the given URL and updates ipset-all.txt.
/// Validates content before saving (only IPv4/IPv6 and CIDR allowed).
/// 
/// Security: URL is validated to prevent SSRF attacks (blocks localhost, private IPs).
#[tauri::command]
pub async fn update_ipset(source_url: String) -> Result<IpsetUpdateResult, IsolateError> {
    // SSRF protection: validate URL points to public address only
    let validated_url = validate_public_url(&source_url)?;
    
    info!(url = %validated_url, "Updating ipset from URL");

    ipset_updater::update_ipset(validated_url.as_str())
        .await
        .network_context("Failed to update ipset")
}

/// Update ipset from configured sources
/// 
/// Tries each configured source in priority order until one succeeds.
/// Sources are configured in configs/ipset_sources.yaml.
#[tauri::command]
pub async fn update_ipset_from_sources() -> Result<IpsetUpdateResult, IsolateError> {
    info!("Updating ipset from configured sources");

    ipset_updater::update_ipset_from_sources()
        .await
        .network_context("Failed to update ipset from sources")
}

/// Set ipset auto-update enabled/disabled
/// 
/// When enabled, ipset will be automatically updated once per day.
#[tauri::command]
pub async fn set_ipset_auto_update(enabled: bool) -> Result<(), IsolateError> {
    info!(enabled = enabled, "Setting ipset auto-update");

    ipset_updater::set_ipset_auto_update(enabled)
        .await
        .io_context("Failed to set ipset auto-update")
}

/// Get ipset sources configuration
/// 
/// Returns the list of configured ipset sources with their URLs and priorities.
#[tauri::command]
pub async fn get_ipset_sources() -> Result<Vec<ipset_updater::IpsetSource>, IsolateError> {
    info!("Getting ipset sources");

    let updater = IpsetUpdater::new()
        .await
        .io_context("Failed to create ipset updater")?;
    
    Ok(updater.get_sources().to_vec())
}

/// Restore ipset from backup
/// 
/// Restores the previous version of ipset-all.txt from backup.
#[tauri::command]
pub async fn restore_ipset_backup() -> Result<(), IsolateError> {
    info!("Restoring ipset from backup");

    let updater = IpsetUpdater::new()
        .await
        .io_context("Failed to create ipset updater")?;
    
    updater.restore_from_backup()
        .await
        .io_context("Failed to restore ipset backup")
}

// ============================================================================
// Ipset Manager Commands (Mode Management)
// ============================================================================

/// Get ipset statistics including mode, IP counts, and update info
/// 
/// Returns comprehensive statistics about the current ipset state.
#[tauri::command]
pub async fn get_ipset_stats() -> Result<IpsetStats, IsolateError> {
    info!("Getting ipset stats");

    ipset_manager::get_ipset_stats()
        .await
        .io_context("Failed to get ipset stats")
}

/// Get current ipset mode
/// 
/// Returns the current mode: "any", "none", or "loaded".
#[tauri::command]
pub async fn get_ipset_mode() -> Result<IpsetMode, IsolateError> {
    info!("Getting ipset mode");

    ipset_manager::get_current_mode()
        .await
        .io_context("Failed to get ipset mode")
}

/// Set ipset mode
/// 
/// Changes the ipset operation mode:
/// - `any`: Allow all traffic (ipset filtering disabled)
/// - `none`: Block all traffic that would match ipset rules  
/// - `loaded`: Use loaded ipset for filtering
/// 
/// Note: Switching to "loaded" mode requires an ipset file to exist.
#[tauri::command]
pub async fn set_ipset_mode(mode: String) -> Result<(), IsolateError> {
    info!(mode = %mode, "Setting ipset mode");

    let mode: IpsetMode = mode.parse()
        .map_err(|e: IsolateError| e)?;

    ipset_manager::set_current_mode(mode)
        .await
        .io_context("Failed to set ipset mode")
}

/// Load ipset from a local file
/// 
/// Loads and validates an ipset file from the specified path.
/// The file is copied to the default location for use.
#[tauri::command]
pub async fn load_ipset_from_file(path: String) -> Result<IpsetValidationResult, IsolateError> {
    info!(path = %path, "Loading ipset from file");

    ipset_manager::load_ipset_from_file(&path)
        .await
        .io_context("Failed to load ipset from file")
}

/// Update ipset from a specific URL (via manager)
/// 
/// Downloads and updates ipset from the given URL.
/// This is an alternative to `update_ipset` that goes through the manager.
#[tauri::command]
pub async fn update_ipset_from_url(url: String) -> Result<IpsetUpdateResult, IsolateError> {
    // SSRF protection: validate URL points to public address only
    let validated_url = validate_public_url(&url)?;
    
    info!(url = %validated_url, "Updating ipset from URL via manager");

    ipset_manager::update_ipset_from_url(validated_url.as_str())
        .await
        .network_context("Failed to update ipset from URL")
}

//! IP Set management commands
//!
//! Commands for managing IP address lists (ipset) for DPI bypass.
//! Supports downloading from remote sources and auto-updates.

use tracing::info;

use crate::commands::validation::validate_public_url;
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::ipset_updater::{
    self, IpsetInfo, IpsetUpdateResult, IpsetUpdater,
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

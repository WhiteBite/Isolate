//! Hosts management commands
//!
//! Tauri IPC команды для управления системным hosts файлом.
//! Используется для добавления Discord voice серверов в hosts.

use tracing::info;

use crate::core::errors::IsolateError;
use crate::core::hosts_manager::{self, HostsStatus};

/// Enable Discord hosts entries
///
/// Adds Discord voice server entries to the system hosts file.
/// Requires administrator privileges.
#[tauri::command]
pub async fn enable_discord_hosts() -> Result<(), IsolateError> {
    info!("Enabling Discord hosts entries");
    
    hosts_manager::add_discord_hosts().await
}

/// Disable Discord hosts entries
///
/// Removes Discord voice server entries from the system hosts file.
/// Requires administrator privileges.
#[tauri::command]
pub async fn disable_discord_hosts() -> Result<(), IsolateError> {
    info!("Disabling Discord hosts entries");
    
    hosts_manager::remove_discord_hosts().await
}

/// Get hosts status
///
/// Returns the current status of Discord hosts entries.
#[tauri::command]
pub async fn get_hosts_status() -> Result<HostsStatus, IsolateError> {
    info!("Getting hosts status");
    
    hosts_manager::get_hosts_status().await
}

/// Backup hosts file
///
/// Creates a backup of the current hosts file.
#[tauri::command]
pub async fn backup_hosts() -> Result<(), IsolateError> {
    info!("Creating hosts backup");
    
    hosts_manager::backup_hosts().await
}

/// Restore hosts from backup
///
/// Restores the hosts file from the backup.
/// Requires administrator privileges.
#[tauri::command]
pub async fn restore_hosts() -> Result<(), IsolateError> {
    info!("Restoring hosts from backup");
    
    hosts_manager::restore_hosts().await
}

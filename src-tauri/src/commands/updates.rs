//! Update management commands
//!
//! Provides two update mechanisms:
//! 1. GitHub Releases API - simple notification with download link (no signing required)
//! 2. Tauri updater plugin - full auto-update (requires signing keys)

use tauri::AppHandle;
use tracing::{error, info};

use crate::core::errors::IsolateError;
use crate::core::update_checker::{check_for_updates as github_check, GitHubUpdateInfo};

/// Check for updates via GitHub Releases API
/// 
/// Returns update info if a newer version is available.
/// Does NOT auto-install - just shows notification with download link.
#[tauri::command]
pub async fn check_github_updates() -> Result<Option<GitHubUpdateInfo>, String> {
    info!("Checking for updates via GitHub API");
    
    match github_check().await {
        Ok(update) => {
            if let Some(ref info) = update {
                info!(version = %info.version, "New version available on GitHub");
            } else {
                info!("Already on latest version");
            }
            Ok(update)
        }
        Err(e) => {
            error!("Failed to check GitHub releases: {}", e);
            Err(e)
        }
    }
}

// Legacy Tauri updater commands (require signing keys)
// Kept for future use when/if signing is implemented

#[cfg(feature = "tauri-updater")]
mod tauri_updater {
    use super::*;
    use tauri_plugin_updater::UpdaterExt;
    use crate::core::errors::{OptionExt, TypedResultExt};
    use crate::core::models::UpdateInfo;

    /// Check for available updates via Tauri updater
    #[tauri::command]
    pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
        info!("Checking for updates via Tauri updater");
        
        let updater = app.updater()
            .tauri_context("Failed to get updater")
            .map_err(|e: IsolateError| e.to_string())?;
        
        match updater.check().await {
            Ok(Some(update)) => {
                info!(version = %update.version, "Update available");
                Ok(Some(UpdateInfo {
                    version: update.version.clone(),
                    notes: update.body.clone(),
                    date: update.date.map(|d| d.to_string()),
                }))
            }
            Ok(None) => {
                info!("No updates available");
                Ok(None)
            }
            Err(e) => {
                error!("Failed to check for updates: {}", e);
                Err(IsolateError::network(format!("Failed to check for updates: {}", e)).to_string())
            }
        }
    }

    /// Download and install available update
    #[tauri::command]
    pub async fn install_update(app: AppHandle) -> Result<(), String> {
        info!("Installing update");
        
        let updater = app.updater()
            .tauri_context("Failed to get updater")
            .map_err(|e: IsolateError| e.to_string())?;
        
        let update = updater
            .check()
            .await
            .network_context("Failed to check for updates")
            .map_err(|e: IsolateError| e.to_string())?
            .ok_or_error("No update available")
            .map_err(|e: IsolateError| e.to_string())?;
        
        info!(version = %update.version, "Downloading update");
        
        update
            .download_and_install(|_downloaded, _total| {}, || {})
            .await
            .network_context("Failed to install update")
            .map_err(|e: IsolateError| e.to_string())?;
        
        info!("Update installed successfully, restart required");
        
        Ok(())
    }
}

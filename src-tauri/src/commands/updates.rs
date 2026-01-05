//! Update management commands

use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;
use tracing::{error, info};

use crate::core::models::UpdateInfo;

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    info!("Checking for updates");
    
    let updater = app.updater().map_err(|e| format!("Failed to get updater: {}", e))?;
    
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
            Err(format!("Failed to check for updates: {}", e))
        }
    }
}

/// Download and install available update
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    info!("Installing update");
    
    let updater = app.updater().map_err(|e| format!("Failed to get updater: {}", e))?;
    
    let update = updater
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?
        .ok_or_else(|| "No update available".to_string())?;
    
    info!(version = %update.version, "Downloading update");
    
    // Download and install the update
    update
        .download_and_install(|_downloaded, _total| {}, || {})
        .await
        .map_err(|e| format!("Failed to install update: {}", e))?;
    
    info!("Update installed successfully, restart required");
    
    Ok(())
}

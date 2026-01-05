//! System commands (version, mode detection, binaries)

use tauri::{Emitter, Window};
use tracing::{info, error};

use crate::core::binaries::{self, BinaryCheckResult, DownloadProgress};

// ============================================================================
// Version & Mode Detection
// ============================================================================

/// Get app version from Cargo.toml
#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Check if running with administrator privileges
#[tauri::command]
pub async fn is_admin() -> Result<bool, String> {
    Ok(crate::core::quic_blocker::is_admin())
}

/// Check if app is running in silent mode (--silent flag)
#[tauri::command]
pub async fn is_silent_mode() -> Result<bool, String> {
    Ok(std::env::args().any(|arg| arg == "--silent"))
}

/// Check if app is running in portable mode
#[tauri::command]
pub async fn is_portable_mode() -> Result<bool, String> {
    Ok(crate::core::paths::is_portable_mode())
}

// ============================================================================
// Binary Management
// ============================================================================

/// Check if all required binaries are present
#[tauri::command]
pub async fn check_binaries() -> Result<BinaryCheckResult, String> {
    info!("Checking required binaries");
    
    binaries::check_binaries()
        .await
        .map_err(|e| format!("Failed to check binaries: {}", e))
}

/// Download missing binaries with progress reporting
#[tauri::command]
pub async fn download_binaries(window: Window) -> Result<(), String> {
    info!("Starting binary download");
    
    let window_clone = window.clone();
    
    binaries::ensure_binaries(move |progress: DownloadProgress| {
        if let Err(e) = window_clone.emit("binaries:progress", &progress) {
            error!("Failed to emit download progress: {}", e);
        }
    })
    .await
    .map_err(|e| {
        error!("Binary download failed: {}", e);
        format!("Failed to download binaries: {}", e)
    })?;
    
    let _ = window.emit("binaries:complete", ());
    info!("Binary download completed");
    
    Ok(())
}

/// Get path to binaries directory
#[tauri::command]
pub async fn get_binaries_dir() -> Result<String, String> {
    Ok(crate::core::paths::get_binaries_dir().display().to_string())
}

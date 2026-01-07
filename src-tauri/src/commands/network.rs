//! Network-related Tauri commands (system proxy, telemetry, crash reporting)

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::sentry_integration;
use crate::state::AppState;

// ============================================================================
// System Proxy Commands
// ============================================================================

/// Set system proxy
///
/// Configures Windows system proxy settings.
#[tauri::command]
pub async fn set_system_proxy(host: String, port: u16, scheme: String) -> Result<(), IsolateError> {
    info!(host = %host, port, scheme = %scheme, "Setting system proxy");
    
    crate::core::system_proxy::set_system_proxy(&host, port, &scheme)
        .await
        .system_proxy_context("Failed to set system proxy")
}

/// Clear system proxy settings
#[tauri::command]
pub async fn clear_system_proxy() -> Result<(), IsolateError> {
    info!("Clearing system proxy");
    
    crate::core::system_proxy::clear_system_proxy()
        .await
        .system_proxy_context("Failed to clear system proxy")
}

/// Check if system proxy is currently set
#[tauri::command]
pub async fn is_system_proxy_set() -> Result<bool, IsolateError> {
    crate::core::system_proxy::is_system_proxy_set()
        .await
        .system_proxy_context("Failed to check system proxy")
}

// ============================================================================
// Telemetry Commands
// ============================================================================

/// Enable or disable telemetry (opt-in)
#[tauri::command]
pub async fn set_telemetry_enabled(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), IsolateError> {
    info!(enabled, "Setting telemetry enabled");
    state.telemetry.set_enabled(enabled);
    Ok(())
}

/// Check if telemetry is enabled
#[tauri::command]
pub async fn is_telemetry_enabled(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, IsolateError> {
    Ok(state.telemetry.is_enabled())
}

/// Get number of pending telemetry events
#[tauri::command]
pub async fn get_telemetry_pending_count(
    state: State<'_, Arc<AppState>>,
) -> Result<usize, IsolateError> {
    Ok(state.telemetry.pending_events().await)
}

/// Manually flush telemetry events
#[tauri::command]
pub async fn flush_telemetry(
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Manually flushing telemetry");
    state.telemetry
        .flush()
        .await
        .network_context("Failed to flush telemetry")
}

/// Clear pending telemetry events without sending
#[tauri::command]
pub async fn clear_telemetry(
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Clearing telemetry events");
    state.telemetry.clear().await;
    Ok(())
}

/// Report optimization result to telemetry
#[tauri::command]
pub async fn report_optimization_telemetry(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
    score: f32,
    success: bool,
) -> Result<(), IsolateError> {
    state.telemetry
        .report_optimization(&strategy_id, score, success)
        .await;
    Ok(())
}

/// Report strategy usage to telemetry
#[tauri::command]
pub async fn report_strategy_usage_telemetry(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
    duration_secs: u64,
) -> Result<(), IsolateError> {
    state.telemetry
        .report_strategy_usage(&strategy_id, duration_secs)
        .await;
    Ok(())
}

// ============================================================================
// Crash Reporting (Sentry) Commands
// ============================================================================

/// Enable or disable crash reporting (opt-in)
///
/// When enabled, anonymous crash reports are sent to help improve the app.
/// Privacy: No IP addresses, usernames, or file paths are collected.
#[tauri::command]
pub async fn set_crash_reporting_enabled(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), IsolateError> {
    info!(enabled, "Setting crash reporting enabled");
    
    // Update Sentry state
    sentry_integration::set_enabled(enabled);
    
    // Persist setting
    state.storage
        .set_setting("crash_reporting_enabled", &enabled)
        .await
        .storage_context("Failed to save crash reporting setting")?;
    
    // Set anonymous user context if enabled
    if enabled {
        // Get or generate anonymous ID
        let anonymous_id = match state.storage.get_setting::<String>("anonymous_id").await {
            Ok(Some(id)) => id,
            _ => {
                let id = uuid::Uuid::new_v4().to_string();
                let _ = state.storage.set_setting("anonymous_id", &id).await;
                id
            }
        };
        sentry_integration::set_user_context(&anonymous_id);
    } else {
        sentry_integration::clear_user_context();
    }
    
    Ok(())
}

/// Check if crash reporting is enabled
#[tauri::command]
pub async fn is_crash_reporting_enabled(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, IsolateError> {
    // Check persisted setting (Sentry state might not be initialized yet)
    let enabled = state.storage
        .get_setting::<bool>("crash_reporting_enabled")
        .await
        .unwrap_or(None)
        .unwrap_or(false);
    
    Ok(enabled)
}

/// Manually report an error to crash reporting
///
/// Used by frontend to report JavaScript errors.
#[tauri::command]
pub async fn report_crash_error(
    error_type: String,
    message: String,
    _context: Option<String>,
) -> Result<(), IsolateError> {
    if !sentry_integration::is_enabled() {
        return Ok(());
    }
    
    info!(error_type = %error_type, "Reporting error to crash reporting");
    
    // Add breadcrumb
    sentry_integration::add_breadcrumb(&error_type, &message);
    
    // Capture as message with context
    sentry_integration::capture_message(
        &format!("[{}] {}", error_type, message),
        sentry::Level::Error,
    );
    
    Ok(())
}

/// Get crash reporting privacy info
///
/// Returns information about what data is collected.
#[tauri::command]
pub async fn get_crash_reporting_info() -> Result<CrashReportingInfo, IsolateError> {
    Ok(CrashReportingInfo {
        enabled: sentry_integration::is_enabled(),
        privacy_url: "https://isolate.app/privacy".to_string(),
        data_collected: vec![
            "Error messages (anonymized)".to_string(),
            "Stack traces".to_string(),
            "App version".to_string(),
            "OS version".to_string(),
        ],
        data_not_collected: vec![
            "IP addresses".to_string(),
            "Usernames".to_string(),
            "File paths".to_string(),
            "Personal data".to_string(),
            "Browsing history".to_string(),
        ],
    })
}

/// Crash reporting privacy information
#[derive(serde::Serialize)]
pub struct CrashReportingInfo {
    pub enabled: bool,
    pub privacy_url: String,
    pub data_collected: Vec<String>,
    pub data_not_collected: Vec<String>,
}

// ============================================================================
// Config Updater Commands
// ============================================================================

use crate::core::config_updater::{ConfigUpdate, UpdateResult};

/// Check for config updates from remote repository
#[tauri::command]
pub async fn check_config_updates() -> Result<Vec<ConfigUpdate>, IsolateError> {
    info!("Checking for config updates");
    
    crate::core::config_updater::check_config_updates()
        .await
        .network_context("Failed to check config updates")
}

/// Download and apply config updates
#[tauri::command]
pub async fn download_config_updates() -> Result<UpdateResult, IsolateError> {
    // Rate limiting: 3 requests per minute (network operation)
    crate::commands::rate_limiter::check_rate_limit_with_config(
        "download_config_updates",
        crate::commands::rate_limiter::limits::DOWNLOAD_CONFIG_UPDATES,
    )?;
    
    info!("Downloading config updates");
    
    crate::core::config_updater::download_config_updates()
        .await
        .network_context("Failed to download config updates")
}

// ============================================================================
// AutoRun Commands
// ============================================================================

/// Get current autorun status
///
/// Returns true if the app is configured to start with Windows.
#[tauri::command]
pub async fn get_autorun_status() -> Result<bool, IsolateError> {
    info!("Checking autorun status");
    
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .io_context("Failed to open registry key")?;
        
        let result: Result<String, _> = run_key.get_value("Isolate");
        Ok(result.is_ok())
    }
    
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

/// Set autorun status
///
/// Enables or disables automatic startup with Windows.
/// When enabled, the app will start in silent mode (minimized to tray).
#[tauri::command]
pub async fn set_autorun(enabled: bool) -> Result<(), IsolateError> {
    info!(enabled, "Setting autorun status");
    
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
            .io_context("Failed to open registry key")?;
        
        if enabled {
            // Get current executable path
            let exe_path = std::env::current_exe()
                .io_context("Failed to get executable path")?;
            
            // Add --silent flag for autorun
            let value = format!("\"{}\" --silent", exe_path.display());
            
            run_key
                .set_value("Isolate", &value)
                .io_context("Failed to set registry value")?;
            
            info!("Autorun enabled");
        } else {
            // Remove the registry value (ignore error if it doesn't exist)
            let _ = run_key.delete_value("Isolate");
            info!("Autorun disabled");
        }
        
        Ok(())
    }
    
    #[cfg(not(windows))]
    {
        Err(IsolateError::Other("Autorun is only supported on Windows".to_string()))
    }
}

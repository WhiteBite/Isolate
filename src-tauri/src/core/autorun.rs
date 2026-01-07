//! AutoRun management for Windows startup
//!
//! Manages application autostart via Windows Registry.
//! Uses HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run
//!
//! NOTE: This module provides autorun functionality for Windows.
//! Functions are called from Tauri commands.

// Public API for autorun management
#![allow(dead_code)]

use crate::core::errors::IsolateError;
use tracing::{debug, error, info};

/// Registry key path for Windows autorun
const AUTORUN_REGISTRY_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

/// Application name in registry
const APP_NAME: &str = "Isolate";

/// Command line flag for minimized startup
const MINIMIZED_FLAG: &str = "--minimized";

/// Enable or disable autorun at Windows startup
///
/// When enabled, adds the application to Windows startup with --minimized flag.
/// When disabled, removes the application from Windows startup.
///
/// # Arguments
/// * `enable` - true to enable autorun, false to disable
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(IsolateError)` on failure
#[cfg(target_os = "windows")]
pub fn set_autorun_enabled(enable: bool) -> Result<(), IsolateError> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(AUTORUN_REGISTRY_KEY)
        .map_err(|e| IsolateError::Config(format!("Failed to open registry key: {}", e)))?;

    if enable {
        let exe_path = std::env::current_exe()
            .map_err(|e| IsolateError::Config(format!("Failed to get executable path: {}", e)))?;

        let autorun_value = format!("\"{}\" {}", exe_path.display(), MINIMIZED_FLAG);

        key.set_value(APP_NAME, &autorun_value)
            .map_err(|e| IsolateError::Config(format!("Failed to set autorun value: {}", e)))?;

        info!("Autorun enabled: {}", autorun_value);
    } else {
        // Try to delete, ignore error if key doesn't exist
        match key.delete_value(APP_NAME) {
            Ok(_) => info!("Autorun disabled"),
            Err(e) => {
                // ERROR_FILE_NOT_FOUND = 2
                if e.raw_os_error() == Some(2) {
                    debug!("Autorun was not enabled, nothing to disable");
                } else {
                    error!("Failed to delete autorun value: {}", e);
                    return Err(IsolateError::Config(format!(
                        "Failed to delete autorun value: {}",
                        e
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Check if autorun is currently enabled
///
/// # Returns
/// * `true` if application is set to start with Windows
/// * `false` if not set or on error
#[cfg(target_os = "windows")]
pub fn is_autorun_enabled() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let key = match hkcu.open_subkey(AUTORUN_REGISTRY_KEY) {
        Ok(k) => k,
        Err(e) => {
            debug!("Failed to open registry key: {}", e);
            return false;
        }
    };

    let value: Result<String, _> = key.get_value(APP_NAME);

    match value {
        Ok(v) => {
            debug!("Autorun value found: {}", v);
            true
        }
        Err(e) => {
            debug!("Autorun value not found: {}", e);
            false
        }
    }
}

/// Get the current autorun command if set
///
/// # Returns
/// * `Some(String)` with the autorun command if enabled
/// * `None` if not enabled or on error
#[cfg(target_os = "windows")]
pub fn get_autorun_command() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(AUTORUN_REGISTRY_KEY).ok()?;
    key.get_value(APP_NAME).ok()
}

// Stub implementations for non-Windows platforms (for compilation)
#[cfg(not(target_os = "windows"))]
pub fn set_autorun_enabled(_enable: bool) -> Result<(), IsolateError> {
    Err(IsolateError::Config(
        "Autorun is only supported on Windows".to_string(),
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn is_autorun_enabled() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn get_autorun_command() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_autorun_toggle() {
        // Note: This test modifies the registry
        // Disable first to ensure clean state
        let _ = set_autorun_enabled(false);
        assert!(!is_autorun_enabled());

        // Enable
        set_autorun_enabled(true).expect("Failed to enable autorun");
        assert!(is_autorun_enabled());
        assert!(get_autorun_command().is_some());

        // Disable
        set_autorun_enabled(false).expect("Failed to disable autorun");
        assert!(!is_autorun_enabled());
        assert!(get_autorun_command().is_none());
    }
}

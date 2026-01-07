//! QUIC blocking via Windows Firewall
//!
//! This module provides functionality to block QUIC protocol (UDP port 443)
//! using Windows Firewall rules. This forces browsers to fall back to TCP/TLS
//! which can then be processed by DPI bypass strategies.

#![allow(dead_code)] // Public QUIC blocking API

use tokio::process::Command;
use tracing::{debug, error, info, warn};

use super::errors::{IsolateError, Result};

/// Name of the firewall rule for QUIC blocking
const FIREWALL_RULE_NAME: &str = "Isolate Block QUIC";

/// Check if the current process is running with administrator privileges
pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use std::mem;
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token: HANDLE = 0;
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return false;
            }

            let mut elevation: TOKEN_ELEVATION = mem::zeroed();
            let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;

            let result = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );

            CloseHandle(token);

            result != 0 && elevation.TokenIsElevated != 0
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}

/// Enable QUIC blocking by adding a Windows Firewall rule
///
/// Creates an outbound firewall rule that blocks UDP traffic on port 443,
/// effectively disabling QUIC protocol and forcing browsers to use TCP.
pub async fn enable_quic_block() -> Result<()> {
    info!("Enabling QUIC block via Windows Firewall");

    if !is_admin() {
        error!("Administrator privileges required to modify firewall rules");
        return Err(IsolateError::RequiresAdmin);
    }

    // First check if rule already exists
    if is_quic_blocked().await? {
        info!("QUIC block rule already exists");
        return Ok(());
    }

    // Add firewall rule to block UDP 443 (outbound)
    let output = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "add",
            "rule",
            &format!("name={}", FIREWALL_RULE_NAME),
            "dir=out",
            "action=block",
            "protocol=UDP",
            "remoteport=443",
        ])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to execute netsh: {}", e)))?;

    if output.status.success() {
        info!("QUIC block rule added successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!(
            "Failed to add firewall rule. stdout: {}, stderr: {}",
            stdout, stderr
        );
        Err(IsolateError::Process(format!(
            "Failed to add firewall rule: {}",
            stderr
        )))
    }
}

/// Disable QUIC blocking by removing the Windows Firewall rule
pub async fn disable_quic_block() -> Result<()> {
    info!("Disabling QUIC block via Windows Firewall");

    if !is_admin() {
        error!("Administrator privileges required to modify firewall rules");
        return Err(IsolateError::RequiresAdmin);
    }

    // Check if rule exists before trying to delete
    if !is_quic_blocked().await? {
        info!("QUIC block rule does not exist, nothing to remove");
        return Ok(());
    }

    // Remove firewall rule
    let output = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "delete",
            "rule",
            &format!("name={}", FIREWALL_RULE_NAME),
        ])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to execute netsh: {}", e)))?;

    if output.status.success() {
        info!("QUIC block rule removed successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if error is because rule doesn't exist (which is fine)
        if stdout.contains("No rules match") || stderr.contains("No rules match") {
            warn!("QUIC block rule was not found (already removed?)");
            return Ok(());
        }
        
        error!(
            "Failed to remove firewall rule. stdout: {}, stderr: {}",
            stdout, stderr
        );
        Err(IsolateError::Process(format!(
            "Failed to remove firewall rule: {}",
            stderr
        )))
    }
}

/// Check if QUIC blocking is currently enabled
///
/// Returns true if the firewall rule exists, false otherwise
pub async fn is_quic_blocked() -> Result<bool> {
    debug!("Checking if QUIC block rule exists");

    let output = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={}", FIREWALL_RULE_NAME),
        ])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to execute netsh: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If rule exists, output will contain rule details
    // If not, it will say "No rules match the specified criteria"
    let exists = !stdout.contains("No rules match") && stdout.contains(FIREWALL_RULE_NAME);

    debug!("QUIC block rule exists: {}", exists);
    Ok(exists)
}

/// Check if running as admin (command wrapper)
pub fn check_admin() -> Result<()> {
    if is_admin() {
        Ok(())
    } else {
        Err(IsolateError::RequiresAdmin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin_returns_bool() {
        // Just verify it doesn't panic
        let _ = is_admin();
    }
}

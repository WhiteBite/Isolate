//! TCP Timestamps management module
//!
//! Provides functionality to enable/disable TCP timestamps via netsh.
//! TCP timestamps can help with some DPI systems by adding timing information
//! to TCP packets, which may confuse certain blocking mechanisms.
//!
//! Requires administrator privileges to modify TCP settings.

use tokio::process::Command;
use tracing::{debug, error, info, warn};

use super::errors::{IsolateError, Result};
use super::quic_blocker::is_admin;

/// TCP timestamps status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum TcpTimestampsStatus {
    /// TCP timestamps are enabled
    Enabled,
    /// TCP timestamps are disabled
    Disabled,
    /// Status could not be determined
    Unknown,
}

impl std::fmt::Display for TcpTimestampsStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TcpTimestampsStatus::Enabled => write!(f, "enabled"),
            TcpTimestampsStatus::Disabled => write!(f, "disabled"),
            TcpTimestampsStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Check current TCP timestamps status
///
/// Uses `netsh int tcp show global` to determine if timestamps are enabled.
/// Returns `TcpTimestampsStatus::Unknown` if the status cannot be determined.
pub async fn check_tcp_timestamps() -> Result<TcpTimestampsStatus> {
    debug!("Checking TCP timestamps status");

    let output = Command::new("netsh")
        .args(["int", "tcp", "show", "global"])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to execute netsh: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse output to find timestamps status
    // Output format varies by Windows version, but typically contains:
    // "Timestamps : enabled" or "Timestamps : disabled"
    // or "RFC 1323 Timestamps : enabled/disabled"
    
    let stdout_lower = stdout.to_lowercase();
    
    // Look for timestamps line
    for line in stdout_lower.lines() {
        if line.contains("timestamp") {
            if line.contains("enabled") {
                debug!("TCP timestamps are enabled");
                return Ok(TcpTimestampsStatus::Enabled);
            } else if line.contains("disabled") {
                debug!("TCP timestamps are disabled");
                return Ok(TcpTimestampsStatus::Disabled);
            }
        }
    }

    // If we couldn't find the status, log the output for debugging
    warn!("Could not determine TCP timestamps status from netsh output");
    debug!("netsh output: {}", stdout);
    
    Ok(TcpTimestampsStatus::Unknown)
}

/// Enable TCP timestamps
///
/// Uses `netsh int tcp set global timestamps=enabled` to enable timestamps.
/// Requires administrator privileges.
pub async fn enable_tcp_timestamps() -> Result<()> {
    info!("Enabling TCP timestamps");

    if !is_admin() {
        error!("Administrator privileges required to modify TCP settings");
        return Err(IsolateError::RequiresAdmin);
    }

    // Check current status first
    let current_status = check_tcp_timestamps().await?;
    if current_status == TcpTimestampsStatus::Enabled {
        info!("TCP timestamps are already enabled");
        return Ok(());
    }

    let output = Command::new("netsh")
        .args(["int", "tcp", "set", "global", "timestamps=enabled"])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to execute netsh: {}", e)))?;

    if output.status.success() {
        info!("TCP timestamps enabled successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!(
            "Failed to enable TCP timestamps. stdout: {}, stderr: {}",
            stdout, stderr
        );
        Err(IsolateError::Process(format!(
            "Failed to enable TCP timestamps: {} {}",
            stdout, stderr
        )))
    }
}

/// Disable TCP timestamps
///
/// Uses `netsh int tcp set global timestamps=disabled` to disable timestamps.
/// Requires administrator privileges.
pub async fn disable_tcp_timestamps() -> Result<()> {
    info!("Disabling TCP timestamps");

    if !is_admin() {
        error!("Administrator privileges required to modify TCP settings");
        return Err(IsolateError::RequiresAdmin);
    }

    // Check current status first
    let current_status = check_tcp_timestamps().await?;
    if current_status == TcpTimestampsStatus::Disabled {
        info!("TCP timestamps are already disabled");
        return Ok(());
    }

    let output = Command::new("netsh")
        .args(["int", "tcp", "set", "global", "timestamps=disabled"])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to execute netsh: {}", e)))?;

    if output.status.success() {
        info!("TCP timestamps disabled successfully");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!(
            "Failed to disable TCP timestamps. stdout: {}, stderr: {}",
            stdout, stderr
        );
        Err(IsolateError::Process(format!(
            "Failed to disable TCP timestamps: {} {}",
            stdout, stderr
        )))
    }
}

/// Set TCP timestamps status
///
/// Convenience function to enable or disable timestamps based on boolean.
pub async fn set_tcp_timestamps(enabled: bool) -> Result<()> {
    if enabled {
        enable_tcp_timestamps().await
    } else {
        disable_tcp_timestamps().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(TcpTimestampsStatus::Enabled.to_string(), "enabled");
        assert_eq!(TcpTimestampsStatus::Disabled.to_string(), "disabled");
        assert_eq!(TcpTimestampsStatus::Unknown.to_string(), "unknown");
    }
}

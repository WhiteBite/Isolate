//! QUIC blocking commands
//!
//! Commands for managing QUIC protocol blocking via Windows Firewall.
//! Blocking QUIC forces browsers to fall back to TCP/TLS connections,
//! which can then be processed by DPI bypass strategies.

use tracing::info;

/// Enable QUIC blocking via Windows Firewall
///
/// Adds a firewall rule to block UDP port 443 (QUIC protocol),
/// forcing browsers to fall back to TCP/TLS connections.
/// Requires administrator privileges.
#[tauri::command]
pub async fn enable_quic_block() -> Result<(), String> {
    info!("Command: enable_quic_block");

    crate::core::quic_blocker::enable_quic_block()
        .await
        .map_err(|e| e.to_string())
}

/// Disable QUIC blocking
///
/// Removes the firewall rule that blocks QUIC protocol.
/// Requires administrator privileges.
#[tauri::command]
pub async fn disable_quic_block() -> Result<(), String> {
    info!("Command: disable_quic_block");

    crate::core::quic_blocker::disable_quic_block()
        .await
        .map_err(|e| e.to_string())
}

/// Set QUIC blocking state
///
/// Enables or disables QUIC blocking based on the `enabled` parameter.
/// Requires administrator privileges.
#[tauri::command]
pub async fn set_quic_block(enabled: bool) -> Result<(), String> {
    info!("Command: set_quic_block enabled={}", enabled);

    if enabled {
        crate::core::quic_blocker::enable_quic_block()
            .await
            .map_err(|e| e.to_string())
    } else {
        crate::core::quic_blocker::disable_quic_block()
            .await
            .map_err(|e| e.to_string())
    }
}

/// Check if QUIC is currently blocked
///
/// Returns true if the QUIC blocking firewall rule exists.
#[tauri::command]
pub async fn is_quic_blocked() -> Result<bool, String> {
    info!("Command: is_quic_blocked");

    crate::core::quic_blocker::is_quic_blocked()
        .await
        .map_err(|e| e.to_string())
}

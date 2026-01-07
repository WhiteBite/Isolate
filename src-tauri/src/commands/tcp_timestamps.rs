//! TCP Timestamps Tauri commands
//!
//! Commands for checking and modifying TCP timestamps settings.

use tracing::info;

use crate::core::errors::IsolateError;
use crate::core::tcp_timestamps::{
    check_tcp_timestamps, set_tcp_timestamps, TcpTimestampsStatus,
};

/// Get current TCP timestamps status
///
/// Returns the current status of TCP timestamps (enabled/disabled/unknown).
/// Does not require admin privileges to check.
#[tauri::command]
pub async fn get_tcp_timestamps_status() -> Result<TcpTimestampsStatus, IsolateError> {
    info!("Getting TCP timestamps status");
    check_tcp_timestamps().await
}

/// Set TCP timestamps enabled/disabled
///
/// Enables or disables TCP timestamps based on the `enabled` parameter.
/// Requires administrator privileges.
#[tauri::command]
pub async fn set_tcp_timestamps_enabled(enabled: bool) -> Result<(), IsolateError> {
    info!(enabled, "Setting TCP timestamps");
    set_tcp_timestamps(enabled).await
}

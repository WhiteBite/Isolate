//! TUN mode commands
//!
//! Commands for managing TUN (transparent proxy) mode that routes
//! all system traffic through a SOCKS proxy.

use tracing::info;

use crate::commands::validation::validate_port;
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::tun_manager::{TunConfig, TunInstance};

// ============================================================================
// TUN Mode Commands
// ============================================================================

/// Start TUN mode
///
/// Routes all system traffic through the specified SOCKS proxy.
/// Requires administrator privileges.
#[tauri::command]
pub async fn start_tun(socks_port: u16) -> Result<TunInstance, String> {
    // Rate limit: max 1 call per 10 seconds
    crate::commands::rate_limiter::check_rate_limit("start_tun", 10)
        .map_err(|e| e.to_string())?;
    
    validate_port(socks_port).map_err(|e| e.to_string())?;
    
    info!(socks_port, "Starting TUN mode");

    let manager = crate::core::tun_manager::get_tun_manager();

    manager
        .start(socks_port)
        .await
        .process_context("Failed to start TUN")
        .map_err(|e: IsolateError| e.to_string())
}

/// Stop TUN mode
///
/// Gracefully stops TUN and restores original network configuration.
#[tauri::command]
pub async fn stop_tun() -> Result<(), String> {
    info!("Stopping TUN mode");

    let manager = crate::core::tun_manager::get_tun_manager();

    manager
        .stop()
        .await
        .process_context("Failed to stop TUN")
        .map_err(|e: IsolateError| e.to_string())
}

/// Check if TUN is running
#[tauri::command]
pub async fn is_tun_running() -> Result<bool, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.is_running().await)
}

/// Get TUN status
///
/// Returns current TUN instance information including port and interface.
#[tauri::command]
pub async fn get_tun_status() -> Result<TunInstance, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.get_instance().await)
}

/// Get TUN configuration
#[tauri::command]
pub async fn get_tun_config() -> Result<TunConfig, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.get_config().await)
}

/// Update TUN configuration
///
/// Note: Changes take effect on next TUN start
#[tauri::command]
pub async fn set_tun_config(config: TunConfig) -> Result<(), String> {
    info!(
        interface = %config.interface_name,
        mtu = config.mtu,
        "Updating TUN config"
    );

    let manager = crate::core::tun_manager::get_tun_manager();
    manager.set_config(config).await;

    Ok(())
}

/// Check if TUN mode is available
///
/// Returns true if sing-box exists and running with admin privileges
#[tauri::command]
pub async fn is_tun_available() -> Result<bool, String> {
    Ok(crate::core::tun_manager::is_tun_available())
}

/// Restart TUN with optional new SOCKS port
///
/// Gracefully restarts TUN mode, optionally with a new SOCKS port.
#[tauri::command]
pub async fn restart_tun(socks_port: Option<u16>) -> Result<TunInstance, String> {
    info!(socks_port = ?socks_port, "Restarting TUN mode");

    let manager = crate::core::tun_manager::get_tun_manager();

    manager
        .restart(socks_port)
        .await
        .process_context("Failed to restart TUN")
        .map_err(|e: IsolateError| e.to_string())
}

//! DNS management commands
//!
//! Tauri IPC команды для управления настройками DNS.

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::dns_manager::{DnsServer, DnsSettings};
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::state::AppState;

/// Get current DNS settings
///
/// Returns the current DNS configuration from storage.
#[tauri::command]
pub async fn get_dns_settings(state: State<'_, Arc<AppState>>) -> Result<DnsSettings, IsolateError> {
    info!("Getting DNS settings");

    crate::core::dns_manager::get_dns_settings(&state.storage)
        .await
        .storage_context("Failed to get DNS settings")
}

/// Set DNS server
///
/// Updates the DNS server configuration.
///
/// # Arguments
/// * `server` - DNS server type (system, cloudflare, google, quad9, openDns, adGuard, custom)
/// * `custom_address` - Custom DNS address (required when server is "custom")
#[tauri::command]
pub async fn set_dns_server(
    state: State<'_, Arc<AppState>>,
    server: DnsServer,
    custom_address: Option<String>,
) -> Result<(), IsolateError> {
    info!(server = ?server, custom_address = ?custom_address, "Setting DNS server");

    crate::core::dns_manager::set_dns_server(&state.storage, server, custom_address)
        .await
        .storage_context("Failed to set DNS server")
}

/// Save full DNS settings
///
/// Saves complete DNS settings including DoH configuration.
#[tauri::command]
pub async fn save_dns_settings(
    state: State<'_, Arc<AppState>>,
    settings: DnsSettings,
) -> Result<(), IsolateError> {
    info!(
        server = ?settings.server,
        doh_enabled = settings.doh_enabled,
        "Saving DNS settings"
    );

    crate::core::dns_manager::set_dns_settings(&state.storage, &settings)
        .await
        .storage_context("Failed to save DNS settings")
}

/// Reset DNS settings to defaults
///
/// Resets DNS configuration to use system DNS.
#[tauri::command]
pub async fn reset_dns_settings(state: State<'_, Arc<AppState>>) -> Result<(), IsolateError> {
    info!("Resetting DNS settings to defaults");

    crate::core::dns_manager::reset_dns_settings(&state.storage)
        .await
        .storage_context("Failed to reset DNS settings")
}

/// Apply DNS settings to system
///
/// Applies the current DNS configuration to Windows network interfaces.
/// Requires administrator privileges.
#[tauri::command]
pub async fn apply_dns_to_system(state: State<'_, Arc<AppState>>) -> Result<(), IsolateError> {
    info!("Applying DNS settings to system");

    let settings = crate::core::dns_manager::get_dns_settings(&state.storage)
        .await
        .storage_context("Failed to get DNS settings")?;

    crate::core::dns_manager::apply_dns_to_system(&settings)
        .await
        .process_context("Failed to apply DNS to system")
}

/// Restore system DNS to DHCP
///
/// Restores DNS configuration to use DHCP (automatic).
/// Requires administrator privileges.
#[tauri::command]
pub async fn restore_system_dns() -> Result<(), IsolateError> {
    info!("Restoring system DNS to DHCP");

    crate::core::dns_manager::restore_system_dns()
        .await
        .process_context("Failed to restore system DNS")
}

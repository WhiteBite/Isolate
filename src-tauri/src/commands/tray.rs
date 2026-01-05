//! System tray commands
//!
//! Commands for managing system tray state, icon, and menu from frontend.

use tauri::AppHandle;
use tracing::info;

/// Update tray status from frontend
#[tauri::command]
pub async fn update_tray(
    app: AppHandle,
    state: String,
    strategy_name: Option<String>,
) -> Result<(), String> {
    info!(state = %state, strategy = ?strategy_name, "Updating tray from frontend");
    
    let tray_state = crate::tray::TrayState::from_str(&state);
    crate::tray::update_tray_state(&app, tray_state, strategy_name);
    
    Ok(())
}

/// Set tray to optimizing state
#[tauri::command]
pub async fn set_tray_optimizing(app: AppHandle) -> Result<(), String> {
    info!("Setting tray to optimizing state");
    crate::tray::set_tray_optimizing(&app);
    Ok(())
}

/// Set tray to error state
#[tauri::command]
pub async fn set_tray_error(app: AppHandle, error_msg: String) -> Result<(), String> {
    info!(error = %error_msg, "Setting tray to error state");
    crate::tray::set_tray_error(&app, &error_msg);
    Ok(())
}

/// Get current tray state
#[tauri::command]
pub async fn get_tray_state() -> Result<String, String> {
    let state = crate::tray::get_tray_state();
    let state_str = match state {
        crate::tray::TrayState::Inactive => "inactive",
        crate::tray::TrayState::Active => "active",
        crate::tray::TrayState::Optimizing => "optimizing",
        crate::tray::TrayState::Error => "error",
    };
    Ok(state_str.to_string())
}

/// Update tray TUN mode status
#[tauri::command]
pub async fn update_tray_tun_status(app: AppHandle, is_tun: bool) -> Result<(), String> {
    info!(is_tun, "Updating tray TUN status");
    crate::tray::update_tray_tun_status(&app, is_tun);
    Ok(())
}

/// Update tray System Proxy status
#[tauri::command]
pub async fn update_tray_proxy_status(app: AppHandle, is_proxy: bool) -> Result<(), String> {
    info!(is_proxy, "Updating tray System Proxy status");
    crate::tray::update_tray_proxy_status(&app, is_proxy);
    Ok(())
}

/// Rebuild tray menu (force refresh)
#[tauri::command]
pub async fn rebuild_tray_menu(app: AppHandle) -> Result<(), String> {
    info!("Rebuilding tray menu");
    crate::tray::rebuild_tray_menu(&app);
    Ok(())
}

//! System tray commands
//!
//! Commands for managing system tray state, icon, and menu from frontend.
//! Includes service protection management and profile switching.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::info;

// ============================================================================
// Data Structures
// ============================================================================

/// Top service information for tray menu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopService {
    /// Service identifier (e.g., "youtube", "discord")
    pub id: String,
    /// Display name (e.g., "YouTube", "Discord")
    pub name: String,
    /// Emoji icon for the service
    pub icon: String,
    /// Whether protection is currently enabled for this service
    pub is_protected: bool,
    /// Usage count (for sorting by popularity)
    pub usage_count: u32,
}

/// Profile mode for tray menu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMode {
    /// Profile identifier (e.g., "default", "game", "work")
    pub id: String,
    /// Display name (e.g., "Default", "Game Mode")
    pub name: String,
    /// Whether this profile is currently active
    pub is_active: bool,
}

/// Update tray status from frontend
#[tauri::command]
pub async fn update_tray(
    app: AppHandle,
    state: String,
    strategy_name: Option<String>,
) -> Result<(), String> {
    info!(state = %state, strategy = ?strategy_name, "Updating tray from frontend");
    
    let tray_state = crate::tray::TrayState::parse(&state);
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

// ============================================================================
// Service Protection Commands
// ============================================================================

/// Toggle protection for a specific service
/// 
/// Enables or disables DPI bypass protection for the given service.
/// Updates the tray menu to reflect the new state.
#[tauri::command]
pub async fn toggle_service_protection(
    app: AppHandle,
    service_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!(
        service_id = %service_id,
        enabled = enabled,
        "Toggling service protection"
    );
    
    // Update tray state
    if enabled {
        // Add service to active list if not present
        let mut services = crate::tray::get_active_services();
        if !services.contains(&service_id) {
            services.push(service_id.clone());
            crate::tray::update_tray_services(&app, services);
        }
    } else {
        // Remove service from active list
        let services: Vec<String> = crate::tray::get_active_services()
            .into_iter()
            .filter(|s| s != &service_id)
            .collect();
        crate::tray::update_tray_services(&app, services);
    }
    
    // TODO: Implement actual protection toggle logic
    // This will integrate with strategy_engine to apply/remove service-specific rules
    
    info!(
        service_id = %service_id,
        enabled = enabled,
        "Service protection toggled successfully"
    );
    
    Ok(())
}

/// Get top N services by usage count
/// 
/// Returns the most frequently used services for display in the tray menu.
/// Currently returns mock data - will be replaced with actual usage statistics.
#[tauri::command]
pub async fn get_top_services(limit: u32) -> Result<Vec<TopService>, String> {
    info!(limit = limit, "Getting top services");
    
    // Mock data for now - will be replaced with actual service registry data
    let all_services = vec![
        TopService {
            id: "youtube".to_string(),
            name: "YouTube".to_string(),
            icon: "🎬".to_string(),
            is_protected: crate::tray::is_service_active("youtube"),
            usage_count: 1500,
        },
        TopService {
            id: "discord".to_string(),
            name: "Discord".to_string(),
            icon: "💬".to_string(),
            is_protected: crate::tray::is_service_active("discord"),
            usage_count: 1200,
        },
        TopService {
            id: "telegram".to_string(),
            name: "Telegram".to_string(),
            icon: "✈️".to_string(),
            is_protected: crate::tray::is_service_active("telegram"),
            usage_count: 1000,
        },
        TopService {
            id: "twitch".to_string(),
            name: "Twitch".to_string(),
            icon: "🎮".to_string(),
            is_protected: crate::tray::is_service_active("twitch"),
            usage_count: 800,
        },
        TopService {
            id: "steam".to_string(),
            name: "Steam".to_string(),
            icon: "🎯".to_string(),
            is_protected: crate::tray::is_service_active("steam"),
            usage_count: 600,
        },
        TopService {
            id: "spotify".to_string(),
            name: "Spotify".to_string(),
            icon: "🎵".to_string(),
            is_protected: crate::tray::is_service_active("spotify"),
            usage_count: 500,
        },
        TopService {
            id: "instagram".to_string(),
            name: "Instagram".to_string(),
            icon: "📷".to_string(),
            is_protected: crate::tray::is_service_active("instagram"),
            usage_count: 400,
        },
        TopService {
            id: "twitter".to_string(),
            name: "Twitter/X".to_string(),
            icon: "🐦".to_string(),
            is_protected: crate::tray::is_service_active("twitter"),
            usage_count: 350,
        },
    ];
    
    // Sort by usage count and take top N
    let mut services = all_services;
    services.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
    let top_services: Vec<TopService> = services.into_iter().take(limit as usize).collect();
    
    info!(
        count = top_services.len(),
        "Returning top services"
    );
    
    Ok(top_services)
}

// ============================================================================
// Profile Mode Commands
// ============================================================================

/// Set the current profile mode
/// 
/// Changes the active profile (default, game, work) which affects
/// which services are protected and how aggressively.
#[tauri::command]
pub async fn set_profile_mode(app: AppHandle, profile: String) -> Result<(), String> {
    info!(profile = %profile, "Setting profile mode");
    
    // Update tray profile
    crate::tray::set_tray_profile_by_name(&app, &profile);
    
    // TODO: Implement actual profile switching logic
    // This will:
    // 1. Load profile-specific service list
    // 2. Apply profile-specific strategy settings
    // 3. Update active services based on profile
    
    info!(profile = %profile, "Profile mode set successfully");
    
    Ok(())
}

/// Get the current profile mode
/// 
/// Returns information about the currently active profile.
#[tauri::command]
pub async fn get_current_profile() -> Result<ProfileMode, String> {
    let current = crate::tray::get_current_profile();
    
    let profile = ProfileMode {
        id: format!("{:?}", current).to_lowercase(),
        name: current.display_name().to_string(),
        is_active: true,
    };
    
    info!(
        profile_id = %profile.id,
        profile_name = %profile.name,
        "Returning current profile"
    );
    
    Ok(profile)
}

/// Get all available profile modes
/// 
/// Returns a list of all profiles with their active status.
#[tauri::command]
pub async fn get_all_profiles() -> Result<Vec<ProfileMode>, String> {
    let current = crate::tray::get_current_profile();
    
    let profiles = vec![
        ProfileMode {
            id: "default".to_string(),
            name: "Default".to_string(),
            is_active: current == crate::tray::TrayProfile::Default,
        },
        ProfileMode {
            id: "game".to_string(),
            name: "Game Mode".to_string(),
            is_active: current == crate::tray::TrayProfile::GameMode,
        },
        ProfileMode {
            id: "work".to_string(),
            name: "Work Mode".to_string(),
            is_active: current == crate::tray::TrayProfile::WorkMode,
        },
    ];
    
    info!("Returning all profiles");
    
    Ok(profiles)
}

// ============================================================================
// Service State Commands
// ============================================================================

/// Get list of currently protected services
#[tauri::command]
pub async fn get_protected_services() -> Result<Vec<String>, String> {
    let services = crate::tray::get_active_services();
    info!(count = services.len(), "Returning protected services");
    Ok(services)
}

/// Set multiple services protection state at once
/// 
/// Useful for bulk operations like "protect all" or "unprotect all"
#[tauri::command]
pub async fn set_services_protection(
    app: AppHandle,
    service_ids: Vec<String>,
) -> Result<(), String> {
    info!(
        count = service_ids.len(),
        services = ?service_ids,
        "Setting services protection"
    );
    
    crate::tray::update_tray_services(&app, service_ids);
    
    Ok(())
}

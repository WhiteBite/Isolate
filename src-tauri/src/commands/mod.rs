//! Tauri commands — IPC interface between frontend and backend

use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, Window};
use tauri_plugin_updater::UpdaterExt;
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::core::binaries::{self, BinaryCheckResult, DownloadProgress};
use crate::core::hostlists::{self, Hostlist};
use crate::core::models::{AppStatus, DiagnosticResult, LogEntry, Service, ServiceWithState, Settings, Strategy, UpdateInfo, VlessConfig};
use crate::core::diagnostics::{DualStackResult, Ipv6Status};
use crate::state::AppState;

/// Get current application status
#[tauri::command]
pub async fn get_status(state: State<'_, Arc<AppState>>) -> Result<AppStatus, String> {
    info!("Getting application status");
    
    let strategy_engine = &state.strategy_engine;
    let current_strategy = strategy_engine.get_global_strategy().await;
    
    Ok(AppStatus {
        is_active: current_strategy.is_some(),
        current_strategy: current_strategy.clone(),
        current_strategy_name: current_strategy,
        services_status: std::collections::HashMap::new(),
    })
}

/// Get list of available strategies
#[tauri::command]
pub async fn get_strategies(state: State<'_, Arc<AppState>>) -> Result<Vec<Strategy>, String> {
    info!("Loading strategies");
    
    let strategies_map = state
        .config_manager
        .load_strategies()
        .await
        .map_err(|e| format!("Failed to load strategies: {}", e))?;
    
    Ok(strategies_map.into_values().collect())
}

/// Get list of services to test
#[tauri::command]
pub async fn get_services(state: State<'_, Arc<AppState>>) -> Result<Vec<Service>, String> {
    info!("Loading services");
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    Ok(services_map.into_values().collect())
}

/// Run optimization process
#[tauri::command]
pub async fn run_optimization(
    window: Window,
    state: State<'_, Arc<AppState>>,
    mode: String,
) -> Result<String, String> {
    info!("Starting optimization in {} mode", mode);
    
    // Emit initial progress
    let _ = window.emit(
        "optimization:progress",
        crate::core::orchestrator::OptimizationProgress::default(),
    );
    
    // Get strategies and services
    let strategies_map = state
        .config_manager
        .load_strategies()
        .await
        .map_err(|e| format!("Failed to load strategies: {}", e))?;
    
    let strategies: Vec<Strategy> = strategies_map.into_values().collect();
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    let service_ids: Vec<String> = services_map.keys().cloned().collect();
    
    // Get environment info
    let env_info = state.get_env_info().await;
    
    // Clone state for async task
    let orchestrator = state.orchestrator.clone();
    let window_clone = window.clone();
    
    // Subscribe to progress events
    let mut progress_rx = orchestrator.subscribe_progress();
    
    // Spawn progress listener
    tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            if let Err(e) = window_clone.emit("optimization:progress", &progress) {
                error!("Failed to emit progress: {}", e);
            }
        }
    });
    
    // Run optimization
    match orchestrator.optimize(&env_info, &strategies, &service_ids).await {
        Ok(result) => {
            info!(
                strategy_id = %result.strategy_id,
                score = result.score,
                "Optimization completed successfully"
            );
            
            let _ = window.emit("optimization:complete", serde_json::json!({
                "strategy_id": result.strategy_id,
                "strategy_name": result.strategy_name,
                "score": result.score,
                "from_cache": result.from_cache,
            }));
            
            Ok(result.strategy_id)
        }
        Err(e) => {
            error!("Optimization failed: {}", e);
            let _ = window.emit("optimization:failed", e.to_string());
            Err(format!("Optimization failed: {}", e))
        }
    }
}

/// Cancel ongoing optimization
#[tauri::command]
pub async fn cancel_optimization(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Cancelling optimization");
    state.orchestrator.cancel().await;
    Ok(())
}

/// Apply specific strategy
#[tauri::command]
pub async fn apply_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<(), String> {
    info!("Applying strategy: {}", strategy_id);
    
    // Load strategy by ID
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await
        .map_err(|e| format!("Strategy not found: {}", e))?;
    
    state
        .strategy_engine
        .start_global(&strategy)
        .await
        .map_err(|e| format!("Failed to apply strategy: {}", e))
}

/// Stop current strategy
#[tauri::command]
pub async fn stop_strategy(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Stopping current strategy");
    
    state
        .strategy_engine
        .stop_global()
        .await
        .map_err(|e| format!("Failed to stop strategy: {}", e))
}

/// Run DPI diagnostics
#[tauri::command]
pub async fn diagnose() -> Result<DiagnosticResult, String> {
    info!("Running DPI diagnostics");
    
    let profile = crate::core::diagnostics::diagnose()
        .await
        .map_err(|e| format!("Diagnostics failed: {}", e))?;
    
    Ok(DiagnosticResult {
        profile,
        tested_services: vec![],
        blocked_services: vec![],
    })
}

/// Run dual-stack (IPv4/IPv6) diagnostics
#[tauri::command]
pub async fn diagnose_dual_stack() -> Result<DualStackResult, String> {
    info!("Running dual-stack diagnostics");
    
    crate::core::diagnostics::diagnose_dual_stack()
        .await
        .map_err(|e| format!("Dual-stack diagnostics failed: {}", e))
}

/// Check IPv6 availability
#[tauri::command]
pub async fn check_ipv6() -> Result<Ipv6Status, String> {
    info!("Checking IPv6 availability");
    
    Ok(crate::core::diagnostics::check_ipv6_availability().await)
}

/// Emergency network reset
#[tauri::command]
pub async fn panic_reset(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    warn!("Panic reset triggered!");
    
    // 1. Stop all running strategies
    if let Err(e) = state.strategy_engine.shutdown_all().await {
        error!("Failed to shutdown strategies: {}", e);
    }
    
    // 2. Reset network (Windows specific)
    #[cfg(windows)]
    {
        // Winsock reset
        let _ = Command::new("netsh")
            .args(["winsock", "reset"])
            .output()
            .await;
        
        // Flush DNS
        let _ = Command::new("ipconfig")
            .args(["/flushdns"])
            .output()
            .await;
        
        info!("Network reset commands executed");
    }
    
    Ok(())
}

// ============================================================================
// Settings Commands
// ============================================================================

/// Get user settings
#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    info!("Getting user settings");
    
    state
        .storage
        .get_settings()
        .map_err(|e| format!("Failed to get settings: {}", e))
}

/// Save user settings
#[tauri::command]
pub async fn save_settings(
    state: State<'_, Arc<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    info!("Saving user settings");
    
    state
        .storage
        .save_settings(&settings)
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Get services with their enabled/disabled state
#[tauri::command]
pub async fn get_services_settings(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ServiceWithState>, String> {
    info!("Loading services with settings");
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    let mut services_with_state = Vec::new();
    
    for service in services_map.into_values() {
        let enabled = state
            .storage
            .get_service_enabled(&service.id)
            .unwrap_or(service.enabled_by_default);
        
        services_with_state.push(ServiceWithState {
            id: service.id,
            name: service.name,
            enabled,
            critical: service.critical,
        });
    }
    
    Ok(services_with_state)
}

/// Toggle a service's enabled state
#[tauri::command]
pub async fn toggle_service(
    state: State<'_, Arc<AppState>>,
    service_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!(service_id = %service_id, enabled, "Toggling service");
    
    state
        .storage
        .set_service_enabled(&service_id, enabled)
        .map_err(|e| format!("Failed to toggle service: {}", e))
}

// ============================================================================
// Generic Setting Commands
// ============================================================================

/// Get a setting by key
#[tauri::command]
pub async fn get_setting(
    state: State<'_, Arc<AppState>>,
    key: String,
) -> Result<serde_json::Value, String> {
    info!(key = %key, "Getting setting");
    
    let value: Option<serde_json::Value> = state
        .storage
        .get_setting(&key)
        .map_err(|e| format!("Failed to get setting: {}", e))?;
    
    Ok(value.unwrap_or(serde_json::Value::Null))
}

/// Set a setting by key
#[tauri::command]
pub async fn set_setting(
    state: State<'_, Arc<AppState>>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    info!(key = %key, "Setting setting");
    
    state
        .storage
        .set_setting(&key, &value)
        .map_err(|e| format!("Failed to set setting: {}", e))
}

/// Get app version from Cargo.toml
#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

// ============================================================================
// Update Commands
// ============================================================================

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    info!("Checking for updates");
    
    let updater = app.updater().map_err(|e| format!("Failed to get updater: {}", e))?;
    
    match updater.check().await {
        Ok(Some(update)) => {
            info!(version = %update.version, "Update available");
            Ok(Some(UpdateInfo {
                version: update.version.clone(),
                notes: update.body.clone(),
                date: update.date.map(|d| d.to_string()),
            }))
        }
        Ok(None) => {
            info!("No updates available");
            Ok(None)
        }
        Err(e) => {
            error!("Failed to check for updates: {}", e);
            Err(format!("Failed to check for updates: {}", e))
        }
    }
}

/// Download and install available update
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    info!("Installing update");
    
    let updater = app.updater().map_err(|e| format!("Failed to get updater: {}", e))?;
    
    let update = updater
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?
        .ok_or_else(|| "No update available".to_string())?;
    
    info!(version = %update.version, "Downloading update");
    
    // Download and install the update
    update
        .download_and_install(|_downloaded, _total| {}, || {})
        .await
        .map_err(|e| format!("Failed to install update: {}", e))?;
    
    info!("Update installed successfully, restart required");
    
    Ok(())
}

// ============================================================================
// Log Commands
// ============================================================================

/// Get recent logs
#[tauri::command]
pub async fn get_logs() -> Result<Vec<LogEntry>, String> {
    // Return empty for now, will be implemented with log capture
    Ok(vec![])
}

/// Export logs to file
#[tauri::command]
pub async fn export_logs() -> Result<String, String> {
    // Save logs to file and return path
    Ok("logs.txt".to_string())
}


// ============================================================================
// Binary Management Commands
// ============================================================================

/// Check if all required binaries are present
#[tauri::command]
pub async fn check_binaries() -> Result<BinaryCheckResult, String> {
    info!("Checking required binaries");
    
    binaries::check_binaries()
        .await
        .map_err(|e| format!("Failed to check binaries: {}", e))
}

/// Download missing binaries with progress reporting
#[tauri::command]
pub async fn download_binaries(window: Window) -> Result<(), String> {
    info!("Starting binary download");
    
    let window_clone = window.clone();
    
    binaries::ensure_binaries(move |progress: DownloadProgress| {
        if let Err(e) = window_clone.emit("binaries:progress", &progress) {
            error!("Failed to emit download progress: {}", e);
        }
    })
    .await
    .map_err(|e| {
        error!("Binary download failed: {}", e);
        format!("Failed to download binaries: {}", e)
    })?;
    
    let _ = window.emit("binaries:complete", ());
    info!("Binary download completed");
    
    Ok(())
}

/// Get path to binaries directory
#[tauri::command]
pub async fn get_binaries_dir() -> Result<String, String> {
    Ok(crate::core::paths::get_binaries_dir().display().to_string())
}

// ============================================================================
// QUIC Blocking Commands
// ============================================================================

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

/// Check if running with administrator privileges
#[tauri::command]
pub async fn is_admin() -> Result<bool, String> {
    Ok(crate::core::quic_blocker::is_admin())
}

// ============================================================================
// VLESS Commands
// ============================================================================

/// Storage key for VLESS configs
const VLESS_CONFIGS_KEY: &str = "vless_configs";

/// Import VLESS config from vless:// URL
#[tauri::command]
pub async fn import_vless(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<VlessConfig, String> {
    info!("Importing VLESS config from URL");

    // Parse the URL
    let config = VlessConfig::from_url(&url)?;

    // Load existing configs
    let mut configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .map_err(|e| format!("Failed to load configs: {}", e))?
        .unwrap_or_default();

    // Check for duplicate UUID
    if configs.iter().any(|c| c.uuid == config.uuid && c.server == config.server) {
        return Err("Config with this UUID and server already exists".to_string());
    }

    // Add new config
    configs.push(config.clone());

    // Save
    state
        .storage
        .set_setting(VLESS_CONFIGS_KEY, &configs)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    info!(id = %config.id, name = %config.name, "VLESS config imported");
    Ok(config)
}

/// Get all saved VLESS configs
#[tauri::command]
pub async fn get_vless_configs(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<VlessConfig>, String> {
    info!("Loading VLESS configs");

    let configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .map_err(|e| format!("Failed to load configs: {}", e))?
        .unwrap_or_default();

    Ok(configs)
}

/// Delete VLESS config by ID
#[tauri::command]
pub async fn delete_vless_config(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    info!(id = %id, "Deleting VLESS config");

    // Load existing configs
    let mut configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .map_err(|e| format!("Failed to load configs: {}", e))?
        .unwrap_or_default();

    // Find and remove
    let initial_len = configs.len();
    configs.retain(|c| c.id != id);

    if configs.len() == initial_len {
        return Err("Config not found".to_string());
    }

    // Save
    state
        .storage
        .set_setting(VLESS_CONFIGS_KEY, &configs)
        .map_err(|e| format!("Failed to save configs: {}", e))?;

    info!(id = %id, "VLESS config deleted");
    Ok(())
}

/// Toggle VLESS config active state
#[tauri::command]
pub async fn toggle_vless_config(
    state: State<'_, Arc<AppState>>,
    id: String,
    active: bool,
) -> Result<(), String> {
    info!(id = %id, active, "Toggling VLESS config");

    // Load existing configs
    let mut configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .map_err(|e| format!("Failed to load configs: {}", e))?
        .unwrap_or_default();

    // Find and update
    let mut found = false;
    for config in &mut configs {
        if config.id == id {
            config.active = active;
            found = true;
        } else if active {
            // Deactivate other configs when activating one
            config.active = false;
        }
    }

    if !found {
        return Err("Config not found".to_string());
    }

    // Save
    state
        .storage
        .set_setting(VLESS_CONFIGS_KEY, &configs)
        .map_err(|e| format!("Failed to save configs: {}", e))?;

    Ok(())
}

// ============================================================================
// Hostlist Management Commands
// ============================================================================

/// Get all available hostlists
#[tauri::command]
pub async fn get_hostlists() -> Result<Vec<Hostlist>, String> {
    info!("Loading all hostlists");

    hostlists::get_all_hostlists()
        .await
        .map_err(|e| format!("Failed to load hostlists: {}", e))
}

/// Get a specific hostlist by ID
#[tauri::command]
pub async fn get_hostlist(id: String) -> Result<Hostlist, String> {
    info!(id = %id, "Loading hostlist");

    hostlists::load_hostlist(&id)
        .await
        .map_err(|e| format!("Failed to load hostlist: {}", e))
}

/// Add domain to a hostlist
#[tauri::command]
pub async fn add_hostlist_domain(hostlist_id: String, domain: String) -> Result<(), String> {
    info!(hostlist_id = %hostlist_id, domain = %domain, "Adding domain to hostlist");

    hostlists::add_domain(&hostlist_id, &domain)
        .await
        .map_err(|e| format!("Failed to add domain: {}", e))
}

/// Remove domain from a hostlist
#[tauri::command]
pub async fn remove_hostlist_domain(hostlist_id: String, domain: String) -> Result<(), String> {
    info!(hostlist_id = %hostlist_id, domain = %domain, "Removing domain from hostlist");

    hostlists::remove_domain(&hostlist_id, &domain)
        .await
        .map_err(|e| format!("Failed to remove domain: {}", e))
}

/// Create a new hostlist
#[tauri::command]
pub async fn create_hostlist(id: String, name: String) -> Result<Hostlist, String> {
    info!(id = %id, name = %name, "Creating new hostlist");

    hostlists::create_hostlist(&id, &name)
        .await
        .map_err(|e| format!("Failed to create hostlist: {}", e))
}

/// Delete a hostlist
#[tauri::command]
pub async fn delete_hostlist(id: String) -> Result<(), String> {
    info!(id = %id, "Deleting hostlist");

    hostlists::delete_hostlist(&id)
        .await
        .map_err(|e| format!("Failed to delete hostlist: {}", e))
}

/// Update hostlist from remote URL
#[tauri::command]
pub async fn update_hostlist_from_url(id: String, url: String) -> Result<Hostlist, String> {
    info!(id = %id, url = %url, "Updating hostlist from URL");

    hostlists::update_hostlist(&id, &url)
        .await
        .map_err(|e| format!("Failed to update hostlist: {}", e))?;

    // Return updated hostlist
    hostlists::load_hostlist(&id)
        .await
        .map_err(|e| format!("Failed to load updated hostlist: {}", e))
}

/// Save hostlist with new domains
#[tauri::command]
pub async fn save_hostlist(hostlist: Hostlist) -> Result<(), String> {
    info!(id = %hostlist.id, domain_count = hostlist.domains.len(), "Saving hostlist");

    hostlists::save_hostlist(&hostlist)
        .await
        .map_err(|e| format!("Failed to save hostlist: {}", e))
}

// ============================================================================
// VLESS Proxy Control Commands
// ============================================================================

/// Start VLESS proxy for a specific config
///
/// Starts sing-box with the given VLESS configuration.
/// Returns the SOCKS port for the proxy.
#[tauri::command]
pub async fn start_vless_proxy(
    state: State<'_, Arc<AppState>>,
    config_id: String,
    socks_port: Option<u16>,
) -> Result<crate::core::singbox_manager::SingboxInstance, String> {
    info!(config_id = %config_id, "Starting VLESS proxy");

    // Load the config
    let configs: Vec<VlessConfig> = state
        .storage
        .get_setting(VLESS_CONFIGS_KEY)
        .map_err(|e| format!("Failed to load configs: {}", e))?
        .unwrap_or_default();

    let config = configs
        .iter()
        .find(|c| c.id == config_id)
        .ok_or_else(|| format!("Config '{}' not found", config_id))?;

    // Convert to vless_engine config
    let vless_config = crate::core::vless_engine::VlessConfig::new(
        config.server.clone(),
        config.port,
        config.uuid.clone(),
    )
    .with_name(&config.name)
    .with_sni(config.sni.clone().unwrap_or_else(|| config.server.clone()));

    // Get manager and allocate port
    let manager = crate::core::singbox_manager::get_manager();
    let port = match socks_port {
        Some(p) => p,
        None => manager.allocate_port(1080).await,
    };

    // Start the proxy
    let instance = manager
        .start(&vless_config, port)
        .await
        .map_err(|e| format!("Failed to start VLESS proxy: {}", e))?;

    info!(
        config_id = %config_id,
        socks_port = instance.socks_port,
        "VLESS proxy started"
    );

    Ok(instance)
}

/// Stop VLESS proxy for a specific config
#[tauri::command]
pub async fn stop_vless_proxy(config_id: String) -> Result<(), String> {
    info!(config_id = %config_id, "Stopping VLESS proxy");

    let manager = crate::core::singbox_manager::get_manager();

    manager
        .stop(&config_id)
        .await
        .map_err(|e| format!("Failed to stop VLESS proxy: {}", e))?;

    info!(config_id = %config_id, "VLESS proxy stopped");
    Ok(())
}

/// Stop all running VLESS proxies
#[tauri::command]
pub async fn stop_all_vless_proxies() -> Result<(), String> {
    info!("Stopping all VLESS proxies");

    let manager = crate::core::singbox_manager::get_manager();

    manager
        .stop_all()
        .await
        .map_err(|e| format!("Failed to stop VLESS proxies: {}", e))?;

    info!("All VLESS proxies stopped");
    Ok(())
}

/// Get status of a specific VLESS proxy
#[tauri::command]
pub async fn get_vless_status(
    config_id: String,
) -> Result<Option<crate::core::singbox_manager::SingboxInstance>, String> {
    let manager = crate::core::singbox_manager::get_manager();
    Ok(manager.get_status(&config_id).await)
}

/// Get status of all running VLESS proxies
#[tauri::command]
pub async fn get_all_vless_status() -> Result<Vec<crate::core::singbox_manager::SingboxInstance>, String> {
    let manager = crate::core::singbox_manager::get_manager();
    Ok(manager.list_instances().await)
}

/// Perform health check on a running VLESS proxy
#[tauri::command]
pub async fn health_check_vless(config_id: String) -> Result<bool, String> {
    info!(config_id = %config_id, "Performing VLESS health check");

    let manager = crate::core::singbox_manager::get_manager();

    manager
        .health_check(&config_id)
        .await
        .map_err(|e| format!("Health check failed: {}", e))
}

/// Test VLESS proxy connectivity
///
/// Makes a test request through the proxy to verify it's working.
#[tauri::command]
pub async fn test_vless_connectivity(
    config_id: String,
    test_url: Option<String>,
) -> Result<u32, String> {
    info!(config_id = %config_id, "Testing VLESS connectivity");

    let manager = crate::core::singbox_manager::get_manager();

    // Get the SOCKS port for this config
    let socks_port = manager
        .get_socks_port(&config_id)
        .await
        .ok_or_else(|| format!("Config '{}' is not running", config_id))?;

    let url = test_url.unwrap_or_else(|| "https://www.google.com".to_string());

    crate::core::vless_engine::test_proxy_connectivity(socks_port, &url)
        .await
        .map_err(|e| format!("Connectivity test failed: {}", e))
}

/// Check if sing-box binary is available
#[tauri::command]
pub async fn is_singbox_available() -> Result<bool, String> {
    Ok(crate::core::singbox_manager::is_singbox_available())
}

/// Get sing-box version
#[tauri::command]
pub async fn get_singbox_version() -> Result<String, String> {
    crate::core::singbox_manager::get_singbox_version()
        .await
        .map_err(|e| format!("Failed to get sing-box version: {}", e))
}


// ============================================================================
// Mode Detection Commands
// ============================================================================

/// Check if app is running in silent mode (--silent flag)
#[tauri::command]
pub async fn is_silent_mode() -> Result<bool, String> {
    Ok(std::env::args().any(|arg| arg == "--silent"))
}

/// Check if app is running in portable mode
#[tauri::command]
pub async fn is_portable_mode() -> Result<bool, String> {
    Ok(crate::core::paths::is_portable_mode())
}

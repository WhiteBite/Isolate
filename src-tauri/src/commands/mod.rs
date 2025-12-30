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

/// Log filter parameters
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LogFilter {
    pub level: Option<String>,
    pub module: Option<String>,
    pub search: Option<String>,
}

/// Get recent logs with optional filtering
#[tauri::command]
pub async fn get_logs(filter: Option<LogFilter>) -> Result<Vec<LogEntry>, String> {
    info!("Getting logs with filter: {:?}", filter);
    
    let logs = match filter {
        Some(f) => crate::core::log_capture::get_filtered_logs(
            f.level.as_deref(),
            f.module.as_deref(),
            f.search.as_deref(),
        ),
        None => crate::core::log_capture::get_all_logs(),
    };
    
    Ok(logs)
}

/// Clear all logs
#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    info!("Clearing logs");
    crate::core::log_capture::clear_logs();
    Ok(())
}

/// Export logs to file
#[tauri::command]
pub async fn export_logs() -> Result<String, String> {
    info!("Exporting logs");
    
    let logs_content = crate::core::log_capture::export_logs_to_string();
    
    // Get logs directory
    let logs_dir = crate::core::paths::get_logs_dir();
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("Failed to create logs directory: {}", e))?;
    
    // Create filename with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("isolate_logs_{}.txt", timestamp);
    let filepath = logs_dir.join(&filename);
    
    // Write logs
    std::fs::write(&filepath, logs_content)
        .map_err(|e| format!("Failed to write logs: {}", e))?;
    
    info!(path = %filepath.display(), "Logs exported");
    Ok(filepath.display().to_string())
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

// ============================================================================
// Proxy Management Commands
// ============================================================================

/// Get all saved proxies
#[tauri::command]
pub async fn get_proxies(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::core::models::ProxyConfig>, String> {
    info!("Loading all proxies");
    
    state
        .storage
        .get_all_proxies()
        .map_err(|e| format!("Failed to get proxies: {}", e))
}

/// Add a new proxy
#[tauri::command]
pub async fn add_proxy(
    state: State<'_, Arc<AppState>>,
    proxy: crate::core::models::ProxyConfig,
) -> Result<crate::core::models::ProxyConfig, String> {
    info!(id = %proxy.id, name = %proxy.name, protocol = ?proxy.protocol, "Adding new proxy");
    
    // Generate ID if empty
    let mut proxy = proxy;
    if proxy.id.is_empty() {
        proxy.id = format!(
            "{}_{}", 
            format!("{:?}", proxy.protocol).to_lowercase(),
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")
        );
    }
    
    state
        .storage
        .save_proxy(&proxy)
        .map_err(|e| format!("Failed to add proxy: {}", e))?;
    
    info!(id = %proxy.id, "Proxy added successfully");
    Ok(proxy)
}

/// Update existing proxy
#[tauri::command]
pub async fn update_proxy(
    state: State<'_, Arc<AppState>>,
    proxy: crate::core::models::ProxyConfig,
) -> Result<(), String> {
    info!(id = %proxy.id, name = %proxy.name, "Updating proxy");
    
    state
        .storage
        .update_proxy(&proxy)
        .map_err(|e| format!("Failed to update proxy: {}", e))
}

/// Delete proxy by ID
#[tauri::command]
pub async fn delete_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    info!(id = %id, "Deleting proxy");
    
    // Stop proxy if running
    let manager = crate::core::singbox_manager::get_manager();
    if manager.is_running(&id).await {
        let _ = manager.stop(&id).await;
    }
    
    state
        .storage
        .delete_proxy(&id)
        .map_err(|e| format!("Failed to delete proxy: {}", e))
}

/// Apply proxy (start sing-box with this proxy)
#[tauri::command]
pub async fn apply_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    info!(id = %id, "Applying proxy");
    
    // Get proxy config
    let proxy = state
        .storage
        .get_proxy(&id)
        .map_err(|e| format!("Failed to get proxy: {}", e))?
        .ok_or_else(|| format!("Proxy '{}' not found", id))?;
    
    // Check if protocol is supported for sing-box
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Vless |
        crate::core::models::ProxyProtocol::Vmess |
        crate::core::models::ProxyProtocol::Shadowsocks |
        crate::core::models::ProxyProtocol::Trojan |
        crate::core::models::ProxyProtocol::Hysteria |
        crate::core::models::ProxyProtocol::Hysteria2 |
        crate::core::models::ProxyProtocol::Tuic => {
            // Convert to VlessConfig for sing-box (generic proxy config)
            let vless_config = crate::core::vless_engine::VlessConfig::new(
                proxy.server.clone(),
                proxy.port,
                proxy.uuid.clone().unwrap_or_default(),
            )
            .with_name(&proxy.name)
            .with_id(&proxy.id)
            .with_sni(proxy.sni.clone().unwrap_or_else(|| proxy.server.clone()));
            
            let manager = crate::core::singbox_manager::get_manager();
            let port = manager.allocate_port(1080).await;
            
            manager
                .start(&vless_config, port)
                .await
                .map_err(|e| format!("Failed to start proxy: {}", e))?;
            
            // Mark as active in storage
            state
                .storage
                .set_proxy_active(&id, true)
                .map_err(|e| format!("Failed to set proxy active: {}", e))?;
            
            info!(id = %id, socks_port = port, "Proxy applied successfully");
            Ok(())
        }
        crate::core::models::ProxyProtocol::Socks5 |
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            // These protocols don't need sing-box, just mark as active
            state
                .storage
                .set_proxy_active(&id, true)
                .map_err(|e| format!("Failed to set proxy active: {}", e))?;
            
            info!(id = %id, "Direct proxy marked as active");
            Ok(())
        }
        _ => Err(format!("Protocol {:?} is not supported for apply", proxy.protocol)),
    }
}

/// Test proxy connectivity
#[tauri::command]
pub async fn test_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<u32, String> {
    info!(id = %id, "Testing proxy connectivity");
    
    // Get proxy config
    let proxy = state
        .storage
        .get_proxy(&id)
        .map_err(|e| format!("Failed to get proxy: {}", e))?
        .ok_or_else(|| format!("Proxy '{}' not found", id))?;
    
    let manager = crate::core::singbox_manager::get_manager();
    
    // Check if already running
    if let Some(socks_port) = manager.get_socks_port(&id).await {
        // Test existing connection
        return crate::core::vless_engine::test_proxy_connectivity(socks_port, "https://www.google.com")
            .await
            .map_err(|e| format!("Connectivity test failed: {}", e));
    }
    
    // Start temporary proxy for testing
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Vless |
        crate::core::models::ProxyProtocol::Vmess |
        crate::core::models::ProxyProtocol::Shadowsocks |
        crate::core::models::ProxyProtocol::Trojan |
        crate::core::models::ProxyProtocol::Hysteria |
        crate::core::models::ProxyProtocol::Hysteria2 |
        crate::core::models::ProxyProtocol::Tuic => {
            let vless_config = crate::core::vless_engine::VlessConfig::new(
                proxy.server.clone(),
                proxy.port,
                proxy.uuid.clone().unwrap_or_default(),
            )
            .with_name(&proxy.name)
            .with_id(&format!("test_{}", proxy.id))
            .with_sni(proxy.sni.clone().unwrap_or_else(|| proxy.server.clone()));
            
            let port = manager.allocate_port(10800).await;
            
            // Start proxy
            manager
                .start(&vless_config, port)
                .await
                .map_err(|e| format!("Failed to start proxy for testing: {}", e))?;
            
            // Wait for proxy to initialize
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            
            // Test connectivity
            let result = crate::core::vless_engine::test_proxy_connectivity(port, "https://www.google.com")
                .await;
            
            // Stop test proxy
            let _ = manager.stop(&format!("test_{}", proxy.id)).await;
            
            result.map_err(|e| format!("Connectivity test failed: {}", e))
        }
        crate::core::models::ProxyProtocol::Socks5 => {
            // Test SOCKS5 directly
            let start = std::time::Instant::now();
            let addr = format!("{}:{}", proxy.server, proxy.port);
            
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tokio::net::TcpStream::connect(&addr)
            ).await {
                Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u32),
                Ok(Err(e)) => Err(format!("Connection failed: {}", e)),
                Err(_) => Err("Connection timeout".to_string()),
            }
        }
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            // Test HTTP proxy
            let start = std::time::Instant::now();
            let proxy_url = format!(
                "{}://{}:{}",
                if proxy.protocol == crate::core::models::ProxyProtocol::Https { "https" } else { "http" },
                proxy.server,
                proxy.port
            );
            
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy URL: {}", e))?)
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| format!("Failed to create client: {}", e))?;
            
            client
                .get("https://www.google.com")
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        _ => Err(format!("Protocol {:?} is not supported for testing", proxy.protocol)),
    }
}

/// Import proxy from URL (vless://, vmess://, ss://, etc.)
#[tauri::command]
pub async fn import_proxy_url(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<crate::core::models::ProxyConfig, String> {
    info!("Importing proxy from URL");
    
    let proxy = crate::core::proxy_parser::parse_proxy_url(&url)
        .map_err(|e| format!("Failed to parse proxy URL: {}", e))?;
    
    // Save to storage
    state
        .storage
        .save_proxy(&proxy)
        .map_err(|e| format!("Failed to save proxy: {}", e))?;
    
    info!(id = %proxy.id, name = %proxy.name, protocol = ?proxy.protocol, "Proxy imported from URL");
    Ok(proxy)
}

/// Import subscription (multiple proxies from URL)
#[tauri::command]
pub async fn import_subscription(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<Vec<crate::core::models::ProxyConfig>, String> {
    info!(url = %url, "Importing subscription");
    
    // Fetch subscription content
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch subscription: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Subscription request failed with status: {}", response.status()));
    }
    
    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read subscription content: {}", e))?;
    
    // Parse subscription
    let proxies = crate::core::proxy_parser::parse_subscription(&content)
        .map_err(|e| format!("Failed to parse subscription: {}", e))?;
    
    // Save all proxies
    let mut saved_proxies = Vec::new();
    for proxy in proxies {
        match state.storage.save_proxy(&proxy) {
            Ok(_) => {
                info!(id = %proxy.id, name = %proxy.name, "Proxy from subscription saved");
                saved_proxies.push(proxy);
            }
            Err(e) => {
                warn!(id = %proxy.id, error = %e, "Failed to save proxy from subscription");
            }
        }
    }
    
    info!(count = saved_proxies.len(), "Subscription import completed");
    Ok(saved_proxies)
}

// ============================================================================
// Domain Routing Commands
// ============================================================================

use crate::core::models::{DomainRoute, AppRoute};
use crate::core::app_routing::InstalledApp;

/// Get all domain routes
#[tauri::command]
pub async fn get_domain_routes(state: State<'_, Arc<AppState>>) -> Result<Vec<DomainRoute>, String> {
    info!("Getting domain routes");
    state.domain_router
        .get_routes()
        .await
        .map_err(|e| format!("Failed to get domain routes: {}", e))
}

/// Add a domain route
#[tauri::command]
pub async fn add_domain_route(
    state: State<'_, Arc<AppState>>,
    domain: String,
    proxy_id: String,
) -> Result<(), String> {
    info!(domain = %domain, proxy_id = %proxy_id, "Adding domain route");
    state.domain_router
        .add_route(&domain, &proxy_id)
        .await
        .map_err(|e| format!("Failed to add domain route: {}", e))
}

/// Remove a domain route
#[tauri::command]
pub async fn remove_domain_route(
    state: State<'_, Arc<AppState>>,
    domain: String,
) -> Result<(), String> {
    info!(domain = %domain, "Removing domain route");
    state.domain_router
        .remove_route(&domain)
        .await
        .map_err(|e| format!("Failed to remove domain route: {}", e))
}

// ============================================================================
// App Routing Commands
// ============================================================================

/// Get all app routes
#[tauri::command]
pub async fn get_app_routes(state: State<'_, Arc<AppState>>) -> Result<Vec<AppRoute>, String> {
    info!("Getting app routes");
    state.app_router
        .get_routes()
        .await
        .map_err(|e| format!("Failed to get app routes: {}", e))
}

/// Add an app route
#[tauri::command]
pub async fn add_app_route(
    state: State<'_, Arc<AppState>>,
    app_name: String,
    app_path: String,
    proxy_id: String,
) -> Result<(), String> {
    info!(app_name = %app_name, proxy_id = %proxy_id, "Adding app route");
    state.app_router
        .add_route(&app_name, &app_path, &proxy_id)
        .await
        .map_err(|e| format!("Failed to add app route: {}", e))
}

/// Remove an app route
#[tauri::command]
pub async fn remove_app_route(
    state: State<'_, Arc<AppState>>,
    app_path: String,
) -> Result<(), String> {
    info!(app_path = %app_path, "Removing app route");
    state.app_router
        .remove_route(&app_path)
        .await
        .map_err(|e| format!("Failed to remove app route: {}", e))
}

/// Get list of installed applications (Windows)
#[tauri::command]
pub async fn get_installed_apps(state: State<'_, Arc<AppState>>) -> Result<Vec<InstalledApp>, String> {
    info!("Getting installed apps");
    state.app_router
        .get_installed_apps()
        .await
        .map_err(|e| format!("Failed to get installed apps: {}", e))
}


// ============================================================================
// Testing Commands
// ============================================================================

use std::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// Tray Commands
// ============================================================================

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

/// Test progress event payload
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestProgress {
    pub current_item: String,
    pub current_type: String, // "proxy" or "strategy"
    pub tested_count: usize,
    pub total_count: usize,
    pub percent: u8,
}

/// Test result for a single item
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestItemResult {
    pub id: String,
    pub name: String,
    pub item_type: String, // "proxy" or "strategy"
    pub success_rate: f32,
    pub latency_ms: u32,
    pub score: f32,
    pub services_tested: Vec<String>,
    pub services_passed: Vec<String>,
}

// Global cancellation flag
static TESTS_CANCELLED: AtomicBool = AtomicBool::new(false);

/// Run tests on proxies and/or strategies
#[tauri::command]
pub async fn run_tests(
    window: Window,
    state: State<'_, Arc<AppState>>,
    proxy_ids: Vec<String>,
    strategy_ids: Vec<String>,
    service_ids: Vec<String>,
    mode: String,
) -> Result<(), String> {
    info!(
        proxy_count = proxy_ids.len(),
        strategy_count = strategy_ids.len(),
        service_count = service_ids.len(),
        mode = %mode,
        "Starting tests"
    );
    
    TESTS_CANCELLED.store(false, Ordering::SeqCst);
    
    let total_count = proxy_ids.len() + strategy_ids.len();
    let mut tested_count = 0;
    let mut results: Vec<TestItemResult> = Vec::new();
    
    // Load services for testing
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    let test_services: Vec<_> = if service_ids.is_empty() {
        services_map.values().cloned().collect()
    } else {
        services_map
            .values()
            .filter(|s| service_ids.contains(&s.id))
            .cloned()
            .collect()
    };
    
    let timeout_secs = if mode == "turbo" { 3 } else { 5 };
    
    // Test proxies
    for proxy_id in &proxy_ids {
        if TESTS_CANCELLED.load(Ordering::SeqCst) {
            info!("Tests cancelled by user");
            break;
        }
        
        // Get proxy
        let proxy = match state.storage.get_proxy(proxy_id) {
            Ok(Some(p)) => p,
            _ => continue,
        };
        
        // Emit progress
        tested_count += 1;
        let progress = TestProgress {
            current_item: proxy.name.clone(),
            current_type: "proxy".to_string(),
            tested_count,
            total_count,
            percent: ((tested_count * 100) / total_count.max(1)) as u8,
        };
        let _ = window.emit("test:progress", &progress);
        
        // Test proxy against services
        let mut services_passed = Vec::new();
        let mut total_latency = 0u32;
        let mut test_count = 0u32;
        
        for service in &test_services {
            if TESTS_CANCELLED.load(Ordering::SeqCst) {
                break;
            }
            
            // Test connectivity through proxy
            match test_proxy_for_service(&state, &proxy, service, timeout_secs).await {
                Ok(latency) => {
                    services_passed.push(service.id.clone());
                    total_latency += latency;
                    test_count += 1;
                }
                Err(_) => {}
            }
        }
        
        let success_rate = if test_services.is_empty() {
            0.0
        } else {
            (services_passed.len() as f32 / test_services.len() as f32) * 100.0
        };
        
        let avg_latency = if test_count > 0 {
            total_latency / test_count
        } else {
            9999
        };
        
        // Score: higher is better (success_rate * 10 - latency_penalty)
        let score = success_rate * 10.0 - (avg_latency as f32 / 100.0);
        
        let result = TestItemResult {
            id: proxy.id.clone(),
            name: proxy.name.clone(),
            item_type: "proxy".to_string(),
            success_rate,
            latency_ms: avg_latency,
            score,
            services_tested: test_services.iter().map(|s| s.id.clone()).collect(),
            services_passed,
        };
        
        let _ = window.emit("test:result", &result);
        results.push(result);
    }
    
    // Test strategies (ВАЖНО: последовательно, не параллельно!)
    for strategy_id in &strategy_ids {
        if TESTS_CANCELLED.load(Ordering::SeqCst) {
            info!("Tests cancelled by user");
            break;
        }
        
        // Load strategy
        let strategy = match state.config_manager.load_strategy_by_id(strategy_id).await {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        // Emit progress
        tested_count += 1;
        let progress = TestProgress {
            current_item: strategy.name.clone(),
            current_type: "strategy".to_string(),
            tested_count,
            total_count,
            percent: ((tested_count * 100) / total_count.max(1)) as u8,
        };
        let _ = window.emit("test:progress", &progress);
        
        // Start strategy
        if let Err(e) = state.strategy_engine.start_global(&strategy).await {
            warn!(strategy_id = %strategy_id, error = %e, "Failed to start strategy for testing");
            continue;
        }
        
        // Wait for strategy to initialize
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Test services
        let mut services_passed = Vec::new();
        let mut total_latency = 0u32;
        let mut test_count = 0u32;
        
        for service in &test_services {
            if TESTS_CANCELLED.load(Ordering::SeqCst) {
                break;
            }
            
            match test_service_direct(service, timeout_secs).await {
                Ok(latency) => {
                    services_passed.push(service.id.clone());
                    total_latency += latency;
                    test_count += 1;
                }
                Err(_) => {}
            }
        }
        
        // Stop strategy
        let _ = state.strategy_engine.stop_global().await;
        
        // Wait before next strategy (prevent BSOD with WinDivert)
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        let success_rate = if test_services.is_empty() {
            0.0
        } else {
            (services_passed.len() as f32 / test_services.len() as f32) * 100.0
        };
        
        let avg_latency = if test_count > 0 {
            total_latency / test_count
        } else {
            9999
        };
        
        let score = success_rate * 10.0 - (avg_latency as f32 / 100.0);
        
        let result = TestItemResult {
            id: strategy.id.clone(),
            name: strategy.name.clone(),
            item_type: "strategy".to_string(),
            success_rate,
            latency_ms: avg_latency,
            score,
            services_tested: test_services.iter().map(|s| s.id.clone()).collect(),
            services_passed,
        };
        
        let _ = window.emit("test:result", &result);
        results.push(result);
    }
    
    // Emit completion
    let _ = window.emit("test:complete", &results);
    
    info!(results_count = results.len(), "Tests completed");
    Ok(())
}

/// Cancel running tests
#[tauri::command]
pub async fn cancel_tests() -> Result<(), String> {
    info!("Cancelling tests");
    TESTS_CANCELLED.store(true, Ordering::SeqCst);
    Ok(())
}

// Helper: test proxy for a specific service
async fn test_proxy_for_service(
    state: &State<'_, Arc<AppState>>,
    proxy: &crate::core::models::ProxyConfig,
    service: &crate::core::models::Service,
    timeout_secs: u64,
) -> Result<u32, String> {
    let start = std::time::Instant::now();
    
    // Get test URL from service
    let test_url = service.get_test_url()
        .ok_or_else(|| "Service has no test URL".to_string())?;
    
    // For SOCKS5/HTTP proxies, test directly
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Socks5 => {
            let proxy_url = format!("socks5://{}:{}", proxy.server, proxy.port);
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| e.to_string())?;
            
            client
                .get(&test_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            let scheme = if proxy.protocol == crate::core::models::ProxyProtocol::Https { "https" } else { "http" };
            let proxy_url = format!("{}://{}:{}", scheme, proxy.server, proxy.port);
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| e.to_string())?;
            
            client
                .get(&test_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        _ => {
            // For other protocols, need sing-box running
            let manager = crate::core::singbox_manager::get_manager();
            
            if let Some(socks_port) = manager.get_socks_port(&proxy.id).await {
                let proxy_url = format!("socks5://127.0.0.1:{}", socks_port);
                let client = reqwest::Client::builder()
                    .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                    .timeout(std::time::Duration::from_secs(timeout_secs))
                    .danger_accept_invalid_certs(true)
                    .build()
                    .map_err(|e| e.to_string())?;
                
                client
                    .get(&test_url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                
                Ok(start.elapsed().as_millis() as u32)
            } else {
                Err("Proxy not running".to_string())
            }
        }
    }
}

// Helper: test service directly (for strategy testing)
async fn test_service_direct(
    service: &crate::core::models::Service,
    timeout_secs: u64,
) -> Result<u32, String> {
    let start = std::time::Instant::now();
    
    // Get test URL from service
    let test_url = service.get_test_url()
        .ok_or_else(|| "Service has no test URL".to_string())?;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;
    
    client
        .get(&test_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(start.elapsed().as_millis() as u32)
}

// ============================================================================
// TUN Mode Commands
// ============================================================================

use crate::core::tun_manager::{TunConfig, TunInstance, TunStatus};

/// Start TUN mode
///
/// Routes all system traffic through the specified SOCKS proxy.
/// Requires administrator privileges.
#[tauri::command]
pub async fn start_tun(socks_port: u16) -> Result<TunInstance, String> {
    info!(socks_port, "Starting TUN mode");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    
    manager
        .start(socks_port)
        .await
        .map_err(|e| format!("Failed to start TUN: {}", e))
}

/// Stop TUN mode
#[tauri::command]
pub async fn stop_tun() -> Result<(), String> {
    info!("Stopping TUN mode");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    
    manager
        .stop()
        .await
        .map_err(|e| format!("Failed to stop TUN: {}", e))
}

/// Check if TUN is running
#[tauri::command]
pub async fn is_tun_running() -> Result<bool, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.is_running())
}

/// Get TUN status
#[tauri::command]
pub async fn get_tun_status() -> Result<TunInstance, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.get_instance())
}

/// Get TUN configuration
#[tauri::command]
pub async fn get_tun_config() -> Result<TunConfig, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.get_config())
}

/// Update TUN configuration
///
/// Note: Changes take effect on next TUN start
#[tauri::command]
pub async fn set_tun_config(config: TunConfig) -> Result<(), String> {
    info!(interface = %config.interface_name, mtu = config.mtu, "Updating TUN config");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    manager.set_config(config);
    
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
#[tauri::command]
pub async fn restart_tun(socks_port: Option<u16>) -> Result<TunInstance, String> {
    info!(socks_port = ?socks_port, "Restarting TUN mode");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    
    manager
        .restart(socks_port)
        .await
        .map_err(|e| format!("Failed to restart TUN: {}", e))
}

// ============================================================================
// System Proxy Commands
// ============================================================================

/// Set system proxy
///
/// Configures Windows system proxy settings.
#[tauri::command]
pub async fn set_system_proxy(host: String, port: u16, scheme: String) -> Result<(), String> {
    info!(host = %host, port, scheme = %scheme, "Setting system proxy");
    
    crate::core::system_proxy::set_system_proxy(&host, port, &scheme)
        .await
        .map_err(|e| format!("Failed to set system proxy: {}", e))
}

/// Clear system proxy settings
#[tauri::command]
pub async fn clear_system_proxy() -> Result<(), String> {
    info!("Clearing system proxy");
    
    crate::core::system_proxy::clear_system_proxy()
        .await
        .map_err(|e| format!("Failed to clear system proxy: {}", e))
}

/// Check if system proxy is currently set
#[tauri::command]
pub async fn is_system_proxy_set() -> Result<bool, String> {
    crate::core::system_proxy::is_system_proxy_set()
        .await
        .map_err(|e| format!("Failed to check system proxy: {}", e))
}

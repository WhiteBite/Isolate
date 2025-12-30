//! Tauri commands — IPC interface between frontend and backend

use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, Window};
use tauri_plugin_updater::UpdaterExt;
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::core::binaries::{self, BinaryCheckResult, DownloadProgress};
use crate::core::models::{AppStatus, DiagnosticResult, LogEntry, Service, ServiceWithState, Settings, Strategy, UpdateInfo};
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
    
    // TODO: Implement full diagnostics
    Ok(DiagnosticResult::default())
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

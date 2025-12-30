//! Tauri commands — IPC interface between frontend and backend

use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::core::models::{AppStatus, DiagnosticResult, Service, ServiceWithState, Settings, Strategy};
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

/// Get app version from Cargo.toml
#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

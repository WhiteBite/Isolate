//! Monitor commands
//!
//! Commands for strategy health monitoring that periodically checks
//! if the active strategy is working correctly.

use tauri::AppHandle;
use tracing::info;

use crate::commands::state_guard::get_state_or_error;
use crate::core::errors::{IsolateError, TypedResultExt};

// ============================================================================
// Monitor Commands
// ============================================================================

/// Start strategy health monitoring
///
/// Begins periodic health checks of the active strategy.
/// Emits events:
/// - `strategy:health` / `monitor:health_check` — current health status
/// - `strategy:degraded` — strategy has degraded
/// - `strategy:recovered` — strategy has recovered
/// - `strategy:restart_requested` — auto-restart requested (if enabled)
#[tauri::command]
pub async fn start_monitor(
    app: AppHandle,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Starting strategy monitor");

    state
        .monitor
        .start(app.clone())
        .await
        .process_context("Failed to start monitor")
}

/// Stop strategy health monitoring
///
/// Gracefully stops the monitor. Any ongoing health check will complete.
#[tauri::command]
pub async fn stop_monitor(app: AppHandle) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Stopping strategy monitor");
    state.monitor.stop();
    Ok(())
}

/// Check if monitor is running
#[tauri::command]
pub async fn is_monitor_running(app: AppHandle) -> Result<bool, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.monitor.is_running())
}

/// Check if strategy is degraded
///
/// Returns true if the current strategy has failed health checks.
#[tauri::command]
pub async fn is_strategy_degraded(app: AppHandle) -> Result<bool, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.monitor.is_degraded())
}

/// Perform manual health check
///
/// Runs an immediate health check regardless of the monitoring schedule.
#[tauri::command]
pub async fn check_strategy_health(app: AppHandle) -> Result<bool, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    state
        .monitor
        .check_strategy_health()
        .await
        .network_context("Health check failed")
}

/// Set monitor test URLs
///
/// Configures which URLs the monitor uses for health checks.
#[tauri::command]
pub async fn set_monitor_urls(
    app: AppHandle,
    urls: Vec<String>,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(count = urls.len(), "Setting monitor test URLs");
    state.monitor.set_test_urls(urls).await;
    Ok(())
}

/// Enable/disable auto-restart on degradation
///
/// When enabled, the monitor will automatically restart the strategy
/// if it detects degradation.
#[tauri::command]
pub async fn set_monitor_auto_restart(
    app: AppHandle,
    enabled: bool,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(enabled, "Setting monitor auto-restart");
    state.monitor.set_auto_restart(enabled);
    Ok(())
}

/// Get monitor configuration
#[tauri::command]
pub async fn get_monitor_config(
    app: AppHandle,
) -> Result<MonitorConfig, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    Ok(MonitorConfig {
        is_running: state.monitor.is_running(),
        is_degraded: state.monitor.is_degraded(),
        consecutive_failures: state.monitor.get_consecutive_failures(),
    })
}

/// Monitor configuration for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitorConfig {
    pub is_running: bool,
    pub is_degraded: bool,
    pub consecutive_failures: u32,
}

//! Strategy-related Tauri commands

use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tracing::info;

use crate::commands::validation::validate_strategy_id;
use crate::core::errors::IsolateError;
use crate::core::models::Strategy;
use crate::core::storage::{StrategyHistoryEntry, StrategyStats};
use crate::state::AppState;

/// Get list of available strategies (high-level format for UI)
#[tauri::command]
pub async fn get_strategies(state: State<'_, Arc<AppState>>) -> Result<Vec<Strategy>, IsolateError> {
    info!("Loading strategies");
    
    let strategies_map = state
        .config_manager
        .load_strategies()
        .await?;
    
    Ok(strategies_map.into_values().collect())
}

/// Get all strategies as DTOs for frontend (unified format)
///
/// Returns both high-level and zapret strategies in a consistent DTO format
/// suitable for UI display. This is the recommended API for frontend.
#[tauri::command]
pub async fn get_strategies_unified() -> Result<Vec<crate::core::unified_strategy_loader::StrategyDto>, IsolateError> {
    info!("Loading all strategies as DTOs (unified)");
    
    let strategies_dir = crate::core::paths::get_configs_dir().join("strategies");
    let loader = crate::core::unified_strategy_loader::UnifiedStrategyLoader::new(&strategies_dir);
    
    loader
        .load_all_as_dto()
        .await
        .map_err(|e| IsolateError::Config(e.to_string()))
}

/// Apply specific strategy
#[tauri::command]
pub async fn apply_strategy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<(), IsolateError> {
    // Rate limit: max 1 call per 5 seconds
    crate::commands::rate_limiter::check_rate_limit("apply_strategy", 5)?;
    
    // Validate strategy_id to prevent path traversal and injection attacks
    validate_strategy_id(&strategy_id)?;
    
    info!("Applying strategy: {}", strategy_id);
    
    // Load strategy by ID
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await?;
    
    let strategy_name = strategy.name.clone();
    
    // Try to start the strategy
    let result = state
        .strategy_engine
        .start_global(&strategy)
        .await;
    
    match result {
        Ok(()) => {
            // Record success for failover tracking
            state.auto_failover.record_success(&strategy_id).await;
            state.auto_failover.set_current_strategy(Some(strategy_id.clone())).await;
            
            // Emit event for frontend to update status
            let _ = app.emit("strategy:applied", serde_json::json!({
                "strategy_id": strategy_id,
                "strategy_name": strategy_name,
            }));
            
            Ok(())
        }
        Err(e) => {
            // Record failure for failover tracking
            let should_failover = state.auto_failover.record_failure(&strategy_id, &e.to_string()).await;
            
            // Check if we should auto-failover
            if should_failover && state.auto_failover.is_enabled().await {
                if let Some(backup_id) = state.auto_failover.get_next_backup_strategy(&strategy_id).await {
                    info!(
                        failed_strategy = %strategy_id,
                        backup_strategy = %backup_id,
                        "Auto failover: switching to backup strategy"
                    );
                    
                    // Emit failover event
                    let _ = app.emit("strategy:failover", serde_json::json!({
                        "failed_strategy": strategy_id,
                        "backup_strategy": backup_id,
                        "reason": e.to_string(),
                    }));
                    
                    // Try to apply backup strategy (recursive call)
                    // Note: This is safe because we track tried strategies
                    if let Ok(backup_strategy) = state.config_manager.load_strategy_by_id(&backup_id).await {
                        if let Ok(()) = state.strategy_engine.start_global(&backup_strategy).await {
                            state.auto_failover.record_success(&backup_id).await;
                            state.auto_failover.set_current_strategy(Some(backup_id.clone())).await;
                            
                            let _ = app.emit("strategy:applied", serde_json::json!({
                                "strategy_id": backup_id,
                                "strategy_name": backup_strategy.name,
                                "is_failover": true,
                            }));
                            
                            return Ok(());
                        }
                    }
                }
            }
            
            Err(e)
        }
    }
}

/// Stop current strategy
#[tauri::command]
pub async fn stop_strategy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Stopping current strategy");
    
    state
        .strategy_engine
        .stop_global()
        .await?;
    
    // Emit event for frontend to update status
    let _ = app.emit("strategy:stopped", ());
    
    Ok(())
}

/// Get current engine mode (mock/real/dpi_test)
#[tauri::command]
pub async fn get_engine_mode(
    state: State<'_, Arc<AppState>>,
) -> Result<String, IsolateError> {
    let mode = state.strategy_engine.get_mode().await;
    Ok(format!("{:?}", mode))
}

/// Set engine mode (mock/real/dpi_test)
///
/// - mock: Simulates strategy execution without running real processes
/// - real: Runs actual winws/sing-box processes
/// - dpi_test: Special mode for DPI testing
#[tauri::command]
pub async fn set_engine_mode(
    state: State<'_, Arc<AppState>>,
    mode: String,
) -> Result<(), IsolateError> {
    use crate::core::strategy_engine::EngineMode;
    
    let engine_mode = match mode.to_lowercase().as_str() {
        "mock" => EngineMode::Mock,
        "real" => EngineMode::Real,
        "dpi_test" | "dpitest" => EngineMode::DpiTest,
        _ => return Err(IsolateError::Validation(
            format!("Unknown mode: {}. Valid modes: mock, real, dpi_test", mode)
        )),
    };
    
    info!(mode = %mode, "Setting engine mode");
    state.strategy_engine.set_mode(engine_mode).await;
    
    Ok(())
}

// ========================================================================
// Strategy Statistics Commands
// ========================================================================

/// Record a strategy test result
///
/// Records success/failure and latency for a strategy execution.
/// Used for tracking strategy performance over time.
#[tauri::command]
pub async fn record_strategy_result(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
    service_id: String,
    success: bool,
    latency_ms: Option<f64>,
) -> Result<(), IsolateError> {
    // Validate strategy_id
    validate_strategy_id(&strategy_id)?;
    
    // Validate service_id (similar format to strategy_id)
    crate::commands::validation::validate_not_empty(&service_id, "Service ID")?;
    if service_id.len() > 64 {
        return Err(IsolateError::Validation(
            "Service ID exceeds maximum length of 64 characters".to_string()
        ));
    }
    if !service_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(IsolateError::Validation(
            "Service ID can only contain alphanumeric characters, underscores, and hyphens".to_string()
        ));
    }
    
    // Validate latency if provided
    if let Some(latency) = latency_ms {
        if latency < 0.0 || latency > 300_000.0 {
            return Err(IsolateError::Validation(
                "Latency must be between 0 and 300000 ms".to_string()
            ));
        }
    }
    
    info!(
        strategy_id = %strategy_id,
        service_id = %service_id,
        success,
        latency_ms = ?latency_ms,
        "Recording strategy result"
    );
    
    state
        .storage
        .record_strategy_result(&strategy_id, &service_id, success, latency_ms)
        .await
}

/// Get history of strategy executions
///
/// Returns the most recent test results for a specific strategy.
#[tauri::command]
pub async fn get_strategy_history(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
    limit: Option<u32>,
) -> Result<Vec<StrategyHistoryEntry>, IsolateError> {
    // Validate strategy_id
    validate_strategy_id(&strategy_id)?;
    
    // Validate and cap limit to prevent excessive queries
    let limit = limit.unwrap_or(50).min(1000);
    
    info!(strategy_id = %strategy_id, limit, "Getting strategy history");
    
    state
        .storage
        .get_strategy_history(&strategy_id, limit)
        .await
}

/// Get aggregated statistics for a specific strategy
///
/// Returns success rate, average latency, and test counts.
#[tauri::command]
pub async fn get_strategy_statistics(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<StrategyStats, IsolateError> {
    // Validate strategy_id
    validate_strategy_id(&strategy_id)?;
    
    info!(strategy_id = %strategy_id, "Getting strategy statistics");
    
    state
        .storage
        .get_strategy_stats(&strategy_id)
        .await
}

/// Get statistics for all strategies
///
/// Returns aggregated stats for every strategy that has test history.
#[tauri::command]
pub async fn get_all_strategy_statistics(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<StrategyStats>, IsolateError> {
    info!("Getting all strategy statistics");
    
    state
        .storage
        .get_all_strategy_stats()
        .await
}

/// Clear history for a specific strategy
#[tauri::command]
pub async fn clear_strategy_history(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<(), IsolateError> {
    // Validate strategy_id
    validate_strategy_id(&strategy_id)?;
    
    info!(strategy_id = %strategy_id, "Clearing strategy history");
    
    state
        .storage
        .clear_strategy_history(&strategy_id)
        .await
}

/// Clear all strategy history
#[tauri::command]
pub async fn clear_all_strategy_history(
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Clearing all strategy history");
    
    state
        .storage
        .clear_all_strategy_history()
        .await
}

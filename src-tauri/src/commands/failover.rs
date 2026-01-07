//! Auto Failover commands — управление автоматическим переключением стратегий

use tauri::AppHandle;
use tracing::info;

use crate::commands::state_guard::get_state_or_error;
use crate::core::auto_failover::{FailoverConfig, FailoverStatus};
use crate::core::errors::IsolateError;

/// Get current failover status
/// 
/// Returns information about failover state including:
/// - Whether failover is enabled
/// - Current failure count
/// - Current and next backup strategy
/// - Cooldown remaining
#[tauri::command]
pub async fn get_failover_status(app: AppHandle) -> Result<FailoverStatus, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Getting failover status");
    
    Ok(state.auto_failover.get_status().await)
}

/// Enable or disable auto failover
/// 
/// When enabled, the system will automatically switch to a backup strategy
/// after a configured number of consecutive failures.
#[tauri::command]
pub async fn set_failover_enabled(
    app: AppHandle,
    enabled: bool,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(enabled, "Setting failover enabled");
    
    state.auto_failover.set_enabled(enabled).await;
    
    // Also update settings
    let mut settings = state.storage.get_settings().await
        .map_err(|e| IsolateError::Storage(e.to_string()))?;
    settings.auto_failover_enabled = enabled;
    state.storage.save_settings(&settings).await
        .map_err(|e| IsolateError::Storage(e.to_string()))?;
    
    Ok(())
}

/// Get failover configuration
#[tauri::command]
pub async fn get_failover_config(app: AppHandle) -> Result<FailoverConfig, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Getting failover config");
    
    Ok(state.auto_failover.get_config().await)
}

/// Update failover configuration
/// 
/// # Arguments
/// * `max_failures` - Number of failures before switching (default: 3)
/// * `cooldown_secs` - Seconds to wait before retry (default: 60)
#[tauri::command]
pub async fn set_failover_config(
    app: AppHandle,
    max_failures: u32,
    cooldown_secs: u32,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(max_failures, cooldown_secs, "Setting failover config");
    
    // Update failover config
    let config = FailoverConfig {
        max_failures,
        cooldown_secs,
        backup_strategies: state.auto_failover.get_config().await.backup_strategies,
    };
    state.auto_failover.update_config(config).await;
    
    // Also update settings
    let mut settings = state.storage.get_settings().await
        .map_err(|e| IsolateError::Storage(e.to_string()))?;
    settings.failover_max_failures = max_failures;
    settings.failover_cooldown_secs = cooldown_secs;
    state.storage.save_settings(&settings).await
        .map_err(|e| IsolateError::Storage(e.to_string()))?;
    
    Ok(())
}

/// Trigger manual failover to backup strategy
/// 
/// Forces immediate switch to the next available backup strategy,
/// regardless of failure count.
/// 
/// Returns the ID of the backup strategy that will be applied,
/// or None if no backup is available.
#[tauri::command]
pub async fn trigger_manual_failover(app: AppHandle) -> Result<Option<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Triggering manual failover");
    
    let backup_strategy = state.auto_failover.trigger_manual_failover().await;
    
    if let Some(ref strategy_id) = backup_strategy {
        info!(strategy_id, "Manual failover: switching to backup strategy");
    } else {
        info!("Manual failover: no backup strategy available");
    }
    
    Ok(backup_strategy)
}

/// Get list of learned strategies (successfully used in the past)
#[tauri::command]
pub async fn get_learned_strategies(app: AppHandle) -> Result<Vec<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    Ok(state.auto_failover.get_learned_strategies().await)
}

/// Reset failover state for a specific strategy
/// 
/// Clears failure count and tried strategies list.
#[tauri::command]
pub async fn reset_failover_state(
    app: AppHandle,
    strategy_id: String,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(strategy_id, "Resetting failover state");
    
    state.auto_failover.reset_strategy_state(&strategy_id).await;
    
    Ok(())
}

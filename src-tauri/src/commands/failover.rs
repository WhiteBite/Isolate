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
/// * `primary_strategy_id` - ID of the primary strategy (optional)
/// * `backup_strategy_ids` - List of backup strategies in priority order
/// * `max_failures` - Number of failures before switching (default: 3)
/// * `cooldown_secs` - Seconds to wait before trying to restore primary (default: 300)
/// * `enabled` - Whether failover is enabled
#[tauri::command]
pub async fn set_failover_config(
    app: AppHandle,
    primary_strategy_id: Option<String>,
    backup_strategy_ids: Option<Vec<String>>,
    max_failures: Option<u32>,
    cooldown_secs: Option<u64>,
    enabled: Option<bool>,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    // Get current config
    let current_config = state.auto_failover.get_config().await;
    
    // Build new config with provided values or defaults
    let config = FailoverConfig {
        primary_strategy_id: primary_strategy_id.or(current_config.primary_strategy_id),
        backup_strategy_ids: backup_strategy_ids.unwrap_or(current_config.backup_strategy_ids),
        max_failures: max_failures.unwrap_or(current_config.max_failures),
        cooldown_secs: cooldown_secs.unwrap_or(current_config.cooldown_secs),
        enabled: enabled.unwrap_or(current_config.enabled),
    };
    
    info!(
        max_failures = config.max_failures,
        cooldown_secs = config.cooldown_secs,
        enabled = config.enabled,
        "Setting failover config"
    );
    
    // Update failover config
    state.auto_failover.update_config(config.clone()).await;
    
    // Update enabled state separately
    state.auto_failover.set_enabled(config.enabled).await;
    
    // Also update settings
    let mut settings = state.storage.get_settings().await
        .map_err(|e| IsolateError::Storage(e.to_string()))?;
    settings.failover_max_failures = config.max_failures;
    settings.failover_cooldown_secs = config.cooldown_secs as u32;
    settings.auto_failover_enabled = config.enabled;
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

/// Force restore primary strategy (ignores cooldown)
/// 
/// Immediately switches back to the primary strategy,
/// regardless of cooldown timer.
/// 
/// Returns the ID of the primary strategy,
/// or None if no primary is configured.
#[tauri::command]
pub async fn force_restore_primary(app: AppHandle) -> Result<Option<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Force restoring primary strategy");
    
    let primary_strategy = state.auto_failover.force_restore_primary().await;
    
    if let Some(ref strategy_id) = primary_strategy {
        info!(strategy_id, "Force restore: switching to primary strategy");
    } else {
        info!("Force restore: no primary strategy configured");
    }
    
    Ok(primary_strategy)
}

/// Try to restore primary strategy (respects cooldown)
/// 
/// Attempts to switch back to the primary strategy if:
/// - Currently on a backup strategy
/// - Cooldown period has elapsed
/// 
/// Returns the ID of the primary strategy if restoration is possible,
/// or None if conditions are not met.
#[tauri::command]
pub async fn try_restore_primary(app: AppHandle) -> Result<Option<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Trying to restore primary strategy");
    
    let primary_strategy = state.auto_failover.try_restore_primary().await;
    
    if let Some(ref strategy_id) = primary_strategy {
        info!(strategy_id, "Restore primary: cooldown expired, switching to primary");
    } else {
        info!("Restore primary: conditions not met (not on backup or cooldown active)");
    }
    
    Ok(primary_strategy)
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

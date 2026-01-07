//! Automation commands
//!
//! Commands for the new automation system:
//! - StrategyOptimizer: one-time strategy optimization
//! - DomainMonitor: continuous domain monitoring

use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Window};
use tracing::{error, info};

use crate::commands::rate_limiter;
use crate::commands::state_guard::get_state_or_error;
use crate::core::automation::{
    AutomationEvent, DomainStatus, MonitorConfig, OptimizationResult,
};
use crate::core::errors::IsolateError;
use crate::core::managers::{Protocol, StrategyStats};
use crate::core::models::Strategy;

// ============================================================================
// Optimizer Commands
// ============================================================================

/// Run strategy optimization using new StrategyOptimizer
///
/// Emits events: automation:progress, automation:complete, automation:error
#[tauri::command]
pub async fn run_optimization_v2(
    app: AppHandle,
    window: Window,
) -> Result<OptimizationResult, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    // Rate limit: max once per 10 seconds
    rate_limiter::check_rate_limit("run_optimization_v2", 10)?;
    
    info!("Starting optimization v2");
    
    // Get strategies and services
    let strategies_map = state.config_manager.load_strategies().await?;
    let strategies: Vec<Strategy> = strategies_map.into_values().collect();
    
    let services_map = state.config_manager.load_services().await?;
    let services: Vec<_> = services_map.values().cloned().collect();
    
    // Get environment info
    let env_info = state.get_env_info().await;
    
    // Subscribe to progress events
    let optimizer = state.optimizer.clone();
    let mut progress_rx = optimizer.subscribe();
    let window_clone = window.clone();
    
    // Spawn progress listener
    tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            if let Err(e) = window_clone.emit("automation:progress", &progress) {
                error!("Failed to emit progress: {}", e);
            }
        }
    });
    
    // Run optimization
    match optimizer.optimize(&env_info, &strategies, &services).await {
        Ok(result) => {
            info!(
                strategy_id = %result.strategy_id,
                score = result.score,
                "Optimization v2 completed"
            );
            
            let _ = window.emit("automation:complete", &result);
            Ok(result)
        }
        Err(e) => {
            error!("Optimization v2 failed: {}", e);
            let _ = window.emit("automation:error", e.to_string());
            Err(e)
        }
    }
}

/// Cancel ongoing optimization
#[tauri::command]
pub async fn cancel_optimization_v2(app: AppHandle) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Cancelling optimization v2");
    state.optimizer.cancel().await;
    Ok(())
}

/// Check if optimization is running
#[tauri::command]
pub async fn is_optimization_v2_running(app: AppHandle) -> Result<bool, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.optimizer.is_running().await)
}

// ============================================================================
// Monitor Commands
// ============================================================================

/// Start domain monitoring
///
/// Emits events: automation:monitor_started, automation:domain_locked, 
/// automation:domain_unlocked, automation:monitor_stopped
#[tauri::command]
pub async fn start_domain_monitor(
    app: AppHandle,
    window: Window,
    domains: Vec<String>,
    _config: Option<MonitorConfig>,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    if domains.is_empty() {
        return Err(IsolateError::Validation("At least one domain is required".into()));
    }
    
    info!(domains = ?domains, "Starting domain monitor");
    
    // Check if already running
    if state.domain_monitor.is_running() {
        return Err(IsolateError::Strategy("Monitor is already running".into()));
    }
    
    // Load strategies
    let strategies_map = state.config_manager.load_strategies().await?;
    let strategies: Vec<Strategy> = strategies_map.into_values().collect();
    
    if strategies.is_empty() {
        return Err(IsolateError::Strategy("No strategies available".into()));
    }
    
    // Subscribe to events
    let monitor = state.domain_monitor.clone();
    let mut event_rx = monitor.subscribe();
    let window_clone = window.clone();
    
    // Spawn event listener
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let event_name = match &event {
                AutomationEvent::MonitorStarted { .. } => "automation:monitor_started",
                AutomationEvent::MonitorStopped => "automation:monitor_stopped",
                AutomationEvent::DomainLocked { .. } => "automation:domain_locked",
                AutomationEvent::DomainUnlocked { .. } => "automation:domain_unlocked",
                AutomationEvent::StrategyBlocked { .. } => "automation:strategy_blocked",
                AutomationEvent::OptimizationProgress(_) => "automation:progress",
            };
            
            if let Err(e) = window_clone.emit(event_name, &event) {
                error!("Failed to emit event: {}", e);
            }
        }
    });
    
    // Start monitor in background
    let monitor_clone = monitor.clone();
    let domains_clone = domains.clone();
    
    tokio::spawn(async move {
        if let Err(e) = monitor_clone.start(&domains_clone, &strategies).await {
            error!("Monitor failed: {}", e);
        }
    });
    
    Ok(())
}

/// Stop domain monitoring
#[tauri::command]
pub async fn stop_domain_monitor(app: AppHandle) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Stopping domain monitor");
    state.domain_monitor.stop().await;
    Ok(())
}

/// Check if domain monitor is running
#[tauri::command]
pub async fn is_domain_monitor_running(app: AppHandle) -> Result<bool, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.domain_monitor.is_running())
}

/// Get domain status
#[tauri::command]
pub async fn get_domain_status(
    app: AppHandle,
    domain: String,
) -> Result<DomainStatus, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.domain_monitor.get_domain_status(&domain).await)
}

/// Get all domain statuses
#[tauri::command]
pub async fn get_all_domain_statuses(
    app: AppHandle,
) -> Result<HashMap<String, DomainStatus>, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.domain_monitor.get_all_statuses().await)
}

// ============================================================================
// Manager Commands
// ============================================================================

/// Get blocked strategies for a domain
#[tauri::command]
pub async fn get_blocked_strategies(
    app: AppHandle,
    domain: String,
) -> Result<Vec<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.blocked_manager.get_blocked_for_domain(&domain).await)
}

/// Block a strategy for a domain
#[tauri::command]
pub async fn block_strategy(
    app: AppHandle,
    domain: String,
    strategy_id: String,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(domain = %domain, strategy_id = %strategy_id, "Blocking strategy");
    state.blocked_manager.block(&domain, &strategy_id).await?;
    Ok(())
}

/// Unblock a strategy for a domain
#[tauri::command]
pub async fn unblock_strategy(
    app: AppHandle,
    domain: String,
    strategy_id: String,
) -> Result<bool, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!(domain = %domain, strategy_id = %strategy_id, "Unblocking strategy");
    Ok(state.blocked_manager.unblock(&domain, &strategy_id).await?)
}

/// Get locked strategy for a domain
#[tauri::command]
pub async fn get_locked_strategy(
    app: AppHandle,
    domain: String,
    protocol: String,
) -> Result<Option<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    let proto = Protocol::from_str(&protocol).ok_or_else(|| {
        IsolateError::Validation(format!("Invalid protocol: {}", protocol))
    })?;
    
    Ok(state.locked_manager.get_locked(&domain, proto).await)
}

/// Lock a strategy for a domain
#[tauri::command]
pub async fn lock_strategy(
    app: AppHandle,
    domain: String,
    strategy_id: String,
    protocol: String,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    let proto = Protocol::from_str(&protocol).ok_or_else(|| {
        IsolateError::Validation(format!("Invalid protocol: {}", protocol))
    })?;
    
    info!(domain = %domain, strategy_id = %strategy_id, protocol = %protocol, "Locking strategy");
    state.locked_manager.lock(&domain, &strategy_id, proto).await?;
    Ok(())
}

/// Unlock a strategy for a domain
#[tauri::command]
pub async fn unlock_strategy(
    app: AppHandle,
    domain: String,
    protocol: String,
) -> Result<Option<String>, IsolateError> {
    let state = get_state_or_error(&app)?;
    
    let proto = Protocol::from_str(&protocol).ok_or_else(|| {
        IsolateError::Validation(format!("Invalid protocol: {}", protocol))
    })?;
    
    info!(domain = %domain, protocol = %protocol, "Unlocking strategy");
    Ok(state.locked_manager.unlock(&domain, proto).await?)
}

/// Get strategy history for a domain (automation version)
#[tauri::command]
pub async fn get_automation_history(
    app: AppHandle,
    domain: String,
) -> Result<HashMap<String, StrategyStats>, IsolateError> {
    let state = get_state_or_error(&app)?;
    Ok(state.history_manager.get_domain_history(&domain).await)
}

/// Clear strategy history (automation version)
#[tauri::command]
pub async fn clear_automation_history(app: AppHandle) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    info!("Clearing automation history");
    state.history_manager.clear().await?;
    Ok(())
}

/// Invalidate strategy cache
#[tauri::command]
pub async fn invalidate_strategy_cache(
    app: AppHandle,
    env_key: Option<String>,
) -> Result<(), IsolateError> {
    let state = get_state_or_error(&app)?;
    
    if let Some(key) = env_key {
        info!(env_key = %key, "Invalidating strategy cache");
        state.cache_manager.invalidate(&key).await?;
    } else {
        info!("Clearing all strategy cache");
        state.cache_manager.invalidate_all().await?;
    }
    Ok(())
}

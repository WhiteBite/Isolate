//! Tauri commands — IPC interface between frontend and backend

// Existing modules
pub mod ab_testing;
pub mod composition;
pub mod diagnostics;
pub mod dns;
pub mod failover;
pub mod health_history;
pub mod hostlists;
pub mod hosts;
pub mod ipset;
pub mod logs;
pub mod monitor;
pub mod plugin_hostlists;
pub mod prewarming;
pub mod proxies;
pub mod quic;
pub mod rate_limiter;
pub mod resources;
pub mod routing;
pub mod scripts;
pub mod settings;
pub mod speedtest;
pub mod state_guard;
pub mod system;
pub mod tcp_timestamps;
pub mod tray;
pub mod tun;
pub mod updates;
pub mod validation;
pub mod vless;

// New refactored modules
pub mod automation;
pub mod metrics;
pub mod network;
pub mod plugins;
pub mod providers;
pub mod services;
pub mod strategies;
pub mod testing;

// Re-exports from existing modules
pub use composition::*;
pub use diagnostics::*;
pub use dns::*;
pub use failover::*;
pub use health_history::*;
pub use hostlists::*;
pub use hosts::*;
pub use ipset::*;
pub use logs::*;
pub use monitor::*;
pub use plugin_hostlists::*;
pub use prewarming::*;
pub use proxies::*;
pub use quic::*;
pub use resources::*;
pub use routing::*;
pub use scripts::*;
pub use settings::*;
pub use speedtest::*;
// Note: state_guard exports are used internally, re-export specific items
// Some exports are for public API but may not be used internally yet
pub use state_guard::get_state_or_error;
#[allow(unused_imports)]
pub use state_guard::{is_state_ready, try_get_state, StateNotReady};
pub use system::*;
pub use tcp_timestamps::*;
pub use tray::*;
pub use tun::*;
pub use updates::*;
pub use vless::*;

// Re-exports from new modules
pub use ab_testing::*;
pub use automation::*;
pub use metrics::*;
pub use network::*;
pub use plugins::*;
pub use providers::*;
pub use services::*;
pub use strategies::*;
pub use testing::*;

use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tracing::info;

use crate::core::errors::IsolateError;
use crate::core::models::AppStatus;
use crate::state::AppState;

/// Check if backend is ready (AppState initialized)
/// This command doesn't require State, so it works even before AppState is ready
#[tauri::command]
pub fn is_backend_ready(app: AppHandle) -> bool {
    app.try_state::<Arc<AppState>>().is_some()
}

/// Verify integrity of all binaries (winws, sing-box, WinDivert, etc.)
/// 
/// Returns detailed verification results including which binaries are valid,
/// missing, or potentially tampered with.
#[tauri::command]
pub async fn verify_binaries_integrity() -> Result<crate::core::integrity::StartupVerificationResult, IsolateError> {
    let binaries_dir = crate::core::paths::get_binaries_dir();
    Ok(crate::core::integrity::verify_on_startup_async(&binaries_dir).await)
}

/// Get current application status
/// 
/// Uses safe state access pattern to handle race condition during initialization.
#[tauri::command]
pub async fn get_status(app: AppHandle) -> Result<AppStatus, IsolateError> {
    let state = get_state_or_error(&app)?;
    
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

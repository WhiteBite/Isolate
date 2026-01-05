//! Diagnostics commands
//!
//! Commands for DPI diagnostics, IPv6 checking, and emergency network reset.

use std::sync::Arc;
use tauri::State;
use tokio::process::Command;
use tracing::{info, warn, error};

use crate::core::models::DiagnosticResult;
use crate::core::diagnostics::{DualStackResult, Ipv6Status};
use crate::state::AppState;

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

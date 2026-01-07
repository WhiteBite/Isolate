//! Tauri commands for process resource limits management
//!
//! Provides IPC interface for:
//! - Setting resource limits on processes (memory, CPU, priority)
//! - Getting current process resource usage
//! - Getting default/recommended limits

use crate::core::resource_limits::{
    get_default_limits, get_recommended_limits, ProcessLimiter, ProcessUsage, ResourceLimits,
};
use crate::core::errors::IsolateError;
use tracing::{info, warn};

/// Set resource limits on a process
///
/// # Arguments
/// * `pid` - Process ID to apply limits to
/// * `limits` - Resource limits configuration
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(IsolateError)` if limits cannot be applied
#[tauri::command]
pub async fn set_process_limits(pid: u32, limits: ResourceLimits) -> Result<(), IsolateError> {
    if pid == 0 {
        return Err(IsolateError::Validation("PID cannot be 0".to_string()));
    }
    
    info!(pid, ?limits, "Setting process resource limits");
    
    let mut limiter = ProcessLimiter::new()?;
    limiter.apply_limits(pid, &limits)?;
    
    info!(pid, "Process resource limits applied successfully");
    Ok(())
}

/// Get current resource usage of a process
///
/// # Arguments
/// * `pid` - Process ID to query
///
/// # Returns
/// * `ProcessUsage` with current memory, CPU usage, etc.
#[tauri::command]
pub async fn get_process_usage(pid: u32) -> Result<ProcessUsage, IsolateError> {
    if pid == 0 {
        return Err(IsolateError::Validation("PID cannot be 0".to_string()));
    }
    
    info!(pid, "Getting process resource usage");
    ProcessLimiter::get_process_usage(pid)
}

/// Get default resource limits (no restrictions)
///
/// # Returns
/// * `ResourceLimits` with default values (unlimited)
#[tauri::command]
pub fn get_default_resource_limits() -> ResourceLimits {
    get_default_limits()
}

/// Get recommended resource limits for a specific process type
///
/// # Arguments
/// * `process_type` - Type of process ("winws", "singbox", etc.)
///
/// # Returns
/// * `ResourceLimits` with recommended values for the process type
#[tauri::command]
pub fn get_recommended_resource_limits(process_type: String) -> ResourceLimits {
    info!(process_type, "Getting recommended resource limits");
    get_recommended_limits(&process_type)
}

/// Get resource usage for multiple processes
///
/// # Arguments
/// * `pids` - List of process IDs to query
///
/// # Returns
/// * Vector of `ProcessUsage` for each process
#[tauri::command]
pub async fn get_multiple_process_usage(pids: Vec<u32>) -> Result<Vec<ProcessUsage>, IsolateError> {
    info!(count = pids.len(), "Getting resource usage for multiple processes");
    
    let mut results = Vec::with_capacity(pids.len());
    for pid in pids {
        match ProcessLimiter::get_process_usage(pid) {
            Ok(usage) => results.push(usage),
            Err(e) => {
                warn!(pid, error = %e, "Failed to get process usage");
                // Include a default entry for failed processes
                results.push(ProcessUsage {
                    pid,
                    is_running: false,
                    ..Default::default()
                });
            }
        }
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_resource_limits() {
        let limits = get_default_resource_limits();
        assert_eq!(limits.max_memory_mb, 0);
        assert!(!limits.has_limits());
    }

    #[test]
    fn test_get_recommended_resource_limits() {
        let winws_limits = get_recommended_resource_limits("winws".to_string());
        assert_eq!(winws_limits.max_memory_mb, 128);
        
        let singbox_limits = get_recommended_resource_limits("singbox".to_string());
        assert_eq!(singbox_limits.max_memory_mb, 256);
    }
}

//! Global Process Runner for Isolate
//!
//! Provides a singleton ProcessRunner instance to ensure all processes
//! are managed centrally. This prevents issues where multiple ProcessRunner
//! instances lose track of each other's processes.
//!
//! CRITICAL: All process management MUST go through this module!
//!
//! NOTE: Some functions are prepared for future process management features.

// Public API for global process management
#![allow(dead_code)]

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::core::errors::Result;
use crate::core::process_runner::{ManagedProcess, ProcessConfig, ProcessRunner};

/// Global ProcessRunner instance
static GLOBAL_RUNNER: Lazy<Arc<RwLock<ProcessRunner>>> =
    Lazy::new(|| Arc::new(RwLock::new(ProcessRunner::new())));

/// Get the global ProcessRunner instance
pub fn get_runner() -> Arc<RwLock<ProcessRunner>> {
    GLOBAL_RUNNER.clone()
}

/// Spawn a process using the global runner
///
/// # Arguments
/// * `id` - Unique identifier for the process
/// * `config` - Process configuration
///
/// # Returns
/// * `Ok(Arc<ManagedProcess>)` - Handle to the spawned process
/// * `Err` - Failed to spawn process
pub async fn spawn(id: &str, config: ProcessConfig) -> Result<Arc<ManagedProcess>> {
    let runner = GLOBAL_RUNNER.read().await;
    runner.spawn(id, config).await
}

/// Stop a process by ID using the global runner
///
/// # Arguments
/// * `id` - Process identifier
///
/// # Returns
/// * `Ok(())` - Process stopped successfully
/// * `Err` - Failed to stop process
pub async fn stop(id: &str) -> Result<()> {
    let runner = GLOBAL_RUNNER.read().await;
    runner.stop(id).await
}

/// Stop all processes managed by the global runner
///
/// # Returns
/// * `Ok(())` - All processes stopped
/// * `Err` - Failed to stop some processes
pub async fn stop_all() -> Result<()> {
    info!("Stopping all processes via global runner");
    let runner = GLOBAL_RUNNER.read().await;
    runner.stop_all().await
}

/// Get a process by ID from the global runner
///
/// # Arguments
/// * `id` - Process identifier
///
/// # Returns
/// * `Some(Arc<ManagedProcess>)` - Process found
/// * `None` - Process not found
pub async fn get(id: &str) -> Option<Arc<ManagedProcess>> {
    let runner = GLOBAL_RUNNER.read().await;
    runner.get(id).await
}

/// List all process IDs managed by the global runner
pub async fn list() -> Vec<String> {
    let runner = GLOBAL_RUNNER.read().await;
    runner.list().await
}

/// Check if a process is running
///
/// # Arguments
/// * `id` - Process identifier
///
/// # Returns
/// * `true` - Process is running
/// * `false` - Process is not running or not found
pub async fn is_running(id: &str) -> bool {
    let runner = GLOBAL_RUNNER.read().await;
    runner.is_running(id).await
}

/// Get count of running processes
pub async fn running_count() -> usize {
    let ids = list().await;
    let mut count = 0;
    for id in ids {
        if is_running(&id).await {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::debug;

    #[tokio::test]
    async fn test_global_runner_singleton() {
        let runner1 = get_runner();
        let runner2 = get_runner();
        
        // Both should point to the same instance
        assert!(Arc::ptr_eq(&runner1, &runner2));
    }

    #[tokio::test]
    async fn test_list_empty() {
        // Initially should be empty (or have processes from other tests)
        let processes = list().await;
        // Just verify it doesn't panic
        debug!("Current processes: {:?}", processes);
    }
}

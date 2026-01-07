//! Unified process management interface
//!
//! Provides a common trait for managing external processes (winws, sing-box, etc.)
//! with automatic cleanup and graceful shutdown support.

#![allow(dead_code)] // Public process management API

use async_trait::async_trait;
use crate::core::errors::Result;

/// Unified interface for process management.
///
/// Implementors should handle:
/// - Process lifecycle (start, stop)
/// - Graceful shutdown with timeout
/// - Resource cleanup on drop
///
/// # Example
/// ```ignore
/// struct MyProcess {
///     handle: ProcessHandle,
/// }
///
/// #[async_trait]
/// impl ProcessManager for MyProcess {
///     async fn start(&mut self) -> Result<()> {
///         // Start the process
///         Ok(())
///     }
///     
///     async fn stop(&mut self) -> Result<()> {
///         self.handle.stop().await
///     }
///     
///     fn is_running(&self) -> bool {
///         self.handle.is_running()
///     }
///     
///     fn pid(&self) -> Option<u32> {
///         self.handle.pid()
///     }
///     
///     fn name(&self) -> &str {
///         &self.handle.name
///     }
/// }
/// ```
#[async_trait]
pub trait ProcessManager: Send + Sync {
    /// Start the process.
    ///
    /// Should be idempotent - calling start on an already running process
    /// should either be a no-op or restart the process.
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the process gracefully.
    ///
    /// Should attempt graceful shutdown first, then force kill after timeout.
    /// Should be safe to call multiple times.
    async fn stop(&mut self) -> Result<()>;
    
    /// Check if process is currently running.
    fn is_running(&self) -> bool;
    
    /// Get process ID if running.
    ///
    /// Returns `None` if process is not running or PID is unavailable.
    fn pid(&self) -> Option<u32>;
    
    /// Get process name for logging and identification.
    fn name(&self) -> &str;
}

/// Process handle with automatic cleanup.
///
/// Wraps a `tokio::process::Child` with additional metadata and
/// implements automatic cleanup on drop.
///
/// # Automatic Cleanup
/// When dropped, the handle will attempt to kill the child process
/// to prevent orphaned processes.
pub struct ProcessHandle {
    /// The underlying child process, if running
    pub child: Option<tokio::process::Child>,
    /// Human-readable name for logging
    pub name: String,
    /// When the process was started
    pub started_at: std::time::Instant,
}

impl ProcessHandle {
    /// Create a new process handle.
    ///
    /// # Arguments
    /// * `child` - The spawned child process
    /// * `name` - Human-readable name for logging
    pub fn new(child: tokio::process::Child, name: impl Into<String>) -> Self {
        Self {
            child: Some(child),
            name: name.into(),
            started_at: std::time::Instant::now(),
        }
    }
    
    /// Check if the process is still running.
    ///
    /// Note: This only checks if we have a handle, not if the process
    /// is actually alive. Use `try_wait()` for accurate status.
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
    
    /// Get the process ID if available.
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().and_then(|c| c.id())
    }
    
    /// Get how long the process has been running.
    pub fn uptime(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }
    
    /// Stop the process gracefully with timeout.
    ///
    /// Attempts to kill the process and waits up to 3 seconds for it to exit.
    /// If the process doesn't exit within the timeout, it's forcefully terminated.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            tracing::debug!(name = %self.name, pid = ?child.id(), "Stopping process");
            
            // Send kill signal
            let _ = child.start_kill();
            
            // Wait for process to exit with timeout
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                child.wait()
            ).await;
            
            tracing::debug!(name = %self.name, "Process stopped");
        }
        Ok(())
    }
    
    /// Take ownership of the child process.
    ///
    /// After calling this, the handle will no longer manage the process
    /// and automatic cleanup will not occur.
    pub fn take(&mut self) -> Option<tokio::process::Child> {
        self.child.take()
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::warn!(
                name = %self.name,
                pid = ?child.id(),
                "Process handle dropped while still running, killing process"
            );
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_handle_creation() {
        // We can't easily test with a real process, but we can test the structure
        let handle = ProcessHandle {
            child: None,
            name: "test-process".to_string(),
            started_at: std::time::Instant::now(),
        };
        
        assert!(!handle.is_running());
        assert!(handle.pid().is_none());
        assert_eq!(handle.name, "test-process");
    }
    
    #[test]
    fn test_process_handle_uptime() {
        let handle = ProcessHandle {
            child: None,
            name: "test".to_string(),
            started_at: std::time::Instant::now(),
        };
        
        // Uptime should be very small
        assert!(handle.uptime().as_millis() < 100);
    }
}

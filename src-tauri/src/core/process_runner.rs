//! Process runner for external binaries (winws, sing-box)
//!
//! Handles process lifecycle: spawn, capture output, graceful shutdown.
//!
//! ## Output Capture Synchronization
//!
//! When a process terminates quickly, its stdout/stderr output might be lost
//! if the reading tasks haven't started yet. To prevent this, we use a
//! `Notify` mechanism to ensure at least one read attempt has been made
//! before returning from `spawn()`.

#![allow(dead_code)] // Public process runner API

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex, Notify, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};

/// Default startup timeout in milliseconds
const DEFAULT_STARTUP_TIMEOUT_MS: u64 = 5000;

/// Default shutdown timeout in milliseconds
const DEFAULT_SHUTDOWN_TIMEOUT_MS: u64 = 3000;

/// Process output line
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub stream: OutputStream,
    pub line: String,
    pub timestamp: std::time::Instant,
}

/// Output stream type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

/// Managed process handle
pub struct ManagedProcess {
    pub id: String,
    pub binary: String,
    state: Arc<RwLock<ProcessState>>,
    child: Arc<Mutex<Option<Child>>>,
    output_tx: broadcast::Sender<OutputLine>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ManagedProcess {
    /// Get current process state
    pub async fn state(&self) -> ProcessState {
        *self.state.read().await
    }

    /// Check if process is running
    pub async fn is_running(&self) -> bool {
        matches!(self.state().await, ProcessState::Running)
    }

    /// Subscribe to process output
    pub fn subscribe_output(&self) -> broadcast::Receiver<OutputLine> {
        self.output_tx.subscribe()
    }

    /// Get process ID (PID) if running
    pub async fn pid(&self) -> Option<u32> {
        let child = self.child.lock().await;
        child.as_ref().and_then(|c| c.id())
    }
}

/// Process runner configuration
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    pub binary: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub startup_timeout_ms: u64,
    pub shutdown_timeout_ms: u64,
    pub requires_admin: bool,
}

impl ProcessConfig {
    pub fn new(binary: PathBuf, args: Vec<String>) -> Self {
        Self {
            binary,
            args,
            env: HashMap::new(),
            working_dir: None,
            startup_timeout_ms: DEFAULT_STARTUP_TIMEOUT_MS,
            shutdown_timeout_ms: DEFAULT_SHUTDOWN_TIMEOUT_MS,
            requires_admin: false,
        }
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_startup_timeout(mut self, timeout_ms: u64) -> Self {
        self.startup_timeout_ms = timeout_ms;
        self
    }

    pub fn with_shutdown_timeout(mut self, timeout_ms: u64) -> Self {
        self.shutdown_timeout_ms = timeout_ms;
        self
    }

    pub fn with_admin(mut self, requires: bool) -> Self {
        self.requires_admin = requires;
        self
    }
}

/// Process runner for managing external processes
pub struct ProcessRunner {
    processes: Arc<RwLock<HashMap<String, Arc<ManagedProcess>>>>,
    /// Lock for atomic check-and-stop operations
    stop_lock: Arc<Mutex<()>>,
}

impl Default for ProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRunner {
    /// Create a new process runner
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            stop_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Spawn a new managed process
    ///
    /// This method ensures that stdout/stderr capture tasks have started
    /// before returning, preventing output loss for fast-terminating processes.
    pub async fn spawn(&self, id: &str, config: ProcessConfig) -> Result<Arc<ManagedProcess>> {
        info!(
            id = %id,
            binary = ?config.binary,
            args = ?config.args,
            "Spawning process"
        );

        // Check if process with this ID already exists
        {
            let processes = self.processes.read().await;
            if let Some(existing) = processes.get(id) {
                if existing.is_running().await {
                    return Err(IsolateError::Process(format!(
                        "Process '{}' is already running",
                        id
                    )));
                }
            }
        }

        // Verify binary exists
        if !config.binary.exists() {
            return Err(IsolateError::Process(format!(
                "Binary not found: {:?}",
                config.binary
            )));
        }

        // Build command
        let mut cmd = Command::new(&config.binary);
        cmd.args(&config.args)
            .envs(&config.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(ref dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| {
            IsolateError::Process(format!("Failed to spawn {}: {}", config.binary.display(), e))
        })?;

        let pid = child.id();
        debug!(id = %id, pid = ?pid, "Process spawned");

        // Create channels
        let (output_tx, _) = broadcast::channel(1000);
        let (shutdown_tx, _) = broadcast::channel(1);

        let state = Arc::new(RwLock::new(ProcessState::Starting));

        // Create Notify instances to signal when output capture has started
        // This prevents output loss for fast-terminating processes
        let stdout_ready = Arc::new(Notify::new());
        let stderr_ready = Arc::new(Notify::new());

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let tx = output_tx.clone();
            let _state_clone = state.clone();
            let id_clone = id.to_string();
            let ready = stdout_ready.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                // Signal that we're ready to read (first read attempt)
                // This is done before the first read to ensure the task has started
                ready.notify_one();

                while let Ok(Some(line)) = lines.next_line().await {
                    debug!(id = %id_clone, stream = "stdout", %line);

                    let _ = tx.send(OutputLine {
                        stream: OutputStream::Stdout,
                        line,
                        timestamp: std::time::Instant::now(),
                    });
                }

                debug!(id = %id_clone, "Stdout stream closed");
            });
        } else {
            // No stdout pipe - signal ready immediately
            stdout_ready.notify_one();
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let tx = output_tx.clone();
            let _state_clone = state.clone();
            let id_clone = id.to_string();
            let ready = stderr_ready.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                // Signal that we're ready to read (first read attempt)
                ready.notify_one();

                while let Ok(Some(line)) = lines.next_line().await {
                    debug!(id = %id_clone, stream = "stderr", %line);

                    let _ = tx.send(OutputLine {
                        stream: OutputStream::Stderr,
                        line,
                        timestamp: std::time::Instant::now(),
                    });
                }

                debug!(id = %id_clone, "Stderr stream closed");
            });
        } else {
            // No stderr pipe - signal ready immediately
            stderr_ready.notify_one();
        }

        // Wait for both output capture tasks to start (with timeout)
        // This ensures we don't lose output from fast-terminating processes
        let wait_timeout = Duration::from_millis(100);
        let _ = timeout(wait_timeout, stdout_ready.notified()).await;
        let _ = timeout(wait_timeout, stderr_ready.notified()).await;

        debug!(id = %id, "Output capture tasks started");

        // Update state to running
        *state.write().await = ProcessState::Running;

        let managed = Arc::new(ManagedProcess {
            id: id.to_string(),
            binary: config.binary.display().to_string(),
            state,
            child: Arc::new(Mutex::new(Some(child))),
            output_tx,
            shutdown_tx,
        });

        // Store process
        {
            let mut processes = self.processes.write().await;
            processes.insert(id.to_string(), managed.clone());
        }

        info!(id = %id, pid = ?pid, "Process started successfully");
        Ok(managed)
    }

    /// Stop a process gracefully (terminate, then kill if needed)
    /// 
    /// This operation is atomic - it acquires a lock to prevent race conditions
    /// between is_running() checks and stop() calls.
    pub async fn stop(&self, id: &str) -> Result<()> {
        info!(id = %id, "Stopping process");

        // Acquire stop lock to prevent race conditions
        let _stop_guard = self.stop_lock.lock().await;

        let process = {
            let processes = self.processes.read().await;
            processes.get(id).cloned()
        };

        let process = match process {
            Some(p) => p,
            None => {
                debug!(id = %id, "Process not found, may already be stopped");
                return Ok(());
            }
        };

        self.stop_process(&process).await
    }
    
    /// Atomically check if running and stop if so.
    /// 
    /// This prevents the race condition where:
    /// 1. Thread A calls is_running() -> true
    /// 2. Thread B calls stop() -> process stopped
    /// 3. Thread A calls stop() -> error or unexpected behavior
    /// 
    /// # Returns
    /// * `Ok(true)` - Process was running and has been stopped
    /// * `Ok(false)` - Process was not running
    /// * `Err(...)` - Error stopping the process
    pub async fn stop_if_running(&self, id: &str) -> Result<bool> {
        // Acquire stop lock for atomic check-and-stop
        let _stop_guard = self.stop_lock.lock().await;
        
        let process = {
            let processes = self.processes.read().await;
            processes.get(id).cloned()
        };
        
        match process {
            Some(p) if p.is_running().await => {
                self.stop_process(&p).await?;
                Ok(true)
            }
            Some(_) => {
                // Process exists but not running - clean up
                let mut processes = self.processes.write().await;
                processes.remove(id);
                Ok(false)
            }
            None => Ok(false),
        }
    }

    /// Internal stop implementation
    async fn stop_process(&self, process: &ManagedProcess) -> Result<()> {
        let id = &process.id;

        // Update state
        *process.state.write().await = ProcessState::Stopping;

        // Signal shutdown
        let _ = process.shutdown_tx.send(());

        let mut child_guard = process.child.lock().await;

        if let Some(ref mut child) = *child_guard {
            // Try graceful termination first (SIGTERM on Unix, TerminateProcess on Windows)
            #[cfg(windows)]
            {
                if let Some(pid) = child.id() {
                    debug!(id = %id, pid = pid, "Sending terminate signal");

                    // Use taskkill for graceful termination
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string()])
                        .output()
                        .await;
                }
            }

            #[cfg(not(windows))]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                if let Some(pid) = child.id() {
                    debug!(id = %id, pid = pid, "Sending SIGTERM");
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
            }

            // Wait for graceful shutdown with timeout
            let shutdown_timeout = Duration::from_millis(DEFAULT_SHUTDOWN_TIMEOUT_MS);

            match timeout(shutdown_timeout, child.wait()).await {
                Ok(Ok(status)) => {
                    info!(id = %id, ?status, "Process terminated gracefully");
                }
                Ok(Err(e)) => {
                    warn!(id = %id, error = %e, "Error waiting for process");
                }
                Err(_) => {
                    // Timeout - force kill
                    warn!(id = %id, "Graceful shutdown timeout, force killing");

                    if let Err(e) = child.kill().await {
                        error!(id = %id, error = %e, "Failed to kill process");
                    }

                    // Wait for kill to complete
                    let _ = child.wait().await;
                    info!(id = %id, "Process force killed");
                }
            }
        }

        // Update state
        *process.state.write().await = ProcessState::Stopped;

        // Remove from active processes
        {
            let mut processes = self.processes.write().await;
            processes.remove(id);
        }

        Ok(())
    }

    /// Stop all running processes
    pub async fn stop_all(&self) -> Result<()> {
        info!("Stopping all processes");

        let processes: Vec<Arc<ManagedProcess>> = {
            let processes = self.processes.read().await;
            processes.values().cloned().collect()
        };

        for process in processes {
            if let Err(e) = self.stop_process(&process).await {
                error!(id = %process.id, error = %e, "Failed to stop process");
            }
        }

        Ok(())
    }

    /// Get a process by ID
    pub async fn get(&self, id: &str) -> Option<Arc<ManagedProcess>> {
        let processes = self.processes.read().await;
        processes.get(id).cloned()
    }

    /// List all managed processes
    pub async fn list(&self) -> Vec<String> {
        let processes = self.processes.read().await;
        processes.keys().cloned().collect()
    }

    /// Check if a process is running
    pub async fn is_running(&self, id: &str) -> bool {
        if let Some(process) = self.get(id).await {
            process.is_running().await
        } else {
            false
        }
    }

    /// Wait for process to exit
    pub async fn wait(&self, id: &str) -> Result<()> {
        let process = self.get(id).await.ok_or_else(|| {
            IsolateError::Process(format!("Process '{}' not found", id))
        })?;

        let mut child_guard = process.child.lock().await;

        if let Some(ref mut child) = *child_guard {
            let status = child.wait().await?;
            info!(id = %id, ?status, "Process exited");
        }

        Ok(())
    }

    /// Wait for process with timeout
    pub async fn wait_with_timeout(&self, id: &str, timeout_ms: u64) -> Result<()> {
        let wait_future = self.wait(id);

        timeout(Duration::from_millis(timeout_ms), wait_future)
            .await
            .map_err(|_| IsolateError::StrategyTimeout(timeout_ms as u32))?
    }
}

impl Drop for ProcessRunner {
    fn drop(&mut self) {
        // Note: Actual cleanup happens via kill_on_drop(true) on Child
        debug!("ProcessRunner dropped");
    }
}

/// Spawn a one-shot process and wait for completion
pub async fn run_command(
    binary: &std::path::Path,
    args: &[&str],
    timeout_ms: u64,
) -> Result<(String, String)> {
    debug!(binary = ?binary, args = ?args, "Running command");

    let output = timeout(
        Duration::from_millis(timeout_ms),
        Command::new(binary).args(args).output(),
    )
    .await
    .map_err(|_| IsolateError::StrategyTimeout(timeout_ms as u32))?
    .map_err(|e| IsolateError::Process(format!("Command failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        warn!(
            binary = ?binary,
            status = ?output.status,
            stderr = %stderr,
            "Command failed"
        );
    }

    Ok((stdout, stderr))
}

/// Gracefully stop a child process with timeout
///
/// Sends a terminate signal (taskkill on Windows, SIGTERM on Unix) and waits
/// for the process to exit. If the process doesn't exit within the timeout,
/// it will be force killed.
///
/// # Arguments
/// * `child` - Mutable reference to Child process
/// * `timeout_ms` - Timeout for graceful shutdown in milliseconds
/// * `process_name` - Name for logging purposes
///
/// # Returns
/// * `Ok(true)` - Process terminated gracefully
/// * `Ok(false)` - Process was force killed after timeout
///
/// # Example
/// ```ignore
/// let mut child = Command::new("some_process").spawn()?;
/// let graceful = graceful_stop_child(&mut child, 3000, "some_process").await?;
/// if graceful {
///     println!("Process stopped gracefully");
/// } else {
///     println!("Process was force killed");
/// }
/// ```
pub async fn graceful_stop_child(
    child: &mut tokio::process::Child,
    timeout_ms: u64,
    process_name: &str,
) -> Result<bool> {
    #[cfg(windows)]
    {
        if let Some(pid) = child.id() {
            debug!(pid, process_name, "Sending terminate signal via taskkill");
            let _ = tokio::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output()
                .await;
        }
    }

    #[cfg(not(windows))]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        if let Some(pid) = child.id() {
            debug!(pid, process_name, "Sending SIGTERM");
            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
        }
    }

    match timeout(Duration::from_millis(timeout_ms), child.wait()).await {
        Ok(Ok(status)) => {
            info!(process_name, ?status, "Process terminated gracefully");
            Ok(true)
        }
        Ok(Err(e)) => {
            warn!(process_name, error = %e, "Error waiting for process, force killing");
            let _ = child.kill().await;
            let _ = child.wait().await;
            Ok(false)
        }
        Err(_) => {
            warn!(process_name, timeout_ms, "Graceful shutdown timeout, force killing");
            let _ = child.kill().await;
            let _ = child.wait().await;
            Ok(false)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== ProcessConfig Tests ====================

    #[test]
    fn test_process_config_new() {
        let binary = PathBuf::from("test.exe");
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        
        let config = ProcessConfig::new(binary.clone(), args.clone());
        
        assert_eq!(config.binary, binary);
        assert_eq!(config.args, args);
        assert!(config.env.is_empty());
        assert!(config.working_dir.is_none());
        assert_eq!(config.startup_timeout_ms, DEFAULT_STARTUP_TIMEOUT_MS);
        assert_eq!(config.shutdown_timeout_ms, DEFAULT_SHUTDOWN_TIMEOUT_MS);
        assert!(!config.requires_admin);
    }

    #[test]
    fn test_process_config_builder_pattern() {
        let binary = PathBuf::from("test.exe");
        let args = vec!["arg1".to_string()];
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "VALUE".to_string());
        let working_dir = PathBuf::from("/tmp");
        
        let config = ProcessConfig::new(binary.clone(), args)
            .with_env(env.clone())
            .with_working_dir(working_dir.clone())
            .with_startup_timeout(10000)
            .with_shutdown_timeout(5000)
            .with_admin(true);
        
        assert_eq!(config.env, env);
        assert_eq!(config.working_dir, Some(working_dir));
        assert_eq!(config.startup_timeout_ms, 10000);
        assert_eq!(config.shutdown_timeout_ms, 5000);
        assert!(config.requires_admin);
    }

    #[test]
    fn test_process_config_empty_args() {
        let config = ProcessConfig::new(PathBuf::from("cmd.exe"), vec![]);
        
        assert!(config.args.is_empty());
    }

    // ==================== ProcessRunner Tests ====================

    #[test]
    fn test_process_runner_new() {
        let runner = ProcessRunner::new();
        
        // Should be empty initially
        let rt = tokio::runtime::Runtime::new().unwrap();
        let list = rt.block_on(runner.list());
        assert!(list.is_empty());
    }

    #[test]
    fn test_process_runner_default() {
        let runner = ProcessRunner::default();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let list = rt.block_on(runner.list());
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_process_runner_is_running_nonexistent() {
        let runner = ProcessRunner::new();
        
        // Non-existent process should return false
        assert!(!runner.is_running("nonexistent").await);
    }

    #[tokio::test]
    async fn test_process_runner_get_nonexistent() {
        let runner = ProcessRunner::new();
        
        // Non-existent process should return None
        assert!(runner.get("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_process_runner_stop_nonexistent() {
        let runner = ProcessRunner::new();
        
        // Stopping non-existent process should succeed (idempotent)
        let result = runner.stop("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_runner_stop_if_running_nonexistent() {
        let runner = ProcessRunner::new();
        
        // Should return Ok(false) for non-existent process
        let result = runner.stop_if_running("nonexistent").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_process_runner_list_empty() {
        let runner = ProcessRunner::new();
        
        let list = runner.list().await;
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_process_runner_stop_all_empty() {
        let runner = ProcessRunner::new();
        
        // Should succeed even with no processes
        let result = runner.stop_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_runner_spawn_missing_binary() {
        let runner = ProcessRunner::new();
        let config = ProcessConfig::new(
            PathBuf::from("nonexistent_binary_12345.exe"),
            vec![],
        );
        
        let result = runner.spawn("test", config).await;
        
        assert!(result.is_err());
        // Verify error message contains info about binary not found
        if let Err(e) = result {
            let err_str = format!("{}", e);
            assert!(err_str.contains("not found") || err_str.contains("Binary"));
        }
    }

    // ==================== Integration Tests (require real processes) ====================

    #[tokio::test]
    #[cfg(windows)]
    async fn test_process_runner_spawn_and_stop_cmd() {
        let runner = ProcessRunner::new();
        
        // Use cmd.exe which exists on all Windows systems
        let config = ProcessConfig::new(
            PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            vec!["/c".to_string(), "ping".to_string(), "localhost".to_string(), "-n".to_string(), "100".to_string()],
        );
        
        // Spawn process
        let result = runner.spawn("test_cmd", config).await;
        assert!(result.is_ok());
        
        let process = result.unwrap();
        assert_eq!(process.id, "test_cmd");
        assert!(process.is_running().await);
        
        // Verify it's in the list
        let list = runner.list().await;
        assert!(list.contains(&"test_cmd".to_string()));
        
        // Verify is_running
        assert!(runner.is_running("test_cmd").await);
        
        // Stop process
        let stop_result = runner.stop("test_cmd").await;
        assert!(stop_result.is_ok());
        
        // Verify it's stopped
        assert!(!runner.is_running("test_cmd").await);
        
        // Verify it's removed from list
        let list = runner.list().await;
        assert!(!list.contains(&"test_cmd".to_string()));
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_process_runner_spawn_duplicate_id() {
        let runner = ProcessRunner::new();
        
        let config = ProcessConfig::new(
            PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            vec!["/c".to_string(), "ping".to_string(), "localhost".to_string(), "-n".to_string(), "100".to_string()],
        );
        
        // Spawn first process
        let result1 = runner.spawn("duplicate_test", config.clone()).await;
        assert!(result1.is_ok());
        
        // Try to spawn with same ID - should fail
        let result2 = runner.spawn("duplicate_test", config).await;
        assert!(result2.is_err());
        // Verify error message contains info about already running
        if let Err(e) = result2 {
            let err_str = format!("{}", e);
            assert!(err_str.contains("already running"));
        }
        
        // Cleanup
        let _ = runner.stop("duplicate_test").await;
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_process_runner_stop_if_running() {
        let runner = ProcessRunner::new();
        
        let config = ProcessConfig::new(
            PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            vec!["/c".to_string(), "ping".to_string(), "localhost".to_string(), "-n".to_string(), "100".to_string()],
        );
        
        // Spawn process
        let _ = runner.spawn("stop_if_test", config).await.unwrap();
        
        // stop_if_running should return true (was running)
        let result = runner.stop_if_running("stop_if_test").await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // Was running
        
        // Second call should return false (not running anymore)
        let result2 = runner.stop_if_running("stop_if_test").await;
        assert!(result2.is_ok());
        assert!(!result2.unwrap()); // Not running
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_process_runner_get_pid() {
        let runner = ProcessRunner::new();
        
        let config = ProcessConfig::new(
            PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            vec!["/c".to_string(), "ping".to_string(), "localhost".to_string(), "-n".to_string(), "100".to_string()],
        );
        
        let process = runner.spawn("pid_test", config).await.unwrap();
        
        // Should have a PID
        let pid = process.pid().await;
        assert!(pid.is_some());
        assert!(pid.unwrap() > 0);
        
        // Cleanup
        let _ = runner.stop("pid_test").await;
    }

    // ==================== run_command Tests ====================

    #[tokio::test]
    #[cfg(windows)]
    async fn test_run_command_success() {
        let result = run_command(
            std::path::Path::new("C:\\Windows\\System32\\cmd.exe"),
            &["/c", "echo", "hello"],
            5000,
        ).await;
        
        assert!(result.is_ok());
        let (stdout, _stderr) = result.unwrap();
        assert!(stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_run_command_missing_binary() {
        let result = run_command(
            std::path::Path::new("nonexistent_binary_12345"),
            &[],
            5000,
        ).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_run_command_timeout() {
        // This test uses a very short timeout to trigger timeout error
        let result = run_command(
            std::path::Path::new("C:\\Windows\\System32\\cmd.exe"),
            &["/c", "ping", "localhost", "-n", "10"],
            100, // Very short timeout
        ).await;
        
        assert!(result.is_err());
        // Verify it's a timeout error by checking the error message
        let err_str = format!("{}", result.unwrap_err());
        assert!(err_str.contains("timeout") || err_str.contains("Timeout"));
    }

    // ==================== OutputLine & OutputStream Tests ====================

    #[test]
    fn test_output_stream_equality() {
        assert_eq!(OutputStream::Stdout, OutputStream::Stdout);
        assert_eq!(OutputStream::Stderr, OutputStream::Stderr);
        assert_ne!(OutputStream::Stdout, OutputStream::Stderr);
    }

    #[test]
    fn test_output_line_creation() {
        let line = OutputLine {
            stream: OutputStream::Stdout,
            line: "test output".to_string(),
            timestamp: std::time::Instant::now(),
        };
        
        assert_eq!(line.stream, OutputStream::Stdout);
        assert_eq!(line.line, "test output");
    }

    // ==================== ProcessState Tests ====================

    #[test]
    fn test_process_state_equality() {
        assert_eq!(ProcessState::Starting, ProcessState::Starting);
        assert_eq!(ProcessState::Running, ProcessState::Running);
        assert_eq!(ProcessState::Stopping, ProcessState::Stopping);
        assert_eq!(ProcessState::Stopped, ProcessState::Stopped);
        assert_eq!(ProcessState::Failed, ProcessState::Failed);
        
        assert_ne!(ProcessState::Running, ProcessState::Stopped);
    }

    // ==================== ManagedProcess Tests ====================

    #[tokio::test]
    async fn test_managed_process_state_transitions() {
        // Create a minimal ManagedProcess for testing state
        let (output_tx, _) = broadcast::channel(10);
        let (shutdown_tx, _) = broadcast::channel(1);
        let state = Arc::new(RwLock::new(ProcessState::Starting));
        
        let process = ManagedProcess {
            id: "test".to_string(),
            binary: "test.exe".to_string(),
            state: state.clone(),
            child: Arc::new(Mutex::new(None)),
            output_tx,
            shutdown_tx,
        };
        
        // Initial state
        assert_eq!(process.state().await, ProcessState::Starting);
        assert!(!process.is_running().await);
        
        // Transition to Running
        *state.write().await = ProcessState::Running;
        assert_eq!(process.state().await, ProcessState::Running);
        assert!(process.is_running().await);
        
        // Transition to Stopping
        *state.write().await = ProcessState::Stopping;
        assert_eq!(process.state().await, ProcessState::Stopping);
        assert!(!process.is_running().await);
        
        // Transition to Stopped
        *state.write().await = ProcessState::Stopped;
        assert_eq!(process.state().await, ProcessState::Stopped);
        assert!(!process.is_running().await);
    }

    #[tokio::test]
    async fn test_managed_process_subscribe_output() {
        let (output_tx, _) = broadcast::channel(10);
        let (shutdown_tx, _) = broadcast::channel(1);
        
        let process = ManagedProcess {
            id: "test".to_string(),
            binary: "test.exe".to_string(),
            state: Arc::new(RwLock::new(ProcessState::Running)),
            child: Arc::new(Mutex::new(None)),
            output_tx: output_tx.clone(),
            shutdown_tx,
        };
        
        // Subscribe to output
        let mut rx = process.subscribe_output();
        
        // Send a message
        let _ = output_tx.send(OutputLine {
            stream: OutputStream::Stdout,
            line: "test message".to_string(),
            timestamp: std::time::Instant::now(),
        });
        
        // Receive the message
        let received = rx.recv().await;
        assert!(received.is_ok());
        let line = received.unwrap();
        assert_eq!(line.line, "test message");
        assert_eq!(line.stream, OutputStream::Stdout);
    }

    #[tokio::test]
    async fn test_managed_process_pid_none_when_no_child() {
        let (output_tx, _) = broadcast::channel(10);
        let (shutdown_tx, _) = broadcast::channel(1);
        
        let process = ManagedProcess {
            id: "test".to_string(),
            binary: "test.exe".to_string(),
            state: Arc::new(RwLock::new(ProcessState::Running)),
            child: Arc::new(Mutex::new(None)), // No child process
            output_tx,
            shutdown_tx,
        };
        
        // PID should be None when there's no child
        assert!(process.pid().await.is_none());
    }
}

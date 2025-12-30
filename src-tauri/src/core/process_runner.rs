//! Process runner for external binaries (winws, sing-box)
//!
//! Handles process lifecycle: spawn, capture output, graceful shutdown.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex, RwLock};
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
        }
    }

    /// Spawn a new managed process
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

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let tx = output_tx.clone();
            let state_clone = state.clone();
            let id_clone = id.to_string();

            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

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
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let tx = output_tx.clone();
            let state_clone = state.clone();
            let id_clone = id.to_string();

            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

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
        }

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
    pub async fn stop(&self, id: &str) -> Result<()> {
        info!(id = %id, "Stopping process");

        let process = {
            let processes = self.processes.read().await;
            processes.get(id).cloned()
        };

        let process = process.ok_or_else(|| {
            IsolateError::Process(format!("Process '{}' not found", id))
        })?;

        self.stop_process(&process).await
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

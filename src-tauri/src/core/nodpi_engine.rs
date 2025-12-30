//! NoDPI Engine module for Isolate
//!
//! Provides functionality for managing DPI bypass engines (Zapret/winws, Flowseal, etc.)
//! Handles engine detection, process lifecycle, and configuration management.
//!
//! CRITICAL: Only ONE winws/WinDivert process can run at a time to avoid BSOD!

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::process::Child;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::{get_binaries_dir, get_hostlists_dir};
use crate::core::process_runner::{ProcessConfig, ProcessRunner};

/// Global flag to track if a WinDivert-based engine is running
/// CRITICAL: Only one WinDivert process can run at a time!
static WINDIVERT_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Default graceful shutdown timeout in milliseconds
const SHUTDOWN_TIMEOUT_MS: u64 = 3000;

/// NoDPI engine type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoDpiEngine {
    /// Zapret/winws engine (uses WinDivert)
    Zapret,
    /// Flowseal engine (uses WinDivert)
    Flowseal,
    /// Custom engine with specified binary name
    Custom(String),
}

impl NoDpiEngine {
    /// Get the binary filename for this engine
    pub fn binary_name(&self) -> &str {
        match self {
            NoDpiEngine::Zapret => "winws.exe",
            NoDpiEngine::Flowseal => "flowseal.exe",
            NoDpiEngine::Custom(name) => name,
        }
    }

    /// Check if this engine uses WinDivert driver
    pub fn uses_windivert(&self) -> bool {
        match self {
            NoDpiEngine::Zapret => true,
            NoDpiEngine::Flowseal => true,
            NoDpiEngine::Custom(_) => true, // Assume custom engines use WinDivert by default
        }
    }
}

impl std::fmt::Display for NoDpiEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoDpiEngine::Zapret => write!(f, "Zapret"),
            NoDpiEngine::Flowseal => write!(f, "Flowseal"),
            NoDpiEngine::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// NoDPI configuration for a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoDpiConfig {
    /// Unique identifier for this configuration
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Engine type to use
    pub engine: NoDpiEngine,
    /// Command-line parameters for the engine
    pub params: Vec<String>,
    /// Optional hostlist name (without path, e.g., "youtube.txt")
    pub hostlist: Option<String>,
}

impl NoDpiConfig {
    /// Create a new NoDPI configuration
    pub fn new(id: impl Into<String>, name: impl Into<String>, engine: NoDpiEngine) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            engine,
            params: Vec::new(),
            hostlist: None,
        }
    }

    /// Add parameters to the configuration
    pub fn with_params(mut self, params: Vec<String>) -> Self {
        self.params = params;
        self
    }

    /// Set hostlist for the configuration
    pub fn with_hostlist(mut self, hostlist: impl Into<String>) -> Self {
        self.hostlist = Some(hostlist.into());
        self
    }
}

/// Information about an available NoDPI engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    /// Engine type
    pub engine: NoDpiEngine,
    /// Full path to the binary
    pub binary_path: PathBuf,
    /// Whether the engine is available (binary exists)
    pub available: bool,
}

/// Detect available NoDPI engines in the binaries directory
///
/// Checks for winws.exe, flowseal.exe, and other known engines.
///
/// # Returns
/// * `Vec<NoDpiEngine>` - List of available engines
pub async fn detect_available_engines() -> Vec<NoDpiEngine> {
    let binaries_dir = get_binaries_dir();
    let mut available = Vec::new();

    // Check for Zapret/winws
    let winws_path = binaries_dir.join("winws.exe");
    if winws_path.exists() {
        debug!("Found Zapret engine: {}", winws_path.display());
        available.push(NoDpiEngine::Zapret);
    }

    // Check for Flowseal
    let flowseal_path = binaries_dir.join("flowseal.exe");
    if flowseal_path.exists() {
        debug!("Found Flowseal engine: {}", flowseal_path.display());
        available.push(NoDpiEngine::Flowseal);
    }

    info!("Detected {} NoDPI engines: {:?}", available.len(), available);
    available
}

/// Get detailed information about all known engines
///
/// # Returns
/// * `Vec<EngineInfo>` - Information about each engine including availability
pub async fn get_engines_info() -> Vec<EngineInfo> {
    let binaries_dir = get_binaries_dir();
    let engines = [NoDpiEngine::Zapret, NoDpiEngine::Flowseal];

    engines
        .into_iter()
        .map(|engine| {
            let binary_path = binaries_dir.join(engine.binary_name());
            let available = binary_path.exists();
            EngineInfo {
                engine,
                binary_path,
                available,
            }
        })
        .collect()
}

/// Check if a specific engine is available
pub async fn is_engine_available(engine: &NoDpiEngine) -> bool {
    let binary_path = get_binaries_dir().join(engine.binary_name());
    binary_path.exists()
}

/// Build command-line arguments for winws from configuration
///
/// Converts NoDpiConfig into winws-compatible command line arguments.
fn build_winws_args(config: &NoDpiConfig) -> Vec<String> {
    let mut args = Vec::new();

    // Add user-specified parameters
    for param in &config.params {
        args.push(param.clone());
    }

    // Add hostlist if specified
    if let Some(ref hostlist) = config.hostlist {
        let hostlist_path = get_hostlists_dir().join(hostlist);
        if hostlist_path.exists() {
            // Check if --hostlist is already in params
            if !config.params.iter().any(|p| p.contains("--hostlist")) {
                args.push("--hostlist".to_string());
                args.push(hostlist_path.display().to_string());
            }
        } else {
            warn!("Hostlist not found: {}", hostlist_path.display());
        }
    }

    debug!("Built winws args: {:?}", args);
    args
}

/// Build command-line arguments for flowseal from configuration
fn build_flowseal_args(config: &NoDpiConfig) -> Vec<String> {
    // Flowseal uses similar argument format to winws
    build_winws_args(config)
}

/// Build command-line arguments based on engine type
fn build_engine_args(config: &NoDpiConfig) -> Vec<String> {
    match config.engine {
        NoDpiEngine::Zapret => build_winws_args(config),
        NoDpiEngine::Flowseal => build_flowseal_args(config),
        NoDpiEngine::Custom(_) => config.params.clone(),
    }
}

/// Handle for a running NoDPI process
pub struct NoDpiHandle {
    /// Configuration used to start the process
    pub config: NoDpiConfig,
    /// Process runner managing the process
    runner: ProcessRunner,
    /// Process ID in the runner
    process_id: String,
}

impl NoDpiHandle {
    /// Check if the process is still running
    pub async fn is_running(&self) -> bool {
        self.runner.is_running(&self.process_id).await
    }

    /// Get the system PID of the process
    pub async fn pid(&self) -> Option<u32> {
        if let Some(process) = self.runner.get(&self.process_id).await {
            process.pid().await
        } else {
            None
        }
    }

    /// Stop the NoDPI process gracefully
    pub async fn stop(&mut self) -> Result<()> {
        stop_nodpi_internal(&self.runner, &self.process_id, &self.config.engine).await
    }
}

/// Start a NoDPI strategy with the given configuration
///
/// CRITICAL: Only ONE winws/WinDivert process can run at a time!
/// Attempting to start a second WinDivert-based engine will return an error.
///
/// # Arguments
/// * `config` - NoDPI configuration to use
///
/// # Returns
/// * `Ok(NoDpiHandle)` - Handle to the running process
/// * `Err(IsolateError)` - Failed to start the engine
pub async fn start_nodpi(config: &NoDpiConfig) -> Result<NoDpiHandle> {
    info!(
        "Starting NoDPI engine: {} ({})",
        config.name, config.engine
    );

    // Check WinDivert exclusivity
    if config.engine.uses_windivert() {
        if WINDIVERT_ACTIVE.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(IsolateError::Process(
                "Another WinDivert-based engine is already running. Only one can run at a time to avoid BSOD!".to_string()
            ));
        }
    }

    // Verify engine is available
    if !is_engine_available(&config.engine).await {
        if config.engine.uses_windivert() {
            WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);
        }
        return Err(IsolateError::Process(format!(
            "Engine binary not found: {}",
            config.engine.binary_name()
        )));
    }

    let binary_path = get_binaries_dir().join(config.engine.binary_name());
    let args = build_engine_args(config);

    debug!(
        "Starting {} with args: {:?}",
        binary_path.display(),
        args
    );

    let process_config = ProcessConfig::new(binary_path, args)
        .with_admin(true) // WinDivert requires admin
        .with_working_dir(get_binaries_dir());

    let runner = ProcessRunner::new();
    let process_id = format!("nodpi-{}", config.id);

    match runner.spawn(&process_id, process_config).await {
        Ok(_) => {
            info!(
                "NoDPI engine started successfully: {} (id: {})",
                config.name, process_id
            );

            Ok(NoDpiHandle {
                config: config.clone(),
                runner,
                process_id,
            })
        }
        Err(e) => {
            // Release WinDivert lock on failure
            if config.engine.uses_windivert() {
                WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);
            }
            error!("Failed to start NoDPI engine: {}", e);
            Err(e)
        }
    }
}

/// Internal function to stop a NoDPI process
async fn stop_nodpi_internal(
    runner: &ProcessRunner,
    process_id: &str,
    engine: &NoDpiEngine,
) -> Result<()> {
    info!("Stopping NoDPI process: {}", process_id);

    let result = runner.stop(process_id).await;

    // Release WinDivert lock
    if engine.uses_windivert() {
        WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);
        debug!("Released WinDivert lock");
    }

    result
}

/// Stop a NoDPI process using its handle
///
/// Performs graceful shutdown with timeout, then force kills if necessary.
///
/// # Arguments
/// * `handle` - Mutable reference to the NoDPI handle
///
/// # Returns
/// * `Ok(())` - Process stopped successfully
/// * `Err(IsolateError)` - Failed to stop the process
pub async fn stop_nodpi(handle: &mut NoDpiHandle) -> Result<()> {
    handle.stop().await
}

/// Stop a NoDPI process by killing the child directly
///
/// Alternative stop method when you have direct access to the Child process.
///
/// # Arguments
/// * `child` - Mutable reference to the child process
///
/// # Returns
/// * `Ok(())` - Process stopped successfully
/// * `Err(IsolateError)` - Failed to stop the process
pub async fn stop_nodpi_child(child: &mut Child) -> Result<()> {
    info!("Stopping NoDPI child process");

    // Try graceful termination first
    #[cfg(windows)]
    {
        if let Some(pid) = child.id() {
            debug!("Sending terminate signal to PID: {}", pid);
            let _ = tokio::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output()
                .await;
        }
    }

    // Wait for graceful shutdown with timeout
    match timeout(Duration::from_millis(SHUTDOWN_TIMEOUT_MS), child.wait()).await {
        Ok(Ok(status)) => {
            info!("NoDPI process terminated gracefully: {:?}", status);
            // Release WinDivert lock
            WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("Error waiting for process: {}", e);
            // Force kill
            force_kill_child(child).await
        }
        Err(_) => {
            warn!("Graceful shutdown timeout, force killing");
            force_kill_child(child).await
        }
    }
}

/// Force kill a child process
async fn force_kill_child(child: &mut Child) -> Result<()> {
    if let Err(e) = child.kill().await {
        error!("Failed to kill process: {}", e);
        return Err(IsolateError::Process(format!("Failed to kill process: {}", e)));
    }

    // Wait for kill to complete
    let _ = child.wait().await;
    info!("NoDPI process force killed");

    // Release WinDivert lock
    WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);

    Ok(())
}

/// Load hostlist from the hostlists directory
///
/// # Arguments
/// * `name` - Hostlist filename (e.g., "youtube.txt")
///
/// # Returns
/// * `Ok(Vec<String>)` - List of domains from the hostlist
/// * `Err(IsolateError)` - Failed to load hostlist
pub async fn load_hostlist(name: &str) -> Result<Vec<String>> {
    let hostlist_path = get_hostlists_dir().join(name);

    debug!("Loading hostlist: {}", hostlist_path.display());

    if !hostlist_path.exists() {
        return Err(IsolateError::Config(format!(
            "Hostlist not found: {}",
            name
        )));
    }

    let content = fs::read_to_string(&hostlist_path).await?;

    let domains: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();

    info!("Loaded {} domains from hostlist: {}", domains.len(), name);
    Ok(domains)
}

/// List available hostlists in the hostlists directory
///
/// # Returns
/// * `Ok(Vec<String>)` - List of hostlist filenames
/// * `Err(IsolateError)` - Failed to read directory
pub async fn list_hostlists() -> Result<Vec<String>> {
    let hostlists_dir = get_hostlists_dir();

    if !hostlists_dir.exists() {
        return Ok(Vec::new());
    }

    let mut hostlists = Vec::new();
    let mut entries = fs::read_dir(&hostlists_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".txt") {
                    hostlists.push(name.to_string());
                }
            }
        }
    }

    debug!("Found {} hostlists", hostlists.len());
    Ok(hostlists)
}

/// Check if WinDivert driver is loaded
///
/// # Returns
/// * `true` - WinDivert driver is available
/// * `false` - WinDivert driver is not loaded
pub async fn is_windivert_available() -> bool {
    let binaries_dir = get_binaries_dir();

    // Check for WinDivert files
    let sys_file = binaries_dir.join("WinDivert64.sys");
    let dll_file = binaries_dir.join("WinDivert.dll");

    sys_file.exists() && dll_file.exists()
}

/// Check if a WinDivert-based engine is currently running
pub fn is_windivert_active() -> bool {
    WINDIVERT_ACTIVE.load(Ordering::SeqCst)
}

/// Reset WinDivert active flag (use with caution, only for recovery)
///
/// This should only be used when you're certain no WinDivert process is running
/// but the flag is stuck in active state.
pub fn reset_windivert_flag() {
    warn!("Resetting WinDivert active flag - use with caution!");
    WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_binary_names() {
        assert_eq!(NoDpiEngine::Zapret.binary_name(), "winws.exe");
        assert_eq!(NoDpiEngine::Flowseal.binary_name(), "flowseal.exe");
        assert_eq!(
            NoDpiEngine::Custom("custom.exe".to_string()).binary_name(),
            "custom.exe"
        );
    }

    #[test]
    fn test_engine_uses_windivert() {
        assert!(NoDpiEngine::Zapret.uses_windivert());
        assert!(NoDpiEngine::Flowseal.uses_windivert());
        assert!(NoDpiEngine::Custom("test.exe".to_string()).uses_windivert());
    }

    #[test]
    fn test_nodpi_config_builder() {
        let config = NoDpiConfig::new("test-1", "Test Strategy", NoDpiEngine::Zapret)
            .with_params(vec!["--fake".to_string(), "--ttl=5".to_string()])
            .with_hostlist("youtube.txt");

        assert_eq!(config.id, "test-1");
        assert_eq!(config.name, "Test Strategy");
        assert_eq!(config.engine, NoDpiEngine::Zapret);
        assert_eq!(config.params.len(), 2);
        assert_eq!(config.hostlist, Some("youtube.txt".to_string()));
    }

    #[test]
    fn test_build_winws_args() {
        let config = NoDpiConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            engine: NoDpiEngine::Zapret,
            params: vec![
                "--fake".to_string(),
                "--ttl=5".to_string(),
            ],
            hostlist: None,
        };

        let args = build_winws_args(&config);
        assert_eq!(args, vec!["--fake", "--ttl=5"]);
    }

    #[test]
    fn test_engine_display() {
        assert_eq!(format!("{}", NoDpiEngine::Zapret), "Zapret");
        assert_eq!(format!("{}", NoDpiEngine::Flowseal), "Flowseal");
        assert_eq!(
            format!("{}", NoDpiEngine::Custom("test.exe".to_string())),
            "Custom(test.exe)"
        );
    }

    #[tokio::test]
    async fn test_detect_engines_returns_vec() {
        // This test just verifies the function runs without panic
        let engines = detect_available_engines().await;
        // Result depends on whether binaries exist in test environment
        assert!(engines.len() <= 2);
    }

    #[test]
    fn test_windivert_flag_operations() {
        // Reset to known state
        WINDIVERT_ACTIVE.store(false, Ordering::SeqCst);
        assert!(!is_windivert_active());

        WINDIVERT_ACTIVE.store(true, Ordering::SeqCst);
        assert!(is_windivert_active());

        reset_windivert_flag();
        assert!(!is_windivert_active());
    }
}

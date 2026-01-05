//! TUN (VPN) mode manager for Isolate
//!
//! Uses sing-box TUN inbound for full traffic capture.
//! TUN mode requires administrator privileges.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::{get_app_data_dir, get_singbox_path};
use crate::core::quic_blocker::is_admin;

// ============================================================================
// Types
// ============================================================================

/// TUN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunConfig {
    /// TUN interface name
    pub interface_name: String,
    /// MTU size (default: 9000 for better performance)
    pub mtu: u32,
    /// IPv4 address for TUN interface
    pub address_v4: String,
    /// IPv6 address for TUN interface (optional)
    pub address_v6: Option<String>,
    /// Enable strict routing (captures all traffic)
    pub strict_route: bool,
    /// Auto route (automatically configure system routes)
    pub auto_route: bool,
    /// Stack implementation: "system", "gvisor", "mixed"
    pub stack: String,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            interface_name: "isolate-tun".to_string(),
            mtu: 9000,
            address_v4: "172.19.0.1/24".to_string(),
            address_v6: None,
            strict_route: false,
            auto_route: true,
            stack: "system".to_string(),
        }
    }
}

/// TUN status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TunStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

/// TUN instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunInstance {
    pub status: TunStatus,
    pub socks_port: u16,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub config: TunConfig,
}

// ============================================================================
// TUN Manager
// ============================================================================

/// Global TUN manager instance
static TUN_MANAGER: Lazy<Arc<TunManager>> = Lazy::new(|| Arc::new(TunManager::new()));

/// Get the global TUN manager
pub fn get_tun_manager() -> Arc<TunManager> {
    TUN_MANAGER.clone()
}

/// TUN manager state
pub struct TunManager {
    config: RwLock<TunConfig>,
    process: RwLock<Option<Child>>,
    status: RwLock<TunStatus>,
    socks_port: RwLock<u16>,
    pid: RwLock<Option<u32>>,
    started_at: RwLock<Option<u64>>,
}

impl TunManager {
    /// Create a new TUN manager
    fn new() -> Self {
        Self {
            config: RwLock::new(TunConfig::default()),
            process: RwLock::new(None),
            status: RwLock::new(TunStatus::Stopped),
            socks_port: RwLock::new(1080),
            pid: RwLock::new(None),
            started_at: RwLock::new(None),
        }
    }

    /// Start TUN mode with sing-box
    ///
    /// # Arguments
    /// * `socks_port` - SOCKS5 proxy port to route traffic through
    ///
    /// # Errors
    /// Returns error if:
    /// - TUN is already running
    /// - Not running with admin privileges
    /// - Failed to generate config or start sing-box
    pub async fn start(&self, socks_port: u16) -> Result<TunInstance> {
        // Check if already running
        if *self.status.read() == TunStatus::Running {
            warn!("TUN already running");
            return Ok(self.get_instance());
        }

        // Check admin privileges - TUN requires admin
        if !is_admin() {
            error!("TUN mode requires administrator privileges");
            return Err(IsolateError::RequiresAdmin);
        }

        info!(socks_port, "Starting TUN mode");
        *self.status.write() = TunStatus::Starting;
        *self.socks_port.write() = socks_port;

        // Generate sing-box config with TUN inbound
        let config_content = self.generate_config(socks_port)?;
        let config_path = self.get_config_path();

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| IsolateError::Io(e.to_string()))?;
        }

        // Write config file
        tokio::fs::write(&config_path, &config_content)
            .await
            .map_err(|e| IsolateError::Io(e.to_string()))?;

        debug!(path = %config_path.display(), "TUN config written");

        // Start sing-box with TUN config
        let singbox_path = get_singbox_path();

        if !singbox_path.exists() {
            error!(path = %singbox_path.display(), "sing-box binary not found");
            *self.status.write() = TunStatus::Failed;
            return Err(IsolateError::Process("sing-box binary not found".into()));
        }

        let child = Command::new(&singbox_path)
            .args(["run", "-c", config_path.to_str().unwrap()])
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                error!(error = %e, "Failed to start sing-box for TUN");
                *self.status.write() = TunStatus::Failed;
                IsolateError::Process(format!("Failed to start sing-box: {}", e))
            })?;

        let pid = child.id();
        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        *self.process.write() = Some(child);
        *self.pid.write() = pid;
        *self.started_at.write() = Some(started_at);
        *self.status.write() = TunStatus::Running;

        info!(pid = ?pid, socks_port, "TUN mode started");

        // Wait a bit for TUN interface to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(self.get_instance())
    }

    /// Stop TUN mode
    pub async fn stop(&self) -> Result<()> {
        let current_status = *self.status.read();
        
        if current_status == TunStatus::Stopped {
            debug!("TUN already stopped");
            return Ok(());
        }

        info!("Stopping TUN mode");
        *self.status.write() = TunStatus::Stopping;

        // Take ownership of the child process
        let child = self.process.write().take();

        if let Some(mut child) = child {
            // Try graceful shutdown first
            #[cfg(windows)]
            {
                // On Windows, try to terminate gracefully
                if let Some(pid) = child.id() {
                    let _ = Command::new("taskkill")
                        .args(["/PID", &pid.to_string()])
                        .output()
                        .await;
                }
            }

            // Wait a bit for graceful shutdown
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Force kill if still running
            if let Err(e) = child.kill().await {
                warn!(error = %e, "Failed to kill TUN process (may have already exited)");
            }

            // Wait for process to fully exit
            let _ = child.wait().await;
        }

        // Clean up config file
        let config_path = self.get_config_path();
        if config_path.exists() {
            if let Err(e) = tokio::fs::remove_file(&config_path).await {
                warn!(error = %e, "Failed to remove TUN config file");
            }
        }

        // Reset state
        *self.pid.write() = None;
        *self.started_at.write() = None;
        *self.status.write() = TunStatus::Stopped;

        info!("TUN mode stopped");
        Ok(())
    }

    /// Check if TUN is running
    pub fn is_running(&self) -> bool {
        *self.status.read() == TunStatus::Running
    }

    /// Get current TUN status
    pub fn get_status(&self) -> TunStatus {
        *self.status.read()
    }

    /// Get current TUN instance information
    pub fn get_instance(&self) -> TunInstance {
        TunInstance {
            status: *self.status.read(),
            socks_port: *self.socks_port.read(),
            pid: *self.pid.read(),
            started_at: *self.started_at.read(),
            config: self.config.read().clone(),
        }
    }

    /// Update TUN configuration
    ///
    /// Note: Changes take effect on next start
    pub fn set_config(&self, config: TunConfig) {
        info!(interface = %config.interface_name, mtu = config.mtu, "Updating TUN config");
        *self.config.write() = config;
    }

    /// Get current TUN configuration
    pub fn get_config(&self) -> TunConfig {
        self.config.read().clone()
    }

    /// Generate sing-box config with TUN inbound
    fn generate_config(&self, socks_port: u16) -> Result<String> {
        let config = self.config.read();

        // Build address array
        let mut addresses = vec![serde_json::json!(config.address_v4)];
        if let Some(ref v6) = config.address_v6 {
            addresses.push(serde_json::json!(v6));
        }

        let json = serde_json::json!({
            "log": {
                "level": "info",
                "timestamp": true
            },
            "inbounds": [
                {
                    "type": "tun",
                    "tag": "tun-in",
                    "interface_name": config.interface_name,
                    "address": addresses,
                    "mtu": config.mtu,
                    "auto_route": config.auto_route,
                    "strict_route": config.strict_route,
                    "stack": config.stack
                }
            ],
            "outbounds": [
                {
                    "type": "socks",
                    "tag": "proxy",
                    "server": "127.0.0.1",
                    "server_port": socks_port
                },
                {
                    "type": "direct",
                    "tag": "direct"
                },
                {
                    "type": "block",
                    "tag": "block"
                }
            ],
            "route": {
                "auto_detect_interface": true,
                "final": "proxy",
                "rules": [
                    {
                        "protocol": "dns",
                        "outbound": "direct"
                    },
                    {
                        "ip_is_private": true,
                        "outbound": "direct"
                    },
                    {
                        // Exclude sing-box itself from routing
                        "process_name": ["sing-box.exe", "sing-box"],
                        "outbound": "direct"
                    }
                ]
            }
        });

        serde_json::to_string_pretty(&json)
            .map_err(|e| IsolateError::Config(format!("Failed to serialize TUN config: {}", e)))
    }

    /// Get path to TUN config file
    fn get_config_path(&self) -> PathBuf {
        get_app_data_dir().join("tun_config.json")
    }

    /// Restart TUN with current configuration
    pub async fn restart(&self, socks_port: Option<u16>) -> Result<TunInstance> {
        let port = socks_port.unwrap_or(*self.socks_port.read());
        
        self.stop().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        self.start(port).await
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

// NOTE: get_singbox_path moved to paths.rs - use crate::core::paths::get_singbox_path

/// Check if TUN mode is available
///
/// Returns true if:
/// - sing-box binary exists
/// - Running with admin privileges
pub fn is_tun_available() -> bool {
    crate::core::paths::get_singbox_path().exists() && is_admin()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TunConfig::default();
        assert_eq!(config.interface_name, "isolate-tun");
        assert_eq!(config.mtu, 9000);
        assert_eq!(config.address_v4, "172.19.0.1/24");
        assert!(config.auto_route);
        assert!(!config.strict_route);
    }

    #[test]
    fn test_generate_config() {
        let manager = TunManager::new();
        let config = manager.generate_config(1080).unwrap();
        
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&config).unwrap();
        
        // Check structure
        assert!(parsed["inbounds"].is_array());
        assert!(parsed["outbounds"].is_array());
        assert!(parsed["route"].is_object());
        
        // Check TUN inbound
        let tun_inbound = &parsed["inbounds"][0];
        assert_eq!(tun_inbound["type"], "tun");
        assert_eq!(tun_inbound["interface_name"], "isolate-tun");
        
        // Check SOCKS outbound
        let socks_outbound = &parsed["outbounds"][0];
        assert_eq!(socks_outbound["type"], "socks");
        assert_eq!(socks_outbound["server_port"], 1080);
    }

    #[test]
    fn test_config_with_ipv6() {
        let manager = TunManager::new();
        
        let mut config = TunConfig::default();
        config.address_v6 = Some("fdfe:dcba:9876::1/96".to_string());
        manager.set_config(config);
        
        let json_str = manager.generate_config(1080).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        
        let addresses = &parsed["inbounds"][0]["address"];
        assert!(addresses.is_array());
        assert_eq!(addresses.as_array().unwrap().len(), 2);
    }
}

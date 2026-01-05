//! Sing-box Process Manager for Isolate
//!
//! Singleton manager for sing-box processes:
//! - Track running instances by config ID
//! - Prevent duplicate instances
//! - Health monitoring
//! - Cleanup on app exit

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{AppRoute, DomainRoute, ProxyConfig};
use crate::core::paths::get_singbox_path;
use crate::core::singbox_config::{
    generate_dns_config_fakeip, generate_dns_config_with_mode, generate_outbound,
    generate_route_rules, DnsMode,
};
use crate::core::vless_engine::{self, VlessConfig};

// ============================================================================
// Types
// ============================================================================

/// Status of a sing-box instance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SingboxStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    HealthCheckFailed,
}

/// Information about a running sing-box instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInstance {
    pub config_id: String,
    pub config_name: String,
    pub socks_port: u16,
    pub status: SingboxStatus,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub last_health_check: Option<u64>,
    pub health_check_failures: u32,
}

/// Internal state for a running instance
struct RunningInstance {
    info: SingboxInstance,
    child: Child,
}

// ============================================================================
// Singleton Manager
// ============================================================================

/// Global sing-box manager instance
static SINGBOX_MANAGER: Lazy<Arc<SingboxManager>> = Lazy::new(|| {
    Arc::new(SingboxManager::new())
});

/// Get the global sing-box manager
pub fn get_manager() -> Arc<SingboxManager> {
    SINGBOX_MANAGER.clone()
}

/// Sing-box process manager
pub struct SingboxManager {
    instances: RwLock<HashMap<String, RunningInstance>>,
    /// Port allocation tracker (to avoid conflicts)
    used_ports: RwLock<HashMap<u16, String>>,
}

impl SingboxManager {
    /// Create a new manager
    fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            used_ports: RwLock::new(HashMap::new()),
        }
    }

    /// Start a VLESS proxy with the given config
    ///
    /// Returns the SOCKS port for the proxy.
    /// If the config is already running, returns the existing port.
    pub async fn start(&self, config: &VlessConfig, socks_port: u16) -> Result<SingboxInstance> {
        let config_id = &config.id;

        // Check if already running
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(config_id) {
                if instance.info.status == SingboxStatus::Running {
                    info!(
                        config_id = %config_id,
                        socks_port = instance.info.socks_port,
                        "Sing-box instance already running"
                    );
                    return Ok(instance.info.clone());
                }
            }
        }

        // Check port availability
        {
            let used_ports = self.used_ports.read().await;
            if let Some(existing_id) = used_ports.get(&socks_port) {
                if existing_id != config_id {
                    return Err(IsolateError::Process(format!(
                        "Port {} is already in use by config '{}'",
                        socks_port, existing_id
                    )));
                }
            }
        }

        info!(
            config_id = %config_id,
            socks_port = socks_port,
            server = %config.server,
            "Starting sing-box instance"
        );

        // Create initial instance info
        let mut instance_info = SingboxInstance {
            config_id: config_id.clone(),
            config_name: config.name.clone(),
            socks_port,
            status: SingboxStatus::Starting,
            pid: None,
            started_at: None,
            last_health_check: None,
            health_check_failures: 0,
        };

        // Start sing-box process
        let child = vless_engine::start_vless(config, socks_port).await?;

        // Update instance info
        instance_info.pid = child.id();
        instance_info.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        instance_info.status = SingboxStatus::Running;

        // Store instance
        {
            let mut instances = self.instances.write().await;
            instances.insert(config_id.clone(), RunningInstance {
                info: instance_info.clone(),
                child,
            });
        }

        // Reserve port
        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.insert(socks_port, config_id.clone());
        }

        // Wait a bit for sing-box to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Perform initial health check
        if let Err(e) = self.health_check(config_id).await {
            warn!(
                config_id = %config_id,
                error = %e,
                "Initial health check failed, but process may still be starting"
            );
        }

        info!(
            config_id = %config_id,
            socks_port = socks_port,
            pid = ?instance_info.pid,
            "Sing-box instance started"
        );

        Ok(instance_info)
    }

    /// Start sing-box with routing rules
    ///
    /// This method generates a sing-box configuration with:
    /// - DNS configuration (fake-ip for TUN mode, direct for SOCKS mode)
    /// - Domain-based routing rules
    /// - Application-based routing rules
    ///
    /// # Arguments
    /// * `proxy` - Proxy configuration to use as outbound
    /// * `domain_routes` - Domain-based routing rules
    /// * `app_routes` - Application-based routing rules
    /// * `socks_port` - SOCKS port for the inbound
    ///
    /// # Returns
    /// SingboxInstance with running process information
    pub async fn start_with_routing(
        &self,
        proxy: &ProxyConfig,
        domain_routes: &[DomainRoute],
        app_routes: &[AppRoute],
        socks_port: u16,
    ) -> Result<SingboxInstance> {
        let config_id = &proxy.id;

        // Check if already running
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(config_id) {
                if instance.info.status == SingboxStatus::Running {
                    info!(
                        config_id = %config_id,
                        socks_port = instance.info.socks_port,
                        "Sing-box instance already running"
                    );
                    return Ok(instance.info.clone());
                }
            }
        }

        // Check port availability
        {
            let used_ports = self.used_ports.read().await;
            if let Some(existing_id) = used_ports.get(&socks_port) {
                if existing_id != config_id {
                    return Err(IsolateError::Process(format!(
                        "Port {} is already in use by config '{}'",
                        socks_port, existing_id
                    )));
                }
            }
        }

        info!(
            config_id = %config_id,
            socks_port = socks_port,
            server = %proxy.server,
            domain_routes = domain_routes.len(),
            app_routes = app_routes.len(),
            "Starting sing-box with routing rules"
        );

        // Generate sing-box configuration with routing
        let singbox_config = self.generate_routing_config(
            proxy,
            domain_routes,
            app_routes,
            socks_port,
        )?;

        // Write config to temp file
        let config_path = get_temp_config_path(config_id);
        let config_json = serde_json::to_string_pretty(&singbox_config)?;
        tokio::fs::write(&config_path, &config_json).await?;

        debug!(
            config_id = %config_id,
            config_path = %config_path.display(),
            "Wrote sing-box config"
        );

        // Create initial instance info
        let mut instance_info = SingboxInstance {
            config_id: config_id.clone(),
            config_name: proxy.name.clone(),
            socks_port,
            status: SingboxStatus::Starting,
            pid: None,
            started_at: None,
            last_health_check: None,
            health_check_failures: 0,
        };

        // Start sing-box process
        let singbox_path = get_singbox_path();
        if !singbox_path.exists() {
            return Err(IsolateError::Process(format!(
                "sing-box binary not found at: {}",
                singbox_path.display()
            )));
        }

        let child = Command::new(&singbox_path)
            .args(["run", "-c", config_path.to_str().unwrap()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| IsolateError::Process(format!("Failed to start sing-box: {}", e)))?;

        // Update instance info
        instance_info.pid = child.id();
        instance_info.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        instance_info.status = SingboxStatus::Running;

        // Store instance
        {
            let mut instances = self.instances.write().await;
            instances.insert(config_id.clone(), RunningInstance {
                info: instance_info.clone(),
                child,
            });
        }

        // Reserve port
        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.insert(socks_port, config_id.clone());
        }

        // Wait a bit for sing-box to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Perform initial health check
        if let Err(e) = self.health_check(config_id).await {
            warn!(
                config_id = %config_id,
                error = %e,
                "Initial health check failed, but process may still be starting"
            );
        }

        info!(
            config_id = %config_id,
            socks_port = socks_port,
            pid = ?instance_info.pid,
            "Sing-box instance with routing started"
        );

        Ok(instance_info)
    }

    /// Generate sing-box configuration with routing rules
    fn generate_routing_config(
        &self,
        proxy: &ProxyConfig,
        domain_routes: &[DomainRoute],
        app_routes: &[AppRoute],
        socks_port: u16,
    ) -> Result<serde_json::Value> {
        // Generate outbound for the proxy
        let proxy_outbound = generate_outbound(proxy)
            .map_err(|e| IsolateError::Config(format!("Failed to generate outbound: {}", e)))?;

        // Build outbounds array
        let outbounds = vec![
            proxy_outbound,
            json!({
                "type": "direct",
                "tag": "direct"
            }),
            json!({
                "type": "block",
                "tag": "block"
            }),
            json!({
                "type": "dns",
                "tag": "dns-out"
            }),
        ];

        // Generate DNS configuration (direct mode for SOCKS proxy)
        let dns = generate_dns_config_with_mode(&[proxy.clone()], DnsMode::Direct);

        // Generate routing rules
        let route = generate_route_rules(domain_routes, app_routes, &proxy.id);

        // Build complete config
        let config = json!({
            "log": {
                "level": "info",
                "timestamp": true
            },
            "dns": dns,
            "inbounds": [
                {
                    "type": "socks",
                    "tag": "socks-in",
                    "listen": "127.0.0.1",
                    "listen_port": socks_port,
                    "sniff": true,
                    "sniff_override_destination": true,
                    "sniff_timeout": "300ms",
                    "domain_strategy": "prefer_ipv4"
                },
                {
                    "type": "http",
                    "tag": "http-in",
                    "listen": "127.0.0.1",
                    "listen_port": socks_port + 1,
                    "sniff": true,
                    "sniff_override_destination": true
                }
            ],
            "outbounds": outbounds,
            "route": route,
            "experimental": {
                "cache_file": {
                    "enabled": false
                }
            }
        });

        Ok(config)
    }

    /// Start sing-box with TUN mode and routing rules
    ///
    /// TUN mode captures all system traffic and routes it through the proxy.
    /// Uses fake-ip DNS for proper domain resolution.
    ///
    /// # Arguments
    /// * `proxy` - Proxy configuration to use as outbound
    /// * `domain_routes` - Domain-based routing rules
    /// * `app_routes` - Application-based routing rules
    /// * `tun_name` - Name for the TUN interface
    ///
    /// # Returns
    /// SingboxInstance with running process information
    pub async fn start_with_tun_routing(
        &self,
        proxy: &ProxyConfig,
        domain_routes: &[DomainRoute],
        app_routes: &[AppRoute],
        tun_name: &str,
    ) -> Result<SingboxInstance> {
        let config_id = format!("{}-tun", proxy.id);

        // Check if already running
        {
            let instances = self.instances.read().await;
            if let Some(instance) = instances.get(&config_id) {
                if instance.info.status == SingboxStatus::Running {
                    info!(
                        config_id = %config_id,
                        "Sing-box TUN instance already running"
                    );
                    return Ok(instance.info.clone());
                }
            }
        }

        info!(
            config_id = %config_id,
            server = %proxy.server,
            tun_name = %tun_name,
            domain_routes = domain_routes.len(),
            app_routes = app_routes.len(),
            "Starting sing-box with TUN mode"
        );

        // Generate sing-box configuration with TUN
        let singbox_config = self.generate_tun_routing_config(
            proxy,
            domain_routes,
            app_routes,
            tun_name,
        )?;

        // Write config to temp file
        let config_path = get_temp_config_path(&config_id);
        let config_json = serde_json::to_string_pretty(&singbox_config)?;
        tokio::fs::write(&config_path, &config_json).await?;

        debug!(
            config_id = %config_id,
            config_path = %config_path.display(),
            "Wrote sing-box TUN config"
        );

        // Create initial instance info
        let mut instance_info = SingboxInstance {
            config_id: config_id.clone(),
            config_name: format!("{} (TUN)", proxy.name),
            socks_port: 0, // TUN mode doesn't use SOCKS port
            status: SingboxStatus::Starting,
            pid: None,
            started_at: None,
            last_health_check: None,
            health_check_failures: 0,
        };

        // Start sing-box process
        let singbox_path = get_singbox_path();
        if !singbox_path.exists() {
            return Err(IsolateError::Process(format!(
                "sing-box binary not found at: {}",
                singbox_path.display()
            )));
        }

        let child = Command::new(&singbox_path)
            .args(["run", "-c", config_path.to_str().unwrap()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| IsolateError::Process(format!("Failed to start sing-box: {}", e)))?;

        // Update instance info
        instance_info.pid = child.id();
        instance_info.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        instance_info.status = SingboxStatus::Running;

        // Store instance
        {
            let mut instances = self.instances.write().await;
            instances.insert(config_id.clone(), RunningInstance {
                info: instance_info.clone(),
                child,
            });
        }

        // Wait a bit for sing-box to initialize
        tokio::time::sleep(Duration::from_millis(1000)).await;

        info!(
            config_id = %config_id,
            pid = ?instance_info.pid,
            "Sing-box TUN instance started"
        );

        Ok(instance_info)
    }

    /// Generate sing-box configuration with TUN mode and routing rules
    fn generate_tun_routing_config(
        &self,
        proxy: &ProxyConfig,
        domain_routes: &[DomainRoute],
        app_routes: &[AppRoute],
        tun_name: &str,
    ) -> Result<serde_json::Value> {
        // Generate outbound for the proxy
        let proxy_outbound = generate_outbound(proxy)
            .map_err(|e| IsolateError::Config(format!("Failed to generate outbound: {}", e)))?;

        // Build outbounds array
        let outbounds = vec![
            proxy_outbound,
            json!({
                "type": "direct",
                "tag": "direct"
            }),
            json!({
                "type": "block",
                "tag": "block"
            }),
            json!({
                "type": "dns",
                "tag": "dns-out"
            }),
        ];

        // Generate DNS configuration with fake-ip for TUN mode
        let dns = generate_dns_config_fakeip(&[proxy.clone()]);

        // Generate routing rules
        let route = generate_route_rules(domain_routes, app_routes, &proxy.id);

        // Build complete config with TUN inbound
        let config = json!({
            "log": {
                "level": "info",
                "timestamp": true
            },
            "dns": dns,
            "inbounds": [
                {
                    "type": "tun",
                    "tag": "tun-in",
                    "interface_name": tun_name,
                    "inet4_address": "172.19.0.1/30",
                    "inet6_address": "fdfe:dcba:9876::1/126",
                    "mtu": 9000,
                    "auto_route": true,
                    "strict_route": true,
                    "stack": "system",
                    "sniff": true,
                    "sniff_override_destination": true
                }
            ],
            "outbounds": outbounds,
            "route": route,
            "experimental": {
                "cache_file": {
                    "enabled": false
                }
            }
        });

        Ok(config)
    }

    /// Stop a running sing-box instance
    pub async fn stop(&self, config_id: &str) -> Result<()> {
        info!(config_id = %config_id, "Stopping sing-box instance");

        let instance = {
            let mut instances = self.instances.write().await;
            instances.remove(config_id)
        };

        let Some(mut instance) = instance else {
            return Err(IsolateError::Process(format!(
                "No running instance for config '{}'",
                config_id
            )));
        };

        // Update status
        instance.info.status = SingboxStatus::Stopping;

        // Release port
        {
            let mut used_ports = self.used_ports.write().await;
            used_ports.remove(&instance.info.socks_port);
        }

        // Stop the process
        vless_engine::stop_vless(config_id, instance.child).await?;

        info!(config_id = %config_id, "Sing-box instance stopped");
        Ok(())
    }

    /// Stop all running instances
    pub async fn stop_all(&self) -> Result<()> {
        info!("Stopping all sing-box instances");

        let config_ids: Vec<String> = {
            let instances = self.instances.read().await;
            instances.keys().cloned().collect()
        };

        for config_id in config_ids {
            if let Err(e) = self.stop(&config_id).await {
                error!(config_id = %config_id, error = %e, "Failed to stop instance");
            }
        }

        Ok(())
    }

    /// Get status of a specific instance
    pub async fn get_status(&self, config_id: &str) -> Option<SingboxInstance> {
        let instances = self.instances.read().await;
        instances.get(config_id).map(|i| i.info.clone())
    }

    /// Get all running instances
    pub async fn list_instances(&self) -> Vec<SingboxInstance> {
        let instances = self.instances.read().await;
        instances.values().map(|i| i.info.clone()).collect()
    }

    /// Check if a config is running
    pub async fn is_running(&self, config_id: &str) -> bool {
        let instances = self.instances.read().await;
        instances.get(config_id)
            .map(|i| i.info.status == SingboxStatus::Running)
            .unwrap_or(false)
    }

    /// Perform health check on a running instance
    ///
    /// Checks if the SOCKS proxy is responding.
    pub async fn health_check(&self, config_id: &str) -> Result<bool> {
        let socks_port = {
            let instances = self.instances.read().await;
            instances.get(config_id)
                .map(|i| i.info.socks_port)
                .ok_or_else(|| IsolateError::Process(format!(
                    "No running instance for config '{}'",
                    config_id
                )))?
        };

        debug!(config_id = %config_id, socks_port = socks_port, "Performing health check");

        // Try to connect to the SOCKS proxy
        let addr = format!("127.0.0.1:{}", socks_port);
        let timeout_duration = Duration::from_secs(3);

        let result = tokio::time::timeout(
            timeout_duration,
            tokio::net::TcpStream::connect(&addr)
        ).await;

        let is_healthy = match result {
            Ok(Ok(_stream)) => {
                debug!(config_id = %config_id, "Health check passed - SOCKS port is open");
                true
            }
            Ok(Err(e)) => {
                warn!(config_id = %config_id, error = %e, "Health check failed - connection error");
                false
            }
            Err(_) => {
                warn!(config_id = %config_id, "Health check failed - timeout");
                false
            }
        };

        // Update health check status
        {
            let mut instances = self.instances.write().await;
            if let Some(instance) = instances.get_mut(config_id) {
                instance.info.last_health_check = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                );

                if is_healthy {
                    instance.info.health_check_failures = 0;
                    instance.info.status = SingboxStatus::Running;
                } else {
                    instance.info.health_check_failures += 1;
                    if instance.info.health_check_failures >= 3 {
                        instance.info.status = SingboxStatus::HealthCheckFailed;
                    }
                }
            }
        }

        Ok(is_healthy)
    }

    /// Perform health check on all running instances
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let config_ids: Vec<String> = {
            let instances = self.instances.read().await;
            instances.keys().cloned().collect()
        };

        let mut results = HashMap::new();

        for config_id in config_ids {
            let is_healthy = self.health_check(&config_id).await.unwrap_or(false);
            results.insert(config_id, is_healthy);
        }

        results
    }

    /// Allocate an available SOCKS port
    ///
    /// Finds an unused port starting from the base port.
    pub async fn allocate_port(&self, base_port: u16) -> u16 {
        let used_ports = self.used_ports.read().await;
        let mut port = base_port;

        while used_ports.contains_key(&port) {
            if port == 65535 {
                port = base_port; // Wrap around (shouldn't happen in practice)
                break;
            }
            port += 1;
        }

        port
    }

    /// Get the SOCKS port for a running config
    pub async fn get_socks_port(&self, config_id: &str) -> Option<u16> {
        let instances = self.instances.read().await;
        instances.get(config_id).map(|i| i.info.socks_port)
    }

    /// Restart a running instance
    pub async fn restart(&self, config: &VlessConfig) -> Result<SingboxInstance> {
        let config_id = &config.id;

        // Get current port if running
        let current_port = self.get_socks_port(config_id).await;

        // Stop if running
        if self.is_running(config_id).await {
            self.stop(config_id).await?;
            // Wait a bit for cleanup
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Start with same port or allocate new one
        let port = current_port.unwrap_or_else(|| 1080);
        self.start(config, port).await
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

// NOTE: get_singbox_path moved to paths.rs - use crate::core::paths::get_singbox_path

/// Get path for temporary config file
fn get_temp_config_path(config_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("isolate-singbox-{}.json", config_id))
}

/// Check if sing-box binary exists
pub fn is_singbox_available() -> bool {
    crate::core::paths::get_singbox_path().exists()
}

/// Get sing-box version
pub async fn get_singbox_version() -> Result<String> {
    let singbox_path = crate::core::paths::get_singbox_path();

    if !singbox_path.exists() {
        return Err(IsolateError::Process("sing-box binary not found".into()));
    }

    let output = tokio::process::Command::new(&singbox_path)
        .arg("version")
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to get sing-box version: {}", e)))?;

    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("unknown");

    Ok(first_line.to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singbox_path() {
        let path = get_singbox_path();
        assert!(path.to_string_lossy().contains("sing-box"));
    }

    #[tokio::test]
    async fn test_port_allocation() {
        let manager = SingboxManager::new();

        // First allocation should return base port
        let port1 = manager.allocate_port(1080).await;
        assert_eq!(port1, 1080);

        // Manually reserve the port
        {
            let mut used_ports = manager.used_ports.write().await;
            used_ports.insert(1080, "test-config".to_string());
        }

        // Next allocation should skip reserved port
        let port2 = manager.allocate_port(1080).await;
        assert_eq!(port2, 1081);
    }
}

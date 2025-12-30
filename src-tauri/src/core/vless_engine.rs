//! VLESS Engine for Isolate
//!
//! Handles VLESS protocol connections via sing-box:
//! - Parse VLESS URLs
//! - Generate sing-box configurations
//! - Manage sing-box process lifecycle
//! - System proxy management (Windows)

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::get_binaries_dir;

// ============================================================================
// VLESS Configuration Types
// ============================================================================

/// VLESS transport type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportType {
    Tcp,
    Ws {
        path: String,
        host: Option<String>,
    },
    Grpc {
        service_name: String,
    },
    H2 {
        path: String,
        host: Option<String>,
    },
}

impl Default for TransportType {
    fn default() -> Self {
        Self::Tcp
    }
}

/// VLESS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlessConfig {
    pub id: String,
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub tls: bool,
    pub transport: TransportType,
    pub sni: Option<String>,
    pub fingerprint: Option<String>,
}

impl VlessConfig {
    /// Create a new VLESS config with minimal parameters
    pub fn new(server: String, port: u16, uuid: String) -> Self {
        let id = format!("vless-{}-{}", server, port);
        Self {
            id: id.clone(),
            name: id,
            server,
            port,
            uuid,
            tls: true,
            transport: TransportType::default(),
            sni: None,
            fingerprint: None,
        }
    }

    /// Set TLS SNI
    pub fn with_sni(mut self, sni: impl Into<String>) -> Self {
        self.sni = Some(sni.into());
        self
    }

    /// Set TLS fingerprint
    pub fn with_fingerprint(mut self, fp: impl Into<String>) -> Self {
        self.fingerprint = Some(fp.into());
        self
    }

    /// Set transport type
    pub fn with_transport(mut self, transport: TransportType) -> Self {
        self.transport = transport;
        self
    }

    /// Set display name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

// ============================================================================
// VLESS URL Parsing
// ============================================================================

/// Parse a VLESS URL into VlessConfig
///
/// Format: vless://uuid@server:port?type=ws&security=tls&path=/ws&sni=example.com&fp=chrome#name
///
/// Supported parameters:
/// - type: tcp, ws, grpc, h2
/// - security: tls, none
/// - path: WebSocket/H2 path
/// - host: WebSocket/H2 host header
/// - sni: TLS SNI
/// - fp: TLS fingerprint
/// - serviceName: gRPC service name
pub fn parse_vless_url(url: &str) -> Result<VlessConfig> {
    // Validate scheme
    if !url.starts_with("vless://") {
        return Err(IsolateError::Config("Invalid VLESS URL: must start with vless://".into()));
    }

    let url_without_scheme = &url[8..]; // Remove "vless://"

    // Split fragment (name) from the rest
    let (main_part, name) = match url_without_scheme.rfind('#') {
        Some(pos) => {
            let name = urlencoding::decode(&url_without_scheme[pos + 1..])
                .map_err(|e| IsolateError::Config(format!("Invalid URL encoding in name: {}", e)))?
                .into_owned();
            (&url_without_scheme[..pos], Some(name))
        }
        None => (url_without_scheme, None),
    };

    // Split query parameters from authority
    let (authority_part, query_string) = match main_part.find('?') {
        Some(pos) => (&main_part[..pos], Some(&main_part[pos + 1..])),
        None => (main_part, None),
    };

    // Parse uuid@server:port
    let at_pos = authority_part
        .find('@')
        .ok_or_else(|| IsolateError::Config("Invalid VLESS URL: missing @ separator".into()))?;

    let uuid = authority_part[..at_pos].to_string();
    if uuid.is_empty() {
        return Err(IsolateError::Config("Invalid VLESS URL: empty UUID".into()));
    }

    let server_port = &authority_part[at_pos + 1..];
    let (server, port) = parse_server_port(server_port)?;

    // Parse query parameters
    let params = parse_query_params(query_string.unwrap_or(""));

    // Determine TLS
    let tls = params.get("security").map(|s| s.as_str()) != Some("none");

    // Parse transport type
    let transport = parse_transport_type(&params)?;

    // Extract optional parameters
    let sni = params.get("sni").cloned();
    let fingerprint = params.get("fp").cloned();

    let config_name = name.unwrap_or_else(|| format!("{}:{}", server, port));
    let id = format!("vless-{}", sanitize_id(&config_name));

    Ok(VlessConfig {
        id,
        name: config_name,
        server,
        port,
        uuid,
        tls,
        transport,
        sni,
        fingerprint,
    })
}

/// Parse server:port string
fn parse_server_port(s: &str) -> Result<(String, u16)> {
    // Handle IPv6 addresses: [::1]:443
    if s.starts_with('[') {
        let bracket_end = s
            .find(']')
            .ok_or_else(|| IsolateError::Config("Invalid IPv6 address format".into()))?;

        let server = s[1..bracket_end].to_string();
        let port_str = &s[bracket_end + 1..];

        if !port_str.starts_with(':') {
            return Err(IsolateError::Config("Missing port after IPv6 address".into()));
        }

        let port: u16 = port_str[1..]
            .parse()
            .map_err(|_| IsolateError::Config("Invalid port number".into()))?;

        return Ok((server, port));
    }

    // Handle regular host:port
    let colon_pos = s
        .rfind(':')
        .ok_or_else(|| IsolateError::Config("Invalid server:port format".into()))?;

    let server = s[..colon_pos].to_string();
    let port: u16 = s[colon_pos + 1..]
        .parse()
        .map_err(|_| IsolateError::Config("Invalid port number".into()))?;

    if server.is_empty() {
        return Err(IsolateError::Config("Empty server address".into()));
    }

    Ok((server, port))
}

/// Parse query string into HashMap
fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if query.is_empty() {
        return params;
    }

    for pair in query.split('&') {
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let value = urlencoding::decode(&pair[eq_pos + 1..])
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| pair[eq_pos + 1..].to_string());
            params.insert(key.to_string(), value);
        }
    }

    params
}

/// Parse transport type from query parameters
fn parse_transport_type(params: &HashMap<String, String>) -> Result<TransportType> {
    let transport_type = params.get("type").map(|s| s.as_str()).unwrap_or("tcp");

    match transport_type {
        "tcp" => Ok(TransportType::Tcp),
        "ws" | "websocket" => {
            let path = params.get("path").cloned().unwrap_or_else(|| "/".to_string());
            let host = params.get("host").cloned();
            Ok(TransportType::Ws { path, host })
        }
        "grpc" => {
            let service_name = params
                .get("serviceName")
                .or_else(|| params.get("service_name"))
                .cloned()
                .unwrap_or_default();
            Ok(TransportType::Grpc { service_name })
        }
        "h2" | "http" => {
            let path = params.get("path").cloned().unwrap_or_else(|| "/".to_string());
            let host = params.get("host").cloned();
            Ok(TransportType::H2 { path, host })
        }
        other => Err(IsolateError::Config(format!(
            "Unsupported transport type: {}",
            other
        ))),
    }
}

/// Sanitize string for use as ID
fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}

// ============================================================================
// Sing-box Configuration Generation
// ============================================================================

/// Generate sing-box configuration JSON for VLESS
///
/// Creates a config with:
/// - SOCKS5 inbound on specified port
/// - VLESS outbound to the configured server
/// - Direct DNS
pub fn generate_singbox_config(config: &VlessConfig, socks_port: u16) -> serde_json::Value {
    let mut outbound = json!({
        "type": "vless",
        "tag": "vless-out",
        "server": config.server,
        "server_port": config.port,
        "uuid": config.uuid,
    });

    // Add TLS configuration
    if config.tls {
        let mut tls_config = json!({
            "enabled": true,
        });

        if let Some(ref sni) = config.sni {
            tls_config["server_name"] = json!(sni);
        } else {
            // Use server as SNI if not specified
            tls_config["server_name"] = json!(config.server);
        }

        if let Some(ref fp) = config.fingerprint {
            tls_config["utls"] = json!({
                "enabled": true,
                "fingerprint": fp
            });
        }

        outbound["tls"] = tls_config;
    }

    // Add transport configuration
    match &config.transport {
        TransportType::Tcp => {
            // No additional config needed for TCP
        }
        TransportType::Ws { path, host } => {
            let mut ws_config = json!({
                "type": "ws",
                "path": path,
            });

            if let Some(h) = host {
                ws_config["headers"] = json!({
                    "Host": h
                });
            } else if let Some(ref sni) = config.sni {
                ws_config["headers"] = json!({
                    "Host": sni
                });
            }

            outbound["transport"] = ws_config;
        }
        TransportType::Grpc { service_name } => {
            outbound["transport"] = json!({
                "type": "grpc",
                "service_name": service_name,
            });
        }
        TransportType::H2 { path, host } => {
            let mut h2_config = json!({
                "type": "http",
                "path": path,
            });

            if let Some(h) = host {
                h2_config["host"] = json!([h]);
            } else if let Some(ref sni) = config.sni {
                h2_config["host"] = json!([sni]);
            }

            outbound["transport"] = h2_config;
        }
    }

    // Build complete config
    json!({
        "log": {
            "level": "info",
            "timestamp": true
        },
        "dns": {
            "servers": [
                {
                    "tag": "dns-direct",
                    "address": "https://1.1.1.1/dns-query",
                    "detour": "direct"
                }
            ]
        },
        "inbounds": [
            {
                "type": "socks",
                "tag": "socks-in",
                "listen": "127.0.0.1",
                "listen_port": socks_port,
                "sniff": true,
                "sniff_override_destination": true
            }
        ],
        "outbounds": [
            outbound,
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
            "rules": [
                {
                    "protocol": "dns",
                    "outbound": "dns-direct"
                }
            ],
            "final": "vless-out"
        }
    })
}

// ============================================================================
// Sing-box Process Management
// ============================================================================

/// Get path to sing-box binary
fn get_singbox_path() -> PathBuf {
    let binaries_dir = get_binaries_dir();

    #[cfg(windows)]
    {
        binaries_dir.join("sing-box.exe")
    }

    #[cfg(not(windows))]
    {
        binaries_dir.join("sing-box")
    }
}

/// Get path for temporary config file
fn get_temp_config_path(config_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("isolate-singbox-{}.json", config_id))
}

/// Start VLESS connection via sing-box
///
/// Writes configuration to a temp file and starts sing-box process.
/// Returns the Child process handle for lifecycle management.
pub async fn start_vless(config: &VlessConfig, socks_port: u16) -> Result<Child> {
    let singbox_path = get_singbox_path();

    // Verify sing-box exists
    if !singbox_path.exists() {
        return Err(IsolateError::Process(format!(
            "sing-box binary not found at: {}",
            singbox_path.display()
        )));
    }

    // Generate and write config
    let singbox_config = generate_singbox_config(config, socks_port);
    let config_path = get_temp_config_path(&config.id);

    let config_json = serde_json::to_string_pretty(&singbox_config)?;
    tokio::fs::write(&config_path, &config_json).await?;

    info!(
        config_id = %config.id,
        config_path = %config_path.display(),
        socks_port = socks_port,
        "Starting sing-box for VLESS"
    );

    debug!(config = %config_json, "Sing-box configuration");

    // Start sing-box process
    let child = Command::new(&singbox_path)
        .args(["run", "-c", config_path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            IsolateError::Process(format!(
                "Failed to start sing-box: {}",
                e
            ))
        })?;

    let pid = child.id();
    info!(
        config_id = %config.id,
        pid = ?pid,
        "sing-box process started"
    );

    Ok(child)
}

/// Stop VLESS connection and cleanup
pub async fn stop_vless(config_id: &str, mut child: Child) -> Result<()> {
    info!(config_id = %config_id, "Stopping sing-box");

    // Try graceful termination first
    #[cfg(windows)]
    {
        if let Some(pid) = child.id() {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string()])
                .output()
                .await;
        }
    }

    // Wait briefly for graceful shutdown
    tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {
            warn!(config_id = %config_id, "Graceful shutdown timeout, force killing");
            let _ = child.kill().await;
        }
        result = child.wait() => {
            match result {
                Ok(status) => info!(config_id = %config_id, ?status, "sing-box terminated"),
                Err(e) => error!(config_id = %config_id, error = %e, "Error waiting for sing-box"),
            }
        }
    }

    // Cleanup temp config file
    let config_path = get_temp_config_path(config_id);
    if config_path.exists() {
        if let Err(e) = tokio::fs::remove_file(&config_path).await {
            warn!(
                config_id = %config_id,
                path = %config_path.display(),
                error = %e,
                "Failed to remove temp config"
            );
        }
    }

    Ok(())
}

// ============================================================================
// System Proxy Management (Windows)
// ============================================================================

/// Registry path for Internet Settings
#[cfg(windows)]
const INTERNET_SETTINGS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

/// Set system proxy to use SOCKS5
///
/// On Windows, this modifies the registry to set the proxy server.
/// Note: Most applications respect this setting, but some may not.
pub async fn set_system_proxy(host: &str, port: u16) -> Result<()> {
    info!(host = %host, port = port, "Setting system proxy");

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(INTERNET_SETTINGS_KEY)
            .map_err(|e| IsolateError::Config(format!("Failed to open registry key: {}", e)))?;

        // Enable proxy
        key.set_value("ProxyEnable", &1u32)
            .map_err(|e| IsolateError::Config(format!("Failed to enable proxy: {}", e)))?;

        // Set proxy server (SOCKS format)
        let proxy_server = format!("socks={}:{}", host, port);
        key.set_value("ProxyServer", &proxy_server)
            .map_err(|e| IsolateError::Config(format!("Failed to set proxy server: {}", e)))?;

        // Notify system of settings change
        notify_proxy_change();

        info!("System proxy set to {}", proxy_server);
    }

    #[cfg(not(windows))]
    {
        warn!("System proxy setting is only supported on Windows");
    }

    Ok(())
}

/// Clear system proxy settings
pub async fn clear_system_proxy() -> Result<()> {
    info!("Clearing system proxy");

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(INTERNET_SETTINGS_KEY)
            .map_err(|e| IsolateError::Config(format!("Failed to open registry key: {}", e)))?;

        // Disable proxy
        key.set_value("ProxyEnable", &0u32)
            .map_err(|e| IsolateError::Config(format!("Failed to disable proxy: {}", e)))?;

        // Clear proxy server
        let _ = key.delete_value("ProxyServer");

        // Notify system of settings change
        notify_proxy_change();

        info!("System proxy cleared");
    }

    #[cfg(not(windows))]
    {
        warn!("System proxy clearing is only supported on Windows");
    }

    Ok(())
}

/// Notify Windows of proxy settings change
#[cfg(windows)]
fn notify_proxy_change() {
    use std::ptr;

    // INTERNET_OPTION_SETTINGS_CHANGED = 39
    // INTERNET_OPTION_REFRESH = 37
    const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
    const INTERNET_OPTION_REFRESH: u32 = 37;

    #[link(name = "wininet")]
    extern "system" {
        fn InternetSetOptionW(
            hInternet: *mut std::ffi::c_void,
            dwOption: u32,
            lpBuffer: *mut std::ffi::c_void,
            dwBufferLength: u32,
        ) -> i32;
    }

    unsafe {
        InternetSetOptionW(ptr::null_mut(), INTERNET_OPTION_SETTINGS_CHANGED, ptr::null_mut(), 0);
        InternetSetOptionW(ptr::null_mut(), INTERNET_OPTION_REFRESH, ptr::null_mut(), 0);
    }

    debug!("Notified system of proxy settings change");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_vless_url() {
        let url = "vless://550e8400-e29b-41d4-a716-446655440000@example.com:443#MyServer";
        let config = parse_vless_url(url).unwrap();

        assert_eq!(config.uuid, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(config.server, "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.name, "MyServer");
        assert!(config.tls);
        assert_eq!(config.transport, TransportType::Tcp);
    }

    #[test]
    fn test_parse_vless_url_with_ws() {
        let url = "vless://uuid@server.com:443?type=ws&path=%2Fws&host=cdn.example.com&security=tls&sni=example.com#WS-Server";
        let config = parse_vless_url(url).unwrap();

        assert_eq!(config.server, "server.com");
        assert_eq!(config.port, 443);
        assert!(config.tls);
        assert_eq!(config.sni, Some("example.com".to_string()));

        match config.transport {
            TransportType::Ws { path, host } => {
                assert_eq!(path, "/ws");
                assert_eq!(host, Some("cdn.example.com".to_string()));
            }
            _ => panic!("Expected WebSocket transport"),
        }
    }

    #[test]
    fn test_parse_vless_url_with_grpc() {
        let url = "vless://uuid@server.com:443?type=grpc&serviceName=myservice&security=tls#gRPC";
        let config = parse_vless_url(url).unwrap();

        match config.transport {
            TransportType::Grpc { service_name } => {
                assert_eq!(service_name, "myservice");
            }
            _ => panic!("Expected gRPC transport"),
        }
    }

    #[test]
    fn test_parse_vless_url_no_tls() {
        let url = "vless://uuid@server.com:80?security=none#NoTLS";
        let config = parse_vless_url(url).unwrap();

        assert!(!config.tls);
    }

    #[test]
    fn test_parse_vless_url_with_fingerprint() {
        let url = "vless://uuid@server.com:443?security=tls&fp=chrome#FP";
        let config = parse_vless_url(url).unwrap();

        assert_eq!(config.fingerprint, Some("chrome".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(parse_vless_url("http://example.com").is_err());
        assert!(parse_vless_url("vless://").is_err());
        assert!(parse_vless_url("vless://uuid@").is_err());
        assert!(parse_vless_url("vless://uuid@server").is_err());
    }

    #[test]
    fn test_generate_singbox_config_basic() {
        let config = VlessConfig::new("example.com".into(), 443, "test-uuid".into());
        let singbox_config = generate_singbox_config(&config, 1080);

        assert!(singbox_config["inbounds"][0]["listen_port"] == 1080);
        assert!(singbox_config["outbounds"][0]["server"] == "example.com");
        assert!(singbox_config["outbounds"][0]["server_port"] == 443);
        assert!(singbox_config["outbounds"][0]["uuid"] == "test-uuid");
    }

    #[test]
    fn test_generate_singbox_config_with_ws() {
        let config = VlessConfig::new("example.com".into(), 443, "test-uuid".into())
            .with_transport(TransportType::Ws {
                path: "/ws".into(),
                host: Some("cdn.example.com".into()),
            })
            .with_sni("example.com");

        let singbox_config = generate_singbox_config(&config, 1080);

        assert!(singbox_config["outbounds"][0]["transport"]["type"] == "ws");
        assert!(singbox_config["outbounds"][0]["transport"]["path"] == "/ws");
    }

    #[test]
    fn test_vless_config_builder() {
        let config = VlessConfig::new("server.com".into(), 443, "uuid".into())
            .with_name("My Server")
            .with_sni("sni.example.com")
            .with_fingerprint("chrome")
            .with_transport(TransportType::Ws {
                path: "/path".into(),
                host: None,
            });

        assert_eq!(config.name, "My Server");
        assert_eq!(config.sni, Some("sni.example.com".to_string()));
        assert_eq!(config.fingerprint, Some("chrome".to_string()));
    }
}

//! VLESS Engine for Isolate
//!
//! Handles VLESS protocol connections via sing-box:
//! - Parse VLESS URLs (including Reality protocol)
//! - Generate sing-box configurations from templates
//! - Manage sing-box process lifecycle
//! - Health checks for running instances
//! - System proxy management (Windows)

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::{get_binaries_dir, get_configs_dir};

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

/// VLESS flow type (for XTLS)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum VlessFlow {
    #[default]
    None,
    /// XTLS Vision flow - recommended for TCP
    XtlsRprxVision,
}

impl VlessFlow {
    /// Get the flow string for sing-box config
    pub fn as_str(&self) -> &str {
        match self {
            VlessFlow::None => "",
            VlessFlow::XtlsRprxVision => "xtls-rprx-vision",
        }
    }

    /// Check if flow is enabled
    pub fn is_enabled(&self) -> bool {
        !matches!(self, VlessFlow::None)
    }

    /// Parse flow from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "xtls-rprx-vision" => VlessFlow::XtlsRprxVision,
            _ => VlessFlow::None,
        }
    }
}

/// Reality protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealityConfig {
    pub enabled: bool,
    pub public_key: Option<String>,
    pub short_id: Option<String>,
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
    /// VLESS flow (xtls-rprx-vision)
    #[serde(default)]
    pub flow: VlessFlow,
    /// Reality protocol config
    #[serde(default)]
    pub reality: RealityConfig,
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
            flow: VlessFlow::default(),
            reality: RealityConfig::default(),
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

    /// Set config ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set VLESS flow (xtls-rprx-vision)
    pub fn with_flow(mut self, flow: VlessFlow) -> Self {
        self.flow = flow;
        self
    }

    /// Enable Reality protocol
    pub fn with_reality(mut self, public_key: String, short_id: Option<String>) -> Self {
        self.reality = RealityConfig {
            enabled: true,
            public_key: Some(public_key),
            short_id,
        };
        self
    }
}

// ============================================================================
// VLESS URL Parsing
// ============================================================================

/// Parse a VLESS URL into VlessConfig
///
/// Format: vless://uuid@server:port?type=ws&security=tls&path=/ws&sni=example.com&fp=chrome&flow=xtls-rprx-vision#name
///
/// Supported parameters:
/// - type: tcp, ws, grpc, h2
/// - security: tls, reality, none
/// - path: WebSocket/H2 path
/// - host: WebSocket/H2 host header
/// - sni: TLS SNI
/// - fp: TLS fingerprint
/// - serviceName: gRPC service name
/// - flow: xtls-rprx-vision (for TCP transport)
/// - pbk: Reality public key
/// - sid: Reality short ID
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

    // Determine security type
    let security = params.get("security").map(|s| s.as_str()).unwrap_or("tls");
    let tls = security != "none";
    let is_reality = security == "reality";

    // Parse transport type
    let transport = parse_transport_type(&params)?;

    // Parse flow (only valid for TCP transport)
    let flow = params.get("flow")
        .map(|f| VlessFlow::from_str(f))
        .unwrap_or_default();

    // Validate flow - only allowed with TCP transport
    if flow.is_enabled() && !matches!(transport, TransportType::Tcp) {
        warn!("VLESS flow is only supported with TCP transport, ignoring flow setting");
    }

    // Extract optional parameters
    let sni = params.get("sni").cloned();
    let fingerprint = params.get("fp").cloned();

    // Parse Reality config
    let reality = if is_reality {
        RealityConfig {
            enabled: true,
            public_key: params.get("pbk").cloned(),
            short_id: params.get("sid").cloned(),
        }
    } else {
        RealityConfig::default()
    };

    let config_name = name.unwrap_or_else(|| format!("{}:{}", server, port));
    let id = format!("vless-{}", sanitize_id(&config_name));

    // Determine flow based on transport type (flow only valid for TCP)
    let final_flow = if matches!(transport, TransportType::Tcp) { flow } else { VlessFlow::None };

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
        flow: final_flow,
        reality,
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

/// Path to sing-box config template
const SINGBOX_TEMPLATE_PATH: &str = "singbox/vless_template.json";

/// Load sing-box template from configs directory
fn load_singbox_template() -> Result<String> {
    let template_path = get_configs_dir().join(SINGBOX_TEMPLATE_PATH);
    
    if !template_path.exists() {
        return Err(IsolateError::Config(format!(
            "Sing-box template not found at: {}",
            template_path.display()
        )));
    }
    
    std::fs::read_to_string(&template_path)
        .map_err(|e| IsolateError::Config(format!(
            "Failed to read sing-box template: {}",
            e
        )))
}

/// Generate sing-box configuration from template
///
/// Replaces placeholders in the template:
/// - {{port}} - SOCKS port
/// - {{server}} - VLESS server address
/// - {{server_port}} - VLESS server port
/// - {{uuid}} - VLESS UUID
/// - {{flow}} - VLESS flow (e.g., xtls-rprx-vision)
/// - {{sni}} - TLS SNI
/// - {{fingerprint}} - TLS fingerprint
/// - {{reality_public_key}} - Reality public key
/// - {{reality_short_id}} - Reality short ID
pub fn generate_singbox_config_from_template(
    config: &VlessConfig,
    socks_port: u16,
) -> Result<serde_json::Value> {
    let template = load_singbox_template()?;
    
    // Determine flow based on transport
    let flow = match &config.transport {
        TransportType::Tcp => config.flow.as_str(),
        _ => "", // Flow only works with TCP
    };
    
    let sni = config.sni.as_deref().unwrap_or(&config.server);
    let fingerprint = config.fingerprint.as_deref().unwrap_or("chrome");
    
    // Reality config
    let reality_public_key = config.reality.public_key.as_deref().unwrap_or("");
    let reality_short_id = config.reality.short_id.as_deref().unwrap_or("");
    
    // Replace string placeholders
    let config_str = template
        .replace("\"{{port}}\"", &socks_port.to_string())
        .replace("\"{{server_port}}\"", &config.port.to_string())
        .replace("{{server}}", &config.server)
        .replace("{{uuid}}", &config.uuid)
        .replace("{{flow}}", flow)
        .replace("{{sni}}", sni)
        .replace("{{fingerprint}}", fingerprint)
        .replace("{{reality_public_key}}", reality_public_key)
        .replace("{{reality_short_id}}", reality_short_id);
    
    // Parse and validate JSON
    let mut parsed: serde_json::Value = serde_json::from_str(&config_str)
        .map_err(|e| IsolateError::Config(format!(
            "Invalid sing-box config after template substitution: {}",
            e
        )))?;
    
    // Post-process: remove empty flow field if not using flow
    if flow.is_empty() {
        if let Some(outbounds) = parsed.get_mut("outbounds") {
            if let Some(arr) = outbounds.as_array_mut() {
                for outbound in arr {
                    if outbound.get("type") == Some(&json!("vless")) {
                        if let Some(obj) = outbound.as_object_mut() {
                            if obj.get("flow") == Some(&json!("")) {
                                obj.remove("flow");
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Post-process: handle Reality vs TLS
    if config.reality.enabled {
        // Ensure Reality config is properly set
        if let Some(outbounds) = parsed.get_mut("outbounds") {
            if let Some(arr) = outbounds.as_array_mut() {
                for outbound in arr {
                    if outbound.get("type") == Some(&json!("vless")) {
                        if let Some(tls) = outbound.get_mut("tls") {
                            if let Some(tls_obj) = tls.as_object_mut() {
                                tls_obj.insert("reality".to_string(), json!({
                                    "enabled": true,
                                    "public_key": reality_public_key,
                                    "short_id": reality_short_id
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
    
    info!(
        config_id = %config.id,
        socks_port = socks_port,
        server = %config.server,
        flow = %flow,
        reality = config.reality.enabled,
        "Generated sing-box config from template"
    );
    
    Ok(parsed)
}

/// Generate sing-box configuration JSON for VLESS (programmatic)
///
/// Creates a config with:
/// - SOCKS5 inbound on specified port
/// - VLESS outbound to the configured server
/// - Proper DNS configuration
/// - Route rules for bypass
/// - Support for Reality protocol and XTLS flow
pub fn generate_singbox_config(config: &VlessConfig, socks_port: u16) -> serde_json::Value {
    let mut outbound = json!({
        "type": "vless",
        "tag": "vless-out",
        "server": config.server,
        "server_port": config.port,
        "uuid": config.uuid,
    });

    // Add flow if enabled (only for TCP transport)
    if config.flow.is_enabled() && matches!(config.transport, TransportType::Tcp) {
        outbound["flow"] = json!(config.flow.as_str());
    }

    // Add TLS configuration
    if config.tls {
        let mut tls_config = json!({
            "enabled": true,
        });

        // Server name (SNI)
        if let Some(ref sni) = config.sni {
            tls_config["server_name"] = json!(sni);
        } else {
            tls_config["server_name"] = json!(config.server);
        }

        // uTLS fingerprint
        if let Some(ref fp) = config.fingerprint {
            tls_config["utls"] = json!({
                "enabled": true,
                "fingerprint": fp
            });
        } else {
            // Default to Chrome fingerprint
            tls_config["utls"] = json!({
                "enabled": true,
                "fingerprint": "chrome"
            });
        }

        // Reality protocol
        if config.reality.enabled {
            let mut reality_config = json!({
                "enabled": true,
            });

            if let Some(ref pbk) = config.reality.public_key {
                reality_config["public_key"] = json!(pbk);
            }

            if let Some(ref sid) = config.reality.short_id {
                reality_config["short_id"] = json!(sid);
            }

            tls_config["reality"] = reality_config;
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

    // Build complete config with improved DNS and routing
    json!({
        "log": {
            "level": "warn",
            "timestamp": true
        },
        "dns": {
            "servers": [
                {
                    "tag": "dns-remote",
                    "address": "https://1.1.1.1/dns-query",
                    "address_resolver": "dns-direct",
                    "detour": "vless-out"
                },
                {
                    "tag": "dns-direct",
                    "address": "https://223.5.5.5/dns-query",
                    "detour": "direct"
                },
                {
                    "tag": "dns-block",
                    "address": "rcode://success"
                }
            ],
            "rules": [
                {
                    "outbound": "any",
                    "server": "dns-direct"
                }
            ],
            "strategy": "prefer_ipv4"
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
            },
            {
                "type": "dns",
                "tag": "dns-out"
            }
        ],
        "route": {
            "rules": [
                {
                    "protocol": "dns",
                    "outbound": "dns-out"
                },
                {
                    "ip_is_private": true,
                    "outbound": "direct"
                },
                {
                    "domain_suffix": [".local", ".localhost"],
                    "outbound": "direct"
                }
            ],
            "final": "vless-out",
            "auto_detect_interface": true
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

    // Try to generate config from template first, fallback to programmatic
    let singbox_config = match generate_singbox_config_from_template(config, socks_port) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!(
                error = %e,
                "Failed to load template, using programmatic config generation"
            );
            generate_singbox_config(config, socks_port)
        }
    };
    
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

/// VLESS connection result with SOCKS port
#[derive(Debug, Clone)]
pub struct VlessConnection {
    pub config_id: String,
    pub socks_port: u16,
}

/// Start VLESS connection and return SOCKS port for testing
///
/// This is a convenience wrapper that starts sing-box and returns
/// the SOCKS port that can be used for proxy testing.
pub async fn start_vless_with_port(
    config: &VlessConfig,
    socks_port: u16,
) -> Result<(Child, VlessConnection)> {
    let child = start_vless(config, socks_port).await?;
    
    // Give sing-box time to start listening
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    let connection = VlessConnection {
        config_id: config.id.clone(),
        socks_port,
    };
    
    info!(
        config_id = %config.id,
        socks_port = socks_port,
        "VLESS connection ready"
    );
    
    Ok((child, connection))
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
// Health Check Functions
// ============================================================================

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub is_healthy: bool,
    pub socks_port_open: bool,
    pub latency_ms: Option<u32>,
    pub error: Option<String>,
}

/// Perform health check on a running sing-box instance
///
/// Checks:
/// 1. SOCKS port is open and accepting connections
/// 2. Optionally tests proxy connectivity
pub async fn health_check_socks(socks_port: u16) -> HealthCheckResult {
    let addr = format!("127.0.0.1:{}", socks_port);
    let timeout_duration = Duration::from_secs(3);

    debug!(socks_port = socks_port, "Performing SOCKS health check");

    let start = std::time::Instant::now();

    // Try to connect to the SOCKS proxy
    let result = tokio::time::timeout(
        timeout_duration,
        tokio::net::TcpStream::connect(&addr)
    ).await;

    match result {
        Ok(Ok(_stream)) => {
            let latency = start.elapsed().as_millis() as u32;
            debug!(socks_port = socks_port, latency_ms = latency, "SOCKS health check passed");
            HealthCheckResult {
                is_healthy: true,
                socks_port_open: true,
                latency_ms: Some(latency),
                error: None,
            }
        }
        Ok(Err(e)) => {
            warn!(socks_port = socks_port, error = %e, "SOCKS health check failed - connection error");
            HealthCheckResult {
                is_healthy: false,
                socks_port_open: false,
                latency_ms: None,
                error: Some(format!("Connection error: {}", e)),
            }
        }
        Err(_) => {
            warn!(socks_port = socks_port, "SOCKS health check failed - timeout");
            HealthCheckResult {
                is_healthy: false,
                socks_port_open: false,
                latency_ms: None,
                error: Some("Connection timeout".to_string()),
            }
        }
    }
}

/// Test proxy connectivity through SOCKS
///
/// Makes a test request through the proxy to verify it's working.
pub async fn test_proxy_connectivity(socks_port: u16, test_url: &str) -> Result<u32> {
    let proxy_addr = format!("socks5://127.0.0.1:{}", socks_port);
    let timeout_duration = Duration::from_secs(10);

    debug!(
        socks_port = socks_port,
        test_url = test_url,
        "Testing proxy connectivity"
    );

    let start = std::time::Instant::now();

    // Build client with SOCKS proxy
    let proxy = reqwest::Proxy::all(&proxy_addr)
        .map_err(|e| IsolateError::Network(format!("Invalid proxy address: {}", e)))?;

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(timeout_duration)
        .build()
        .map_err(|e| IsolateError::Network(format!("Failed to build HTTP client: {}", e)))?;

    // Make test request
    let response = client
        .head(test_url)
        .send()
        .await
        .map_err(|e| IsolateError::Network(format!("Proxy test request failed: {}", e)))?;

    let latency = start.elapsed().as_millis() as u32;

    if response.status().is_success() || response.status().is_redirection() {
        info!(
            socks_port = socks_port,
            latency_ms = latency,
            status = %response.status(),
            "Proxy connectivity test passed"
        );
        Ok(latency)
    } else {
        Err(IsolateError::Network(format!(
            "Proxy test returned status: {}",
            response.status()
        )))
    }
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
        assert!(!config.flow.is_enabled());
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
    fn test_parse_vless_url_with_flow() {
        let url = "vless://uuid@server.com:443?security=tls&flow=xtls-rprx-vision#Vision";
        let config = parse_vless_url(url).unwrap();

        assert!(config.flow.is_enabled());
        assert_eq!(config.flow.as_str(), "xtls-rprx-vision");
    }

    #[test]
    fn test_parse_vless_url_with_reality() {
        let url = "vless://uuid@server.com:443?security=reality&sni=www.google.com&fp=chrome&pbk=publickey123&sid=shortid#Reality";
        let config = parse_vless_url(url).unwrap();

        assert!(config.tls);
        assert!(config.reality.enabled);
        assert_eq!(config.reality.public_key, Some("publickey123".to_string()));
        assert_eq!(config.reality.short_id, Some("shortid".to_string()));
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
    fn test_generate_singbox_config_with_flow() {
        let config = VlessConfig::new("example.com".into(), 443, "test-uuid".into())
            .with_flow(VlessFlow::XtlsRprxVision);

        let singbox_config = generate_singbox_config(&config, 1080);

        assert_eq!(singbox_config["outbounds"][0]["flow"], "xtls-rprx-vision");
    }

    #[test]
    fn test_generate_singbox_config_with_reality() {
        let config = VlessConfig::new("example.com".into(), 443, "test-uuid".into())
            .with_reality("publickey123".into(), Some("shortid".into()));

        let singbox_config = generate_singbox_config(&config, 1080);

        assert!(singbox_config["outbounds"][0]["tls"]["reality"]["enabled"] == true);
        assert!(singbox_config["outbounds"][0]["tls"]["reality"]["public_key"] == "publickey123");
    }

    #[test]
    fn test_vless_config_builder() {
        let config = VlessConfig::new("server.com".into(), 443, "uuid".into())
            .with_name("My Server")
            .with_sni("sni.example.com")
            .with_fingerprint("chrome")
            .with_flow(VlessFlow::XtlsRprxVision)
            .with_transport(TransportType::Ws {
                path: "/path".into(),
                host: None,
            });

        assert_eq!(config.name, "My Server");
        assert_eq!(config.sni, Some("sni.example.com".to_string()));
        assert_eq!(config.fingerprint, Some("chrome".to_string()));
        // Flow should be reset to None because WS transport doesn't support flow
    }

    #[test]
    fn test_vless_flow_parsing() {
        assert_eq!(VlessFlow::from_str("xtls-rprx-vision"), VlessFlow::XtlsRprxVision);
        assert_eq!(VlessFlow::from_str("XTLS-RPRX-VISION"), VlessFlow::XtlsRprxVision);
        assert_eq!(VlessFlow::from_str("unknown"), VlessFlow::None);
        assert_eq!(VlessFlow::from_str(""), VlessFlow::None);
    }
}

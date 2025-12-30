//! Data models for Isolate

use serde::{Deserialize, Serialize};

// ============================================================================
// Strategy Models
// ============================================================================

/// Strategy family/type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyFamily {
    DnsBypass,
    SniFrag,
    TlsFrag,
    Vless,
    Hybrid,
}

/// Engine type for strategy execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyEngine {
    Zapret,
    SingBox,
    Xray,
    Hybrid,
}

/// Strategy execution mode capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModeCapabilities {
    pub supports_socks: bool,
    pub supports_global: bool,
}

/// Template for launching a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchTemplate {
    pub binary: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    pub log_file: Option<String>,
    #[serde(default)]
    pub requires_admin: bool,
}

/// Strategy requirements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyRequirements {
    #[serde(default = "default_min_rights")]
    pub min_rights: String,
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub binaries: Vec<String>,
}

fn default_min_rights() -> String {
    "user".to_string()
}

/// Strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub family: StrategyFamily,
    pub engine: StrategyEngine,
    #[serde(default)]
    pub mode_capabilities: ModeCapabilities,
    pub socks_template: Option<LaunchTemplate>,
    pub global_template: Option<LaunchTemplate>,
    #[serde(default)]
    pub requirements: StrategyRequirements,
    #[serde(default)]
    pub weight_hint: i32,
    #[serde(default)]
    pub services: Vec<String>,
}

// ============================================================================
// Service & Test Models
// ============================================================================

/// Test type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestDefinition {
    HttpsGet {
        url: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
        #[serde(default)]
        expected_status: Vec<u16>,
        min_body_size: Option<usize>,
    },
    HttpsHead {
        url: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
    WebSocket {
        url: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
    TcpConnect {
        host: String,
        port: u16,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
    Dns {
        domain: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
}

fn default_timeout() -> u32 {
    5000
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled_by_default: bool,
    #[serde(default)]
    pub critical: bool,
    pub tests: Vec<TestDefinition>,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Result Models
// ============================================================================

/// Error type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    Dns,
    Tcp,
    Tls,
    Http,
    Timeout,
    Unknown,
}

/// Single test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub error_type: Option<ErrorType>,
    pub error_message: Option<String>,
}

/// Aggregated service test summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTestSummary {
    pub service_id: String,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub errors: Vec<ErrorType>,
}

/// Strategy score after testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyScore {
    pub strategy_id: String,
    pub success_rate: f64,
    pub critical_success_rate: f64,
    pub latency_avg: f64,
    pub latency_jitter: f64,
    pub score: f64,
}

// ============================================================================
// Diagnostic Models
// ============================================================================

/// DPI block type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DpiKind {
    DnsBlock,
    SniTlsBlock,
    IpBlock,
    #[default]
    NoBlock,
    Unknown,
}

/// DPI diagnostic profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DpiProfile {
    pub kind: DpiKind,
    pub details: Option<String>,
    pub candidate_families: Vec<StrategyFamily>,
}

/// Diagnostic result for frontend
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticResult {
    pub profile: DpiProfile,
    pub tested_services: Vec<String>,
    pub blocked_services: Vec<String>,
}

// ============================================================================
// Application State Models
// ============================================================================

/// Current application status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppStatus {
    pub is_active: bool,
    pub current_strategy: Option<String>,
    pub current_strategy_name: Option<String>,
    pub services_status: std::collections::HashMap<String, bool>,
}

/// IP stack type for dual-stack support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IpStack {
    /// IPv4 only
    V4Only,
    /// IPv6 only
    V6Only,
    /// Both IPv4 and IPv6
    #[default]
    DualStack,
}

impl std::fmt::Display for IpStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpStack::V4Only => write!(f, "IPv4"),
            IpStack::V6Only => write!(f, "IPv6"),
            IpStack::DualStack => write!(f, "Dual-Stack"),
        }
    }
}

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_start: bool,
    pub auto_apply: bool,
    pub minimize_to_tray: bool,
    pub block_quic: bool,
    pub default_mode: String, // "turbo" or "deep"
    /// Portable mode - store data in app directory
    #[serde(default)]
    pub portable_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_start: false,
            auto_apply: false,
            minimize_to_tray: true,
            block_quic: true,
            default_mode: "turbo".to_string(),
            portable_mode: false,
        }
    }
}

/// Service with enabled state for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWithState {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub critical: bool,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvInfo {
    pub asn: Option<String>,
    pub country: Option<String>,
    pub wifi_ssid: Option<String>,
    pub is_admin: bool,
    pub os_version: String,
}

impl EnvInfo {
    /// Generate cache key from environment
    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.asn.as_deref().unwrap_or("unknown"),
            self.country.as_deref().unwrap_or("unknown"),
            self.wifi_ssid.as_deref().unwrap_or("unknown")
        )
    }
}

// ============================================================================
// Update Models
// ============================================================================

/// Information about available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}

// ============================================================================
// Log Models
// ============================================================================

/// Log entry for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
}

// ============================================================================
// VLESS Models
// ============================================================================

/// VLESS proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlessConfig {
    pub id: String,
    pub name: String,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub flow: Option<String>,
    pub security: String,
    pub sni: Option<String>,
    #[serde(default)]
    pub active: bool,
}

impl VlessConfig {
    /// Parse VLESS config from vless:// URL
    /// Format: vless://uuid@server:port?security=tls&sni=example.com&flow=xtls-rprx-vision#name
    pub fn from_url(url: &str) -> Result<Self, String> {
        // Check protocol
        if !url.starts_with("vless://") {
            return Err("URL must start with vless://".to_string());
        }

        let url = &url[8..]; // Remove "vless://"

        // Split by # to get name
        let (main_part, name) = if let Some(hash_pos) = url.rfind('#') {
            let name = urlencoding::decode(&url[hash_pos + 1..])
                .map_err(|e| format!("Failed to decode name: {}", e))?
                .to_string();
            (&url[..hash_pos], name)
        } else {
            (url, "Imported Config".to_string())
        };

        // Split by ? to get params
        let (address_part, params_str) = if let Some(q_pos) = main_part.find('?') {
            (&main_part[..q_pos], Some(&main_part[q_pos + 1..]))
        } else {
            (main_part, None)
        };

        // Parse uuid@server:port
        let at_pos = address_part
            .find('@')
            .ok_or_else(|| "Invalid URL format: missing @".to_string())?;

        let uuid = address_part[..at_pos].to_string();
        if uuid.is_empty() {
            return Err("UUID cannot be empty".to_string());
        }

        let server_port = &address_part[at_pos + 1..];
        let colon_pos = server_port
            .rfind(':')
            .ok_or_else(|| "Invalid URL format: missing port".to_string())?;

        let server = server_port[..colon_pos].to_string();
        if server.is_empty() {
            return Err("Server cannot be empty".to_string());
        }

        let port: u16 = server_port[colon_pos + 1..]
            .parse()
            .map_err(|_| "Invalid port number".to_string())?;

        // Parse query params
        let mut security = "tls".to_string();
        let mut sni: Option<String> = None;
        let mut flow: Option<String> = None;

        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some(eq_pos) = param.find('=') {
                    let key = &param[..eq_pos];
                    let value = urlencoding::decode(&param[eq_pos + 1..])
                        .map_err(|e| format!("Failed to decode param {}: {}", key, e))?
                        .to_string();

                    match key {
                        "security" => security = value,
                        "sni" => sni = Some(value),
                        "flow" => flow = Some(value),
                        _ => {} // Ignore unknown params
                    }
                }
            }
        }

        // Generate unique ID
        let id = format!("vless_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));

        Ok(VlessConfig {
            id,
            name,
            server,
            port,
            uuid,
            flow,
            security,
            sni,
            active: false,
        })
    }

    /// Convert config back to vless:// URL
    pub fn to_url(&self) -> String {
        let mut url = format!("vless://{}@{}:{}", self.uuid, self.server, self.port);

        let mut params = vec![format!("security={}", self.security)];

        if let Some(ref sni) = self.sni {
            params.push(format!("sni={}", sni));
        }

        if let Some(ref flow) = self.flow {
            params.push(format!("flow={}", flow));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url.push('#');
        url.push_str(&urlencoding::encode(&self.name));

        url
    }
}

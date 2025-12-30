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

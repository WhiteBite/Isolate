//! Data models for Isolate

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub env: HashMap<String, String>,
    pub log_file: Option<String>,
    #[serde(default)]
    pub requires_admin: bool,
}

/// Strategy requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRequirements {
    #[serde(default = "default_min_rights")]
    pub min_rights: String,
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub binaries: Vec<String>,
}

impl Default for StrategyRequirements {
    fn default() -> Self {
        Self {
            min_rights: default_min_rights(),
            os: Vec::new(),
            binaries: Vec::new(),
        }
    }
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
    /// Simple test URL for quick connectivity checks
    #[serde(default)]
    pub test_url: Option<String>,
}

impl Service {
    /// Get the test URL for this service
    /// Falls back to extracting URL from first HTTPS test if test_url is not set
    pub fn get_test_url(&self) -> Option<String> {
        if let Some(ref url) = self.test_url {
            return Some(url.clone());
        }
        
        // Try to extract from tests
        for test in &self.tests {
            match test {
                TestDefinition::HttpsGet { url, .. } => return Some(url.clone()),
                TestDefinition::HttpsHead { url, .. } => return Some(url.clone()),
                TestDefinition::WebSocket { url, .. } => return Some(url.clone()),
                _ => continue,
            }
        }
        
        None
    }
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
    pub services_status: HashMap<String, bool>,
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
// Proxy Protocol Models
// ============================================================================

/// Supported proxy protocols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocol {
    #[default]
    Socks5,
    Http,
    Https,
    Shadowsocks,
    Trojan,
    Vmess,
    Vless,
    Tuic,
    Hysteria,
    Hysteria2,
    Wireguard,
    Ssh,
}

impl std::fmt::Display for ProxyProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyProtocol::Socks5 => write!(f, "SOCKS5"),
            ProxyProtocol::Http => write!(f, "HTTP"),
            ProxyProtocol::Https => write!(f, "HTTPS"),
            ProxyProtocol::Shadowsocks => write!(f, "Shadowsocks"),
            ProxyProtocol::Trojan => write!(f, "Trojan"),
            ProxyProtocol::Vmess => write!(f, "VMess"),
            ProxyProtocol::Vless => write!(f, "VLESS"),
            ProxyProtocol::Tuic => write!(f, "TUIC"),
            ProxyProtocol::Hysteria => write!(f, "Hysteria"),
            ProxyProtocol::Hysteria2 => write!(f, "Hysteria2"),
            ProxyProtocol::Wireguard => write!(f, "WireGuard"),
            ProxyProtocol::Ssh => write!(f, "SSH"),
        }
    }
}

/// Universal proxy configuration supporting all protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: String,
    pub name: String,
    pub protocol: ProxyProtocol,
    pub server: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub uuid: Option<String>,
    pub tls: bool,
    pub sni: Option<String>,
    pub transport: Option<String>,
    #[serde(default)]
    pub custom_fields: HashMap<String, String>,
    #[serde(default)]
    pub active: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            protocol: ProxyProtocol::default(),
            server: String::new(),
            port: 1080,
            username: None,
            password: None,
            uuid: None,
            tls: false,
            sni: None,
            transport: None,
            custom_fields: HashMap::new(),
            active: false,
        }
    }
}

// ============================================================================
// Routing Models
// ============================================================================

/// Domain-based routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRoute {
    pub domain: String,
    pub proxy_id: String,
}

/// Application-based routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRoute {
    pub app_name: String,
    pub app_path: String,
    pub proxy_id: String,
}

// ============================================================================
// VLESS Models
// ============================================================================

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_serialization() {
        // Test Strategy serialization/deserialization
        let strategy = Strategy {
            id: "test-strategy".to_string(),
            name: "Test Strategy".to_string(),
            description: "A test strategy".to_string(),
            family: StrategyFamily::DnsBypass,
            engine: StrategyEngine::Zapret,
            mode_capabilities: ModeCapabilities {
                supports_socks: true,
                supports_global: true,
            },
            socks_template: Some(LaunchTemplate {
                binary: "winws.exe".to_string(),
                args: vec!["--arg1".to_string(), "--arg2".to_string()],
                env: HashMap::new(),
                log_file: Some("log.txt".to_string()),
                requires_admin: true,
            }),
            global_template: None,
            requirements: StrategyRequirements {
                min_rights: "admin".to_string(),
                os: vec!["windows".to_string()],
                binaries: vec!["winws.exe".to_string()],
            },
            weight_hint: 100,
            services: vec!["youtube".to_string(), "discord".to_string()],
        };

        // Serialize to JSON
        let json = serde_json::to_string(&strategy).unwrap();
        
        // Deserialize back
        let deserialized: Strategy = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, "test-strategy");
        assert_eq!(deserialized.name, "Test Strategy");
        assert_eq!(deserialized.family, StrategyFamily::DnsBypass);
        assert_eq!(deserialized.engine, StrategyEngine::Zapret);
        assert!(deserialized.mode_capabilities.supports_socks);
        assert!(deserialized.mode_capabilities.supports_global);
        assert!(deserialized.socks_template.is_some());
        assert!(deserialized.global_template.is_none());
        assert_eq!(deserialized.requirements.min_rights, "admin");
        assert_eq!(deserialized.weight_hint, 100);
        assert_eq!(deserialized.services.len(), 2);
    }

    #[test]
    fn test_strategy_family_serialization() {
        // Test StrategyFamily enum serialization
        let families = vec![
            (StrategyFamily::DnsBypass, "\"dns_bypass\""),
            (StrategyFamily::SniFrag, "\"sni_frag\""),
            (StrategyFamily::TlsFrag, "\"tls_frag\""),
            (StrategyFamily::Vless, "\"vless\""),
            (StrategyFamily::Hybrid, "\"hybrid\""),
        ];

        for (family, expected_json) in families {
            let json = serde_json::to_string(&family).unwrap();
            assert_eq!(json, expected_json);
            
            let deserialized: StrategyFamily = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, family);
        }
    }

    #[test]
    fn test_engine_enum() {
        // Test StrategyEngine enum serialization/deserialization
        let engines = vec![
            (StrategyEngine::Zapret, "\"zapret\""),
            (StrategyEngine::SingBox, "\"sing_box\""),
            (StrategyEngine::Xray, "\"xray\""),
            (StrategyEngine::Hybrid, "\"hybrid\""),
        ];

        for (engine, expected_json) in engines {
            let json = serde_json::to_string(&engine).unwrap();
            assert_eq!(json, expected_json);
            
            let deserialized: StrategyEngine = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, engine);
        }
    }

    #[test]
    fn test_engine_equality() {
        assert_eq!(StrategyEngine::Zapret, StrategyEngine::Zapret);
        assert_ne!(StrategyEngine::Zapret, StrategyEngine::SingBox);
        assert_ne!(StrategyEngine::Xray, StrategyEngine::Hybrid);
    }

    #[test]
    fn test_dpi_kind_default() {
        let kind: DpiKind = Default::default();
        assert_eq!(kind, DpiKind::NoBlock);
    }

    #[test]
    fn test_ip_stack_display() {
        assert_eq!(IpStack::V4Only.to_string(), "IPv4");
        assert_eq!(IpStack::V6Only.to_string(), "IPv6");
        assert_eq!(IpStack::DualStack.to_string(), "Dual-Stack");
    }

    #[test]
    fn test_ip_stack_default() {
        let stack: IpStack = Default::default();
        assert_eq!(stack, IpStack::DualStack);
    }

    #[test]
    fn test_proxy_protocol_display() {
        assert_eq!(ProxyProtocol::Socks5.to_string(), "SOCKS5");
        assert_eq!(ProxyProtocol::Http.to_string(), "HTTP");
        assert_eq!(ProxyProtocol::Vless.to_string(), "VLESS");
        assert_eq!(ProxyProtocol::Wireguard.to_string(), "WireGuard");
    }

    #[test]
    fn test_proxy_protocol_default() {
        let protocol: ProxyProtocol = Default::default();
        assert_eq!(protocol, ProxyProtocol::Socks5);
    }

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(!settings.auto_start);
        assert!(!settings.auto_apply);
        assert!(settings.minimize_to_tray);
        assert!(settings.block_quic);
        assert_eq!(settings.default_mode, "turbo");
        assert!(!settings.portable_mode);
    }

    #[test]
    fn test_env_info_cache_key() {
        let env = EnvInfo {
            asn: Some("AS12345".to_string()),
            country: Some("RU".to_string()),
            wifi_ssid: Some("MyWiFi".to_string()),
            is_admin: false,
            os_version: "Windows 11".to_string(),
        };
        
        assert_eq!(env.cache_key(), "AS12345:RU:MyWiFi");
        
        let env_partial = EnvInfo {
            asn: None,
            country: Some("US".to_string()),
            wifi_ssid: None,
            is_admin: true,
            os_version: "Windows 10".to_string(),
        };
        
        assert_eq!(env_partial.cache_key(), "unknown:US:unknown");
    }

    #[test]
    fn test_service_get_test_url() {
        // Test with explicit test_url
        let service_with_url = Service {
            id: "test".to_string(),
            name: "Test Service".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![],
            test_url: Some("https://example.com".to_string()),
        };
        assert_eq!(service_with_url.get_test_url(), Some("https://example.com".to_string()));

        // Test extracting from HttpsGet test
        let service_with_test = Service {
            id: "test2".to_string(),
            name: "Test Service 2".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![
                TestDefinition::HttpsGet {
                    url: "https://test.com/api".to_string(),
                    timeout_ms: 5000,
                    expected_status: vec![200],
                    min_body_size: None,
                },
            ],
            test_url: None,
        };
        assert_eq!(service_with_test.get_test_url(), Some("https://test.com/api".to_string()));

        // Test with no URL available
        let service_no_url = Service {
            id: "test3".to_string(),
            name: "Test Service 3".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![
                TestDefinition::TcpConnect {
                    host: "example.com".to_string(),
                    port: 443,
                    timeout_ms: 5000,
                },
            ],
            test_url: None,
        };
        assert_eq!(service_no_url.get_test_url(), None);
    }

    #[test]
    fn test_test_definition_variants() {
        // Test all TestDefinition variants can be serialized/deserialized
        let tests = vec![
            TestDefinition::HttpsGet {
                url: "https://example.com".to_string(),
                timeout_ms: 5000,
                expected_status: vec![200, 201],
                min_body_size: Some(100),
            },
            TestDefinition::HttpsHead {
                url: "https://example.com".to_string(),
                timeout_ms: 3000,
            },
            TestDefinition::WebSocket {
                url: "wss://example.com/ws".to_string(),
                timeout_ms: 5000,
            },
            TestDefinition::TcpConnect {
                host: "example.com".to_string(),
                port: 443,
                timeout_ms: 5000,
            },
            TestDefinition::Dns {
                domain: "example.com".to_string(),
                timeout_ms: 2000,
            },
        ];

        for test in tests {
            let json = serde_json::to_string(&test).unwrap();
            let _deserialized: TestDefinition = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_proxy_config_default() {
        let config = ProxyConfig::default();
        assert!(config.id.is_empty());
        assert!(config.name.is_empty());
        assert_eq!(config.protocol, ProxyProtocol::Socks5);
        assert!(config.server.is_empty());
        assert_eq!(config.port, 1080);
        assert!(config.username.is_none());
        assert!(config.password.is_none());
        assert!(!config.tls);
        assert!(!config.active);
    }

    #[test]
    fn test_mode_capabilities_default() {
        let caps = ModeCapabilities::default();
        assert!(!caps.supports_socks);
        assert!(!caps.supports_global);
    }

    #[test]
    fn test_strategy_requirements_default() {
        let reqs = StrategyRequirements::default();
        assert_eq!(reqs.min_rights, "user");
        assert!(reqs.os.is_empty());
        assert!(reqs.binaries.is_empty());
    }

    // ========================================================================
    // VlessConfig Tests
    // ========================================================================

    #[test]
    fn test_vless_config_from_url_basic() {
        let url = "vless://550e8400-e29b-41d4-a716-446655440000@example.com:443?security=tls&sni=example.com#MyServer";
        let config = VlessConfig::from_url(url).unwrap();
        
        assert_eq!(config.uuid, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(config.server, "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.security, "tls");
        assert_eq!(config.sni, Some("example.com".to_string()));
        assert_eq!(config.name, "MyServer");
        assert!(!config.active);
    }

    #[test]
    fn test_vless_config_from_url_with_flow() {
        let url = "vless://test-uuid@server.net:8443?security=tls&flow=xtls-rprx-vision#Test";
        let config = VlessConfig::from_url(url).unwrap();
        
        assert_eq!(config.uuid, "test-uuid");
        assert_eq!(config.server, "server.net");
        assert_eq!(config.port, 8443);
        assert_eq!(config.flow, Some("xtls-rprx-vision".to_string()));
    }

    #[test]
    fn test_vless_config_from_url_minimal() {
        // URL without query params and name
        let url = "vless://uuid123@host.com:1234";
        let config = VlessConfig::from_url(url).unwrap();
        
        assert_eq!(config.uuid, "uuid123");
        assert_eq!(config.server, "host.com");
        assert_eq!(config.port, 1234);
        assert_eq!(config.security, "tls"); // default
        assert_eq!(config.name, "Imported Config"); // default
    }

    #[test]
    fn test_vless_config_from_url_encoded_name() {
        let url = "vless://uuid@server.com:443#My%20Server%20%231";
        let config = VlessConfig::from_url(url).unwrap();
        
        assert_eq!(config.name, "My Server #1");
    }

    #[test]
    fn test_vless_config_from_url_ipv6() {
        let url = "vless://uuid@[2001:db8::1]:443?security=tls#IPv6Server";
        let config = VlessConfig::from_url(url).unwrap();
        
        assert_eq!(config.server, "[2001:db8::1]");
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_vless_config_from_url_invalid_protocol() {
        let result = VlessConfig::from_url("vmess://uuid@server:443");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("vless://"));
    }

    #[test]
    fn test_vless_config_from_url_empty_uuid() {
        let result = VlessConfig::from_url("vless://@server:443");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UUID"));
    }

    #[test]
    fn test_vless_config_from_url_empty_server() {
        let result = VlessConfig::from_url("vless://uuid@:443");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Server"));
    }

    #[test]
    fn test_vless_config_from_url_invalid_port() {
        let result = VlessConfig::from_url("vless://uuid@server:notaport");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port"));
    }

    #[test]
    fn test_vless_config_from_url_missing_at() {
        let result = VlessConfig::from_url("vless://uuidserver:443");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("@"));
    }

    #[test]
    fn test_vless_config_from_url_missing_port() {
        let result = VlessConfig::from_url("vless://uuid@server");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("port"));
    }

    #[test]
    fn test_vless_config_to_url_basic() {
        let config = VlessConfig {
            id: "test-id".to_string(),
            name: "Test Server".to_string(),
            server: "example.com".to_string(),
            port: 443,
            uuid: "test-uuid-123".to_string(),
            flow: None,
            security: "tls".to_string(),
            sni: Some("example.com".to_string()),
            active: false,
            per_domain_routing: Vec::new(),
            per_app_routing: Vec::new(),
        };
        
        let url = config.to_url();
        
        assert!(url.starts_with("vless://test-uuid-123@example.com:443"));
        assert!(url.contains("security=tls"));
        assert!(url.contains("sni=example.com"));
        assert!(url.contains("#Test%20Server"));
    }

    #[test]
    fn test_vless_config_to_url_with_flow() {
        let config = VlessConfig {
            id: "id".to_string(),
            name: "Server".to_string(),
            server: "host.net".to_string(),
            port: 8443,
            uuid: "uuid".to_string(),
            flow: Some("xtls-rprx-vision".to_string()),
            security: "reality".to_string(),
            sni: None,
            active: true,
            per_domain_routing: Vec::new(),
            per_app_routing: Vec::new(),
        };
        
        let url = config.to_url();
        
        assert!(url.contains("flow=xtls-rprx-vision"));
        assert!(url.contains("security=reality"));
        assert!(!url.contains("sni="));
    }

    #[test]
    fn test_vless_config_roundtrip() {
        let original_url = "vless://550e8400-e29b-41d4-a716-446655440000@example.com:443?security=tls&sni=example.com&flow=xtls-rprx-vision#TestServer";
        let config = VlessConfig::from_url(original_url).unwrap();
        let generated_url = config.to_url();
        
        // Parse again and compare
        let config2 = VlessConfig::from_url(&generated_url).unwrap();
        
        assert_eq!(config.uuid, config2.uuid);
        assert_eq!(config.server, config2.server);
        assert_eq!(config.port, config2.port);
        assert_eq!(config.security, config2.security);
        assert_eq!(config.sni, config2.sni);
        assert_eq!(config.flow, config2.flow);
        assert_eq!(config.name, config2.name);
    }

    // ========================================================================
    // ErrorType Tests
    // ========================================================================

    #[test]
    fn test_error_type_serialization() {
        let error_types = vec![
            (ErrorType::Dns, "\"dns\""),
            (ErrorType::Tcp, "\"tcp\""),
            (ErrorType::Tls, "\"tls\""),
            (ErrorType::Http, "\"http\""),
            (ErrorType::Timeout, "\"timeout\""),
            (ErrorType::Unknown, "\"unknown\""),
        ];

        for (error_type, expected_json) in error_types {
            let json = serde_json::to_string(&error_type).unwrap();
            assert_eq!(json, expected_json);
            
            let deserialized: ErrorType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, error_type);
        }
    }

    #[test]
    fn test_error_type_equality() {
        assert_eq!(ErrorType::Dns, ErrorType::Dns);
        assert_ne!(ErrorType::Dns, ErrorType::Tcp);
        assert_ne!(ErrorType::Timeout, ErrorType::Unknown);
    }

    #[test]
    fn test_error_type_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(ErrorType::Dns);
        set.insert(ErrorType::Tcp);
        set.insert(ErrorType::Dns); // duplicate
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&ErrorType::Dns));
        assert!(set.contains(&ErrorType::Tcp));
    }

    // ========================================================================
    // TestResult Tests
    // ========================================================================

    #[test]
    fn test_test_result_success() {
        let result = TestResult {
            test_id: "test-1".to_string(),
            success: true,
            latency_ms: Some(150),
            error_type: None,
            error_message: None,
        };
        
        assert!(result.success);
        assert_eq!(result.latency_ms, Some(150));
        assert!(result.error_type.is_none());
    }

    #[test]
    fn test_test_result_failure() {
        let result = TestResult {
            test_id: "test-2".to_string(),
            success: false,
            latency_ms: None,
            error_type: Some(ErrorType::Timeout),
            error_message: Some("Connection timed out".to_string()),
        };
        
        assert!(!result.success);
        assert!(result.latency_ms.is_none());
        assert_eq!(result.error_type, Some(ErrorType::Timeout));
    }

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult {
            test_id: "https-get-1".to_string(),
            success: true,
            latency_ms: Some(200),
            error_type: None,
            error_message: None,
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TestResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.test_id, "https-get-1");
        assert!(deserialized.success);
        assert_eq!(deserialized.latency_ms, Some(200));
    }

    // ========================================================================
    // StrategyScore Tests
    // ========================================================================

    #[test]
    fn test_strategy_score_creation() {
        let score = StrategyScore {
            strategy_id: "strategy-1".to_string(),
            success_rate: 0.95,
            critical_success_rate: 1.0,
            latency_avg: 150.5,
            latency_jitter: 25.0,
            score: 85.5,
        };
        
        assert_eq!(score.strategy_id, "strategy-1");
        assert!((score.success_rate - 0.95).abs() < f64::EPSILON);
        assert!((score.critical_success_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_strategy_score_serialization() {
        let score = StrategyScore {
            strategy_id: "test-strategy".to_string(),
            success_rate: 0.8,
            critical_success_rate: 0.9,
            latency_avg: 200.0,
            latency_jitter: 50.0,
            score: 75.0,
        };
        
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: StrategyScore = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.strategy_id, "test-strategy");
        assert!((deserialized.success_rate - 0.8).abs() < f64::EPSILON);
        assert!((deserialized.score - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_strategy_score_edge_values() {
        let score = StrategyScore {
            strategy_id: "edge-case".to_string(),
            success_rate: 0.0,
            critical_success_rate: 0.0,
            latency_avg: 0.0,
            latency_jitter: 0.0,
            score: 0.0,
        };
        
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: StrategyScore = serde_json::from_str(&json).unwrap();
        
        assert!((deserialized.success_rate - 0.0).abs() < f64::EPSILON);
    }

    // ========================================================================
    // DiagnosticResult Tests
    // ========================================================================

    #[test]
    fn test_diagnostic_result_default() {
        let result = DiagnosticResult::default();
        
        assert_eq!(result.profile.kind, DpiKind::NoBlock);
        assert!(result.profile.details.is_none());
        assert!(result.profile.candidate_families.is_empty());
        assert!(result.tested_services.is_empty());
        assert!(result.blocked_services.is_empty());
    }

    #[test]
    fn test_diagnostic_result_serialization() {
        let result = DiagnosticResult {
            profile: DpiProfile {
                kind: DpiKind::SniTlsBlock,
                details: Some("SNI blocking detected".to_string()),
                candidate_families: vec![StrategyFamily::SniFrag, StrategyFamily::TlsFrag],
            },
            tested_services: vec!["youtube".to_string(), "discord".to_string()],
            blocked_services: vec!["youtube".to_string()],
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DiagnosticResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.profile.kind, DpiKind::SniTlsBlock);
        assert_eq!(deserialized.tested_services.len(), 2);
        assert_eq!(deserialized.blocked_services.len(), 1);
    }

    #[test]
    fn test_dpi_profile_default() {
        let profile = DpiProfile::default();
        
        assert_eq!(profile.kind, DpiKind::NoBlock);
        assert!(profile.details.is_none());
        assert!(profile.candidate_families.is_empty());
    }

    // ========================================================================
    // AppStatus Tests
    // ========================================================================

    #[test]
    fn test_app_status_default() {
        let status = AppStatus::default();
        
        assert!(!status.is_active);
        assert!(status.current_strategy.is_none());
        assert!(status.current_strategy_name.is_none());
        assert!(status.services_status.is_empty());
    }

    #[test]
    fn test_app_status_active() {
        let mut services = HashMap::new();
        services.insert("youtube".to_string(), true);
        services.insert("discord".to_string(), false);
        
        let status = AppStatus {
            is_active: true,
            current_strategy: Some("strategy-1".to_string()),
            current_strategy_name: Some("YouTube Fix".to_string()),
            services_status: services,
        };
        
        assert!(status.is_active);
        assert_eq!(status.current_strategy, Some("strategy-1".to_string()));
        assert_eq!(status.services_status.get("youtube"), Some(&true));
        assert_eq!(status.services_status.get("discord"), Some(&false));
    }

    #[test]
    fn test_app_status_serialization() {
        let status = AppStatus {
            is_active: true,
            current_strategy: Some("test".to_string()),
            current_strategy_name: Some("Test Strategy".to_string()),
            services_status: HashMap::new(),
        };
        
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: AppStatus = serde_json::from_str(&json).unwrap();
        
        assert!(deserialized.is_active);
        assert_eq!(deserialized.current_strategy, Some("test".to_string()));
    }

    // ========================================================================
    // DomainRoute and AppRoute Tests
    // ========================================================================

    #[test]
    fn test_domain_route_creation() {
        let route = DomainRoute {
            domain: "youtube.com".to_string(),
            proxy_id: "proxy-1".to_string(),
        };
        
        assert_eq!(route.domain, "youtube.com");
        assert_eq!(route.proxy_id, "proxy-1");
    }

    #[test]
    fn test_domain_route_serialization() {
        let route = DomainRoute {
            domain: "*.google.com".to_string(),
            proxy_id: "vless-proxy".to_string(),
        };
        
        let json = serde_json::to_string(&route).unwrap();
        let deserialized: DomainRoute = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.domain, "*.google.com");
        assert_eq!(deserialized.proxy_id, "vless-proxy");
    }

    #[test]
    fn test_app_route_creation() {
        let route = AppRoute {
            app_name: "Chrome".to_string(),
            app_path: "C:\\Program Files\\Google\\Chrome\\chrome.exe".to_string(),
            proxy_id: "proxy-2".to_string(),
        };
        
        assert_eq!(route.app_name, "Chrome");
        assert!(route.app_path.contains("chrome.exe"));
        assert_eq!(route.proxy_id, "proxy-2");
    }

    #[test]
    fn test_app_route_serialization() {
        let route = AppRoute {
            app_name: "Firefox".to_string(),
            app_path: "/usr/bin/firefox".to_string(),
            proxy_id: "socks-proxy".to_string(),
        };
        
        let json = serde_json::to_string(&route).unwrap();
        let deserialized: AppRoute = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.app_name, "Firefox");
        assert_eq!(deserialized.app_path, "/usr/bin/firefox");
        assert_eq!(deserialized.proxy_id, "socks-proxy");
    }

    #[test]
    fn test_domain_route_with_wildcards() {
        let routes = vec![
            DomainRoute { domain: "*.youtube.com".to_string(), proxy_id: "p1".to_string() },
            DomainRoute { domain: "discord.gg".to_string(), proxy_id: "p2".to_string() },
            DomainRoute { domain: "*.*.example.com".to_string(), proxy_id: "p3".to_string() },
        ];
        
        for route in routes {
            let json = serde_json::to_string(&route).unwrap();
            let _: DomainRoute = serde_json::from_str(&json).unwrap();
        }
    }

    // ========================================================================
    // ServiceTestSummary Tests
    // ========================================================================

    #[test]
    fn test_service_test_summary_serialization() {
        let summary = ServiceTestSummary {
            service_id: "youtube".to_string(),
            total_tests: 5,
            passed_tests: 4,
            success_rate: 0.8,
            avg_latency_ms: 150.5,
            errors: vec![ErrorType::Timeout],
        };
        
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: ServiceTestSummary = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.service_id, "youtube");
        assert_eq!(deserialized.total_tests, 5);
        assert_eq!(deserialized.passed_tests, 4);
        assert!((deserialized.success_rate - 0.8).abs() < f64::EPSILON);
    }
}

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
    /// Per-domain routing rules
    #[serde(default)]
    pub per_domain_routing: Vec<DomainRoute>,
    /// Per-application routing rules
    #[serde(default)]
    pub per_app_routing: Vec<AppRoute>,
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
            per_domain_routing: Vec::new(),
            per_app_routing: Vec::new(),
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

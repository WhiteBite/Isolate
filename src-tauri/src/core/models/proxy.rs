//! Proxy-related models

#![allow(dead_code)] // Public proxy models

use crate::core::errors::IsolateError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn from_url(url: &str) -> Result<Self, IsolateError> {
        // Check protocol
        if !url.starts_with("vless://") {
            return Err(IsolateError::Validation("URL must start with vless://".to_string()));
        }

        let url = &url[8..]; // Remove "vless://"

        // Split by # to get name
        let (main_part, name) = if let Some(hash_pos) = url.rfind('#') {
            let name = urlencoding::decode(&url[hash_pos + 1..])
                .map_err(|e| IsolateError::Validation(format!("Failed to decode name: {}", e)))?
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
            .ok_or_else(|| IsolateError::Validation("Invalid URL format: missing @".to_string()))?;

        let uuid = address_part[..at_pos].to_string();
        if uuid.is_empty() {
            return Err(IsolateError::Validation("UUID cannot be empty".to_string()));
        }

        let server_port = &address_part[at_pos + 1..];
        let colon_pos = server_port
            .rfind(':')
            .ok_or_else(|| IsolateError::Validation("Invalid URL format: missing port".to_string()))?;

        let server = server_port[..colon_pos].to_string();
        if server.is_empty() {
            return Err(IsolateError::Validation("Server cannot be empty".to_string()));
        }

        let port: u16 = server_port[colon_pos + 1..]
            .parse()
            .map_err(|_| IsolateError::Validation("Invalid port number".to_string()))?;

        // Parse query params
        let mut security = "tls".to_string();
        let mut sni: Option<String> = None;
        let mut flow: Option<String> = None;

        if let Some(params) = params_str {
            for param in params.split('&') {
                if let Some(eq_pos) = param.find('=') {
                    let key = &param[..eq_pos];
                    let value = urlencoding::decode(&param[eq_pos + 1..])
                        .map_err(|e| IsolateError::Validation(format!("Failed to decode param {}: {}", key, e)))?
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_domain_route_creation() {
        let route = DomainRoute {
            domain: "youtube.com".to_string(),
            proxy_id: "proxy-1".to_string(),
        };
        
        assert_eq!(route.domain, "youtube.com");
        assert_eq!(route.proxy_id, "proxy-1");
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
    fn test_vless_config_from_url_invalid_protocol() {
        let result = VlessConfig::from_url("vmess://uuid@server:443");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("vless://"));
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
        assert!(url.contains("sni=example.com"));
        assert!(url.contains("#Test%20Server"));
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
}

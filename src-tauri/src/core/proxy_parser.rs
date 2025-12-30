//! Proxy URL parser for all supported protocols
//!
//! Supports parsing URLs for:
//! - VLESS: vless://uuid@server:port?params#name
//! - VMess: vmess://base64(json)
//! - Shadowsocks: ss://base64(method:password)@server:port#name
//! - Trojan: trojan://password@server:port?params#name
//! - TUIC: tuic://uuid:password@server:port?params#name
//! - Hysteria/Hysteria2: hysteria2://auth@server:port?params#name
//! - SOCKS5: socks5://user:pass@server:port
//! - HTTP/HTTPS: http://user:pass@server:port

use crate::core::models::{ProxyConfig, ProxyProtocol};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;

/// Parses a proxy URL and returns ProxyConfig
pub fn parse_proxy_url(url: &str) -> Result<ProxyConfig> {
    let url = url.trim();

    if url.starts_with("vless://") {
        parse_vless_url(url)
    } else if url.starts_with("vmess://") {
        parse_vmess_url(url)
    } else if url.starts_with("ss://") {
        parse_shadowsocks_url(url)
    } else if url.starts_with("trojan://") {
        parse_trojan_url(url)
    } else if url.starts_with("tuic://") {
        parse_tuic_url(url)
    } else if url.starts_with("hysteria://") {
        parse_hysteria_url(url, false)
    } else if url.starts_with("hysteria2://") || url.starts_with("hy2://") {
        parse_hysteria_url(url, true)
    } else if url.starts_with("socks5://") || url.starts_with("socks://") {
        parse_socks_url(url)
    } else if url.starts_with("http://") || url.starts_with("https://") {
        parse_http_url(url)
    } else {
        Err(anyhow!("Unsupported proxy protocol: {}", url.split("://").next().unwrap_or("unknown")))
    }
}

/// Parses subscription content and returns list of ProxyConfig
pub fn parse_subscription(content: &str) -> Result<Vec<ProxyConfig>> {
    let content = content.trim();

    // Try base64 decode first
    if let Ok(decoded) = general_purpose::STANDARD.decode(content) {
        if let Ok(decoded_str) = String::from_utf8(decoded) {
            return parse_subscription_content(&decoded_str);
        }
    }

    // Try URL-safe base64
    if let Ok(decoded) = general_purpose::URL_SAFE.decode(content) {
        if let Ok(decoded_str) = String::from_utf8(decoded) {
            return parse_subscription_content(&decoded_str);
        }
    }

    // Try as plain text
    parse_subscription_content(content)
}

fn parse_subscription_content(content: &str) -> Result<Vec<ProxyConfig>> {
    let content = content.trim();

    // Try JSON array
    if content.starts_with('[') {
        if let Ok(configs) = serde_json::from_str::<Vec<ProxyConfig>>(content) {
            return Ok(configs);
        }
    }

    // Parse line by line
    let mut configs = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        match parse_proxy_url(line) {
            Ok(config) => configs.push(config),
            Err(e) => {
                tracing::warn!("Failed to parse proxy URL '{}': {}", truncate_url(line), e);
            }
        }
    }

    if configs.is_empty() {
        Err(anyhow!("No valid proxy configurations found in subscription"))
    } else {
        Ok(configs)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generates unique ID for proxy config
fn generate_id(protocol: &str) -> String {
    let uuid_part = uuid::Uuid::new_v4()
        .to_string()
        .split('-')
        .next()
        .unwrap_or("unknown")
        .to_string();
    format!("{}_{}", protocol, uuid_part)
}

/// Extracts name from URL fragment or generates from server:port
fn extract_name(fragment: Option<&str>, server: &str, port: u16) -> String {
    if let Some(name) = fragment {
        if !name.is_empty() {
            return urlencoding::decode(name)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| name.to_string());
        }
    }
    format!("{}:{}", server, port)
}

/// Parses query parameters from URL
fn parse_query_params(query: Option<&str>) -> HashMap<String, String> {
    let mut params = HashMap::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let decoded_value = urlencoding::decode(value)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| value.to_string());
                params.insert(key.to_string(), decoded_value);
            }
        }
    }
    params
}

/// Truncates URL for logging (hides sensitive parts)
fn truncate_url(url: &str) -> String {
    if url.len() > 50 {
        format!("{}...", &url[..50])
    } else {
        url.to_string()
    }
}

// ============================================================================
// VLESS Parser
// ============================================================================

/// Parses VLESS URL: vless://uuid@server:port?params#name
fn parse_vless_url(url: &str) -> Result<ProxyConfig> {
    let url_obj = url::Url::parse(url)?;

    let uuid = url_obj.username().to_string();
    if uuid.is_empty() {
        return Err(anyhow!("VLESS URL missing UUID"));
    }

    let server = url_obj
        .host_str()
        .ok_or_else(|| anyhow!("VLESS URL missing server"))?
        .to_string();

    let port = url_obj.port().unwrap_or(443);
    let params = parse_query_params(url_obj.query());
    let name = extract_name(url_obj.fragment(), &server, port);

    let security = params.get("security").map(|s| s.as_str()).unwrap_or("tls");
    let tls = security == "tls" || security == "reality";

    let mut custom_fields = HashMap::new();

    // Flow (xtls-rprx-vision, etc.)
    if let Some(flow) = params.get("flow") {
        custom_fields.insert("flow".to_string(), flow.clone());
    }

    // Transport type
    if let Some(transport_type) = params.get("type") {
        custom_fields.insert("transport_type".to_string(), transport_type.clone());
    }

    // Reality params
    if let Some(pbk) = params.get("pbk") {
        custom_fields.insert("public_key".to_string(), pbk.clone());
    }
    if let Some(sid) = params.get("sid") {
        custom_fields.insert("short_id".to_string(), sid.clone());
    }
    if let Some(fp) = params.get("fp") {
        custom_fields.insert("fingerprint".to_string(), fp.clone());
    }

    // WebSocket params
    if let Some(path) = params.get("path") {
        custom_fields.insert("path".to_string(), path.clone());
    }
    if let Some(host) = params.get("host") {
        custom_fields.insert("host".to_string(), host.clone());
    }

    Ok(ProxyConfig {
        id: generate_id("vless"),
        name,
        protocol: ProxyProtocol::Vless,
        server,
        port,
        username: None,
        password: None,
        uuid: Some(uuid),
        tls,
        sni: params.get("sni").cloned(),
        transport: params.get("type").cloned(),
        custom_fields,
        active: false,
    })
}

// ============================================================================
// VMess Parser
// ============================================================================

/// Parses VMess URL: vmess://base64(json)
fn parse_vmess_url(url: &str) -> Result<ProxyConfig> {
    let encoded = url.strip_prefix("vmess://").ok_or_else(|| anyhow!("Invalid VMess URL"))?;

    // Decode base64
    let decoded = general_purpose::STANDARD
        .decode(encoded.trim())
        .or_else(|_| general_purpose::URL_SAFE.decode(encoded.trim()))
        .map_err(|e| anyhow!("Failed to decode VMess base64: {}", e))?;

    let json_str = String::from_utf8(decoded).map_err(|e| anyhow!("Invalid UTF-8 in VMess: {}", e))?;

    // Parse JSON
    let json: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| anyhow!("Invalid VMess JSON: {}", e))?;

    let server = json["add"]
        .as_str()
        .or_else(|| json["address"].as_str())
        .ok_or_else(|| anyhow!("VMess missing server address"))?
        .to_string();

    let port = json["port"]
        .as_u64()
        .or_else(|| json["port"].as_str().and_then(|s| s.parse().ok()))
        .ok_or_else(|| anyhow!("VMess missing port"))? as u16;

    let uuid = json["id"]
        .as_str()
        .ok_or_else(|| anyhow!("VMess missing UUID"))?
        .to_string();

    let name = json["ps"]
        .as_str()
        .or_else(|| json["remarks"].as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}:{}", server, port));

    let tls = json["tls"].as_str().map(|s| s == "tls").unwrap_or(false);

    let mut custom_fields = HashMap::new();

    // Alter ID
    if let Some(aid) = json["aid"].as_u64().or_else(|| json["alterId"].as_u64()) {
        custom_fields.insert("alter_id".to_string(), aid.to_string());
    }

    // Network/Transport
    if let Some(net) = json["net"].as_str().or_else(|| json["network"].as_str()) {
        custom_fields.insert("network".to_string(), net.to_string());
    }

    // Path for WebSocket
    if let Some(path) = json["path"].as_str() {
        custom_fields.insert("path".to_string(), path.to_string());
    }

    // Host
    if let Some(host) = json["host"].as_str() {
        custom_fields.insert("host".to_string(), host.to_string());
    }

    Ok(ProxyConfig {
        id: generate_id("vmess"),
        name,
        protocol: ProxyProtocol::Vmess,
        server,
        port,
        username: None,
        password: None,
        uuid: Some(uuid),
        tls,
        sni: json["sni"].as_str().map(|s| s.to_string()),
        transport: json["net"].as_str().or_else(|| json["network"].as_str()).map(|s| s.to_string()),
        custom_fields,
        active: false,
    })
}

// ============================================================================
// Shadowsocks Parser
// ============================================================================

/// Parses Shadowsocks URL: ss://base64(method:password)@server:port#name
/// Also supports: ss://base64(method:password@server:port)#name (SIP002)
fn parse_shadowsocks_url(url: &str) -> Result<ProxyConfig> {
    let url_str = url.strip_prefix("ss://").ok_or_else(|| anyhow!("Invalid Shadowsocks URL"))?;

    // Extract fragment (name)
    let (main_part, fragment) = if let Some(hash_pos) = url_str.rfind('#') {
        (&url_str[..hash_pos], Some(&url_str[hash_pos + 1..]))
    } else {
        (url_str, None)
    };

    // Try SIP002 format: base64(method:password)@server:port
    if let Some(at_pos) = main_part.rfind('@') {
        let encoded_part = &main_part[..at_pos];
        let server_part = &main_part[at_pos + 1..];

        // Decode method:password
        let decoded = general_purpose::STANDARD
            .decode(encoded_part)
            .or_else(|_| general_purpose::URL_SAFE.decode(encoded_part))
            .map_err(|e| anyhow!("Failed to decode Shadowsocks base64: {}", e))?;

        let method_pass = String::from_utf8(decoded).map_err(|e| anyhow!("Invalid UTF-8: {}", e))?;

        let (method, password) = method_pass
            .split_once(':')
            .ok_or_else(|| anyhow!("Invalid Shadowsocks format: missing method:password"))?;

        // Parse server:port
        let (server, port) = parse_server_port(server_part)?;
        let name = extract_name(fragment, &server, port);

        let mut custom_fields = HashMap::new();
        custom_fields.insert("method".to_string(), method.to_string());

        return Ok(ProxyConfig {
            id: generate_id("ss"),
            name,
            protocol: ProxyProtocol::Shadowsocks,
            server,
            port,
            username: None,
            password: Some(password.to_string()),
            uuid: None,
            tls: false,
            sni: None,
            transport: None,
            custom_fields,
            active: false,
        });
    }

    // Try legacy format: base64(method:password@server:port)
    let decoded = general_purpose::STANDARD
        .decode(main_part)
        .or_else(|_| general_purpose::URL_SAFE.decode(main_part))
        .map_err(|e| anyhow!("Failed to decode Shadowsocks base64: {}", e))?;

    let decoded_str = String::from_utf8(decoded).map_err(|e| anyhow!("Invalid UTF-8: {}", e))?;

    let at_pos = decoded_str
        .rfind('@')
        .ok_or_else(|| anyhow!("Invalid Shadowsocks format"))?;

    let method_pass = &decoded_str[..at_pos];
    let server_part = &decoded_str[at_pos + 1..];

    let (method, password) = method_pass
        .split_once(':')
        .ok_or_else(|| anyhow!("Invalid Shadowsocks format: missing method:password"))?;

    let (server, port) = parse_server_port(server_part)?;
    let name = extract_name(fragment, &server, port);

    let mut custom_fields = HashMap::new();
    custom_fields.insert("method".to_string(), method.to_string());

    Ok(ProxyConfig {
        id: generate_id("ss"),
        name,
        protocol: ProxyProtocol::Shadowsocks,
        server,
        port,
        username: None,
        password: Some(password.to_string()),
        uuid: None,
        tls: false,
        sni: None,
        transport: None,
        custom_fields,
        active: false,
    })
}

/// Parses server:port string
fn parse_server_port(s: &str) -> Result<(String, u16)> {
    // Handle IPv6: [::1]:port
    if s.starts_with('[') {
        let bracket_end = s.find(']').ok_or_else(|| anyhow!("Invalid IPv6 format"))?;
        let server = s[1..bracket_end].to_string();
        let port_str = &s[bracket_end + 1..];
        let port: u16 = port_str
            .strip_prefix(':')
            .ok_or_else(|| anyhow!("Missing port after IPv6"))?
            .parse()
            .map_err(|_| anyhow!("Invalid port"))?;
        return Ok((server, port));
    }

    // IPv4 or hostname
    let colon_pos = s.rfind(':').ok_or_else(|| anyhow!("Missing port"))?;
    let server = s[..colon_pos].to_string();
    let port: u16 = s[colon_pos + 1..]
        .parse()
        .map_err(|_| anyhow!("Invalid port"))?;

    Ok((server, port))
}

// ============================================================================
// Trojan Parser
// ============================================================================

/// Parses Trojan URL: trojan://password@server:port?params#name
fn parse_trojan_url(url: &str) -> Result<ProxyConfig> {
    let url_obj = url::Url::parse(url)?;

    let password = url_obj.username().to_string();
    if password.is_empty() {
        return Err(anyhow!("Trojan URL missing password"));
    }

    let server = url_obj
        .host_str()
        .ok_or_else(|| anyhow!("Trojan URL missing server"))?
        .to_string();

    let port = url_obj.port().unwrap_or(443);
    let params = parse_query_params(url_obj.query());
    let name = extract_name(url_obj.fragment(), &server, port);

    let security = params.get("security").map(|s| s.as_str()).unwrap_or("tls");
    let tls = security != "none";

    let mut custom_fields = HashMap::new();

    // Transport type
    if let Some(transport_type) = params.get("type") {
        custom_fields.insert("transport_type".to_string(), transport_type.clone());
    }

    // WebSocket params
    if let Some(path) = params.get("path") {
        custom_fields.insert("path".to_string(), path.clone());
    }
    if let Some(host) = params.get("host") {
        custom_fields.insert("host".to_string(), host.clone());
    }

    // Fingerprint
    if let Some(fp) = params.get("fp") {
        custom_fields.insert("fingerprint".to_string(), fp.clone());
    }

    // ALPN
    if let Some(alpn) = params.get("alpn") {
        custom_fields.insert("alpn".to_string(), alpn.clone());
    }

    Ok(ProxyConfig {
        id: generate_id("trojan"),
        name,
        protocol: ProxyProtocol::Trojan,
        server,
        port,
        username: None,
        password: Some(urlencoding::decode(&password).unwrap_or(password.clone().into()).to_string()),
        uuid: None,
        tls,
        sni: params.get("sni").cloned(),
        transport: params.get("type").cloned(),
        custom_fields,
        active: false,
    })
}

// ============================================================================
// TUIC Parser
// ============================================================================

/// Parses TUIC URL: tuic://uuid:password@server:port?params#name
fn parse_tuic_url(url: &str) -> Result<ProxyConfig> {
    let url_obj = url::Url::parse(url)?;

    let uuid = url_obj.username().to_string();
    if uuid.is_empty() {
        return Err(anyhow!("TUIC URL missing UUID"));
    }

    let password = url_obj.password().map(|s| s.to_string());

    let server = url_obj
        .host_str()
        .ok_or_else(|| anyhow!("TUIC URL missing server"))?
        .to_string();

    let port = url_obj.port().unwrap_or(443);
    let params = parse_query_params(url_obj.query());
    let name = extract_name(url_obj.fragment(), &server, port);

    let mut custom_fields = HashMap::new();

    // Congestion control
    if let Some(cc) = params.get("congestion_control").or_else(|| params.get("cc")) {
        custom_fields.insert("congestion_control".to_string(), cc.clone());
    }

    // UDP relay mode
    if let Some(udp) = params.get("udp_relay_mode") {
        custom_fields.insert("udp_relay_mode".to_string(), udp.clone());
    }

    // ALPN
    if let Some(alpn) = params.get("alpn") {
        custom_fields.insert("alpn".to_string(), alpn.clone());
    }

    // Disable SNI
    if let Some(disable_sni) = params.get("disable_sni") {
        custom_fields.insert("disable_sni".to_string(), disable_sni.clone());
    }

    Ok(ProxyConfig {
        id: generate_id("tuic"),
        name,
        protocol: ProxyProtocol::Tuic,
        server,
        port,
        username: None,
        password,
        uuid: Some(uuid),
        tls: true, // TUIC always uses TLS
        sni: params.get("sni").cloned(),
        transport: Some("quic".to_string()),
        custom_fields,
        active: false,
    })
}

// ============================================================================
// Hysteria Parser
// ============================================================================

/// Parses Hysteria/Hysteria2 URL: hysteria2://auth@server:port?params#name
fn parse_hysteria_url(url: &str, is_v2: bool) -> Result<ProxyConfig> {
    // Normalize URL prefix
    let normalized = if url.starts_with("hy2://") {
        url.replacen("hy2://", "hysteria2://", 1)
    } else {
        url.to_string()
    };

    let url_obj = url::Url::parse(&normalized)?;

    let auth = url_obj.username().to_string();
    let password = if auth.is_empty() {
        url_obj.password().map(|s| s.to_string())
    } else {
        Some(auth)
    };

    let server = url_obj
        .host_str()
        .ok_or_else(|| anyhow!("Hysteria URL missing server"))?
        .to_string();

    let port = url_obj.port().unwrap_or(443);
    let params = parse_query_params(url_obj.query());
    let name = extract_name(url_obj.fragment(), &server, port);

    let mut custom_fields = HashMap::new();

    // Obfuscation
    if let Some(obfs) = params.get("obfs") {
        custom_fields.insert("obfs".to_string(), obfs.clone());
    }
    if let Some(obfs_password) = params.get("obfs-password").or_else(|| params.get("obfsPassword")) {
        custom_fields.insert("obfs_password".to_string(), obfs_password.clone());
    }

    // Bandwidth (Hysteria v1)
    if let Some(up) = params.get("up").or_else(|| params.get("upmbps")) {
        custom_fields.insert("up_mbps".to_string(), up.clone());
    }
    if let Some(down) = params.get("down").or_else(|| params.get("downmbps")) {
        custom_fields.insert("down_mbps".to_string(), down.clone());
    }

    // ALPN
    if let Some(alpn) = params.get("alpn") {
        custom_fields.insert("alpn".to_string(), alpn.clone());
    }

    // Insecure
    if let Some(insecure) = params.get("insecure") {
        custom_fields.insert("insecure".to_string(), insecure.clone());
    }

    let protocol = if is_v2 {
        ProxyProtocol::Hysteria2
    } else {
        ProxyProtocol::Hysteria
    };

    Ok(ProxyConfig {
        id: generate_id(if is_v2 { "hy2" } else { "hysteria" }),
        name,
        protocol,
        server,
        port,
        username: None,
        password,
        uuid: None,
        tls: true, // Hysteria always uses TLS/QUIC
        sni: params.get("sni").cloned(),
        transport: Some("quic".to_string()),
        custom_fields,
        active: false,
    })
}

// ============================================================================
// SOCKS5 Parser
// ============================================================================

/// Parses SOCKS5 URL: socks5://user:pass@server:port or socks://user:pass@server:port
fn parse_socks_url(url: &str) -> Result<ProxyConfig> {
    let url_obj = url::Url::parse(url)?;

    let server = url_obj
        .host_str()
        .ok_or_else(|| anyhow!("SOCKS URL missing server"))?
        .to_string();

    let port = url_obj.port().unwrap_or(1080);

    let username = if url_obj.username().is_empty() {
        None
    } else {
        Some(
            urlencoding::decode(url_obj.username())
                .unwrap_or(url_obj.username().into())
                .to_string(),
        )
    };

    let password = url_obj.password().map(|p| {
        urlencoding::decode(p)
            .unwrap_or(p.into())
            .to_string()
    });

    let name = extract_name(url_obj.fragment(), &server, port);

    Ok(ProxyConfig {
        id: generate_id("socks5"),
        name,
        protocol: ProxyProtocol::Socks5,
        server,
        port,
        username,
        password,
        uuid: None,
        tls: false,
        sni: None,
        transport: None,
        custom_fields: HashMap::new(),
        active: false,
    })
}

// ============================================================================
// HTTP/HTTPS Parser
// ============================================================================

/// Parses HTTP/HTTPS proxy URL: http://user:pass@server:port
fn parse_http_url(url: &str) -> Result<ProxyConfig> {
    let url_obj = url::Url::parse(url)?;

    let server = url_obj
        .host_str()
        .ok_or_else(|| anyhow!("HTTP proxy URL missing server"))?
        .to_string();

    let is_https = url_obj.scheme() == "https";
    let default_port = if is_https { 443 } else { 8080 };
    let port = url_obj.port().unwrap_or(default_port);

    let username = if url_obj.username().is_empty() {
        None
    } else {
        Some(
            urlencoding::decode(url_obj.username())
                .unwrap_or(url_obj.username().into())
                .to_string(),
        )
    };

    let password = url_obj.password().map(|p| {
        urlencoding::decode(p)
            .unwrap_or(p.into())
            .to_string()
    });

    let name = extract_name(url_obj.fragment(), &server, port);

    let protocol = if is_https {
        ProxyProtocol::Https
    } else {
        ProxyProtocol::Http
    };

    Ok(ProxyConfig {
        id: generate_id(if is_https { "https" } else { "http" }),
        name,
        protocol,
        server,
        port,
        username,
        password,
        uuid: None,
        tls: is_https,
        sni: None,
        transport: None,
        custom_fields: HashMap::new(),
        active: false,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vless_url() {
        let url = "vless://550e8400-e29b-41d4-a716-446655440000@example.com:443?security=tls&sni=example.com&flow=xtls-rprx-vision#Test%20Server";
        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Vless);
        assert_eq!(config.server, "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.uuid, Some("550e8400-e29b-41d4-a716-446655440000".to_string()));
        assert!(config.tls);
        assert_eq!(config.name, "Test Server");
    }

    #[test]
    fn test_parse_socks_url() {
        let url = "socks5://user:pass@127.0.0.1:1080";
        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Socks5);
        assert_eq!(config.server, "127.0.0.1");
        assert_eq!(config.port, 1080);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
    }

    #[test]
    fn test_parse_trojan_url() {
        let url = "trojan://password123@server.com:443?sni=server.com#MyTrojan";
        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Trojan);
        assert_eq!(config.server, "server.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.password, Some("password123".to_string()));
        assert!(config.tls);
    }

    #[test]
    fn test_unsupported_protocol() {
        let url = "unknown://server:1234";
        assert!(parse_proxy_url(url).is_err());
    }
}

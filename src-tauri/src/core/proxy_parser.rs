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
// Export Functions
// ============================================================================

/// Exports ProxyConfig to URL format
pub fn export_proxy_url(proxy: &ProxyConfig) -> Result<String> {
    match proxy.protocol {
        ProxyProtocol::Vless => export_vless_url(proxy),
        ProxyProtocol::Vmess => export_vmess_url(proxy),
        ProxyProtocol::Shadowsocks => export_shadowsocks_url(proxy),
        ProxyProtocol::Trojan => export_trojan_url(proxy),
        ProxyProtocol::Tuic => export_tuic_url(proxy),
        ProxyProtocol::Hysteria | ProxyProtocol::Hysteria2 => export_hysteria_url(proxy),
        ProxyProtocol::Socks5 => export_socks_url(proxy),
        ProxyProtocol::Http | ProxyProtocol::Https => export_http_url(proxy),
        _ => Err(anyhow!("Protocol {:?} export not supported", proxy.protocol)),
    }
}

fn export_vless_url(proxy: &ProxyConfig) -> Result<String> {
    let uuid = proxy.uuid.as_ref().ok_or_else(|| anyhow!("VLESS requires UUID"))?;
    let mut params = vec![];
    
    if proxy.tls {
        params.push("security=tls".to_string());
    }
    if let Some(ref sni) = proxy.sni {
        params.push(format!("sni={}", urlencoding::encode(sni)));
    }
    if let Some(flow) = proxy.custom_fields.get("flow") {
        params.push(format!("flow={}", urlencoding::encode(flow)));
    }
    if let Some(ref transport) = proxy.transport {
        params.push(format!("type={}", urlencoding::encode(transport)));
    }
    if let Some(path) = proxy.custom_fields.get("path") {
        params.push(format!("path={}", urlencoding::encode(path)));
    }
    if let Some(host) = proxy.custom_fields.get("host") {
        params.push(format!("host={}", urlencoding::encode(host)));
    }
    if let Some(pbk) = proxy.custom_fields.get("public_key") {
        params.push(format!("pbk={}", urlencoding::encode(pbk)));
    }
    if let Some(sid) = proxy.custom_fields.get("short_id") {
        params.push(format!("sid={}", urlencoding::encode(sid)));
    }
    if let Some(fp) = proxy.custom_fields.get("fingerprint") {
        params.push(format!("fp={}", urlencoding::encode(fp)));
    }
    
    let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    let fragment = urlencoding::encode(&proxy.name);
    
    Ok(format!("vless://{}@{}:{}{}#{}", uuid, proxy.server, proxy.port, query, fragment))
}

fn export_vmess_url(proxy: &ProxyConfig) -> Result<String> {
    let uuid = proxy.uuid.as_ref().ok_or_else(|| anyhow!("VMess requires UUID"))?;
    
    let mut json = serde_json::json!({
        "v": "2",
        "ps": proxy.name,
        "add": proxy.server,
        "port": proxy.port,
        "id": uuid,
        "aid": proxy.custom_fields.get("alter_id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0),
        "net": proxy.transport.as_deref().unwrap_or("tcp"),
        "tls": if proxy.tls { "tls" } else { "" }
    });
    
    if let Some(sni) = &proxy.sni {
        json["sni"] = serde_json::json!(sni);
    }
    if let Some(path) = proxy.custom_fields.get("path") {
        json["path"] = serde_json::json!(path);
    }
    if let Some(host) = proxy.custom_fields.get("host") {
        json["host"] = serde_json::json!(host);
    }
    
    let encoded = general_purpose::STANDARD.encode(json.to_string());
    Ok(format!("vmess://{}", encoded))
}

fn export_shadowsocks_url(proxy: &ProxyConfig) -> Result<String> {
    let password = proxy.password.as_ref().ok_or_else(|| anyhow!("Shadowsocks requires password"))?;
    let method = proxy.custom_fields.get("method").map(|s| s.as_str()).unwrap_or("aes-256-gcm");
    
    let user_info = format!("{}:{}", method, password);
    let encoded = general_purpose::STANDARD.encode(&user_info);
    let fragment = urlencoding::encode(&proxy.name);
    
    Ok(format!("ss://{}@{}:{}#{}", encoded, proxy.server, proxy.port, fragment))
}

fn export_trojan_url(proxy: &ProxyConfig) -> Result<String> {
    let password = proxy.password.as_ref().ok_or_else(|| anyhow!("Trojan requires password"))?;
    let mut params = vec![];
    
    if let Some(ref sni) = proxy.sni {
        params.push(format!("sni={}", urlencoding::encode(sni)));
    }
    if let Some(ref transport) = proxy.transport {
        params.push(format!("type={}", urlencoding::encode(transport)));
    }
    
    let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    let fragment = urlencoding::encode(&proxy.name);
    
    Ok(format!("trojan://{}@{}:{}{}#{}", urlencoding::encode(password), proxy.server, proxy.port, query, fragment))
}

fn export_tuic_url(proxy: &ProxyConfig) -> Result<String> {
    let uuid = proxy.uuid.as_ref().ok_or_else(|| anyhow!("TUIC requires UUID"))?;
    let mut params = vec![];
    
    if let Some(ref sni) = proxy.sni {
        params.push(format!("sni={}", urlencoding::encode(sni)));
    }
    if let Some(cc) = proxy.custom_fields.get("congestion_control") {
        params.push(format!("congestion_control={}", urlencoding::encode(cc)));
    }
    
    let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    let fragment = urlencoding::encode(&proxy.name);
    let password_part = proxy.password.as_ref().map(|p| format!(":{}", p)).unwrap_or_default();
    
    Ok(format!("tuic://{}{}@{}:{}{}#{}", uuid, password_part, proxy.server, proxy.port, query, fragment))
}

fn export_hysteria_url(proxy: &ProxyConfig) -> Result<String> {
    let prefix = if proxy.protocol == ProxyProtocol::Hysteria2 { "hysteria2" } else { "hysteria" };
    let mut params = vec![];
    
    if let Some(ref sni) = proxy.sni {
        params.push(format!("sni={}", urlencoding::encode(sni)));
    }
    if let Some(obfs) = proxy.custom_fields.get("obfs") {
        params.push(format!("obfs={}", urlencoding::encode(obfs)));
    }
    
    let query = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    let fragment = urlencoding::encode(&proxy.name);
    let auth = proxy.password.as_ref().map(|p| format!("{}@", p)).unwrap_or_default();
    
    Ok(format!("{}://{}{}:{}{}#{}", prefix, auth, proxy.server, proxy.port, query, fragment))
}

fn export_socks_url(proxy: &ProxyConfig) -> Result<String> {
    let auth = match (&proxy.username, &proxy.password) {
        (Some(u), Some(p)) => format!("{}:{}@", urlencoding::encode(u), urlencoding::encode(p)),
        (Some(u), None) => format!("{}@", urlencoding::encode(u)),
        _ => String::new(),
    };
    let fragment = if proxy.name.is_empty() || proxy.name == format!("{}:{}", proxy.server, proxy.port) {
        String::new()
    } else {
        format!("#{}", urlencoding::encode(&proxy.name))
    };
    
    Ok(format!("socks5://{}{}:{}{}", auth, proxy.server, proxy.port, fragment))
}

fn export_http_url(proxy: &ProxyConfig) -> Result<String> {
    let scheme = if proxy.protocol == ProxyProtocol::Https { "https" } else { "http" };
    let auth = match (&proxy.username, &proxy.password) {
        (Some(u), Some(p)) => format!("{}:{}@", urlencoding::encode(u), urlencoding::encode(p)),
        (Some(u), None) => format!("{}@", urlencoding::encode(u)),
        _ => String::new(),
    };
    
    Ok(format!("{}://{}{}:{}", scheme, auth, proxy.server, proxy.port))
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

    // ========================================================================
    // VMess Tests
    // ========================================================================

    #[test]
    fn test_parse_vmess_url_basic() {
        // VMess JSON: aid as number (not string)
        let json = r#"{"add":"server.com","port":"443","id":"550e8400-e29b-41d4-a716-446655440000","aid":64,"net":"tcp","tls":"tls","ps":"Test VMess"}"#;
        let encoded = general_purpose::STANDARD.encode(json);
        let url = format!("vmess://{}", encoded);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Vmess);
        assert_eq!(config.server, "server.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.uuid, Some("550e8400-e29b-41d4-a716-446655440000".to_string()));
        assert!(config.tls);
        assert_eq!(config.name, "Test VMess");
        assert_eq!(config.custom_fields.get("alter_id"), Some(&"64".to_string()));
        assert_eq!(config.custom_fields.get("network"), Some(&"tcp".to_string()));
    }

    #[test]
    fn test_parse_vmess_url_websocket() {
        let json = r#"{"add":"ws.example.com","port":"80","id":"test-uuid-1234","aid":"0","net":"ws","path":"/ws","host":"cdn.example.com","ps":"WS Server"}"#;
        let encoded = general_purpose::STANDARD.encode(json);
        let url = format!("vmess://{}", encoded);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Vmess);
        assert_eq!(config.server, "ws.example.com");
        assert_eq!(config.port, 80);
        assert!(!config.tls);
        assert_eq!(config.transport, Some("ws".to_string()));
        assert_eq!(config.custom_fields.get("path"), Some(&"/ws".to_string()));
        assert_eq!(config.custom_fields.get("host"), Some(&"cdn.example.com".to_string()));
    }

    #[test]
    fn test_parse_vmess_url_with_address_field() {
        // Some VMess configs use "address" instead of "add"
        let json = r#"{"address":"alt.server.com","port":8443,"id":"uuid-test","ps":"Alt Server"}"#;
        let encoded = general_purpose::STANDARD.encode(json);
        let url = format!("vmess://{}", encoded);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.server, "alt.server.com");
        assert_eq!(config.port, 8443);
    }

    #[test]
    fn test_parse_vmess_url_port_as_number() {
        let json = r#"{"add":"server.com","port":443,"id":"uuid-123"}"#;
        let encoded = general_purpose::STANDARD.encode(json);
        let url = format!("vmess://{}", encoded);

        let config = parse_proxy_url(&url).unwrap();
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_parse_vmess_url_missing_uuid() {
        let json = r#"{"add":"server.com","port":"443"}"#;
        let encoded = general_purpose::STANDARD.encode(json);
        let url = format!("vmess://{}", encoded);

        assert!(parse_proxy_url(&url).is_err());
    }

    #[test]
    fn test_parse_vmess_url_missing_server() {
        let json = r#"{"port":"443","id":"uuid-123"}"#;
        let encoded = general_purpose::STANDARD.encode(json);
        let url = format!("vmess://{}", encoded);

        assert!(parse_proxy_url(&url).is_err());
    }

    #[test]
    fn test_parse_vmess_url_invalid_base64() {
        let url = "vmess://not-valid-base64!!!";
        assert!(parse_proxy_url(url).is_err());
    }

    #[test]
    fn test_parse_vmess_url_invalid_json() {
        let encoded = general_purpose::STANDARD.encode("not json");
        let url = format!("vmess://{}", encoded);
        assert!(parse_proxy_url(&url).is_err());
    }

    // ========================================================================
    // Shadowsocks Tests
    // ========================================================================

    #[test]
    fn test_parse_shadowsocks_sip002_format() {
        // SIP002: ss://base64(method:password)@server:port#name
        let method_pass = general_purpose::STANDARD.encode("aes-256-gcm:mypassword");
        let url = format!("ss://{}@ss.example.com:8388#SS%20Server", method_pass);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Shadowsocks);
        assert_eq!(config.server, "ss.example.com");
        assert_eq!(config.port, 8388);
        assert_eq!(config.password, Some("mypassword".to_string()));
        assert_eq!(config.custom_fields.get("method"), Some(&"aes-256-gcm".to_string()));
        assert_eq!(config.name, "SS Server");
    }

    #[test]
    fn test_parse_shadowsocks_legacy_format() {
        // Legacy: ss://base64(method:password@server:port)#name
        let full = general_purpose::STANDARD.encode("chacha20-ietf-poly1305:secretpass@legacy.server.com:1234");
        let url = format!("ss://{}#Legacy%20SS", full);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Shadowsocks);
        assert_eq!(config.server, "legacy.server.com");
        assert_eq!(config.port, 1234);
        assert_eq!(config.password, Some("secretpass".to_string()));
        assert_eq!(config.custom_fields.get("method"), Some(&"chacha20-ietf-poly1305".to_string()));
    }

    #[test]
    fn test_parse_shadowsocks_url_safe_base64() {
        // URL-safe base64 variant
        let method_pass = general_purpose::URL_SAFE.encode("aes-128-gcm:pass123");
        let url = format!("ss://{}@urlsafe.server.com:9999", method_pass);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.server, "urlsafe.server.com");
        assert_eq!(config.port, 9999);
    }

    #[test]
    fn test_parse_shadowsocks_no_name() {
        let method_pass = general_purpose::STANDARD.encode("aes-256-gcm:pass");
        let url = format!("ss://{}@noname.com:8388", method_pass);

        let config = parse_proxy_url(&url).unwrap();

        // Name should be generated from server:port
        assert_eq!(config.name, "noname.com:8388");
    }

    #[test]
    fn test_parse_shadowsocks_ipv6() {
        let full = general_purpose::STANDARD.encode("aes-256-gcm:pass@[::1]:8388");
        let url = format!("ss://{}", full);

        let config = parse_proxy_url(&url).unwrap();

        assert_eq!(config.server, "::1");
        assert_eq!(config.port, 8388);
    }

    #[test]
    fn test_parse_shadowsocks_invalid_format() {
        // Missing method:password separator
        let encoded = general_purpose::STANDARD.encode("invalidformat");
        let url = format!("ss://{}@server.com:8388", encoded);

        assert!(parse_proxy_url(&url).is_err());
    }

    // ========================================================================
    // TUIC Tests
    // ========================================================================

    #[test]
    fn test_parse_tuic_url_basic() {
        let url = "tuic://550e8400-e29b-41d4-a716-446655440000:password123@tuic.server.com:443#TUIC%20Server";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Tuic);
        assert_eq!(config.server, "tuic.server.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.uuid, Some("550e8400-e29b-41d4-a716-446655440000".to_string()));
        assert_eq!(config.password, Some("password123".to_string()));
        assert!(config.tls); // TUIC always uses TLS
        assert_eq!(config.transport, Some("quic".to_string()));
        assert_eq!(config.name, "TUIC Server");
    }

    #[test]
    fn test_parse_tuic_url_with_params() {
        let url = "tuic://uuid-test:pass@server.com:443?congestion_control=bbr&udp_relay_mode=native&alpn=h3&sni=custom.sni.com#TUIC";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.custom_fields.get("congestion_control"), Some(&"bbr".to_string()));
        assert_eq!(config.custom_fields.get("udp_relay_mode"), Some(&"native".to_string()));
        assert_eq!(config.custom_fields.get("alpn"), Some(&"h3".to_string()));
        assert_eq!(config.sni, Some("custom.sni.com".to_string()));
    }

    #[test]
    fn test_parse_tuic_url_cc_shorthand() {
        let url = "tuic://uuid:pass@server.com:443?cc=cubic";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.custom_fields.get("congestion_control"), Some(&"cubic".to_string()));
    }

    #[test]
    fn test_parse_tuic_url_no_password() {
        let url = "tuic://uuid-only@server.com:443";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.uuid, Some("uuid-only".to_string()));
        assert_eq!(config.password, None);
    }

    #[test]
    fn test_parse_tuic_url_default_port() {
        let url = "tuic://uuid:pass@server.com";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.port, 443); // Default TUIC port
    }

    #[test]
    fn test_parse_tuic_url_missing_uuid() {
        let url = "tuic://:password@server.com:443";

        assert!(parse_proxy_url(url).is_err());
    }

    // ========================================================================
    // Hysteria/Hysteria2 Tests
    // ========================================================================

    #[test]
    fn test_parse_hysteria2_url_basic() {
        let url = "hysteria2://authpassword@hy2.server.com:443#Hysteria2%20Server";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Hysteria2);
        assert_eq!(config.server, "hy2.server.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.password, Some("authpassword".to_string()));
        assert!(config.tls); // Hysteria always uses TLS
        assert_eq!(config.transport, Some("quic".to_string()));
    }

    #[test]
    fn test_parse_hysteria2_hy2_prefix() {
        let url = "hy2://authpass@hy2.server.com:443#HY2";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Hysteria2);
        assert_eq!(config.server, "hy2.server.com");
    }

    #[test]
    fn test_parse_hysteria2_with_params() {
        let url = "hysteria2://auth@server.com:443?obfs=salamander&obfs-password=obfspass&sni=custom.sni.com&insecure=1#HY2";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.custom_fields.get("obfs"), Some(&"salamander".to_string()));
        assert_eq!(config.custom_fields.get("obfs_password"), Some(&"obfspass".to_string()));
        assert_eq!(config.sni, Some("custom.sni.com".to_string()));
        assert_eq!(config.custom_fields.get("insecure"), Some(&"1".to_string()));
    }

    #[test]
    fn test_parse_hysteria1_url() {
        let url = "hysteria://auth@hy1.server.com:443?upmbps=100&downmbps=200#Hysteria1";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Hysteria);
        assert_eq!(config.custom_fields.get("up_mbps"), Some(&"100".to_string()));
        assert_eq!(config.custom_fields.get("down_mbps"), Some(&"200".to_string()));
    }

    #[test]
    fn test_parse_hysteria_bandwidth_params() {
        let url = "hysteria://auth@server.com:443?up=50&down=100";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.custom_fields.get("up_mbps"), Some(&"50".to_string()));
        assert_eq!(config.custom_fields.get("down_mbps"), Some(&"100".to_string()));
    }

    #[test]
    fn test_parse_hysteria_password_in_password_field() {
        // Some URLs put auth in password position: hysteria2://:password@server
        let url = "hysteria2://:secretauth@server.com:443";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.password, Some("secretauth".to_string()));
    }

    #[test]
    fn test_parse_hysteria_missing_server() {
        let url = "hysteria2://auth@:443";

        assert!(parse_proxy_url(url).is_err());
    }

    // ========================================================================
    // HTTP/HTTPS Proxy Tests
    // ========================================================================

    #[test]
    fn test_parse_http_url_basic() {
        let url = "http://proxy.example.com:8080";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Http);
        assert_eq!(config.server, "proxy.example.com");
        assert_eq!(config.port, 8080);
        assert!(!config.tls);
        assert_eq!(config.username, None);
        assert_eq!(config.password, None);
    }

    #[test]
    fn test_parse_http_url_with_auth() {
        let url = "http://user:pass123@proxy.example.com:3128";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Http);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass123".to_string()));
    }

    #[test]
    fn test_parse_http_url_encoded_credentials() {
        let url = "http://user%40domain:pass%23word@proxy.com:8080";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.username, Some("user@domain".to_string()));
        assert_eq!(config.password, Some("pass#word".to_string()));
    }

    #[test]
    fn test_parse_https_url() {
        let url = "https://secure.proxy.com:443";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Https);
        assert!(config.tls);
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_parse_http_default_port() {
        let url = "http://proxy.com";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.port, 8080); // Default HTTP proxy port
    }

    #[test]
    fn test_parse_https_default_port() {
        let url = "https://proxy.com";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.port, 443); // Default HTTPS port
    }

    #[test]
    fn test_parse_http_with_fragment() {
        let url = "http://proxy.com:8080#My%20HTTP%20Proxy";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.name, "My HTTP Proxy");
    }

    // ========================================================================
    // Subscription Tests
    // ========================================================================

    #[test]
    fn test_parse_subscription_base64() {
        let content = "vless://uuid1@server1.com:443#Server1\nvless://uuid2@server2.com:443#Server2";
        let encoded = general_purpose::STANDARD.encode(content);

        let configs = parse_subscription(&encoded).unwrap();

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].server, "server1.com");
        assert_eq!(configs[1].server, "server2.com");
    }

    #[test]
    fn test_parse_subscription_plain_text() {
        let content = "vless://uuid1@server1.com:443#Server1\nvless://uuid2@server2.com:443#Server2";

        let configs = parse_subscription(content).unwrap();

        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn test_parse_subscription_with_comments() {
        let content = "# This is a comment\nvless://uuid@server.com:443#Server\n// Another comment\ntrojan://pass@trojan.com:443";

        let configs = parse_subscription(content).unwrap();

        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn test_parse_subscription_mixed_protocols() {
        let content = "vless://uuid@vless.com:443\nss://YWVzLTI1Ni1nY206cGFzcw==@ss.com:8388\ntrojan://pass@trojan.com:443";

        let configs = parse_subscription(content).unwrap();

        assert_eq!(configs.len(), 3);
        assert_eq!(configs[0].protocol, ProxyProtocol::Vless);
        assert_eq!(configs[1].protocol, ProxyProtocol::Shadowsocks);
        assert_eq!(configs[2].protocol, ProxyProtocol::Trojan);
    }

    #[test]
    fn test_parse_subscription_skips_invalid() {
        let content = "vless://uuid@valid.com:443\ninvalid-url\nvless://uuid2@valid2.com:443";

        let configs = parse_subscription(content).unwrap();

        assert_eq!(configs.len(), 2); // Invalid URL skipped
    }

    #[test]
    fn test_parse_subscription_empty_lines() {
        let content = "\n\nvless://uuid@server.com:443\n\n\n";

        let configs = parse_subscription(content).unwrap();

        assert_eq!(configs.len(), 1);
    }

    #[test]
    fn test_parse_subscription_url_safe_base64() {
        let content = "vless://uuid@server.com:443#Test";
        let encoded = general_purpose::URL_SAFE.encode(content);

        let configs = parse_subscription(&encoded).unwrap();

        assert_eq!(configs.len(), 1);
    }

    #[test]
    fn test_parse_subscription_empty() {
        let content = "";

        assert!(parse_subscription(content).is_err());
    }

    #[test]
    fn test_parse_subscription_only_comments() {
        let content = "# Comment 1\n// Comment 2\n# Comment 3";

        assert!(parse_subscription(content).is_err());
    }

    #[test]
    fn test_parse_subscription_all_invalid() {
        let content = "invalid1\ninvalid2\nnot-a-url";

        assert!(parse_subscription(content).is_err());
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_url() {
        assert!(parse_proxy_url("").is_err());
    }

    #[test]
    fn test_whitespace_url() {
        assert!(parse_proxy_url("   ").is_err());
    }

    #[test]
    fn test_url_with_whitespace_trimmed() {
        let url = "  socks5://127.0.0.1:1080  ";
        let config = parse_proxy_url(url).unwrap();
        assert_eq!(config.server, "127.0.0.1");
    }

    #[test]
    fn test_invalid_port_too_large() {
        let url = "socks5://127.0.0.1:99999";
        assert!(parse_proxy_url(url).is_err());
    }

    #[test]
    fn test_vless_missing_uuid() {
        let url = "vless://@server.com:443";
        assert!(parse_proxy_url(url).is_err());
    }

    #[test]
    fn test_vless_missing_server() {
        let url = "vless://uuid@:443";
        assert!(parse_proxy_url(url).is_err());
    }

    #[test]
    fn test_trojan_missing_password() {
        let url = "trojan://@server.com:443";
        assert!(parse_proxy_url(url).is_err());
    }

    #[test]
    fn test_socks_missing_server() {
        let url = "socks5://:1080";
        assert!(parse_proxy_url(url).is_err());
    }

    #[test]
    fn test_vless_reality_params() {
        let url = "vless://uuid@server.com:443?security=reality&pbk=publickey123&sid=shortid&fp=chrome#Reality";

        let config = parse_proxy_url(url).unwrap();

        assert!(config.tls); // reality counts as TLS
        assert_eq!(config.custom_fields.get("public_key"), Some(&"publickey123".to_string()));
        assert_eq!(config.custom_fields.get("short_id"), Some(&"shortid".to_string()));
        assert_eq!(config.custom_fields.get("fingerprint"), Some(&"chrome".to_string()));
    }

    #[test]
    fn test_vless_websocket_transport() {
        let url = "vless://uuid@server.com:443?type=ws&path=/websocket&host=cdn.example.com";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.transport, Some("ws".to_string()));
        assert_eq!(config.custom_fields.get("path"), Some(&"/websocket".to_string()));
        assert_eq!(config.custom_fields.get("host"), Some(&"cdn.example.com".to_string()));
    }

    #[test]
    fn test_trojan_with_transport() {
        let url = "trojan://pass@server.com:443?type=ws&path=/trojan&fp=firefox&alpn=h2,http/1.1";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.transport, Some("ws".to_string()));
        assert_eq!(config.custom_fields.get("fingerprint"), Some(&"firefox".to_string()));
        assert_eq!(config.custom_fields.get("alpn"), Some(&"h2,http/1.1".to_string()));
    }

    #[test]
    fn test_socks_without_auth() {
        let url = "socks5://proxy.local:1080";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.username, None);
        assert_eq!(config.password, None);
    }

    #[test]
    fn test_socks_scheme_variant() {
        let url = "socks://user:pass@proxy.local:1080";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.protocol, ProxyProtocol::Socks5);
    }

    #[test]
    fn test_socks_default_port() {
        let url = "socks5://proxy.local";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.port, 1080); // Default SOCKS port
    }

    #[test]
    fn test_url_encoded_name() {
        let url = "vless://uuid@server.com:443#%E6%B5%8B%E8%AF%95%E6%9C%8D%E5%8A%A1%E5%99%A8";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.name, "测试服务器"); // Chinese characters
    }

    #[test]
    fn test_special_characters_in_password() {
        let url = "trojan://p%40ss%3Aword%21@server.com:443";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.password, Some("p@ss:word!".to_string()));
    }

    #[test]
    fn test_ipv4_server() {
        let url = "vless://uuid@192.168.1.100:443";

        let config = parse_proxy_url(url).unwrap();

        assert_eq!(config.server, "192.168.1.100");
    }

    #[test]
    fn test_ipv6_server_vless() {
        let url = "vless://uuid@[2001:db8::1]:443";

        let config = parse_proxy_url(url).unwrap();

        // URL parser keeps brackets for IPv6
        assert_eq!(config.server, "[2001:db8::1]");
        assert_eq!(config.port, 443);
    }

    #[test]
    fn test_generate_unique_ids() {
        let url1 = "vless://uuid@server1.com:443";
        let url2 = "vless://uuid@server2.com:443";

        let config1 = parse_proxy_url(url1).unwrap();
        let config2 = parse_proxy_url(url2).unwrap();

        // IDs should be unique
        assert_ne!(config1.id, config2.id);
        // IDs should start with protocol prefix
        assert!(config1.id.starts_with("vless_"));
        assert!(config2.id.starts_with("vless_"));
    }
}

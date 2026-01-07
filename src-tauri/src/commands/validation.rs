//! Input validation helpers for Tauri commands
//!
//! Provides validation functions for common input types:
//! - Domains, URLs, IPs
//! - Ports, strategy IDs
//! - Proxy configurations

#![allow(dead_code)] // Public validation API

use crate::core::errors::IsolateError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

/// Validate that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), IsolateError> {
    if value.trim().is_empty() {
        return Err(IsolateError::Validation(format!("{} cannot be empty", field_name)));
    }
    Ok(())
}

/// Validate domain format
/// 
/// Rules:
/// - Must contain at least one dot
/// - No spaces or newlines
/// - Each label max 63 chars, total max 253 chars
/// - Labels can contain alphanumeric and hyphens (not at start/end)
pub fn validate_domain(domain: &str) -> Result<(), IsolateError> {
    validate_not_empty(domain, "Domain")?;
    
    let domain = domain.trim().to_lowercase();
    
    // Max length check
    if domain.len() > 253 {
        return Err(IsolateError::Validation(
            "Domain exceeds maximum length of 253 characters".to_string()
        ));
    }
    
    if !domain.contains('.') {
        return Err(IsolateError::Validation(
            "Invalid domain format: must contain at least one dot".to_string()
        ));
    }
    
    // Check for invalid characters
    if domain.contains(' ') || domain.contains('\n') || domain.contains('\t') {
        return Err(IsolateError::Validation(
            "Domain contains invalid whitespace characters".to_string()
        ));
    }
    
    // Validate each label
    for label in domain.split('.') {
        if label.is_empty() {
            return Err(IsolateError::Validation(
                "Domain contains empty label (consecutive dots)".to_string()
            ));
        }
        if label.len() > 63 {
            return Err(IsolateError::Validation(
                format!("Domain label '{}' exceeds 63 characters", label)
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(IsolateError::Validation(
                format!("Domain label '{}' cannot start or end with hyphen", label)
            ));
        }
        // Allow alphanumeric and hyphens
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(IsolateError::Validation(
                format!("Domain label '{}' contains invalid characters", label)
            ));
        }
    }
    
    Ok(())
}

/// Validate port number (1-65535)
pub fn validate_port(port: u16) -> Result<(), IsolateError> {
    if port == 0 {
        return Err(IsolateError::Validation("Port cannot be 0".to_string()));
    }
    // u16 max is 65535, so upper bound is implicit
    Ok(())
}

/// Validate port range for specific use cases
pub fn validate_port_range(port: u16, min: u16, max: u16, context: &str) -> Result<(), IsolateError> {
    if port < min || port > max {
        return Err(IsolateError::Validation(
            format!("{} port must be between {} and {}, got {}", context, min, max, port)
        ));
    }
    Ok(())
}

/// Validate URL format
/// 
/// Supports http:// and https:// schemes
pub fn validate_url(url: &str) -> Result<(), IsolateError> {
    validate_not_empty(url, "URL")?;
    
    let url = url.trim();
    
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(IsolateError::Validation(
            "URL must start with http:// or https://".to_string()
        ));
    }
    
    // Check for basic structure after scheme
    let after_scheme = if url.starts_with("https://") {
        &url[8..]
    } else {
        &url[7..]
    };
    
    if after_scheme.is_empty() {
        return Err(IsolateError::Validation(
            "URL must have a host after the scheme".to_string()
        ));
    }
    
    // Extract host part (before path/query)
    let host = after_scheme.split('/').next().unwrap_or("");
    let host = host.split('?').next().unwrap_or("");
    let host = host.split(':').next().unwrap_or(""); // Remove port
    
    if host.is_empty() {
        return Err(IsolateError::Validation(
            "URL must have a valid host".to_string()
        ));
    }
    
    Ok(())
}

/// Check if an IP address is in a private/internal range
/// 
/// Blocks:
/// - IPv4: 127.0.0.0/8 (loopback), 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 (RFC 1918), 169.254.0.0/16 (link-local)
/// - IPv6: ::1 (loopback), fc00::/7 (unique local), fe80::/10 (link-local)
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Loopback: 127.0.0.0/8
            if ipv4.octets()[0] == 127 {
                return true;
            }
            
            // RFC 1918 private ranges
            // 10.0.0.0/8
            if ipv4.octets()[0] == 10 {
                return true;
            }
            
            // 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
            if ipv4.octets()[0] == 172 && (ipv4.octets()[1] >= 16 && ipv4.octets()[1] <= 31) {
                return true;
            }
            
            // 192.168.0.0/16
            if ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168 {
                return true;
            }
            
            // Link-local: 169.254.0.0/16
            if ipv4.octets()[0] == 169 && ipv4.octets()[1] == 254 {
                return true;
            }
            
            false
        }
        IpAddr::V6(ipv6) => {
            // Loopback: ::1
            if ipv6.is_loopback() {
                return true;
            }
            
            // Unique local addresses: fc00::/7 (fc00:: - fdff::)
            let segments = ipv6.segments();
            if segments[0] >= 0xfc00 && segments[0] <= 0xfdff {
                return true;
            }
            
            // Link-local: fe80::/10 (fe80:: - febf::)
            if segments[0] >= 0xfe80 && segments[0] <= 0xfebf {
                return true;
            }
            
            false
        }
    }
}

/// Validate that a URL points to a public (non-private) address
/// 
/// This function prevents SSRF attacks by:
/// 1. Parsing the URL to extract the hostname
/// 2. Resolving the hostname to IP address(es)
/// 3. Checking that all resolved IPs are public (not private/internal)
/// 
/// Blocked address ranges:
/// - IPv4: localhost, 127.x.x.x, 10.x.x.x, 172.16-31.x.x, 192.168.x.x, 169.254.x.x
/// - IPv6: ::1, fc00::/7, fe80::/10
/// 
/// # Examples
/// ```
/// // Public URLs - OK
/// validate_public_url("https://google.com")?;
/// validate_public_url("https://github.com/user/repo")?;
/// 
/// // Private URLs - Blocked
/// validate_public_url("http://localhost:8080")?; // Error
/// validate_public_url("http://127.0.0.1")?; // Error
/// validate_public_url("http://192.168.1.1")?; // Error
/// ```
pub fn validate_public_url(url_str: &str) -> Result<url::Url, IsolateError> {
    // First, validate basic URL format
    validate_url(url_str)?;
    
    // Parse URL using url crate for proper parsing
    let parsed_url = url::Url::parse(url_str)
        .map_err(|e| IsolateError::Validation(format!("Invalid URL format: {}", e)))?;
    
    // Extract host
    let host = parsed_url.host_str()
        .ok_or_else(|| IsolateError::Validation("URL must have a host".to_string()))?;
    
    // Check for localhost explicitly (case-insensitive)
    let host_lower = host.to_lowercase();
    if host_lower == "localhost" || host_lower == "localhost." {
        return Err(IsolateError::Validation(
            "Access to localhost is not allowed".to_string()
        ));
    }
    
    // If host is already an IP address, check it directly
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            return Err(IsolateError::Validation(
                format!("Access to private IP address {} is not allowed", ip)
            ));
        }
        return Ok(parsed_url);
    }
    
    // Resolve hostname to IP addresses
    // Use port 80 as dummy port for resolution (required by ToSocketAddrs)
    let socket_addr = format!("{}:80", host);
    let resolved_addrs: Vec<IpAddr> = socket_addr
        .to_socket_addrs()
        .map_err(|e| IsolateError::Validation(format!("Failed to resolve hostname '{}': {}", host, e)))?
        .map(|addr| addr.ip())
        .collect();
    
    if resolved_addrs.is_empty() {
        return Err(IsolateError::Validation(
            format!("Hostname '{}' did not resolve to any IP addresses", host)
        ));
    }
    
    // Check all resolved IPs - if ANY are private, reject
    for ip in &resolved_addrs {
        if is_private_ip(ip) {
            return Err(IsolateError::Validation(
                format!("Hostname '{}' resolves to private IP address {} which is not allowed", host, ip)
            ));
        }
    }
    
    Ok(parsed_url)
}

/// Validate IPv4 address format
pub fn validate_ipv4(ip: &str) -> Result<(), IsolateError> {
    validate_not_empty(ip, "IPv4 address")?;
    
    ip.trim().parse::<Ipv4Addr>()
        .map_err(|_| IsolateError::Validation(
            format!("Invalid IPv4 address format: {}", ip)
        ))?;
    
    Ok(())
}

/// Validate IPv6 address format
pub fn validate_ipv6(ip: &str) -> Result<(), IsolateError> {
    validate_not_empty(ip, "IPv6 address")?;
    
    // Handle bracketed IPv6 (common in URLs)
    let ip = ip.trim().trim_start_matches('[').trim_end_matches(']');
    
    ip.parse::<Ipv6Addr>()
        .map_err(|_| IsolateError::Validation(
            format!("Invalid IPv6 address format: {}", ip)
        ))?;
    
    Ok(())
}

/// Validate IP address (IPv4 or IPv6)
pub fn validate_ip(ip: &str) -> Result<(), IsolateError> {
    validate_not_empty(ip, "IP address")?;
    
    let ip = ip.trim();
    
    // Try IPv4 first
    if validate_ipv4(ip).is_ok() {
        return Ok(());
    }
    
    // Try IPv6
    if validate_ipv6(ip).is_ok() {
        return Ok(());
    }
    
    Err(IsolateError::Validation(
        format!("Invalid IP address format (neither IPv4 nor IPv6): {}", ip)
    ))
}

/// Validate strategy ID format
/// 
/// Rules:
/// - Non-empty
/// - Max 64 characters
/// - Alphanumeric, underscores, hyphens only
/// - Cannot start with hyphen or underscore
pub fn validate_strategy_id(id: &str) -> Result<(), IsolateError> {
    validate_not_empty(id, "Strategy ID")?;
    
    let id = id.trim();
    
    if id.len() > 64 {
        return Err(IsolateError::Validation(
            "Strategy ID exceeds maximum length of 64 characters".to_string()
        ));
    }
    
    if id.starts_with('-') || id.starts_with('_') {
        return Err(IsolateError::Validation(
            "Strategy ID cannot start with hyphen or underscore".to_string()
        ));
    }
    
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(IsolateError::Validation(
            "Strategy ID can only contain alphanumeric characters, underscores, and hyphens".to_string()
        ));
    }
    
    Ok(())
}

/// Validate proxy host (domain or IP)
pub fn validate_proxy_host(host: &str) -> Result<(), IsolateError> {
    validate_not_empty(host, "Proxy host")?;
    
    let host = host.trim();
    
    // Try as IP first
    if validate_ip(host).is_ok() {
        return Ok(());
    }
    
    // Try as domain
    validate_domain(host)
        .map_err(|_| IsolateError::Validation(
            format!("Proxy host must be a valid IP address or domain: {}", host)
        ))
}

/// Proxy configuration for validation
#[derive(Debug)]
pub struct ProxyConfigValidation<'a> {
    pub host: &'a str,
    pub port: u16,
    pub protocol: &'a str,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
}

/// Validate complete proxy configuration
pub fn validate_proxy_config(config: &ProxyConfigValidation) -> Result<(), IsolateError> {
    // Validate host
    validate_proxy_host(config.host)?;
    
    // Validate port
    validate_port(config.port)?;
    
    // Validate protocol
    let valid_protocols = ["http", "https", "socks4", "socks5", "vless", "vmess", "trojan", "shadowsocks"];
    let protocol = config.protocol.to_lowercase();
    if !valid_protocols.contains(&protocol.as_str()) {
        return Err(IsolateError::Validation(
            format!("Invalid proxy protocol '{}'. Supported: {}", config.protocol, valid_protocols.join(", "))
        ));
    }
    
    // Validate credentials if provided
    if let Some(username) = config.username {
        if username.trim().is_empty() {
            return Err(IsolateError::Validation(
                "Proxy username cannot be empty if provided".to_string()
            ));
        }
        if username.len() > 255 {
            return Err(IsolateError::Validation(
                "Proxy username exceeds maximum length of 255 characters".to_string()
            ));
        }
    }
    
    if let Some(password) = config.password {
        if password.len() > 255 {
            return Err(IsolateError::Validation(
                "Proxy password exceeds maximum length of 255 characters".to_string()
            ));
        }
    }
    
    // Check for auth consistency
    if config.username.is_some() != config.password.is_some() {
        // Some protocols allow username without password
        if !["socks5", "http"].contains(&protocol.as_str()) && config.password.is_none() {
            return Err(IsolateError::Validation(
                "Proxy password is required when username is provided for this protocol".to_string()
            ));
        }
    }
    
    Ok(())
}

/// Validate VLESS UUID format
pub fn validate_uuid(uuid: &str) -> Result<(), IsolateError> {
    validate_not_empty(uuid, "UUID")?;
    
    let uuid = uuid.trim();
    
    // UUID format: 8-4-4-4-12 hex chars
    let parts: Vec<&str> = uuid.split('-').collect();
    if parts.len() != 5 {
        return Err(IsolateError::Validation(
            "Invalid UUID format: must have 5 parts separated by hyphens".to_string()
        ));
    }
    
    let expected_lengths = [8, 4, 4, 4, 12];
    for (i, (part, &expected_len)) in parts.iter().zip(expected_lengths.iter()).enumerate() {
        if part.len() != expected_len {
            return Err(IsolateError::Validation(
                format!("Invalid UUID format: part {} should be {} characters", i + 1, expected_len)
            ));
        }
        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(IsolateError::Validation(
                format!("Invalid UUID format: part {} contains non-hex characters", i + 1)
            ));
        }
    }
    
    Ok(())
}

// ==================== TTL Validators ====================

/// Validate TTL value (1-255, as per IP header field size)
/// 
/// TTL (Time To Live) is an 8-bit field in IP header, valid range is 1-255.
/// Value 0 is invalid as it would cause immediate packet drop.
pub fn validate_ttl(value: &str) -> Result<u8, IsolateError> {
    let value = value.trim();
    
    let ttl: u8 = value.parse()
        .map_err(|_| IsolateError::Validation(
            format!("Invalid TTL value '{}': must be a number 1-255", value)
        ))?;
    
    if ttl == 0 {
        return Err(IsolateError::Validation(
            "TTL cannot be 0".to_string()
        ));
    }
    
    Ok(ttl)
}

/// Validate autottl format (min:max:delta, all values 1-255)
/// 
/// Format: "min:max:delta" where:
/// - min: minimum TTL value (1-255)
/// - max: maximum TTL value (1-255, must be >= min)
/// - delta: step value for TTL adjustment (1-255)
/// 
/// Example: "2:10:2" means TTL range from 2 to 10 with step 2
pub fn validate_autottl(value: &str) -> Result<(u8, u8, u8), IsolateError> {
    let value = value.trim();
    
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 3 {
        return Err(IsolateError::Validation(
            format!("Invalid autottl format '{}': expected min:max:delta (e.g., 2:10:2)", value)
        ));
    }
    
    let min = validate_ttl(parts[0])
        .map_err(|_| IsolateError::Validation(
            format!("Invalid autottl min value '{}': must be 1-255", parts[0])
        ))?;
    
    let max = validate_ttl(parts[1])
        .map_err(|_| IsolateError::Validation(
            format!("Invalid autottl max value '{}': must be 1-255", parts[1])
        ))?;
    
    let delta = validate_ttl(parts[2])
        .map_err(|_| IsolateError::Validation(
            format!("Invalid autottl delta value '{}': must be 1-255", parts[2])
        ))?;
    
    if min > max {
        return Err(IsolateError::Validation(
            format!("Invalid autottl '{}': min ({}) cannot be greater than max ({})", value, min, max)
        ));
    }
    
    Ok((min, max, delta))
}

// ==================== Capabilities Validators ====================

/// Validators from capabilities/default.json for winws arguments
/// These regex patterns define what arguments are allowed to be passed to winws.exe
pub mod capabilities {
    use regex::Regex;
    use std::sync::LazyLock;

    /// All validators from capabilities/default.json for winws
    static WINWS_VALIDATORS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
        vec![
            // Flag arguments (--flag format)
            Regex::new(r"^--wf-tcp$").unwrap(),
            Regex::new(r"^--wf-udp$").unwrap(),
            Regex::new(r"^--filter$").unwrap(),
            Regex::new(r"^--hostlist$").unwrap(),
            Regex::new(r"^--hostlist-exclude$").unwrap(),
            Regex::new(r"^--hostlist-domains$").unwrap(),
            Regex::new(r"^--ipset$").unwrap(),
            Regex::new(r"^--ipset-exclude$").unwrap(),
            Regex::new(r"^--dpi-desync$").unwrap(),
            Regex::new(r"^--dpi-desync-l7$").unwrap(),
            Regex::new(r"^--dpi-desync-ipid$").unwrap(),
            Regex::new(r"^--dpi-desync-repeats$").unwrap(),
            Regex::new(r"^--dpi-desync-split-seqovl$").unwrap(),
            Regex::new(r"^--dpi-desync-split-pos$").unwrap(),
            Regex::new(r"^--dpi-desync-split-seqovl-pattern$").unwrap(),
            Regex::new(r"^--dpi-desync-fooling$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-tls$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-quic$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-tls-mod$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-wireguard$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-dht$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-unknown-udp$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-tcp-mod$").unwrap(),
            Regex::new(r"^--dpi-desync-fake-syndata$").unwrap(),
            Regex::new(r"^--dpi-desync-ttl$").unwrap(),
            Regex::new(r"^--dpi-desync-ttl6$").unwrap(),
            Regex::new(r"^--dpi-desync-autottl$").unwrap(),
            Regex::new(r"^--dpi-desync-badseq-increment$").unwrap(),
            Regex::new(r"^--dpi-desync-badack-increment$").unwrap(),
            Regex::new(r"^--dpi-desync-ts-increment$").unwrap(),
            Regex::new(r"^--dpi-desync-cutoff$").unwrap(),
            Regex::new(r"^--dpi-desync-hostfakesplit-mod$").unwrap(),
            Regex::new(r"^--dpi-desync-hostfakesplit-midhost$").unwrap(),
            Regex::new(r"^--dpi-desync-fakedsplit-mod$").unwrap(),
            Regex::new(r"^--dpi-desync-dup$").unwrap(),
            Regex::new(r"^--dpi-desync-dup-replace$").unwrap(),
            Regex::new(r"^--dpi-desync-dup-ttl$").unwrap(),
            Regex::new(r"^--dpi-desync-dup-autottl$").unwrap(),
            Regex::new(r"^--dpi-desync-dup-fooling$").unwrap(),
            Regex::new(r"^--wsize$").unwrap(),
            Regex::new(r"^--wssize$").unwrap(),
            Regex::new(r"^--wssize-cutoff$").unwrap(),
            Regex::new(r"^--filter-l3$").unwrap(),
            Regex::new(r"^--filter-l7$").unwrap(),
            Regex::new(r"^--filter-ssid$").unwrap(),
            Regex::new(r"^--nlm-filter$").unwrap(),
            Regex::new(r"^--new$").unwrap(),
            // Value patterns
            Regex::new(r"^[0-9,]+$").unwrap(),  // Port numbers: 80,443
            Regex::new(r"^(tcp|udp|ip|ipv4|ipv6|http|tls|quic|dns|stun|discord|wireguard|dht|unknown)$").unwrap(),
            Regex::new(r"^(fake|rst|rstack|synack|syndata|disorder|disorder2|split|split2|ipfrag1|ipfrag2|hopbyhop|destopt|ipfrag1\+destopt|hopbyhop\+destopt|udplen|tamper|fakedsplit|multisplit|multidisorder|hostfakesplit)(,(fake|rst|rstack|synack|syndata|disorder|disorder2|split|split2|ipfrag1|ipfrag2|hopbyhop|destopt|ipfrag1\+destopt|hopbyhop\+destopt|udplen|tamper|fakedsplit|multisplit|multidisorder|hostfakesplit))*$").unwrap(),
            Regex::new(r"^(md5sig|badsum|datanoack|hopbyhop|hopbyhop2|badseq|badack|ts|ts2)(,(md5sig|badsum|datanoack|hopbyhop|hopbyhop2|badseq|badack|ts|ts2))*$").unwrap(),
            Regex::new(r#"^[a-zA-Z]:[\\][^<>:"\|\?\*]+$"#).unwrap(),  // Windows path
            Regex::new(r"^[0-9]+$").unwrap(),  // Single number
            Regex::new(r"^[0-9]+:[0-9]+:[0-9]+$").unwrap(),  // TTL format: min:max:delta
            Regex::new(r"^(n|d|s)[0-9]+$").unwrap(),  // Cutoff format: n5, d10, s100
            Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap(),  // General alphanumeric with dots/hyphens
        ]
    });

    /// Check if a single argument matches any of the winws validators
    pub fn validate_winws_arg(arg: &str) -> bool {
        WINWS_VALIDATORS.iter().any(|re| re.is_match(arg))
    }

    /// Validate all arguments for winws command
    /// Returns Ok(()) if all arguments are valid, Err with invalid args otherwise
    pub fn validate_winws_args(args: &[&str]) -> Result<(), Vec<String>> {
        let invalid: Vec<String> = args
            .iter()
            .filter(|arg| !validate_winws_arg(arg))
            .map(|s| s.to_string())
            .collect();
        
        if invalid.is_empty() {
            Ok(())
        } else {
            Err(invalid)
        }
    }

    /// Parse a winws command line (e.g., "--wf-tcp=80,443 --dpi-desync=fake")
    /// into individual arguments and validate each one
    pub fn validate_winws_cmdline(cmdline: &str) -> Result<(), Vec<String>> {
        let args: Vec<&str> = cmdline
            .split_whitespace()
            .flat_map(|part| {
                // Split --flag=value into ["--flag", "value"]
                if let Some(eq_pos) = part.find('=') {
                    vec![&part[..eq_pos], &part[eq_pos + 1..]]
                } else {
                    vec![part]
                }
            })
            .collect();
        
        validate_winws_args(&args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::capabilities::*;

    // ==================== validate_not_empty ====================
    
    #[test]
    fn test_validate_not_empty_valid() {
        assert!(validate_not_empty("test", "field").is_ok());
        assert!(validate_not_empty("a", "field").is_ok());
        assert!(validate_not_empty("  text  ", "field").is_ok());
    }

    #[test]
    fn test_validate_not_empty_invalid() {
        assert!(validate_not_empty("", "field").is_err());
        assert!(validate_not_empty("   ", "field").is_err());
        assert!(validate_not_empty("\t\n", "field").is_err());
    }

    // ==================== validate_domain ====================
    
    #[test]
    fn test_validate_domain_valid() {
        assert!(validate_domain("example.com").is_ok());
        assert!(validate_domain("sub.example.com").is_ok());
        assert!(validate_domain("deep.sub.example.com").is_ok());
        assert!(validate_domain("example-site.com").is_ok());
        assert!(validate_domain("123.example.com").is_ok());
        assert!(validate_domain("a.b").is_ok());
    }

    #[test]
    fn test_validate_domain_invalid_format() {
        assert!(validate_domain("example").is_err()); // no dot
        assert!(validate_domain("").is_err());
        assert!(validate_domain("   ").is_err());
    }

    #[test]
    fn test_validate_domain_invalid_chars() {
        assert!(validate_domain("example .com").is_err()); // space
        assert!(validate_domain("example\n.com").is_err()); // newline
        assert!(validate_domain("example\t.com").is_err()); // tab
        assert!(validate_domain("exam_ple.com").is_err()); // underscore
        assert!(validate_domain("exam@ple.com").is_err()); // @
    }

    #[test]
    fn test_validate_domain_invalid_labels() {
        assert!(validate_domain("-example.com").is_err()); // starts with hyphen
        assert!(validate_domain("example-.com").is_err()); // ends with hyphen
        assert!(validate_domain("example..com").is_err()); // empty label
    }

    #[test]
    fn test_validate_domain_length_limits() {
        // Label > 63 chars
        let long_label = "a".repeat(64);
        assert!(validate_domain(&format!("{}.com", long_label)).is_err());
        
        // Total > 253 chars (need 254+ chars)
        // 63 + 1 + 63 + 1 + 63 + 1 + 63 = 255 chars
        let long_domain = format!("{}.{}.{}.{}", "a".repeat(63), "b".repeat(63), "c".repeat(63), "d".repeat(63));
        assert!(validate_domain(&long_domain).is_err());
        
        // 253 chars should be ok
        let max_domain = format!("{}.{}.{}.com", "a".repeat(60), "b".repeat(60), "c".repeat(60));
        assert!(validate_domain(&max_domain).is_ok());
    }

    // ==================== validate_port ====================
    
    #[test]
    fn test_validate_port_valid() {
        assert!(validate_port(1).is_ok());
        assert!(validate_port(80).is_ok());
        assert!(validate_port(443).is_ok());
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_validate_port_range() {
        assert!(validate_port_range(1024, 1024, 65535, "User").is_ok());
        assert!(validate_port_range(8080, 1024, 65535, "User").is_ok());
        assert!(validate_port_range(80, 1024, 65535, "User").is_err()); // below min
    }

    // ==================== validate_url ====================
    
    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://example.com/path").is_ok());
        assert!(validate_url("https://example.com:8080/path").is_ok());
        assert!(validate_url("http://localhost").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("example.com").is_err()); // no scheme
        assert!(validate_url("https://").is_err()); // no host
        assert!(validate_url("http://").is_err());
    }

    // ==================== validate_ip ====================
    
    #[test]
    fn test_validate_ipv4_valid() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("0.0.0.0").is_ok());
        assert!(validate_ipv4("255.255.255.255").is_ok());
        assert!(validate_ipv4("127.0.0.1").is_ok());
        assert!(validate_ipv4("  10.0.0.1  ").is_ok()); // with whitespace
    }

    #[test]
    fn test_validate_ipv4_invalid() {
        assert!(validate_ipv4("").is_err());
        assert!(validate_ipv4("256.1.1.1").is_err()); // octet > 255
        assert!(validate_ipv4("192.168.1").is_err()); // missing octet
        assert!(validate_ipv4("192.168.1.1.1").is_err()); // extra octet
        assert!(validate_ipv4("abc.def.ghi.jkl").is_err());
        assert!(validate_ipv4("::1").is_err()); // IPv6
    }

    #[test]
    fn test_validate_ipv6_valid() {
        assert!(validate_ipv6("::1").is_ok());
        assert!(validate_ipv6("fe80::1").is_ok());
        assert!(validate_ipv6("2001:db8::1").is_ok());
        assert!(validate_ipv6("::ffff:192.168.1.1").is_ok()); // IPv4-mapped
        assert!(validate_ipv6("[::1]").is_ok()); // bracketed
        assert!(validate_ipv6("[2001:db8::1]").is_ok());
    }

    #[test]
    fn test_validate_ipv6_invalid() {
        assert!(validate_ipv6("").is_err());
        assert!(validate_ipv6("192.168.1.1").is_err()); // IPv4
        assert!(validate_ipv6("gggg::1").is_err()); // invalid hex
        assert!(validate_ipv6("2001:db8::1::2").is_err()); // multiple ::
    }

    #[test]
    fn test_validate_ip_both() {
        // IPv4
        assert!(validate_ip("192.168.1.1").is_ok());
        assert!(validate_ip("127.0.0.1").is_ok());
        
        // IPv6
        assert!(validate_ip("::1").is_ok());
        assert!(validate_ip("2001:db8::1").is_ok());
        
        // Invalid
        assert!(validate_ip("").is_err());
        assert!(validate_ip("not-an-ip").is_err());
    }

    // ==================== validate_strategy_id ====================
    
    #[test]
    fn test_validate_strategy_id_valid() {
        assert!(validate_strategy_id("discord_multisplit").is_ok());
        assert!(validate_strategy_id("youtube-fake").is_ok());
        assert!(validate_strategy_id("general1").is_ok());
        assert!(validate_strategy_id("ALT2").is_ok());
        assert!(validate_strategy_id("a").is_ok());
    }

    #[test]
    fn test_validate_strategy_id_invalid() {
        assert!(validate_strategy_id("").is_err());
        assert!(validate_strategy_id("   ").is_err());
        assert!(validate_strategy_id("-starts-with-hyphen").is_err());
        assert!(validate_strategy_id("_starts_with_underscore").is_err());
        assert!(validate_strategy_id("has space").is_err());
        assert!(validate_strategy_id("has@special").is_err());
        assert!(validate_strategy_id("has.dot").is_err());
    }

    #[test]
    fn test_validate_strategy_id_length() {
        let long_id = "a".repeat(65);
        assert!(validate_strategy_id(&long_id).is_err());
        
        let max_id = "a".repeat(64);
        assert!(validate_strategy_id(&max_id).is_ok());
    }

    // ==================== validate_proxy_host ====================
    
    #[test]
    fn test_validate_proxy_host_valid() {
        // IPs
        assert!(validate_proxy_host("192.168.1.1").is_ok());
        assert!(validate_proxy_host("::1").is_ok());
        
        // Domains
        assert!(validate_proxy_host("proxy.example.com").is_ok());
        assert!(validate_proxy_host("vpn.server.net").is_ok());
    }

    #[test]
    fn test_validate_proxy_host_invalid() {
        assert!(validate_proxy_host("").is_err());
        assert!(validate_proxy_host("not valid").is_err());
    }

    // ==================== validate_proxy_config ====================
    
    #[test]
    fn test_validate_proxy_config_valid() {
        let config = ProxyConfigValidation {
            host: "proxy.example.com",
            port: 8080,
            protocol: "http",
            username: None,
            password: None,
        };
        assert!(validate_proxy_config(&config).is_ok());
        
        let config_with_auth = ProxyConfigValidation {
            host: "192.168.1.1",
            port: 1080,
            protocol: "socks5",
            username: Some("user"),
            password: Some("pass"),
        };
        assert!(validate_proxy_config(&config_with_auth).is_ok());
    }

    #[test]
    fn test_validate_proxy_config_protocols() {
        for protocol in ["http", "https", "socks4", "socks5", "vless", "vmess", "trojan", "shadowsocks"] {
            let config = ProxyConfigValidation {
                host: "example.com",
                port: 8080,
                protocol,
                username: None,
                password: None,
            };
            assert!(validate_proxy_config(&config).is_ok(), "Protocol {} should be valid", protocol);
        }
        
        let invalid = ProxyConfigValidation {
            host: "example.com",
            port: 8080,
            protocol: "ftp",
            username: None,
            password: None,
        };
        assert!(validate_proxy_config(&invalid).is_err());
    }

    #[test]
    fn test_validate_proxy_config_invalid() {
        // Invalid host
        let config = ProxyConfigValidation {
            host: "",
            port: 8080,
            protocol: "http",
            username: None,
            password: None,
        };
        assert!(validate_proxy_config(&config).is_err());
        
        // Invalid port
        let config = ProxyConfigValidation {
            host: "example.com",
            port: 0,
            protocol: "http",
            username: None,
            password: None,
        };
        assert!(validate_proxy_config(&config).is_err());
        
        // Empty username
        let config = ProxyConfigValidation {
            host: "example.com",
            port: 8080,
            protocol: "http",
            username: Some(""),
            password: Some("pass"),
        };
        assert!(validate_proxy_config(&config).is_err());
    }

    // ==================== validate_uuid ====================
    
    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("00000000-0000-0000-0000-000000000000").is_ok());
        assert!(validate_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff").is_ok());
        assert!(validate_uuid("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("").is_err());
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("550e8400e29b41d4a716446655440000").is_err()); // no hyphens
        assert!(validate_uuid("550e8400-e29b-41d4-a716").is_err()); // too short
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000-extra").is_err()); // too long
        assert!(validate_uuid("gggggggg-gggg-gggg-gggg-gggggggggggg").is_err()); // invalid hex
    }

    // ==================== SSRF Protection Tests ====================
    
    #[test]
    fn test_is_private_ip_loopback_ipv4() {
        // 127.0.0.0/8 - all addresses starting with 127
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))), "127.0.0.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0))), "127.0.0.0 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 1, 1, 1))), "127.1.1.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255))), "127.255.255.255 should be private");
    }

    #[test]
    fn test_is_private_ip_rfc1918_10() {
        // 10.0.0.0/8
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0))), "10.0.0.0 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))), "10.0.0.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))), "10.255.255.255 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 123, 45, 67))), "10.123.45.67 should be private");
    }

    #[test]
    fn test_is_private_ip_rfc1918_172() {
        // 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0))), "172.16.0.0 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))), "172.16.0.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 20, 10, 5))), "172.20.10.5 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255))), "172.31.255.255 should be private");
        
        // Edge cases - just outside the range
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 15, 255, 255))), "172.15.255.255 should be public");
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 32, 0, 0))), "172.32.0.0 should be public");
    }

    #[test]
    fn test_is_private_ip_rfc1918_192() {
        // 192.168.0.0/16
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0))), "192.168.0.0 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))), "192.168.0.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))), "192.168.1.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255))), "192.168.255.255 should be private");
        
        // Edge cases
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 167, 1, 1))), "192.167.1.1 should be public");
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 169, 1, 1))), "192.169.1.1 should be public");
    }

    #[test]
    fn test_is_private_ip_link_local() {
        // 169.254.0.0/16
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 0, 0))), "169.254.0.0 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))), "169.254.1.1 should be private");
        assert!(is_private_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 255, 255))), "169.254.255.255 should be private");
        
        // Edge cases
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(169, 253, 1, 1))), "169.253.1.1 should be public");
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(169, 255, 1, 1))), "169.255.1.1 should be public");
    }

    #[test]
    fn test_is_private_ip_public_ipv4() {
        // Common public IPs should NOT be private
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))), "8.8.8.8 (Google DNS) should be public");
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))), "1.1.1.1 (Cloudflare DNS) should be public");
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(142, 250, 185, 46))), "142.250.185.46 should be public");
        assert!(!is_private_ip(&IpAddr::V4(Ipv4Addr::new(151, 101, 1, 140))), "151.101.1.140 should be public");
    }

    #[test]
    fn test_is_private_ip_ipv6_loopback() {
        // ::1
        let loopback = "::1".parse::<Ipv6Addr>().unwrap();
        assert!(is_private_ip(&IpAddr::V6(loopback)), "::1 should be private");
    }

    #[test]
    fn test_is_private_ip_ipv6_unique_local() {
        // fc00::/7 (fc00:: - fdff::)
        let fc00 = "fc00::1".parse::<Ipv6Addr>().unwrap();
        assert!(is_private_ip(&IpAddr::V6(fc00)), "fc00::1 should be private");
        
        let fd00 = "fd00::1".parse::<Ipv6Addr>().unwrap();
        assert!(is_private_ip(&IpAddr::V6(fd00)), "fd00::1 should be private");
        
        let fdff = "fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff".parse::<Ipv6Addr>().unwrap();
        assert!(is_private_ip(&IpAddr::V6(fdff)), "fdff:ffff:... should be private");
    }

    #[test]
    fn test_is_private_ip_ipv6_link_local() {
        // fe80::/10 (fe80:: - febf::)
        let fe80 = "fe80::1".parse::<Ipv6Addr>().unwrap();
        assert!(is_private_ip(&IpAddr::V6(fe80)), "fe80::1 should be private");
        
        let fe80_full = "fe80:1234:5678:9abc:def0:1234:5678:9abc".parse::<Ipv6Addr>().unwrap();
        assert!(is_private_ip(&IpAddr::V6(fe80_full)), "fe80:... should be private");
    }

    #[test]
    fn test_is_private_ip_ipv6_public() {
        // Public IPv6 addresses
        let google = "2001:4860:4860::8888".parse::<Ipv6Addr>().unwrap();
        assert!(!is_private_ip(&IpAddr::V6(google)), "2001:4860:4860::8888 (Google DNS) should be public");
        
        let cloudflare = "2606:4700:4700::1111".parse::<Ipv6Addr>().unwrap();
        assert!(!is_private_ip(&IpAddr::V6(cloudflare)), "2606:4700:4700::1111 (Cloudflare DNS) should be public");
    }

    #[test]
    fn test_validate_public_url_localhost_string() {
        // Explicit localhost string should be blocked
        assert!(validate_public_url("http://localhost").is_err(), "http://localhost should be blocked");
        assert!(validate_public_url("https://localhost").is_err(), "https://localhost should be blocked");
        assert!(validate_public_url("http://localhost:8080").is_err(), "http://localhost:8080 should be blocked");
        assert!(validate_public_url("http://LOCALHOST").is_err(), "http://LOCALHOST should be blocked (case-insensitive)");
        assert!(validate_public_url("http://LocalHost").is_err(), "http://LocalHost should be blocked");
    }

    #[test]
    fn test_validate_public_url_loopback_ips() {
        // 127.x.x.x should be blocked
        assert!(validate_public_url("http://127.0.0.1").is_err(), "http://127.0.0.1 should be blocked");
        assert!(validate_public_url("http://127.0.0.1:8080").is_err(), "http://127.0.0.1:8080 should be blocked");
        assert!(validate_public_url("http://127.1.1.1").is_err(), "http://127.1.1.1 should be blocked");
        assert!(validate_public_url("http://127.255.255.255").is_err(), "http://127.255.255.255 should be blocked");
    }

    #[test]
    fn test_validate_public_url_rfc1918_10() {
        // 10.x.x.x should be blocked
        assert!(validate_public_url("http://10.0.0.1").is_err(), "http://10.0.0.1 should be blocked");
        assert!(validate_public_url("http://10.0.0.1:8080").is_err(), "http://10.0.0.1:8080 should be blocked");
        assert!(validate_public_url("http://10.123.45.67").is_err(), "http://10.123.45.67 should be blocked");
        assert!(validate_public_url("http://10.255.255.255").is_err(), "http://10.255.255.255 should be blocked");
    }

    #[test]
    fn test_validate_public_url_rfc1918_172() {
        // 172.16.x.x - 172.31.x.x should be blocked
        assert!(validate_public_url("http://172.16.0.1").is_err(), "http://172.16.0.1 should be blocked");
        assert!(validate_public_url("http://172.20.10.5").is_err(), "http://172.20.10.5 should be blocked");
        assert!(validate_public_url("http://172.31.255.255").is_err(), "http://172.31.255.255 should be blocked");
        
        // Edge cases - just outside range should be allowed (if they resolve)
        // Note: These tests may fail if the IPs don't resolve or are unreachable
        // In production, 172.15.x.x and 172.32.x.x are public but may not resolve
    }

    #[test]
    fn test_validate_public_url_rfc1918_192() {
        // 192.168.x.x should be blocked
        assert!(validate_public_url("http://192.168.0.1").is_err(), "http://192.168.0.1 should be blocked");
        assert!(validate_public_url("http://192.168.1.1").is_err(), "http://192.168.1.1 should be blocked");
        assert!(validate_public_url("http://192.168.1.1:8080").is_err(), "http://192.168.1.1:8080 should be blocked");
        assert!(validate_public_url("http://192.168.255.255").is_err(), "http://192.168.255.255 should be blocked");
    }

    #[test]
    fn test_validate_public_url_link_local() {
        // 169.254.x.x should be blocked
        assert!(validate_public_url("http://169.254.1.1").is_err(), "http://169.254.1.1 should be blocked");
        assert!(validate_public_url("http://169.254.169.254").is_err(), "http://169.254.169.254 (AWS metadata) should be blocked");
    }

    #[test]
    fn test_validate_public_url_ipv6_loopback() {
        // ::1 should be blocked
        assert!(validate_public_url("http://[::1]").is_err(), "http://[::1] should be blocked");
        assert!(validate_public_url("http://[::1]:8080").is_err(), "http://[::1]:8080 should be blocked");
    }

    #[test]
    fn test_validate_public_url_ipv6_link_local() {
        // fe80:: should be blocked
        assert!(validate_public_url("http://[fe80::1]").is_err(), "http://[fe80::1] should be blocked");
    }

    #[test]
    fn test_validate_public_url_ipv6_unique_local() {
        // fc00::/7 should be blocked
        assert!(validate_public_url("http://[fc00::1]").is_err(), "http://[fc00::1] should be blocked");
        assert!(validate_public_url("http://[fd00::1]").is_err(), "http://[fd00::1] should be blocked");
    }

    #[test]
    #[ignore] // Requires network access
    fn test_validate_public_url_public_domains() {
        // Well-known public domains should be allowed
        assert!(validate_public_url("https://google.com").is_ok(), "https://google.com should be allowed");
        assert!(validate_public_url("https://github.com").is_ok(), "https://github.com should be allowed");
        assert!(validate_public_url("https://example.com").is_ok(), "https://example.com should be allowed");
        assert!(validate_public_url("https://cloudflare.com").is_ok(), "https://cloudflare.com should be allowed");
    }

    #[test]
    #[ignore] // Requires network access and may fail if localhost resolves
    fn test_validate_public_url_localhost_dns_resolution() {
        // If "localhost" resolves to 127.0.0.1, it should be blocked
        // This test is ignored because DNS behavior varies by system
        let result = validate_public_url("http://localhost");
        assert!(result.is_err(), "localhost should be blocked even after DNS resolution");
    }

    #[test]
    fn test_validate_public_url_invalid_format() {
        // Invalid URL formats should be rejected
        assert!(validate_public_url("").is_err(), "Empty URL should be rejected");
        assert!(validate_public_url("not-a-url").is_err(), "Invalid URL should be rejected");
        assert!(validate_public_url("ftp://example.com").is_err(), "FTP URL should be rejected");
    }

    #[test]
    fn test_validate_public_url_with_path_and_query() {
        // URLs with paths and query strings should work (if host is public)
        // These tests use direct IPs to avoid DNS resolution
        
        // Private IPs with paths should still be blocked
        assert!(validate_public_url("http://127.0.0.1/api/data").is_err(), "http://127.0.0.1/api/data should be blocked");
        assert!(validate_public_url("http://192.168.1.1/admin?token=abc").is_err(), "http://192.168.1.1/admin?token=abc should be blocked");
        assert!(validate_public_url("http://10.0.0.1:8080/secret").is_err(), "http://10.0.0.1:8080/secret should be blocked");
    }

    #[test]
    fn test_validate_public_url_returns_parsed_url() {
        // For valid public URLs, should return parsed URL object
        // Using direct public IP to avoid DNS resolution in tests
        let result = validate_public_url("http://8.8.8.8");
        assert!(result.is_ok(), "http://8.8.8.8 should be allowed");
        
        let parsed = result.unwrap();
        assert_eq!(parsed.scheme(), "http");
        assert_eq!(parsed.host_str(), Some("8.8.8.8"));
    }

    // ==================== Capabilities Validators (winws) ====================
    
    /// Tests for winws argument validation based on capabilities/default.json validators
    mod capabilities_tests {
        use super::*;

        // ==================== Flag Arguments ====================
        
        #[test]
        fn test_winws_flag_wf_tcp() {
            assert!(validate_winws_arg("--wf-tcp"), "--wf-tcp should be valid");
        }

        #[test]
        fn test_winws_flag_wf_udp() {
            assert!(validate_winws_arg("--wf-udp"), "--wf-udp should be valid");
        }

        #[test]
        fn test_winws_flag_filter() {
            assert!(validate_winws_arg("--filter"), "--filter should be valid");
        }

        #[test]
        fn test_winws_flag_hostlist() {
            assert!(validate_winws_arg("--hostlist"), "--hostlist should be valid");
            assert!(validate_winws_arg("--hostlist-exclude"), "--hostlist-exclude should be valid");
            assert!(validate_winws_arg("--hostlist-domains"), "--hostlist-domains should be valid");
        }

        #[test]
        fn test_winws_flag_ipset() {
            assert!(validate_winws_arg("--ipset"), "--ipset should be valid");
            assert!(validate_winws_arg("--ipset-exclude"), "--ipset-exclude should be valid");
        }

        #[test]
        fn test_winws_flag_dpi_desync() {
            assert!(validate_winws_arg("--dpi-desync"), "--dpi-desync should be valid");
            assert!(validate_winws_arg("--dpi-desync-l7"), "--dpi-desync-l7 should be valid");
            assert!(validate_winws_arg("--dpi-desync-ttl"), "--dpi-desync-ttl should be valid");
            assert!(validate_winws_arg("--dpi-desync-ttl6"), "--dpi-desync-ttl6 should be valid");
            assert!(validate_winws_arg("--dpi-desync-autottl"), "--dpi-desync-autottl should be valid");
            assert!(validate_winws_arg("--dpi-desync-fooling"), "--dpi-desync-fooling should be valid");
            assert!(validate_winws_arg("--dpi-desync-repeats"), "--dpi-desync-repeats should be valid");
            assert!(validate_winws_arg("--dpi-desync-split-pos"), "--dpi-desync-split-pos should be valid");
            assert!(validate_winws_arg("--dpi-desync-split-seqovl"), "--dpi-desync-split-seqovl should be valid");
            assert!(validate_winws_arg("--dpi-desync-cutoff"), "--dpi-desync-cutoff should be valid");
        }

        #[test]
        fn test_winws_flag_dpi_desync_fake() {
            assert!(validate_winws_arg("--dpi-desync-fake-tls"), "--dpi-desync-fake-tls should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-quic"), "--dpi-desync-fake-quic should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-tls-mod"), "--dpi-desync-fake-tls-mod should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-wireguard"), "--dpi-desync-fake-wireguard should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-dht"), "--dpi-desync-fake-dht should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-unknown-udp"), "--dpi-desync-fake-unknown-udp should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-tcp-mod"), "--dpi-desync-fake-tcp-mod should be valid");
            assert!(validate_winws_arg("--dpi-desync-fake-syndata"), "--dpi-desync-fake-syndata should be valid");
        }

        #[test]
        fn test_winws_flag_dpi_desync_dup() {
            assert!(validate_winws_arg("--dpi-desync-dup"), "--dpi-desync-dup should be valid");
            assert!(validate_winws_arg("--dpi-desync-dup-replace"), "--dpi-desync-dup-replace should be valid");
            assert!(validate_winws_arg("--dpi-desync-dup-ttl"), "--dpi-desync-dup-ttl should be valid");
            assert!(validate_winws_arg("--dpi-desync-dup-autottl"), "--dpi-desync-dup-autottl should be valid");
            assert!(validate_winws_arg("--dpi-desync-dup-fooling"), "--dpi-desync-dup-fooling should be valid");
        }

        #[test]
        fn test_winws_flag_wsize() {
            assert!(validate_winws_arg("--wsize"), "--wsize should be valid");
            assert!(validate_winws_arg("--wssize"), "--wssize should be valid");
            assert!(validate_winws_arg("--wssize-cutoff"), "--wssize-cutoff should be valid");
        }

        #[test]
        fn test_winws_flag_filter_l() {
            assert!(validate_winws_arg("--filter-l3"), "--filter-l3 should be valid");
            assert!(validate_winws_arg("--filter-l7"), "--filter-l7 should be valid");
            assert!(validate_winws_arg("--filter-ssid"), "--filter-ssid should be valid");
            assert!(validate_winws_arg("--nlm-filter"), "--nlm-filter should be valid");
        }

        #[test]
        fn test_winws_flag_new() {
            assert!(validate_winws_arg("--new"), "--new should be valid");
        }

        // ==================== Value Arguments ====================

        #[test]
        fn test_winws_value_port_numbers() {
            // Single port
            assert!(validate_winws_arg("80"), "80 should be valid");
            assert!(validate_winws_arg("443"), "443 should be valid");
            assert!(validate_winws_arg("8080"), "8080 should be valid");
            
            // Multiple ports
            assert!(validate_winws_arg("80,443"), "80,443 should be valid");
            assert!(validate_winws_arg("80,443,8080"), "80,443,8080 should be valid");
            assert!(validate_winws_arg("1,2,3,4,5"), "1,2,3,4,5 should be valid");
        }

        #[test]
        fn test_winws_value_protocols() {
            let protocols = ["tcp", "udp", "ip", "ipv4", "ipv6", "http", "tls", "quic", "dns", "stun", "discord", "wireguard", "dht", "unknown"];
            for proto in protocols {
                assert!(validate_winws_arg(proto), "{} should be valid protocol", proto);
            }
        }

        #[test]
        fn test_winws_value_desync_methods() {
            // Single methods
            assert!(validate_winws_arg("fake"), "fake should be valid");
            assert!(validate_winws_arg("split"), "split should be valid");
            assert!(validate_winws_arg("split2"), "split2 should be valid");
            assert!(validate_winws_arg("disorder"), "disorder should be valid");
            assert!(validate_winws_arg("disorder2"), "disorder2 should be valid");
            assert!(validate_winws_arg("fakedsplit"), "fakedsplit should be valid");
            assert!(validate_winws_arg("multisplit"), "multisplit should be valid");
            assert!(validate_winws_arg("multidisorder"), "multidisorder should be valid");
            assert!(validate_winws_arg("hostfakesplit"), "hostfakesplit should be valid");
            
            // Combined methods
            assert!(validate_winws_arg("fake,split"), "fake,split should be valid");
            assert!(validate_winws_arg("fake,split2"), "fake,split2 should be valid");
            assert!(validate_winws_arg("disorder,split"), "disorder,split should be valid");
            assert!(validate_winws_arg("fake,disorder,split"), "fake,disorder,split should be valid");
            
            // Special combinations
            assert!(validate_winws_arg("hopbyhop"), "hopbyhop should be valid");
            assert!(validate_winws_arg("destopt"), "destopt should be valid");
            assert!(validate_winws_arg("ipfrag1+destopt"), "ipfrag1+destopt should be valid");
            assert!(validate_winws_arg("hopbyhop+destopt"), "hopbyhop+destopt should be valid");
        }

        #[test]
        fn test_winws_value_fooling_methods() {
            // Single methods
            assert!(validate_winws_arg("md5sig"), "md5sig should be valid");
            assert!(validate_winws_arg("badsum"), "badsum should be valid");
            assert!(validate_winws_arg("datanoack"), "datanoack should be valid");
            assert!(validate_winws_arg("badseq"), "badseq should be valid");
            assert!(validate_winws_arg("badack"), "badack should be valid");
            
            // Combined methods
            assert!(validate_winws_arg("md5sig,badsum"), "md5sig,badsum should be valid");
            assert!(validate_winws_arg("badseq,badack"), "badseq,badack should be valid");
            assert!(validate_winws_arg("md5sig,badsum,datanoack"), "md5sig,badsum,datanoack should be valid");
        }

        #[test]
        fn test_winws_value_ttl_numbers() {
            assert!(validate_winws_arg("5"), "5 should be valid TTL");
            assert!(validate_winws_arg("10"), "10 should be valid TTL");
            assert!(validate_winws_arg("128"), "128 should be valid TTL");
        }

        #[test]
        fn test_winws_value_autottl_format() {
            // Format: min:max:delta
            assert!(validate_winws_arg("1:5:1"), "1:5:1 should be valid autottl");
            assert!(validate_winws_arg("2:10:2"), "2:10:2 should be valid autottl");
            assert!(validate_winws_arg("5:64:5"), "5:64:5 should be valid autottl");
        }

        #[test]
        fn test_winws_value_cutoff_format() {
            // Format: n/d/s + number
            assert!(validate_winws_arg("n5"), "n5 should be valid cutoff");
            assert!(validate_winws_arg("d10"), "d10 should be valid cutoff");
            assert!(validate_winws_arg("s100"), "s100 should be valid cutoff");
            assert!(validate_winws_arg("n1"), "n1 should be valid cutoff");
        }

        #[test]
        fn test_winws_value_windows_path() {
            assert!(validate_winws_arg(r"C:\path\to\file.txt"), r"C:\path\to\file.txt should be valid");
            assert!(validate_winws_arg(r"D:\hostlist\youtube.txt"), r"D:\hostlist\youtube.txt should be valid");
            assert!(validate_winws_arg(r"C:\Program Files\Isolate\hostlists\discord.txt"), "Path with spaces should be valid");
        }

        #[test]
        fn test_winws_value_alphanumeric() {
            assert!(validate_winws_arg("youtube.txt"), "youtube.txt should be valid");
            assert!(validate_winws_arg("discord-hosts.txt"), "discord-hosts.txt should be valid");
            assert!(validate_winws_arg("my_hostlist.txt"), "my_hostlist.txt should be valid");
            assert!(validate_winws_arg("file123"), "file123 should be valid");
        }

        // ==================== Invalid Arguments ====================

        #[test]
        fn test_winws_invalid_flags() {
            assert!(!validate_winws_arg("--invalid-flag"), "--invalid-flag should be invalid");
            assert!(!validate_winws_arg("--exec"), "--exec should be invalid (security)");
            assert!(!validate_winws_arg("--shell"), "--shell should be invalid (security)");
            assert!(!validate_winws_arg("-c"), "-c should be invalid");
            assert!(!validate_winws_arg("--help"), "--help should be invalid (not in whitelist)");
        }

        #[test]
        fn test_winws_invalid_values() {
            assert!(!validate_winws_arg("invalid_protocol"), "invalid_protocol should be invalid");
            assert!(!validate_winws_arg("abc"), "abc should be invalid (not a valid protocol)");
            assert!(!validate_winws_arg("80;443"), "80;443 should be invalid (semicolon not allowed)");
            assert!(!validate_winws_arg("80|443"), "80|443 should be invalid (pipe not allowed)");
        }

        #[test]
        fn test_winws_invalid_injection_attempts() {
            // Command injection attempts
            assert!(!validate_winws_arg("; rm -rf /"), "Command injection should be invalid");
            assert!(!validate_winws_arg("| cat /etc/passwd"), "Pipe injection should be invalid");
            assert!(!validate_winws_arg("$(whoami)"), "Command substitution should be invalid");
            assert!(!validate_winws_arg("`whoami`"), "Backtick injection should be invalid");
            assert!(!validate_winws_arg("&& malicious"), "AND injection should be invalid");
            assert!(!validate_winws_arg("|| malicious"), "OR injection should be invalid");
        }

        #[test]
        fn test_winws_invalid_path_traversal() {
            assert!(!validate_winws_arg("../../../etc/passwd"), "Path traversal should be invalid");
            assert!(!validate_winws_arg("..\\..\\windows\\system32"), "Windows path traversal should be invalid");
        }

        // ==================== Full Command Line Validation ====================

        #[test]
        fn test_winws_cmdline_basic() {
            assert!(validate_winws_cmdline("--wf-tcp 80,443").is_ok(), "Basic TCP filter should be valid");
            assert!(validate_winws_cmdline("--wf-udp 443").is_ok(), "Basic UDP filter should be valid");
        }

        #[test]
        fn test_winws_cmdline_with_equals() {
            assert!(validate_winws_cmdline("--wf-tcp=80,443").is_ok(), "--wf-tcp=80,443 should be valid");
            assert!(validate_winws_cmdline("--dpi-desync=fake,split").is_ok(), "--dpi-desync=fake,split should be valid");
            assert!(validate_winws_cmdline("--dpi-desync-ttl=5").is_ok(), "--dpi-desync-ttl=5 should be valid");
        }

        #[test]
        fn test_winws_cmdline_complex_strategy() {
            let cmdline = "--wf-tcp 80,443 --dpi-desync fake,split2 --dpi-desync-ttl 5 --dpi-desync-fooling md5sig";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Complex strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_youtube_strategy() {
            // Typical YouTube bypass strategy
            let cmdline = "--wf-tcp 80,443 --dpi-desync fake,split2 --dpi-desync-ttl 3 --dpi-desync-fooling md5sig,badsum";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "YouTube strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_discord_strategy() {
            // Typical Discord bypass strategy
            let cmdline = "--wf-udp 443 --dpi-desync fake --dpi-desync-repeats 6";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Discord strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_with_hostlist() {
            let cmdline = r"--wf-tcp 80,443 --hostlist C:\Isolate\hostlists\youtube.txt --dpi-desync fake";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Strategy with hostlist should be valid");
        }

        #[test]
        fn test_winws_cmdline_multisplit() {
            let cmdline = "--wf-tcp 80,443 --dpi-desync multisplit --dpi-desync-split-pos 1,2,3";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Multisplit strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_with_new_filter() {
            let cmdline = "--new --wf-tcp 80 --dpi-desync fake --new --wf-tcp 443 --dpi-desync split";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Multiple filters with --new should be valid");
        }

        #[test]
        fn test_winws_cmdline_invalid() {
            let result = validate_winws_cmdline("--wf-tcp 80,443 --invalid-flag value");
            assert!(result.is_err(), "Invalid flag should fail");
            let invalid = result.unwrap_err();
            assert!(invalid.contains(&"--invalid-flag".to_string()), "Should report --invalid-flag as invalid");
        }

        #[test]
        fn test_winws_cmdline_injection_attempt() {
            let result = validate_winws_cmdline("--wf-tcp 80,443 ; rm -rf /");
            assert!(result.is_err(), "Injection attempt should fail");
        }

        // ==================== validate_winws_args ====================

        #[test]
        fn test_winws_args_array_valid() {
            let args = vec!["--wf-tcp", "80,443", "--dpi-desync", "fake,split"];
            assert!(validate_winws_args(&args).is_ok(), "Valid args array should pass");
        }

        #[test]
        fn test_winws_args_array_invalid() {
            let args = vec!["--wf-tcp", "80,443", "--invalid", "value"];
            let result = validate_winws_args(&args);
            assert!(result.is_err(), "Invalid args should fail");
            let invalid = result.unwrap_err();
            assert_eq!(invalid.len(), 2, "Should have 2 invalid args");
            assert!(invalid.contains(&"--invalid".to_string()));
            assert!(invalid.contains(&"value".to_string()));
        }

        #[test]
        fn test_winws_args_empty() {
            let args: Vec<&str> = vec![];
            assert!(validate_winws_args(&args).is_ok(), "Empty args should be valid");
        }

        // ==================== Additional Port Validation Tests ====================

        #[test]
        fn test_winws_port_edge_cases() {
            // Minimum valid port
            assert!(validate_winws_arg("1"), "Port 1 should be valid");
            
            // Maximum valid port (65535)
            assert!(validate_winws_arg("65535"), "Port 65535 should be valid");
            
            // Port ranges
            assert!(validate_winws_arg("1,65535"), "Port range 1,65535 should be valid");
            assert!(validate_winws_arg("80,443,8080,8443"), "Multiple common ports should be valid");
        }

        #[test]
        fn test_winws_port_invalid_values() {
            // Port > 65535 - this is a string, regex allows any digits
            // The semantic validation (port > 65535) should be done at a higher level
            // Here we test that non-numeric values are rejected
            assert!(!validate_winws_arg("80a"), "Port with letter should be invalid");
            assert!(!validate_winws_arg("port80"), "Port with prefix should be invalid");
            assert!(!validate_winws_arg("-80"), "Negative port should be invalid");
            assert!(!validate_winws_arg("80.0"), "Port with decimal should be invalid");
        }

        // ==================== Domain/Hostlist Pattern Tests ====================

        #[test]
        fn test_winws_domain_patterns() {
            // Domain-like values (alphanumeric with dots/hyphens)
            assert!(validate_winws_arg("youtube.com"), "youtube.com should be valid");
            assert!(validate_winws_arg("google.com"), "google.com should be valid");
            assert!(validate_winws_arg("sub.domain.com"), "sub.domain.com should be valid");
            assert!(validate_winws_arg("my-domain.net"), "my-domain.net should be valid");
        }

        #[test]
        fn test_winws_wildcard_domains_invalid() {
            // Wildcard domains are NOT valid as direct arguments
            // They should be in hostlist files, not command line
            assert!(!validate_winws_arg("*.google.com"), "Wildcard domain should be invalid");
            assert!(!validate_winws_arg("*.youtube.com"), "Wildcard domain should be invalid");
            assert!(!validate_winws_arg("*"), "Single wildcard should be invalid");
        }

        // ==================== Path Validation Tests ====================

        #[test]
        fn test_winws_relative_paths_invalid() {
            // Relative paths are NOT valid - only absolute Windows paths
            assert!(!validate_winws_arg("configs/hostlists/general.txt"), "Unix relative path should be invalid");
            assert!(!validate_winws_arg("./hostlists/youtube.txt"), "Relative path with ./ should be invalid");
            assert!(!validate_winws_arg("hostlists/discord.txt"), "Simple relative path should be invalid");
        }

        #[test]
        fn test_winws_absolute_windows_paths() {
            // Valid Windows absolute paths
            assert!(validate_winws_arg(r"C:\hostlists\general.txt"), "C: drive path should be valid");
            assert!(validate_winws_arg(r"D:\configs\strategies\youtube.txt"), "D: drive path should be valid");
            assert!(validate_winws_arg(r"E:\Isolate\hostlists\discord.txt"), "E: drive path should be valid");
            
            // Path with multiple subdirectories
            assert!(validate_winws_arg(r"C:\Users\User\AppData\Local\Isolate\hostlists\general.txt"), 
                "Deep path should be valid");
        }

        #[test]
        fn test_winws_path_invalid_characters() {
            // Paths with invalid characters
            assert!(!validate_winws_arg(r"C:\path\file<name>.txt"), "Path with < should be invalid");
            assert!(!validate_winws_arg(r"C:\path\file>name.txt"), "Path with > should be invalid");
            assert!(!validate_winws_arg(r#"C:\path\file"name.txt"#), "Path with quote should be invalid");
            assert!(!validate_winws_arg(r"C:\path\file|name.txt"), "Path with pipe should be invalid");
            assert!(!validate_winws_arg(r"C:\path\file?name.txt"), "Path with ? should be invalid");
            assert!(!validate_winws_arg(r"C:\path\file*name.txt"), "Path with * should be invalid");
        }

        // ==================== TTL Validation Tests ====================

        #[test]
        fn test_winws_ttl_values() {
            // Valid TTL values (1-255 typically)
            assert!(validate_winws_arg("1"), "TTL 1 should be valid");
            assert!(validate_winws_arg("5"), "TTL 5 should be valid");
            assert!(validate_winws_arg("64"), "TTL 64 should be valid");
            assert!(validate_winws_arg("128"), "TTL 128 should be valid");
            assert!(validate_winws_arg("255"), "TTL 255 should be valid");
        }

        #[test]
        fn test_winws_autottl_extended() {
            // Extended autottl format tests
            assert!(validate_winws_arg("1:255:1"), "Full TTL range should be valid");
            assert!(validate_winws_arg("3:10:1"), "Common autottl should be valid");
            assert!(validate_winws_arg("5:64:5"), "Step 5 autottl should be valid");
        }

        // ==================== Empty/Whitespace Tests ====================

        #[test]
        fn test_winws_empty_and_whitespace() {
            // Empty string
            assert!(!validate_winws_arg(""), "Empty string should be invalid");
            
            // Whitespace only
            assert!(!validate_winws_arg(" "), "Single space should be invalid");
            assert!(!validate_winws_arg("  "), "Multiple spaces should be invalid");
            assert!(!validate_winws_arg("\t"), "Tab should be invalid");
            assert!(!validate_winws_arg("\n"), "Newline should be invalid");
        }

        // ==================== Command Injection Prevention Tests ====================

        #[test]
        fn test_winws_shell_metacharacters() {
            // Shell metacharacters that could be used for injection
            assert!(!validate_winws_arg("$HOME"), "Shell variable should be invalid");
            assert!(!validate_winws_arg("${PATH}"), "Shell variable expansion should be invalid");
            assert!(!validate_winws_arg("%USERPROFILE%"), "Windows env var should be invalid");
            assert!(!validate_winws_arg("!important!"), "Exclamation marks should be invalid");
        }

        #[test]
        fn test_winws_command_chaining() {
            // Command chaining attempts
            assert!(!validate_winws_arg("80 && echo pwned"), "AND chaining should be invalid");
            assert!(!validate_winws_arg("80 || echo pwned"), "OR chaining should be invalid");
            assert!(!validate_winws_arg("80; echo pwned"), "Semicolon chaining should be invalid");
            assert!(!validate_winws_arg("80\necho pwned"), "Newline chaining should be invalid");
        }

        #[test]
        fn test_winws_subshell_injection() {
            // Subshell/command substitution attempts
            assert!(!validate_winws_arg("$(cat /etc/passwd)"), "Subshell should be invalid");
            assert!(!validate_winws_arg("`cat /etc/passwd`"), "Backticks should be invalid");
            assert!(!validate_winws_arg("$((1+1))"), "Arithmetic expansion should be invalid");
        }

        #[test]
        fn test_winws_redirect_injection() {
            // Redirect attempts
            assert!(!validate_winws_arg("> /tmp/pwned"), "Output redirect should be invalid");
            assert!(!validate_winws_arg("< /etc/passwd"), "Input redirect should be invalid");
            assert!(!validate_winws_arg(">> /tmp/pwned"), "Append redirect should be invalid");
            assert!(!validate_winws_arg("2>&1"), "Stderr redirect should be invalid");
        }

        // ==================== Real-World Strategy Command Lines ====================

        #[test]
        fn test_winws_cmdline_youtube_full() {
            // Full YouTube bypass strategy from configs
            let cmdline = "--wf-tcp 80,443 --dpi-desync fake,split2 --dpi-desync-ttl 3 --dpi-desync-fooling md5sig,badsum --dpi-desync-split-pos 1";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Full YouTube strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_discord_full() {
            // Full Discord bypass strategy
            let cmdline = "--wf-udp 443 --filter-l7 quic --dpi-desync fake --dpi-desync-repeats 6 --dpi-desync-fake-quic";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Full Discord strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_general_full() {
            // General strategy with hostlist
            let cmdline = r"--wf-tcp 80,443 --hostlist C:\Isolate\hostlists\general.txt --dpi-desync fake,disorder2 --dpi-desync-ttl 5 --dpi-desync-autottl 2:10:2";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "General strategy with hostlist should be valid");
        }

        #[test]
        fn test_winws_cmdline_multi_filter() {
            // Multiple filters using --new
            let cmdline = "--new --wf-tcp 80 --dpi-desync fake --new --wf-tcp 443 --dpi-desync split2 --new --wf-udp 443 --dpi-desync fake";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Multi-filter strategy should be valid");
        }

        #[test]
        fn test_winws_cmdline_with_cutoff() {
            // Strategy with cutoff parameters
            let cmdline = "--wf-tcp 80,443 --dpi-desync fake,split --dpi-desync-cutoff n5 --wssize-cutoff d10";
            assert!(validate_winws_cmdline(cmdline).is_ok(), "Strategy with cutoff should be valid");
        }
    }
}

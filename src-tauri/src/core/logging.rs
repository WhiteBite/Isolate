//! Logging utilities for sensitive data masking
//!
//! Provides functions to mask sensitive information in logs
//! to prevent accidental exposure of credentials, IPs, etc.
//!
//! NOTE: Some masking functions are prepared for future logging features.

// Public API for log masking
#![allow(dead_code)]

/// Masks a UUID, showing only first 8 characters
/// Example: "550e8400-e29b-41d4-a716-446655440000" -> "550e8400..."
pub fn mask_uuid(uuid: &str) -> String {
    if uuid.len() > 8 {
        format!("{}...", &uuid[..8])
    } else {
        "***".to_string()
    }
}

/// Masks an IP address, showing only first octet
/// Example: "192.168.1.100" -> "192.***"
pub fn mask_ip(ip: &str) -> String {
    if let Some(first_dot) = ip.find('.') {
        format!("{}.***", &ip[..first_dot])
    } else if ip.contains(':') {
        // IPv6
        if let Some(first_colon) = ip.find(':') {
            format!("{}:***", &ip[..first_colon])
        } else {
            "***".to_string()
        }
    } else {
        ip.to_string() // localhost or hostname
    }
}

/// Masks a URL, hiding path and query parameters
/// Example: "https://api.example.com/path?token=secret" -> "https://api.***/<masked>"
pub fn mask_url(url: &str) -> String {
    // Simple parsing without url crate dependency
    if let Some(scheme_end) = url.find("://") {
        let after_scheme = &url[scheme_end + 3..];
        if let Some(path_start) = after_scheme.find('/') {
            let host = &after_scheme[..path_start];
            let scheme = &url[..scheme_end];
            return format!("{}://{}/***", scheme, mask_ip(host));
        } else {
            return format!("{}://{}", &url[..scheme_end], mask_ip(after_scheme));
        }
    }
    "***".to_string()
}

/// Masks proxy host for logging
pub fn mask_proxy_host(host: &str) -> String {
    mask_ip(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_uuid() {
        assert_eq!(mask_uuid("550e8400-e29b-41d4-a716-446655440000"), "550e8400...");
        assert_eq!(mask_uuid("short"), "***");
        assert_eq!(mask_uuid("12345678"), "***");
        assert_eq!(mask_uuid("123456789"), "12345678...");
    }

    #[test]
    fn test_mask_ip_v4() {
        assert_eq!(mask_ip("192.168.1.100"), "192.***");
        assert_eq!(mask_ip("10.0.0.1"), "10.***");
        assert_eq!(mask_ip("localhost"), "localhost");
    }

    #[test]
    fn test_mask_ip_v6() {
        assert_eq!(mask_ip("2001:db8::1"), "2001:***");
        assert_eq!(mask_ip("::1"), ":***");
    }

    #[test]
    fn test_mask_url() {
        assert_eq!(
            mask_url("https://api.example.com/path?token=secret"),
            "https://api.***/***"
        );
        assert_eq!(
            mask_url("http://192.168.1.1:8080/api"),
            "http://192.***/***"
        );
    }

    #[test]
    fn test_mask_proxy_host() {
        assert_eq!(mask_proxy_host("proxy.example.com"), "proxy.***");
        assert_eq!(mask_proxy_host("192.168.1.1"), "192.***");
    }
}

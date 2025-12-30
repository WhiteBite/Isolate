//! Sing-box configuration generator for all proxy protocols
//!
//! Generates JSON configuration for sing-box supporting:
//! - VLESS, VMess, Shadowsocks, Trojan
//! - TUIC, Hysteria/Hysteria2
//! - SOCKS5, HTTP/HTTPS proxies
//! - Domain and application-based routing
//! - TUN mode with fake-ip DNS

use crate::core::models::{AppRoute, DomainRoute, ProxyConfig, ProxyProtocol};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// DNS mode for configuration generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DnsMode {
    /// Direct DNS resolution (for SOCKS/HTTP proxy mode)
    Direct,
    /// Fake-IP DNS (for TUN mode)
    FakeIp,
}

/// Generates complete sing-box configuration
pub fn generate_singbox_config(
    proxies: &[ProxyConfig],
    domain_routes: &[DomainRoute],
    app_routes: &[AppRoute],
    socks_port: u16,
    http_port: u16,
) -> Result<Value> {
    let mut outbounds = vec![
        json!({
            "type": "direct",
            "tag": "direct"
        }),
        json!({
            "type": "block",
            "tag": "block"
        }),
        json!({
            "type": "dns",
            "tag": "dns-out"
        }),
    ];

    // Add outbound for each proxy
    for proxy in proxies {
        match generate_outbound(proxy) {
            Ok(outbound) => outbounds.push(outbound),
            Err(e) => {
                tracing::warn!("Failed to generate outbound for {}: {}", proxy.id, e);
            }
        }
    }

    // Generate routing rules
    let route = generate_routing(domain_routes, app_routes, proxies);

    // Generate DNS configuration
    let dns = generate_dns_config(proxies);

    Ok(json!({
        "log": {
            "level": "info",
            "timestamp": true
        },
        "dns": dns,
        "inbounds": [
            {
                "type": "socks",
                "tag": "socks-in",
                "listen": "127.0.0.1",
                "listen_port": socks_port,
                "sniff": true,
                "sniff_override_destination": true,
                "sniff_timeout": "300ms",
                "domain_strategy": "prefer_ipv4"
            },
            {
                "type": "http",
                "tag": "http-in",
                "listen": "127.0.0.1",
                "listen_port": http_port,
                "sniff": true,
                "sniff_override_destination": true
            }
        ],
        "outbounds": outbounds,
        "route": route,
        "experimental": {
            "cache_file": {
                "enabled": false
            }
        }
    }))
}

/// Generates outbound configuration for a specific proxy
pub fn generate_outbound(proxy: &ProxyConfig) -> Result<Value> {
    match proxy.protocol {
        ProxyProtocol::Vless => generate_vless_outbound(proxy),
        ProxyProtocol::Vmess => generate_vmess_outbound(proxy),
        ProxyProtocol::Shadowsocks => generate_shadowsocks_outbound(proxy),
        ProxyProtocol::Trojan => generate_trojan_outbound(proxy),
        ProxyProtocol::Tuic => generate_tuic_outbound(proxy),
        ProxyProtocol::Hysteria | ProxyProtocol::Hysteria2 => generate_hysteria_outbound(proxy),
        ProxyProtocol::Socks5 => generate_socks_outbound(proxy),
        ProxyProtocol::Http | ProxyProtocol::Https => generate_http_outbound(proxy),
        ProxyProtocol::Wireguard => generate_wireguard_outbound(proxy),
        ProxyProtocol::Ssh => Err(anyhow!("SSH protocol not supported by sing-box")),
    }
}

// ============================================================================
// VLESS Outbound
// ============================================================================

fn generate_vless_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let uuid = proxy.uuid.as_ref().ok_or_else(|| anyhow!("VLESS requires UUID"))?;

    let mut outbound = json!({
        "type": "vless",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "uuid": uuid,
        "packet_encoding": "xudp"
    });

    // Add flow if specified
    if let Some(flow) = proxy.custom_fields.get("flow") {
        if !flow.is_empty() {
            outbound["flow"] = json!(flow);
        }
    }

    // Add TLS configuration
    if proxy.tls {
        let mut tls = json!({
            "enabled": true,
            "insecure": false
        });

        if let Some(ref sni) = proxy.sni {
            tls["server_name"] = json!(sni);
        }

        // Check for Reality
        if proxy.custom_fields.get("security").map(|s| s.as_str()) == Some("reality") {
            tls["reality"] = json!({
                "enabled": true,
                "public_key": proxy.custom_fields.get("pbk").unwrap_or(&String::new()),
                "short_id": proxy.custom_fields.get("sid").unwrap_or(&String::new())
            });
        } else {
            // Regular TLS with UTLS fingerprint
            let fingerprint = proxy.custom_fields.get("fp")
                .map(|s| s.as_str())
                .unwrap_or("chrome");
            
            tls["utls"] = json!({
                "enabled": true,
                "fingerprint": fingerprint
            });

            // ALPN
            if let Some(alpn) = proxy.custom_fields.get("alpn") {
                let alpn_list: Vec<&str> = alpn.split(',').collect();
                tls["alpn"] = json!(alpn_list);
            }
        }

        outbound["tls"] = tls;
    }

    // Add transport configuration
    add_transport_config(&mut outbound, proxy)?;

    Ok(outbound)
}

// ============================================================================
// VMess Outbound
// ============================================================================

fn generate_vmess_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let uuid = proxy.uuid.as_ref().ok_or_else(|| anyhow!("VMess requires UUID"))?;

    let mut outbound = json!({
        "type": "vmess",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "uuid": uuid,
        "security": proxy.custom_fields.get("encryption").unwrap_or(&"auto".to_string()),
        "alter_id": proxy.custom_fields.get("aid")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
    });

    // Add TLS if enabled
    if proxy.tls {
        let mut tls = json!({
            "enabled": true,
            "insecure": false
        });

        if let Some(ref sni) = proxy.sni {
            tls["server_name"] = json!(sni);
        }

        let fingerprint = proxy.custom_fields.get("fp")
            .map(|s| s.as_str())
            .unwrap_or("chrome");
        
        tls["utls"] = json!({
            "enabled": true,
            "fingerprint": fingerprint
        });

        outbound["tls"] = tls;
    }

    // Add transport configuration
    add_transport_config(&mut outbound, proxy)?;

    Ok(outbound)
}

// ============================================================================
// Shadowsocks Outbound
// ============================================================================

fn generate_shadowsocks_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let password = proxy.password.as_ref()
        .ok_or_else(|| anyhow!("Shadowsocks requires password"))?;

    let method = proxy.custom_fields.get("method")
        .map(|s| s.as_str())
        .unwrap_or("aes-256-gcm");

    let mut outbound = json!({
        "type": "shadowsocks",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "method": method,
        "password": password
    });

    // Plugin support (e.g., obfs, v2ray-plugin)
    if let Some(plugin) = proxy.custom_fields.get("plugin") {
        outbound["plugin"] = json!(plugin);
        
        if let Some(plugin_opts) = proxy.custom_fields.get("plugin_opts") {
            outbound["plugin_opts"] = json!(plugin_opts);
        }
    }

    // UDP over TCP
    if proxy.custom_fields.get("uot").map(|s| s == "true").unwrap_or(false) {
        outbound["udp_over_tcp"] = json!({
            "enabled": true,
            "version": 2
        });
    }

    Ok(outbound)
}

// ============================================================================
// Trojan Outbound
// ============================================================================

fn generate_trojan_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let password = proxy.password.as_ref()
        .ok_or_else(|| anyhow!("Trojan requires password"))?;

    let mut outbound = json!({
        "type": "trojan",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "password": password
    });

    // TLS is mandatory for Trojan
    let mut tls = json!({
        "enabled": true,
        "insecure": false
    });

    if let Some(ref sni) = proxy.sni {
        tls["server_name"] = json!(sni);
    }

    let fingerprint = proxy.custom_fields.get("fp")
        .map(|s| s.as_str())
        .unwrap_or("chrome");
    
    tls["utls"] = json!({
        "enabled": true,
        "fingerprint": fingerprint
    });

    if let Some(alpn) = proxy.custom_fields.get("alpn") {
        let alpn_list: Vec<&str> = alpn.split(',').collect();
        tls["alpn"] = json!(alpn_list);
    }

    outbound["tls"] = tls;

    // Add transport configuration
    add_transport_config(&mut outbound, proxy)?;

    Ok(outbound)
}

// ============================================================================
// TUIC Outbound
// ============================================================================

fn generate_tuic_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let uuid = proxy.uuid.as_ref().ok_or_else(|| anyhow!("TUIC requires UUID"))?;
    let password = proxy.password.as_ref()
        .ok_or_else(|| anyhow!("TUIC requires password"))?;

    let mut outbound = json!({
        "type": "tuic",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "uuid": uuid,
        "password": password,
        "congestion_control": proxy.custom_fields.get("congestion_control")
            .map(|s| s.as_str())
            .unwrap_or("bbr"),
        "udp_relay_mode": proxy.custom_fields.get("udp_relay_mode")
            .map(|s| s.as_str())
            .unwrap_or("native"),
        "zero_rtt_handshake": proxy.custom_fields.get("zero_rtt")
            .map(|s| s == "true")
            .unwrap_or(false),
        "heartbeat": "10s"
    });

    // TLS configuration (mandatory for TUIC)
    let mut tls = json!({
        "enabled": true,
        "insecure": false
    });

    if let Some(ref sni) = proxy.sni {
        tls["server_name"] = json!(sni);
    }

    if let Some(alpn) = proxy.custom_fields.get("alpn") {
        let alpn_list: Vec<&str> = alpn.split(',').collect();
        tls["alpn"] = json!(alpn_list);
    } else {
        tls["alpn"] = json!(["h3"]);
    }

    outbound["tls"] = tls;

    Ok(outbound)
}

// ============================================================================
// Hysteria/Hysteria2 Outbound
// ============================================================================

fn generate_hysteria_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let is_v2 = proxy.protocol == ProxyProtocol::Hysteria2;
    
    let mut outbound = json!({
        "type": if is_v2 { "hysteria2" } else { "hysteria" },
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port
    });

    if is_v2 {
        // Hysteria2 uses password
        let password = proxy.password.as_ref()
            .ok_or_else(|| anyhow!("Hysteria2 requires password"))?;
        outbound["password"] = json!(password);
    } else {
        // Hysteria v1 uses auth_str
        if let Some(auth) = proxy.custom_fields.get("auth") {
            outbound["auth_str"] = json!(auth);
        } else if let Some(ref password) = proxy.password {
            outbound["auth_str"] = json!(password);
        }

        // Hysteria v1 specific options
        if let Some(protocol) = proxy.custom_fields.get("protocol") {
            outbound["protocol"] = json!(protocol);
        }
    }

    // Bandwidth settings
    if let Some(up) = proxy.custom_fields.get("up_mbps") {
        outbound["up_mbps"] = json!(up.parse::<i32>().unwrap_or(100));
    }
    if let Some(down) = proxy.custom_fields.get("down_mbps") {
        outbound["down_mbps"] = json!(down.parse::<i32>().unwrap_or(100));
    }

    // Obfuscation
    if let Some(obfs) = proxy.custom_fields.get("obfs") {
        outbound["obfs"] = json!({
            "type": obfs,
            "password": proxy.custom_fields.get("obfs_password").unwrap_or(&String::new())
        });
    }

    // TLS configuration
    let mut tls = json!({
        "enabled": true,
        "insecure": proxy.custom_fields.get("insecure")
            .map(|s| s == "true")
            .unwrap_or(false)
    });

    if let Some(ref sni) = proxy.sni {
        tls["server_name"] = json!(sni);
    }

    if let Some(alpn) = proxy.custom_fields.get("alpn") {
        let alpn_list: Vec<&str> = alpn.split(',').collect();
        tls["alpn"] = json!(alpn_list);
    }

    outbound["tls"] = tls;

    Ok(outbound)
}

// ============================================================================
// SOCKS5 Outbound
// ============================================================================

fn generate_socks_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let mut outbound = json!({
        "type": "socks",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "version": "5"
    });

    if let Some(ref username) = proxy.username {
        outbound["username"] = json!(username);
    }

    if let Some(ref password) = proxy.password {
        outbound["password"] = json!(password);
    }

    // UDP support
    if proxy.custom_fields.get("udp").map(|s| s == "true").unwrap_or(true) {
        outbound["udp_over_tcp"] = json!(false);
    }

    Ok(outbound)
}

// ============================================================================
// HTTP/HTTPS Outbound
// ============================================================================

fn generate_http_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let mut outbound = json!({
        "type": "http",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port
    });

    if let Some(ref username) = proxy.username {
        outbound["username"] = json!(username);
    }

    if let Some(ref password) = proxy.password {
        outbound["password"] = json!(password);
    }

    // TLS for HTTPS proxy
    if proxy.tls || proxy.protocol == ProxyProtocol::Https {
        let mut tls = json!({
            "enabled": true,
            "insecure": false
        });

        if let Some(ref sni) = proxy.sni {
            tls["server_name"] = json!(sni);
        }

        outbound["tls"] = tls;
    }

    Ok(outbound)
}

// ============================================================================
// WireGuard Outbound
// ============================================================================

fn generate_wireguard_outbound(proxy: &ProxyConfig) -> Result<Value> {
    let private_key = proxy.custom_fields.get("private_key")
        .ok_or_else(|| anyhow!("WireGuard requires private_key"))?;
    let peer_public_key = proxy.custom_fields.get("peer_public_key")
        .ok_or_else(|| anyhow!("WireGuard requires peer_public_key"))?;

    let mut outbound = json!({
        "type": "wireguard",
        "tag": proxy.id,
        "server": proxy.server,
        "server_port": proxy.port,
        "private_key": private_key,
        "peer_public_key": peer_public_key
    });

    // Local address (required)
    if let Some(local_address) = proxy.custom_fields.get("local_address") {
        let addresses: Vec<&str> = local_address.split(',').collect();
        outbound["local_address"] = json!(addresses);
    }

    // Pre-shared key (optional)
    if let Some(psk) = proxy.custom_fields.get("pre_shared_key") {
        outbound["pre_shared_key"] = json!(psk);
    }

    // Reserved bytes (optional)
    if let Some(reserved) = proxy.custom_fields.get("reserved") {
        let reserved_bytes: Vec<u8> = reserved
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if reserved_bytes.len() == 3 {
            outbound["reserved"] = json!(reserved_bytes);
        }
    }

    // MTU
    if let Some(mtu) = proxy.custom_fields.get("mtu") {
        outbound["mtu"] = json!(mtu.parse::<u32>().unwrap_or(1280));
    }

    Ok(outbound)
}

// ============================================================================
// Transport Configuration Helper
// ============================================================================

fn add_transport_config(outbound: &mut Value, proxy: &ProxyConfig) -> Result<()> {
    let transport_type = proxy.transport.as_deref()
        .or_else(|| proxy.custom_fields.get("type").map(|s| s.as_str()));

    let transport = match transport_type {
        Some("ws") | Some("websocket") => {
            let mut ws = json!({
                "type": "ws"
            });

            if let Some(path) = proxy.custom_fields.get("path") {
                ws["path"] = json!(path);
            }

            if let Some(host) = proxy.custom_fields.get("host") {
                ws["headers"] = json!({
                    "Host": host
                });
            }

            // Early data for WebSocket
            if let Some(ed) = proxy.custom_fields.get("ed") {
                ws["max_early_data"] = json!(ed.parse::<u32>().unwrap_or(2048));
            }

            if let Some(ed_header) = proxy.custom_fields.get("eh") {
                ws["early_data_header_name"] = json!(ed_header);
            }

            Some(ws)
        }
        Some("grpc") => {
            let mut grpc = json!({
                "type": "grpc"
            });

            if let Some(service_name) = proxy.custom_fields.get("serviceName") {
                grpc["service_name"] = json!(service_name);
            }

            Some(grpc)
        }
        Some("h2") | Some("http") => {
            let mut h2 = json!({
                "type": "http"
            });

            if let Some(path) = proxy.custom_fields.get("path") {
                h2["path"] = json!(path);
            }

            if let Some(host) = proxy.custom_fields.get("host") {
                h2["host"] = json!([host]);
            }

            Some(h2)
        }
        Some("quic") => {
            Some(json!({
                "type": "quic"
            }))
        }
        Some("httpupgrade") => {
            let mut httpupgrade = json!({
                "type": "httpupgrade"
            });

            if let Some(path) = proxy.custom_fields.get("path") {
                httpupgrade["path"] = json!(path);
            }

            if let Some(host) = proxy.custom_fields.get("host") {
                httpupgrade["host"] = json!(host);
            }

            Some(httpupgrade)
        }
        _ => None,
    };

    if let Some(t) = transport {
        outbound["transport"] = t;
    }

    Ok(())
}

// ============================================================================
// Routing Configuration
// ============================================================================

/// Generate routing rules for domain and application-based routing
/// 
/// This is a public API for generating route rules that can be used
/// when building custom sing-box configurations.
/// 
/// # Arguments
/// * `domain_routes` - Domain-based routing rules
/// * `app_routes` - Application-based routing rules
/// * `default_outbound` - Default outbound tag for unmatched traffic
/// 
/// # Returns
/// Array of routing rules as JSON Value
pub fn generate_route_rules(
    domain_routes: &[DomainRoute],
    app_routes: &[AppRoute],
    default_outbound: &str,
) -> Value {
    let mut rules = vec![
        // DNS rule
        json!({
            "protocol": "dns",
            "outbound": "dns-out"
        }),
        // Private IP addresses go direct
        json!({
            "ip_is_private": true,
            "outbound": "direct"
        }),
        // Local domains go direct
        json!({
            "domain_suffix": [".local", ".localhost", ".lan"],
            "outbound": "direct"
        }),
        json!({
            "domain": ["localhost"],
            "outbound": "direct"
        }),
        // Private IP ranges go direct
        json!({
            "ip_cidr": [
                "127.0.0.0/8",
                "::1/128",
                "10.0.0.0/8",
                "172.16.0.0/12",
                "192.168.0.0/16",
                "fc00::/7"
            ],
            "outbound": "direct"
        }),
    ];

    // Group domain routes by proxy_id for efficiency
    let mut domain_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for route in domain_routes {
        domain_groups
            .entry(route.proxy_id.clone())
            .or_default()
            .push(route.domain.clone());
    }

    // Add domain-based routing rules
    for (proxy_id, domains) in &domain_groups {
        // Separate domain suffixes and exact domains
        let (suffixes, exact): (Vec<_>, Vec<_>) = domains
            .iter()
            .partition(|d| d.starts_with('.'));

        if !suffixes.is_empty() {
            rules.push(json!({
                "domain_suffix": suffixes,
                "outbound": proxy_id
            }));
        }

        if !exact.is_empty() {
            rules.push(json!({
                "domain": exact,
                "outbound": proxy_id
            }));
        }
    }

    // Group app routes by proxy_id
    let mut app_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for route in app_routes {
        app_groups
            .entry(route.proxy_id.clone())
            .or_default()
            .push(route.app_name.clone());
    }

    // Add application-based routing rules
    for (proxy_id, apps) in &app_groups {
        rules.push(json!({
            "process_name": apps,
            "outbound": proxy_id
        }));
    }

    json!({
        "rules": rules,
        "final": default_outbound,
        "auto_detect_interface": true,
        "override_android_vpn": false
    })
}

fn generate_routing(
    domain_routes: &[DomainRoute],
    app_routes: &[AppRoute],
    proxies: &[ProxyConfig],
) -> Value {
    // Determine default outbound from proxies
    let default_outbound = proxies
        .iter()
        .find(|p| p.active)
        .map(|p| p.id.as_str())
        .unwrap_or_else(|| {
            proxies.first().map(|p| p.id.as_str()).unwrap_or("direct")
        });

    generate_route_rules(domain_routes, app_routes, default_outbound)
}

// ============================================================================
// DNS Configuration
// ============================================================================

fn generate_dns_config(proxies: &[ProxyConfig]) -> Value {
    generate_dns_config_internal(proxies, DnsMode::Direct)
}

/// Generate DNS configuration with specified mode
/// 
/// # Arguments
/// * `proxies` - List of proxy configurations
/// * `mode` - DNS mode (Direct or FakeIp)
/// 
/// # Returns
/// DNS configuration as JSON Value
pub fn generate_dns_config_with_mode(proxies: &[ProxyConfig], mode: DnsMode) -> Value {
    generate_dns_config_internal(proxies, mode)
}

/// Generate DNS configuration for fake-ip mode (TUN)
/// 
/// Fake-IP mode returns fake IP addresses for DNS queries,
/// which are then resolved by the proxy server.
/// This is required for TUN mode to work properly.
pub fn generate_dns_config_fakeip(proxies: &[ProxyConfig]) -> Value {
    generate_dns_config_internal(proxies, DnsMode::FakeIp)
}

fn generate_dns_config_internal(proxies: &[ProxyConfig], mode: DnsMode) -> Value {
    // Find active proxy for remote DNS
    let proxy_tag = proxies
        .iter()
        .find(|p| p.active)
        .map(|p| p.id.as_str())
        .unwrap_or_else(|| {
            proxies.first().map(|p| p.id.as_str()).unwrap_or("direct")
        });

    match mode {
        DnsMode::Direct => {
            json!({
                "servers": [
                    {
                        "tag": "dns-remote",
                        "address": "https://1.1.1.1/dns-query",
                        "address_resolver": "dns-direct",
                        "detour": proxy_tag
                    },
                    {
                        "tag": "dns-direct",
                        "address": "https://8.8.8.8/dns-query",
                        "detour": "direct"
                    },
                    {
                        "tag": "dns-local",
                        "address": "local",
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
                    },
                    {
                        "clash_mode": "Direct",
                        "server": "dns-direct"
                    },
                    {
                        "clash_mode": "Global",
                        "server": "dns-remote"
                    }
                ],
                "strategy": "prefer_ipv4",
                "disable_cache": false,
                "disable_expire": false
            })
        }
        DnsMode::FakeIp => {
            json!({
                "servers": [
                    {
                        "tag": "dns-remote",
                        "address": "https://1.1.1.1/dns-query",
                        "address_resolver": "dns-direct",
                        "detour": proxy_tag
                    },
                    {
                        "tag": "dns-direct",
                        "address": "https://8.8.8.8/dns-query",
                        "detour": "direct"
                    },
                    {
                        "tag": "dns-local",
                        "address": "local",
                        "detour": "direct"
                    },
                    {
                        "tag": "dns-fakeip",
                        "address": "fakeip"
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
                    },
                    {
                        "clash_mode": "Direct",
                        "server": "dns-direct"
                    },
                    {
                        "clash_mode": "Global",
                        "server": "dns-fakeip"
                    },
                    {
                        "query_type": ["A", "AAAA"],
                        "server": "dns-fakeip"
                    }
                ],
                "fakeip": {
                    "enabled": true,
                    "inet4_range": "198.18.0.0/15",
                    "inet6_range": "fc00::/18"
                },
                "strategy": "prefer_ipv4",
                "disable_cache": false,
                "disable_expire": false,
                "independent_cache": true
            })
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Generate config for a single proxy (simplified version)
pub fn generate_single_proxy_config(
    proxy: &ProxyConfig,
    socks_port: u16,
) -> Result<Value> {
    generate_singbox_config(
        &[proxy.clone()],
        &[],
        &[],
        socks_port,
        socks_port + 1,
    )
}

/// Validate sing-box configuration
pub fn validate_config(config: &Value) -> Result<()> {
    // Check required fields
    if config.get("inbounds").is_none() {
        return Err(anyhow!("Missing inbounds configuration"));
    }
    if config.get("outbounds").is_none() {
        return Err(anyhow!("Missing outbounds configuration"));
    }

    // Check that outbounds array is not empty
    let outbounds = config.get("outbounds")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Outbounds must be an array"))?;

    if outbounds.is_empty() {
        return Err(anyhow!("Outbounds array cannot be empty"));
    }

    // Check that each outbound has type and tag
    for (i, outbound) in outbounds.iter().enumerate() {
        if outbound.get("type").is_none() {
            return Err(anyhow!("Outbound {} missing type", i));
        }
        if outbound.get("tag").is_none() {
            return Err(anyhow!("Outbound {} missing tag", i));
        }
    }

    Ok(())
}

/// Convert config to pretty-printed JSON string
pub fn config_to_string(config: &Value) -> Result<String> {
    serde_json::to_string_pretty(config)
        .map_err(|e| anyhow!("Failed to serialize config: {}", e))
}

/// Write config to file
pub fn write_config_to_file(config: &Value, path: &std::path::Path) -> Result<()> {
    let json_str = config_to_string(config)?;
    std::fs::write(path, json_str)
        .map_err(|e| anyhow!("Failed to write config file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vless_proxy() -> ProxyConfig {
        let mut custom_fields = std::collections::HashMap::new();
        custom_fields.insert("flow".to_string(), "xtls-rprx-vision".to_string());
        custom_fields.insert("fp".to_string(), "chrome".to_string());

        ProxyConfig {
            id: "test-vless".to_string(),
            name: "Test VLESS".to_string(),
            protocol: ProxyProtocol::Vless,
            server: "example.com".to_string(),
            port: 443,
            username: None,
            password: None,
            uuid: Some("test-uuid-1234".to_string()),
            tls: true,
            sni: Some("example.com".to_string()),
            transport: None,
            custom_fields,
            active: true,
        }
    }

    #[test]
    fn test_generate_vless_outbound() {
        let proxy = create_test_vless_proxy();
        let outbound = generate_vless_outbound(&proxy).unwrap();

        assert_eq!(outbound["type"], "vless");
        assert_eq!(outbound["tag"], "test-vless");
        assert_eq!(outbound["server"], "example.com");
        assert_eq!(outbound["server_port"], 443);
        assert_eq!(outbound["uuid"], "test-uuid-1234");
        assert_eq!(outbound["flow"], "xtls-rprx-vision");
        assert!(outbound["tls"]["enabled"].as_bool().unwrap());
    }

    #[test]
    fn test_generate_full_config() {
        let proxy = create_test_vless_proxy();
        let config = generate_singbox_config(
            &[proxy],
            &[],
            &[],
            1080,
            1081,
        ).unwrap();

        assert!(config.get("log").is_some());
        assert!(config.get("dns").is_some());
        assert!(config.get("inbounds").is_some());
        assert!(config.get("outbounds").is_some());
        assert!(config.get("route").is_some());

        // Validate config
        validate_config(&config).unwrap();
    }

    #[test]
    fn test_generate_shadowsocks_outbound() {
        let mut custom_fields = std::collections::HashMap::new();
        custom_fields.insert("method".to_string(), "aes-256-gcm".to_string());

        let proxy = ProxyConfig {
            id: "test-ss".to_string(),
            name: "Test SS".to_string(),
            protocol: ProxyProtocol::Shadowsocks,
            server: "ss.example.com".to_string(),
            port: 8388,
            username: None,
            password: Some("test-password".to_string()),
            uuid: None,
            tls: false,
            sni: None,
            transport: None,
            custom_fields,
            active: false,
        };

        let outbound = generate_shadowsocks_outbound(&proxy).unwrap();

        assert_eq!(outbound["type"], "shadowsocks");
        assert_eq!(outbound["method"], "aes-256-gcm");
        assert_eq!(outbound["password"], "test-password");
    }

    #[test]
    fn test_routing_with_domain_routes() {
        let proxy = create_test_vless_proxy();
        let domain_routes = vec![
            DomainRoute {
                domain: ".youtube.com".to_string(),
                proxy_id: "test-vless".to_string(),
            },
            DomainRoute {
                domain: "google.com".to_string(),
                proxy_id: "test-vless".to_string(),
            },
        ];

        let config = generate_singbox_config(
            &[proxy],
            &domain_routes,
            &[],
            1080,
            1081,
        ).unwrap();

        let route = config.get("route").unwrap();
        let rules = route.get("rules").unwrap().as_array().unwrap();

        // Should have domain routing rules
        let has_domain_rule = rules.iter().any(|r| {
            r.get("domain_suffix").is_some() || r.get("domain").is_some()
        });
        assert!(has_domain_rule);
    }
}

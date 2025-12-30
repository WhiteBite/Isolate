//! DPI diagnostics module for Isolate
//!
//! Provides network diagnostics to classify the type of DPI blocking:
//! - DNS blocking (domain resolution fails)
//! - SNI/TLS blocking (connection reset during TLS handshake)
//! - IP blocking (TCP connection fails)
//! - No blocking detected
//!
//! Supports dual-stack IPv4/IPv6 diagnostics.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info, instrument, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{DpiKind, DpiProfile, IpStack, StrategyFamily};

/// Default timeout for all diagnostic operations (5 seconds as per project rules)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Test domains for diagnostics
const TEST_DOMAINS: &[&str] = &[
    "www.youtube.com",
    "discord.com",
    "www.instagram.com",
];

/// Known working domain for baseline comparison
const BASELINE_DOMAIN: &str = "www.google.com";

/// DNS resolution result
#[derive(Debug, Clone)]
struct DnsResult {
    domain: String,
    success: bool,
    resolved_ips: Vec<SocketAddr>,
    latency_ms: u32,
    error: Option<String>,
}

/// TCP connection result
#[derive(Debug, Clone)]
struct TcpResult {
    host: String,
    port: u16,
    success: bool,
    latency_ms: u32,
    error: Option<String>,
}

/// TLS handshake result
#[derive(Debug, Clone)]
struct TlsResult {
    host: String,
    success: bool,
    latency_ms: u32,
    error: Option<String>,
    reset_during_handshake: bool,
}

/// IPv6 availability result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ipv6Status {
    pub available: bool,
    pub can_resolve: bool,
    pub can_connect: bool,
    pub latency_ms: Option<u32>,
    pub error: Option<String>,
}

/// Dual-stack diagnostic result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DualStackResult {
    pub ip_stack: IpStack,
    pub ipv4_profile: DpiProfile,
    pub ipv6_profile: Option<DpiProfile>,
    pub ipv6_status: Ipv6Status,
}

/// Performs DNS resolution test
#[instrument(skip_all, fields(domain = %domain))]
async fn test_dns_resolve(domain: &str) -> DnsResult {
    let start = Instant::now();
    let lookup_host = format!("{}:443", domain);

    debug!("Starting DNS resolution for {}", domain);

    let result = timeout(DEFAULT_TIMEOUT, tokio::net::lookup_host(&lookup_host)).await;

    let latency_ms = start.elapsed().as_millis() as u32;

    match result {
        Ok(Ok(addrs)) => {
            let resolved: Vec<SocketAddr> = addrs.collect();
            if resolved.is_empty() {
                debug!("DNS resolution returned empty for {}", domain);
                DnsResult {
                    domain: domain.to_string(),
                    success: false,
                    resolved_ips: vec![],
                    latency_ms,
                    error: Some("No addresses resolved".to_string()),
                }
            } else {
                debug!("DNS resolved {} to {:?}", domain, resolved);
                DnsResult {
                    domain: domain.to_string(),
                    success: true,
                    resolved_ips: resolved,
                    latency_ms,
                    error: None,
                }
            }
        }
        Ok(Err(e)) => {
            warn!("DNS resolution failed for {}: {}", domain, e);
            DnsResult {
                domain: domain.to_string(),
                success: false,
                resolved_ips: vec![],
                latency_ms,
                error: Some(e.to_string()),
            }
        }
        Err(_) => {
            warn!("DNS resolution timeout for {}", domain);
            DnsResult {
                domain: domain.to_string(),
                success: false,
                resolved_ips: vec![],
                latency_ms: DEFAULT_TIMEOUT.as_millis() as u32,
                error: Some("Timeout".to_string()),
            }
        }
    }
}

/// Performs TCP connection test to port 443
#[instrument(skip_all, fields(host = %host, port = %port))]
async fn test_tcp_connect(host: &str, port: u16) -> TcpResult {
    let start = Instant::now();
    let addr = format!("{}:{}", host, port);

    debug!("Starting TCP connection to {}", addr);

    let result = timeout(DEFAULT_TIMEOUT, TcpStream::connect(&addr)).await;

    let latency_ms = start.elapsed().as_millis() as u32;

    match result {
        Ok(Ok(_stream)) => {
            debug!("TCP connection successful to {}", addr);
            TcpResult {
                host: host.to_string(),
                port,
                success: true,
                latency_ms,
                error: None,
            }
        }
        Ok(Err(e)) => {
            warn!("TCP connection failed to {}: {}", addr, e);
            TcpResult {
                host: host.to_string(),
                port,
                success: false,
                latency_ms,
                error: Some(e.to_string()),
            }
        }
        Err(_) => {
            warn!("TCP connection timeout to {}", addr);
            TcpResult {
                host: host.to_string(),
                port,
                success: false,
                latency_ms: DEFAULT_TIMEOUT.as_millis() as u32,
                error: Some("Timeout".to_string()),
            }
        }
    }
}

/// Performs a basic TLS handshake test by sending ClientHello
/// This helps detect SNI-based blocking
#[instrument(skip_all, fields(host = %host))]
async fn test_tls_handshake(host: &str, addr: SocketAddr) -> TlsResult {
    let start = Instant::now();

    debug!("Starting TLS handshake test to {} ({})", host, addr);

    // Build a minimal TLS ClientHello with SNI
    let client_hello = build_client_hello(host);

    let result = timeout(DEFAULT_TIMEOUT, async {
        let mut stream = TcpStream::connect(addr).await?;
        stream.write_all(&client_hello).await?;

        // Try to read response
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).await?;

        Ok::<(usize, Vec<u8>), std::io::Error>((n, buf[..n].to_vec()))
    })
    .await;

    let latency_ms = start.elapsed().as_millis() as u32;

    match result {
        Ok(Ok((n, response))) => {
            // Check if we got a valid TLS response (ServerHello starts with 0x16 0x03)
            let is_valid_tls = n >= 2 && response[0] == 0x16 && response[1] == 0x03;
            
            if is_valid_tls {
                debug!("TLS handshake successful for {}", host);
                TlsResult {
                    host: host.to_string(),
                    success: true,
                    latency_ms,
                    error: None,
                    reset_during_handshake: false,
                }
            } else {
                debug!("TLS handshake got unexpected response for {}", host);
                TlsResult {
                    host: host.to_string(),
                    success: false,
                    latency_ms,
                    error: Some("Invalid TLS response".to_string()),
                    reset_during_handshake: false,
                }
            }
        }
        Ok(Err(e)) => {
            let error_str = e.to_string();
            let is_reset = error_str.contains("reset")
                || error_str.contains("forcibly closed")
                || e.kind() == std::io::ErrorKind::ConnectionReset;

            warn!("TLS handshake failed for {}: {} (reset: {})", host, e, is_reset);

            TlsResult {
                host: host.to_string(),
                success: false,
                latency_ms,
                error: Some(error_str),
                reset_during_handshake: is_reset,
            }
        }
        Err(_) => {
            warn!("TLS handshake timeout for {}", host);
            TlsResult {
                host: host.to_string(),
                success: false,
                latency_ms: DEFAULT_TIMEOUT.as_millis() as u32,
                error: Some("Timeout".to_string()),
                reset_during_handshake: false,
            }
        }
    }
}

/// Builds a minimal TLS 1.2 ClientHello with SNI extension
fn build_client_hello(hostname: &str) -> Vec<u8> {
    let hostname_bytes = hostname.as_bytes();
    let hostname_len = hostname_bytes.len();

    // SNI extension
    let sni_extension: Vec<u8> = {
        let mut ext = Vec::new();
        // Extension type: server_name (0x0000)
        ext.extend_from_slice(&[0x00, 0x00]);
        // Extension length
        let ext_len = (hostname_len + 5) as u16;
        ext.extend_from_slice(&ext_len.to_be_bytes());
        // Server name list length
        let list_len = (hostname_len + 3) as u16;
        ext.extend_from_slice(&list_len.to_be_bytes());
        // Server name type: hostname (0)
        ext.push(0x00);
        // Hostname length
        let name_len = hostname_len as u16;
        ext.extend_from_slice(&name_len.to_be_bytes());
        // Hostname
        ext.extend_from_slice(hostname_bytes);
        ext
    };

    // Supported versions extension (TLS 1.2)
    let supported_versions: Vec<u8> = vec![
        0x00, 0x2b, // Extension type: supported_versions
        0x00, 0x03, // Length
        0x02,       // Versions length
        0x03, 0x03, // TLS 1.2
    ];

    let extensions_len = sni_extension.len() + supported_versions.len();

    // Build ClientHello
    let mut client_hello = Vec::new();

    // Handshake header
    // Client version: TLS 1.2 (0x0303)
    client_hello.extend_from_slice(&[0x03, 0x03]);

    // Random (32 bytes)
    client_hello.extend_from_slice(&[0u8; 32]);

    // Session ID length (0)
    client_hello.push(0x00);

    // Cipher suites
    let cipher_suites: &[u8] = &[
        0x00, 0x04, // Length (2 cipher suites)
        0x13, 0x01, // TLS_AES_128_GCM_SHA256
        0x00, 0xff, // TLS_EMPTY_RENEGOTIATION_INFO_SCSV
    ];
    client_hello.extend_from_slice(cipher_suites);

    // Compression methods
    client_hello.extend_from_slice(&[0x01, 0x00]); // 1 method: null

    // Extensions length
    let ext_len = extensions_len as u16;
    client_hello.extend_from_slice(&ext_len.to_be_bytes());

    // Extensions
    client_hello.extend_from_slice(&sni_extension);
    client_hello.extend_from_slice(&supported_versions);

    // Wrap in Handshake message
    let handshake_len = client_hello.len();
    let mut handshake = Vec::new();
    handshake.push(0x01); // ClientHello
    // Length (3 bytes)
    handshake.push(((handshake_len >> 16) & 0xff) as u8);
    handshake.push(((handshake_len >> 8) & 0xff) as u8);
    handshake.push((handshake_len & 0xff) as u8);
    handshake.extend_from_slice(&client_hello);

    // Wrap in TLS record
    let record_len = handshake.len();
    let mut record = Vec::new();
    record.push(0x16); // Handshake
    record.extend_from_slice(&[0x03, 0x01]); // TLS 1.0 for compatibility
    record.extend_from_slice(&(record_len as u16).to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

/// Classifies the type of DPI blocking based on diagnostic results
fn classify_blocking(
    dns_results: &[DnsResult],
    tcp_results: &[TcpResult],
    tls_results: &[TlsResult],
    baseline_works: bool,
) -> DpiKind {
    // If baseline doesn't work, network is broken
    if !baseline_works {
        return DpiKind::Unknown;
    }

    let dns_failures = dns_results.iter().filter(|r| !r.success).count();
    let tcp_failures = tcp_results.iter().filter(|r| !r.success).count();
    let tls_resets = tls_results.iter().filter(|r| r.reset_during_handshake).count();
    let tls_failures = tls_results.iter().filter(|r| !r.success).count();

    let total_tests = dns_results.len();

    // If most DNS resolutions fail, it's DNS blocking
    if dns_failures > total_tests / 2 {
        return DpiKind::DnsBlock;
    }

    // If DNS works but TCP fails, it's IP blocking
    if dns_failures == 0 && tcp_failures > total_tests / 2 {
        return DpiKind::IpBlock;
    }

    // If DNS and TCP work but TLS gets reset, it's SNI/TLS blocking
    if dns_failures == 0 && tcp_failures == 0 && (tls_resets > 0 || tls_failures > total_tests / 2) {
        return DpiKind::SniTlsBlock;
    }

    // No blocking detected
    if dns_failures == 0 && tcp_failures == 0 && tls_failures == 0 {
        return DpiKind::NoBlock;
    }

    DpiKind::Unknown
}

/// Suggests strategy families based on DPI kind
fn suggest_strategies(kind: &DpiKind) -> Vec<StrategyFamily> {
    match kind {
        DpiKind::DnsBlock => vec![StrategyFamily::DnsBypass, StrategyFamily::Vless],
        DpiKind::SniTlsBlock => vec![
            StrategyFamily::SniFrag,
            StrategyFamily::TlsFrag,
            StrategyFamily::Vless,
        ],
        DpiKind::IpBlock => vec![StrategyFamily::Vless],
        DpiKind::NoBlock => vec![],
        DpiKind::Unknown => vec![
            StrategyFamily::SniFrag,
            StrategyFamily::TlsFrag,
            StrategyFamily::DnsBypass,
            StrategyFamily::Vless,
        ],
    }
}

/// Main diagnostic function
///
/// Performs comprehensive network diagnostics to determine the type of DPI blocking.
/// Returns a DpiProfile with classification and recommended strategy families.
#[instrument(skip_all)]
pub async fn diagnose() -> Result<DpiProfile> {
    info!("Starting DPI diagnostics");

    // First, test baseline connectivity
    let baseline_dns = test_dns_resolve(BASELINE_DOMAIN).await;
    if !baseline_dns.success {
        warn!("Baseline DNS failed, network may be down");
        return Ok(DpiProfile {
            kind: DpiKind::Unknown,
            details: Some("Baseline connectivity check failed".to_string()),
            candidate_families: vec![],
        });
    }

    let baseline_tcp = if let Some(addr) = baseline_dns.resolved_ips.first() {
        test_tcp_connect(&addr.ip().to_string(), 443).await
    } else {
        return Ok(DpiProfile {
            kind: DpiKind::Unknown,
            details: Some("No baseline IP resolved".to_string()),
            candidate_families: vec![],
        });
    };

    let baseline_works = baseline_dns.success && baseline_tcp.success;

    // Run diagnostics for test domains in parallel
    let mut dns_results = Vec::new();
    let mut tcp_results = Vec::new();
    let mut tls_results = Vec::new();

    for domain in TEST_DOMAINS {
        let dns_result = test_dns_resolve(domain).await;

        if dns_result.success {
            if let Some(addr) = dns_result.resolved_ips.first() {
                let ip = addr.ip().to_string();

                // Run TCP and TLS tests in parallel
                let (tcp_result, tls_result) = tokio::join!(
                    test_tcp_connect(&ip, 443),
                    test_tls_handshake(domain, *addr)
                );

                tcp_results.push(tcp_result);
                tls_results.push(tls_result);
            }
        }

        dns_results.push(dns_result);
    }

    // Classify blocking type
    let kind = classify_blocking(&dns_results, &tcp_results, &tls_results, baseline_works);
    let candidate_families = suggest_strategies(&kind);

    // Build details string
    let details = format!(
        "DNS: {}/{} ok, TCP: {}/{} ok, TLS: {}/{} ok ({} resets)",
        dns_results.iter().filter(|r| r.success).count(),
        dns_results.len(),
        tcp_results.iter().filter(|r| r.success).count(),
        tcp_results.len(),
        tls_results.iter().filter(|r| r.success).count(),
        tls_results.len(),
        tls_results.iter().filter(|r| r.reset_during_handshake).count()
    );

    info!("Diagnostics complete: {:?} - {}", kind, details);

    Ok(DpiProfile {
        kind,
        details: Some(details),
        candidate_families,
    })
}

/// Diagnose a specific domain
#[instrument(skip_all, fields(domain = %domain))]
pub async fn diagnose_domain(domain: &str) -> Result<DpiProfile> {
    info!("Diagnosing domain: {}", domain);

    let dns_result = test_dns_resolve(domain).await;

    if !dns_result.success {
        return Ok(DpiProfile {
            kind: DpiKind::DnsBlock,
            details: dns_result.error,
            candidate_families: suggest_strategies(&DpiKind::DnsBlock),
        });
    }

    let addr = dns_result
        .resolved_ips
        .first()
        .ok_or_else(|| IsolateError::Network("No IP resolved".to_string()))?;

    let ip = addr.ip().to_string();

    let (tcp_result, tls_result) =
        tokio::join!(test_tcp_connect(&ip, 443), test_tls_handshake(domain, *addr));

    if !tcp_result.success {
        return Ok(DpiProfile {
            kind: DpiKind::IpBlock,
            details: tcp_result.error,
            candidate_families: suggest_strategies(&DpiKind::IpBlock),
        });
    }

    if !tls_result.success {
        let kind = if tls_result.reset_during_handshake {
            DpiKind::SniTlsBlock
        } else {
            DpiKind::Unknown
        };
        return Ok(DpiProfile {
            kind: kind.clone(),
            details: tls_result.error,
            candidate_families: suggest_strategies(&kind),
        });
    }

    Ok(DpiProfile {
        kind: DpiKind::NoBlock,
        details: Some(format!("Domain {} is accessible", domain)),
        candidate_families: vec![],
    })
}

// ============================================================================
// IPv6 / Dual-Stack Diagnostics
// ============================================================================

/// IPv6 test domain (Google's IPv6-only hostname)
const IPV6_TEST_HOST: &str = "ipv6.google.com";

/// Alternative IPv6 test targets
const IPV6_TEST_TARGETS: &[&str] = &[
    "2001:4860:4860::8888", // Google DNS
    "2606:4700:4700::1111", // Cloudflare DNS
];

/// Checks if IPv6 is available on the system
#[instrument(skip_all)]
pub async fn check_ipv6_availability() -> Ipv6Status {
    info!("Checking IPv6 availability");

    // Step 1: Try to resolve an IPv6-only hostname
    let can_resolve = check_ipv6_dns().await;

    // Step 2: Try to connect to a known IPv6 address
    let (can_connect, latency_ms, error) = check_ipv6_connectivity().await;

    let available = can_resolve || can_connect;

    info!(
        available,
        can_resolve,
        can_connect,
        "IPv6 availability check complete"
    );

    Ipv6Status {
        available,
        can_resolve,
        can_connect,
        latency_ms,
        error,
    }
}

/// Checks if DNS can resolve IPv6 addresses
async fn check_ipv6_dns() -> bool {
    let lookup_host = format!("{}:443", IPV6_TEST_HOST);

    let result = timeout(DEFAULT_TIMEOUT, tokio::net::lookup_host(lookup_host)).await;
    
    match result {
        Ok(Ok(addrs)) => {
            let has_ipv6 = addrs.into_iter().any(|addr| addr.ip().is_ipv6());
            debug!("IPv6 DNS resolution: {}", has_ipv6);
            has_ipv6
        }
        Ok(Err(e)) => {
            debug!("IPv6 DNS resolution failed: {}", e);
            false
        }
        Err(_) => {
            debug!("IPv6 DNS resolution timeout");
            false
        }
    }
}

/// Checks if we can connect to IPv6 addresses
async fn check_ipv6_connectivity() -> (bool, Option<u32>, Option<String>) {
    for target in IPV6_TEST_TARGETS {
        let start = Instant::now();
        let addr = format!("[{}]:443", target);

        match timeout(Duration::from_secs(3), TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => {
                let latency = start.elapsed().as_millis() as u32;
                debug!("IPv6 connectivity OK to {}, latency {}ms", target, latency);
                return (true, Some(latency), None);
            }
            Ok(Err(e)) => {
                debug!("IPv6 connection to {} failed: {}", target, e);
            }
            Err(_) => {
                debug!("IPv6 connection to {} timeout", target);
            }
        }
    }

    (false, None, Some("Cannot connect to any IPv6 target".to_string()))
}

/// Filters addresses by IP version
fn filter_addresses_by_version(addrs: &[SocketAddr], ipv6: bool) -> Vec<SocketAddr> {
    addrs
        .iter()
        .filter(|addr| {
            if ipv6 {
                addr.ip().is_ipv6()
            } else {
                addr.ip().is_ipv4()
            }
        })
        .cloned()
        .collect()
}

/// Runs diagnostics on a specific IP stack (IPv4 or IPv6)
#[instrument(skip_all, fields(ipv6 = %ipv6))]
async fn diagnose_stack(ipv6: bool) -> Result<DpiProfile> {
    let stack_name = if ipv6 { "IPv6" } else { "IPv4" };
    info!("Running {} diagnostics", stack_name);

    // Test baseline connectivity for this stack
    let baseline_dns = test_dns_resolve(BASELINE_DOMAIN).await;
    let stack_addrs = filter_addresses_by_version(&baseline_dns.resolved_ips, ipv6);

    if stack_addrs.is_empty() {
        return Ok(DpiProfile {
            kind: DpiKind::Unknown,
            details: Some(format!("No {} addresses resolved for baseline", stack_name)),
            candidate_families: vec![],
        });
    }

    let baseline_addr = stack_addrs[0];
    let baseline_tcp = test_tcp_connect(&baseline_addr.ip().to_string(), 443).await;
    let baseline_works = baseline_tcp.success;

    if !baseline_works {
        return Ok(DpiProfile {
            kind: DpiKind::Unknown,
            details: Some(format!("{} baseline connectivity failed", stack_name)),
            candidate_families: vec![],
        });
    }

    // Run diagnostics for test domains
    let mut dns_results = Vec::new();
    let mut tcp_results = Vec::new();
    let mut tls_results = Vec::new();

    for domain in TEST_DOMAINS {
        let dns_result = test_dns_resolve(domain).await;
        let stack_addrs = filter_addresses_by_version(&dns_result.resolved_ips, ipv6);

        if !stack_addrs.is_empty() {
            let addr = stack_addrs[0];
            let ip = addr.ip().to_string();

            let (tcp_result, tls_result) = tokio::join!(
                test_tcp_connect(&ip, 443),
                test_tls_handshake(domain, addr)
            );

            tcp_results.push(tcp_result);
            tls_results.push(tls_result);

            // Mark DNS as successful for this stack
            dns_results.push(DnsResult {
                domain: domain.to_string(),
                success: true,
                resolved_ips: stack_addrs,
                latency_ms: dns_result.latency_ms,
                error: None,
            });
        } else {
            // No addresses for this stack
            dns_results.push(DnsResult {
                domain: domain.to_string(),
                success: false,
                resolved_ips: vec![],
                latency_ms: dns_result.latency_ms,
                error: Some(format!("No {} addresses", stack_name)),
            });
        }
    }

    let kind = classify_blocking(&dns_results, &tcp_results, &tls_results, baseline_works);
    let candidate_families = suggest_strategies(&kind);

    let details = format!(
        "{}: DNS {}/{} ok, TCP {}/{} ok, TLS {}/{} ok ({} resets)",
        stack_name,
        dns_results.iter().filter(|r| r.success).count(),
        dns_results.len(),
        tcp_results.iter().filter(|r| r.success).count(),
        tcp_results.len(),
        tls_results.iter().filter(|r| r.success).count(),
        tls_results.len(),
        tls_results.iter().filter(|r| r.reset_during_handshake).count()
    );

    Ok(DpiProfile {
        kind,
        details: Some(details),
        candidate_families,
    })
}

/// Performs dual-stack diagnostics (IPv4 and IPv6)
///
/// Detects which IP stacks are available and runs diagnostics on both.
/// Reports which stack has issues.
#[instrument(skip_all)]
pub async fn diagnose_dual_stack() -> Result<DualStackResult> {
    info!("Starting dual-stack diagnostics");

    // Check IPv6 availability first
    let ipv6_status = check_ipv6_availability().await;

    // Always run IPv4 diagnostics
    let ipv4_profile = diagnose_stack(false).await?;

    // Run IPv6 diagnostics if available
    let (ipv6_profile, ip_stack) = if ipv6_status.available {
        let profile = diagnose_stack(true).await?;

        // Determine overall stack status
        let stack = match (&ipv4_profile.kind, &profile.kind) {
            (DpiKind::NoBlock, DpiKind::NoBlock) => IpStack::DualStack,
            (DpiKind::NoBlock, _) => IpStack::V4Only,
            (_, DpiKind::NoBlock) => IpStack::V6Only,
            _ => {
                // Both have issues, prefer IPv4
                if ipv4_profile.kind == DpiKind::Unknown {
                    IpStack::V6Only
                } else {
                    IpStack::V4Only
                }
            }
        };

        (Some(profile), stack)
    } else {
        (None, IpStack::V4Only)
    };

    info!(
        ip_stack = %ip_stack,
        ipv4_kind = ?ipv4_profile.kind,
        ipv6_available = ipv6_status.available,
        "Dual-stack diagnostics complete"
    );

    Ok(DualStackResult {
        ip_stack,
        ipv4_profile,
        ipv6_profile,
        ipv6_status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_client_hello() {
        let hello = build_client_hello("example.com");
        // Should start with TLS record header
        assert_eq!(hello[0], 0x16); // Handshake
        assert_eq!(hello[1], 0x03); // TLS major version
        assert_eq!(hello[2], 0x01); // TLS minor version (1.0 for compat)
    }

    #[test]
    fn test_classify_blocking_dns() {
        let dns_results = vec![
            DnsResult {
                domain: "test.com".to_string(),
                success: false,
                resolved_ips: vec![],
                latency_ms: 100,
                error: Some("NXDOMAIN".to_string()),
            },
            DnsResult {
                domain: "test2.com".to_string(),
                success: false,
                resolved_ips: vec![],
                latency_ms: 100,
                error: Some("NXDOMAIN".to_string()),
            },
        ];

        let kind = classify_blocking(&dns_results, &[], &[], true);
        assert_eq!(kind, DpiKind::DnsBlock);
    }

    #[test]
    fn test_suggest_strategies() {
        let strategies = suggest_strategies(&DpiKind::SniTlsBlock);
        assert!(strategies.contains(&StrategyFamily::SniFrag));
        assert!(strategies.contains(&StrategyFamily::TlsFrag));
    }
}

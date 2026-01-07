//! Test engine module for Isolate
//!
//! Provides HTTP GET/HEAD tests, TCP connect tests with SOCKS5 proxy support.
//! All tests have configurable timeouts (max 5 seconds as per project rules).
//!
//! NOTE: This module provides low-level testing primitives for strategy validation.
//! Used internally by the testing subsystem.

// Public API for strategy testing infrastructure
#![allow(dead_code)]

use std::time::{Duration, Instant};

use reqwest::{Client, Proxy};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info, instrument, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{ErrorType, ServiceTestSummary, TestDefinition, TestResult};

/// Maximum timeout for any operation (5 seconds as per project rules)
const MAX_TIMEOUT: Duration = Duration::from_secs(5);

/// SOCKS5 proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
}

impl ProxyConfig {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    pub fn url(&self) -> String {
        format!("socks5://{}:{}", self.host, self.port)
    }
}

/// Test engine for running service tests
pub struct TestEngine {
    client: Client,
    proxy_client: Option<Client>,
    proxy_config: Option<ProxyConfig>,
}

impl TestEngine {
    /// Creates a new test engine without proxy
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(MAX_TIMEOUT)
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| IsolateError::Network(e.to_string()))?;

        Ok(Self {
            client,
            proxy_client: None,
            proxy_config: None,
        })
    }

    /// Creates a new test engine with SOCKS5 proxy
    pub fn with_proxy(proxy: ProxyConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(MAX_TIMEOUT)
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| IsolateError::Network(e.to_string()))?;

        let proxy_url = proxy.url();
        let reqwest_proxy =
            Proxy::all(&proxy_url).map_err(|e| IsolateError::Network(e.to_string()))?;

        let proxy_client = Client::builder()
            .timeout(MAX_TIMEOUT)
            .proxy(reqwest_proxy)
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| IsolateError::Network(e.to_string()))?;

        Ok(Self {
            client,
            proxy_client: Some(proxy_client),
            proxy_config: Some(proxy),
        })
    }

    /// Returns the appropriate client (proxy or direct)
    fn get_client(&self, use_proxy: bool) -> &Client {
        if use_proxy {
            self.proxy_client.as_ref().unwrap_or(&self.client)
        } else {
            &self.client
        }
    }

    /// Performs HTTP GET test
    #[instrument(skip(self), fields(url = %url))]
    pub async fn test_http_get(
        &self,
        url: &str,
        timeout_ms: u32,
        expected_status: &[u16],
        min_body_size: Option<usize>,
        use_proxy: bool,
    ) -> TestResult {
        let test_id = format!("http_get:{}", url);
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms.min(MAX_TIMEOUT.as_millis() as u32) as u64);

        debug!("Starting HTTP GET test for {}", url);

        let client = self.get_client(use_proxy);
        let result = timeout(timeout_duration, client.get(url).send()).await;

        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(Ok(response)) => {
                let status = response.status().as_u16();
                let status_ok = expected_status.is_empty() || expected_status.contains(&status);

                if !status_ok {
                    return TestResult {
                        test_id,
                        success: false,
                        latency_ms: Some(latency_ms),
                        error_type: Some(ErrorType::Http),
                        error_message: Some(format!("Unexpected status: {}", status)),
                    };
                }

                // Check body size if required
                if let Some(min_size) = min_body_size {
                    match response.bytes().await {
                        Ok(body) if body.len() >= min_size => {
                            debug!("HTTP GET success for {} ({}ms)", url, latency_ms);
                            TestResult {
                                test_id,
                                success: true,
                                latency_ms: Some(latency_ms),
                                error_type: None,
                                error_message: None,
                            }
                        }
                        Ok(body) => TestResult {
                            test_id,
                            success: false,
                            latency_ms: Some(latency_ms),
                            error_type: Some(ErrorType::Http),
                            error_message: Some(format!(
                                "Body too small: {} < {}",
                                body.len(),
                                min_size
                            )),
                        },
                        Err(e) => TestResult {
                            test_id,
                            success: false,
                            latency_ms: Some(latency_ms),
                            error_type: Some(ErrorType::Http),
                            error_message: Some(e.to_string()),
                        },
                    }
                } else {
                    debug!("HTTP GET success for {} ({}ms)", url, latency_ms);
                    TestResult {
                        test_id,
                        success: true,
                        latency_ms: Some(latency_ms),
                        error_type: None,
                        error_message: None,
                    }
                }
            }
            Ok(Err(e)) => {
                let error_type = classify_reqwest_error(&e);
                warn!("HTTP GET failed for {}: {}", url, e);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(latency_ms),
                    error_type: Some(error_type),
                    error_message: Some(e.to_string()),
                }
            }
            Err(_) => {
                warn!("HTTP GET timeout for {}", url);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(timeout_ms),
                    error_type: Some(ErrorType::Timeout),
                    error_message: Some("Request timeout".to_string()),
                }
            }
        }
    }

    /// Performs HTTP HEAD test
    #[instrument(skip(self), fields(url = %url))]
    pub async fn test_http_head(&self, url: &str, timeout_ms: u32, use_proxy: bool) -> TestResult {
        let test_id = format!("http_head:{}", url);
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms.min(MAX_TIMEOUT.as_millis() as u32) as u64);

        debug!("Starting HTTP HEAD test for {}", url);

        let client = self.get_client(use_proxy);
        let result = timeout(timeout_duration, client.head(url).send()).await;

        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                if status.is_success() || status.is_redirection() {
                    debug!("HTTP HEAD success for {} ({}ms)", url, latency_ms);
                    TestResult {
                        test_id,
                        success: true,
                        latency_ms: Some(latency_ms),
                        error_type: None,
                        error_message: None,
                    }
                } else {
                    TestResult {
                        test_id,
                        success: false,
                        latency_ms: Some(latency_ms),
                        error_type: Some(ErrorType::Http),
                        error_message: Some(format!("Status: {}", status)),
                    }
                }
            }
            Ok(Err(e)) => {
                let error_type = classify_reqwest_error(&e);
                warn!("HTTP HEAD failed for {}: {}", url, e);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(latency_ms),
                    error_type: Some(error_type),
                    error_message: Some(e.to_string()),
                }
            }
            Err(_) => {
                warn!("HTTP HEAD timeout for {}", url);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(timeout_ms),
                    error_type: Some(ErrorType::Timeout),
                    error_message: Some("Request timeout".to_string()),
                }
            }
        }
    }

    /// Performs TCP connect test
    #[instrument(skip(self), fields(host = %host, port = %port))]
    pub async fn test_tcp_connect(
        &self,
        host: &str,
        port: u16,
        timeout_ms: u32,
        use_proxy: bool,
    ) -> TestResult {
        let test_id = format!("tcp:{}:{}", host, port);
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms.min(MAX_TIMEOUT.as_millis() as u32) as u64);

        debug!("Starting TCP connect test for {}:{}", host, port);

        let result = if use_proxy {
            if let Some(proxy) = &self.proxy_config {
                timeout(
                    timeout_duration,
                    connect_via_socks5(&proxy.host, proxy.port, host, port),
                )
                .await
            } else {
                // No proxy configured, fall back to direct
                let addr = format!("{}:{}", host, port);
                timeout(timeout_duration, TcpStream::connect(&addr)).await
                    .map(|r| r.map(|_| ()))
            }
        } else {
            let addr = format!("{}:{}", host, port);
            timeout(timeout_duration, TcpStream::connect(&addr)).await
                .map(|r| r.map(|_| ()))
        };

        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(Ok(_)) => {
                debug!("TCP connect success for {}:{} ({}ms)", host, port, latency_ms);
                TestResult {
                    test_id,
                    success: true,
                    latency_ms: Some(latency_ms),
                    error_type: None,
                    error_message: None,
                }
            }
            Ok(Err(e)) => {
                let error_type = classify_io_error(&e);
                warn!("TCP connect failed for {}:{}: {}", host, port, e);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(latency_ms),
                    error_type: Some(error_type),
                    error_message: Some(e.to_string()),
                }
            }
            Err(_) => {
                warn!("TCP connect timeout for {}:{}", host, port);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(timeout_ms),
                    error_type: Some(ErrorType::Timeout),
                    error_message: Some("Connection timeout".to_string()),
                }
            }
        }
    }

    /// Runs a single test definition
    pub async fn run_test(&self, test: &TestDefinition, use_proxy: bool) -> TestResult {
        match test {
            TestDefinition::HttpsGet {
                url,
                timeout_ms,
                expected_status,
                min_body_size,
            } => {
                self.test_http_get(url, *timeout_ms, expected_status, *min_body_size, use_proxy)
                    .await
            }
            TestDefinition::HttpsHead { url, timeout_ms } => {
                self.test_http_head(url, *timeout_ms, use_proxy).await
            }
            TestDefinition::TcpConnect {
                host,
                port,
                timeout_ms,
            } => {
                self.test_tcp_connect(host, *port, *timeout_ms, use_proxy)
                    .await
            }
            TestDefinition::WebSocket { url, timeout_ms } => {
                // WebSocket test - use HTTP upgrade check
                self.test_http_head(url, *timeout_ms, use_proxy).await
            }
            TestDefinition::Dns { domain, timeout_ms } => {
                self.test_dns(domain, *timeout_ms).await
            }
        }
    }

    /// Performs DNS resolution test
    #[instrument(skip(self), fields(domain = %domain))]
    async fn test_dns(&self, domain: &str, timeout_ms: u32) -> TestResult {
        let test_id = format!("dns:{}", domain);
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms.min(MAX_TIMEOUT.as_millis() as u32) as u64);
        let lookup_host = format!("{}:443", domain);

        debug!("Starting DNS test for {}", domain);

        let result = timeout(timeout_duration, tokio::net::lookup_host(&lookup_host)).await;

        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(Ok(addrs)) => {
                let resolved: Vec<_> = addrs.collect();
                if resolved.is_empty() {
                    TestResult {
                        test_id,
                        success: false,
                        latency_ms: Some(latency_ms),
                        error_type: Some(ErrorType::Dns),
                        error_message: Some("No addresses resolved".to_string()),
                    }
                } else {
                    debug!("DNS success for {} ({}ms)", domain, latency_ms);
                    TestResult {
                        test_id,
                        success: true,
                        latency_ms: Some(latency_ms),
                        error_type: None,
                        error_message: None,
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("DNS failed for {}: {}", domain, e);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(latency_ms),
                    error_type: Some(ErrorType::Dns),
                    error_message: Some(e.to_string()),
                }
            }
            Err(_) => {
                warn!("DNS timeout for {}", domain);
                TestResult {
                    test_id,
                    success: false,
                    latency_ms: Some(timeout_ms),
                    error_type: Some(ErrorType::Timeout),
                    error_message: Some("DNS timeout".to_string()),
                }
            }
        }
    }
}

/// Runs all tests for a service in parallel and aggregates results
#[instrument(skip_all, fields(service_id = %service_id))]
pub async fn run_service_tests(
    engine: &TestEngine,
    service_id: &str,
    tests: &[TestDefinition],
    use_proxy: bool,
) -> ServiceTestSummary {
    info!("Running {} tests for service {}", tests.len(), service_id);

    // Run all tests in parallel using tokio::join
    let futures: Vec<_> = tests
        .iter()
        .map(|test| engine.run_test(test, use_proxy))
        .collect();

    let results = futures::future::join_all(futures).await;

    // Aggregate results
    aggregate_results(service_id, &results)
}

/// Aggregates test results into a summary
fn aggregate_results(service_id: &str, results: &[TestResult]) -> ServiceTestSummary {
    let total_tests = results.len() as u32;
    let passed_tests = results.iter().filter(|r| r.success).count() as u32;

    let success_rate = if total_tests > 0 {
        passed_tests as f64 / total_tests as f64
    } else {
        0.0
    };

    // Calculate average latency from successful tests
    let latencies: Vec<u32> = results
        .iter()
        .filter(|r| r.success && r.latency_ms.is_some())
        .filter_map(|r| r.latency_ms)
        .collect();

    let avg_latency_ms = if !latencies.is_empty() {
        latencies.iter().sum::<u32>() as f64 / latencies.len() as f64
    } else {
        0.0
    };

    // Collect unique error types
    let errors: Vec<ErrorType> = results
        .iter()
        .filter_map(|r| r.error_type.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    ServiceTestSummary {
        service_id: service_id.to_string(),
        total_tests,
        passed_tests,
        success_rate,
        avg_latency_ms,
        errors,
    }
}

/// Runs tests for multiple services in parallel
#[instrument(skip_all)]
pub async fn run_all_service_tests(
    engine: &TestEngine,
    services: &[(String, Vec<TestDefinition>)],
    use_proxy: bool,
) -> Vec<ServiceTestSummary> {
    info!("Running tests for {} services", services.len());

    let futures: Vec<_> = services
        .iter()
        .map(|(service_id, tests)| run_service_tests(engine, service_id, tests, use_proxy))
        .collect();

    futures::future::join_all(futures).await
}

/// SOCKS5 proxy connection helper
async fn connect_via_socks5(
    proxy_host: &str,
    proxy_port: u16,
    target_host: &str,
    target_port: u16,
) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);
    let mut stream = TcpStream::connect(&proxy_addr).await?;

    // SOCKS5 greeting: version 5, 1 auth method (no auth)
    stream.write_all(&[0x05, 0x01, 0x00]).await?;

    // Read server response
    let mut response = [0u8; 2];
    stream.read_exact(&mut response).await?;

    if response[0] != 0x05 || response[1] != 0x00 {
        return Err(std::io::Error::other("SOCKS5 auth failed"));
    }

    // SOCKS5 connect request
    let mut request = vec![
        0x05, // version
        0x01, // connect
        0x00, // reserved
        0x03, // domain name
        target_host.len() as u8,
    ];
    request.extend_from_slice(target_host.as_bytes());
    request.extend_from_slice(&target_port.to_be_bytes());

    stream.write_all(&request).await?;

    // Read connect response
    let mut response = [0u8; 10];
    stream.read_exact(&mut response).await?;

    if response[0] != 0x05 || response[1] != 0x00 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("SOCKS5 connect failed: {}", response[1]),
        ));
    }

    Ok(())
}

/// Classifies reqwest errors into ErrorType
fn classify_reqwest_error(error: &reqwest::Error) -> ErrorType {
    if error.is_timeout() {
        ErrorType::Timeout
    } else if error.is_connect() {
        ErrorType::Tcp
    } else if error.is_request() {
        // Check if it's a DNS error
        let error_str = error.to_string().to_lowercase();
        if error_str.contains("dns") || error_str.contains("resolve") {
            ErrorType::Dns
        } else if error_str.contains("tls") || error_str.contains("ssl") || error_str.contains("certificate") {
            ErrorType::Tls
        } else {
            ErrorType::Http
        }
    } else {
        ErrorType::Unknown
    }
}

/// Classifies IO errors into ErrorType
fn classify_io_error(error: &std::io::Error) -> ErrorType {
    match error.kind() {
        std::io::ErrorKind::TimedOut => ErrorType::Timeout,
        std::io::ErrorKind::ConnectionRefused
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::ConnectionAborted => ErrorType::Tcp,
        std::io::ErrorKind::NotFound => ErrorType::Dns,
        _ => {
            let error_str = error.to_string().to_lowercase();
            if error_str.contains("tls") || error_str.contains("ssl") {
                ErrorType::Tls
            } else {
                ErrorType::Unknown
            }
        }
    }
}

impl Default for TestEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default TestEngine")
    }
}

//! HTTP prober for testing strategy connectivity
//!
//! Provides HTTP probing capabilities for testing DPI bypass strategies.
//! Supports both direct connections (for Zapret in GLOBAL mode) and
//! SOCKS5 proxy connections (for VLESS strategies).
//!
//! NOTE: This module is part of the testing infrastructure for strategy validation.

// Public API for HTTP probing
#![allow(dead_code)]

use std::time::{Duration, Instant};

use reqwest::Client;
use serde::Serialize;
use tracing::{debug, instrument, warn};

use super::endpoints::TestEndpoint;

/// Configuration for HTTP probing
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    /// Total request timeout
    pub timeout: Duration,
    /// Connection establishment timeout
    pub connect_timeout: Duration,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(5),
        }
    }
}

impl ProbeConfig {
    /// Create config with custom timeouts
    pub fn new(timeout: Duration, connect_timeout: Duration) -> Self {
        Self {
            timeout,
            connect_timeout,
        }
    }

    /// Create config with fast timeouts for quick testing
    pub fn fast() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            connect_timeout: Duration::from_secs(3),
        }
    }

    /// Create config with slow timeouts for unreliable networks
    pub fn slow() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(15),
        }
    }
}

/// Result of a single probe attempt
#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    /// URL that was tested
    pub url: String,
    /// Whether the probe was successful
    pub success: bool,
    /// Response latency in milliseconds (if successful)
    pub latency_ms: Option<f64>,
    /// HTTP status code (if received)
    pub status_code: Option<u16>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Whether this was a critical endpoint
    pub is_critical: bool,
}

impl ProbeResult {
    /// Create a successful probe result
    fn success(url: String, latency_ms: f64, status_code: u16, is_critical: bool) -> Self {
        Self {
            url,
            success: true,
            latency_ms: Some(latency_ms),
            status_code: Some(status_code),
            error: None,
            is_critical,
        }
    }

    /// Create a failed probe result
    fn failure(url: String, error: String, is_critical: bool) -> Self {
        Self {
            url,
            success: false,
            latency_ms: None,
            status_code: None,
            error: Some(error),
            is_critical,
        }
    }

    /// Create a failed probe result with partial info (got status but considered failure)
    fn failure_with_status(
        url: String,
        status_code: u16,
        latency_ms: f64,
        error: String,
        is_critical: bool,
    ) -> Self {
        Self {
            url,
            success: false,
            latency_ms: Some(latency_ms),
            status_code: Some(status_code),
            error: Some(error),
            is_critical,
        }
    }
}

/// HTTP prober for testing connectivity
pub struct HttpProber {
    config: ProbeConfig,
    /// Client for direct connections
    direct_client: Client,
}

impl HttpProber {
    /// Create a new prober with the given configuration
    pub fn new(config: ProbeConfig) -> Self {
        let direct_client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .danger_accept_invalid_certs(false)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            direct_client,
        }
    }

    /// Create a prober with default configuration
    pub fn with_default_config() -> Self {
        Self::new(ProbeConfig::default())
    }

    /// Get the current configuration
    pub fn config(&self) -> &ProbeConfig {
        &self.config
    }

    /// Test endpoint directly (for Zapret in GLOBAL mode)
    ///
    /// Makes a direct HTTP request without any proxy.
    #[instrument(skip(self), fields(url = %endpoint.url))]
    pub async fn probe_direct(&self, endpoint: &TestEndpoint) -> ProbeResult {
        debug!("Probing {} directly", endpoint.name);
        self.do_probe(&self.direct_client, endpoint).await
    }

    /// Test endpoint through SOCKS5 proxy (for VLESS strategies)
    ///
    /// Creates a new client with SOCKS5 proxy configuration and makes the request.
    #[instrument(skip(self), fields(url = %endpoint.url, socks_port = %socks_port))]
    pub async fn probe_via_socks(&self, endpoint: &TestEndpoint, socks_port: u16) -> ProbeResult {
        debug!(
            "Probing {} via SOCKS5 proxy on port {}",
            endpoint.name, socks_port
        );

        // Create SOCKS5 proxy client
        let proxy_url = format!("socks5://127.0.0.1:{}", socks_port);
        let proxy = match reqwest::Proxy::all(&proxy_url) {
            Ok(p) => p,
            Err(e) => {
                return ProbeResult::failure(
                    endpoint.url.clone(),
                    format!("Failed to create SOCKS5 proxy: {}", e),
                    endpoint.is_critical,
                );
            }
        };

        let socks_client = match Client::builder()
            .timeout(self.config.timeout)
            .connect_timeout(self.config.connect_timeout)
            .proxy(proxy)
            .danger_accept_invalid_certs(false)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return ProbeResult::failure(
                    endpoint.url.clone(),
                    format!("Failed to create SOCKS5 client: {}", e),
                    endpoint.is_critical,
                );
            }
        };

        self.do_probe(&socks_client, endpoint).await
    }

    /// Test all endpoints
    ///
    /// If `socks_port` is provided, uses SOCKS5 proxy; otherwise tests directly.
    #[instrument(skip(self, endpoints))]
    pub async fn probe_all(
        &self,
        endpoints: &[TestEndpoint],
        socks_port: Option<u16>,
    ) -> Vec<ProbeResult> {
        let mut results = Vec::with_capacity(endpoints.len());

        for endpoint in endpoints {
            let result = match socks_port {
                Some(port) => self.probe_via_socks(endpoint, port).await,
                None => self.probe_direct(endpoint).await,
            };
            results.push(result);
        }

        results
    }

    /// Test all endpoints concurrently
    ///
    /// Faster than sequential probing but may cause issues with some DPI systems.
    #[instrument(skip(self, endpoints))]
    pub async fn probe_all_concurrent(
        &self,
        endpoints: &[TestEndpoint],
        socks_port: Option<u16>,
    ) -> Vec<ProbeResult> {
        use futures::future::join_all;

        let futures: Vec<_> = endpoints
            .iter()
            .map(|endpoint| async move {
                match socks_port {
                    Some(port) => self.probe_via_socks(endpoint, port).await,
                    None => self.probe_direct(endpoint).await,
                }
            })
            .collect();

        join_all(futures).await
    }

    /// Internal probe implementation
    async fn do_probe(&self, client: &Client, endpoint: &TestEndpoint) -> ProbeResult {
        let start = Instant::now();

        let response = match client.get(&endpoint.url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                let error_msg = Self::format_error(&e);
                warn!("Probe failed for {}: {}", endpoint.name, error_msg);
                return ProbeResult::failure(endpoint.url.clone(), error_msg, endpoint.is_critical);
            }
        };

        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        let status = response.status().as_u16();

        // Consider these status codes as success (connection established)
        // 200 - OK
        // 204 - No Content (used by generate_204 endpoints)
        // 301, 302 - Redirects (connection works)
        // 401, 403 - Auth errors (connection works, just not authorized)
        let success_codes = [200, 204, 301, 302, 401, 403];

        if success_codes.contains(&status) {
            debug!(
                "Probe successful for {}: {} in {:.1}ms",
                endpoint.name, status, latency_ms
            );
            ProbeResult::success(endpoint.url.clone(), latency_ms, status, endpoint.is_critical)
        } else {
            // Got a response but unexpected status
            let error_msg = format!("Unexpected status code: {}", status);
            warn!(
                "Probe returned unexpected status for {}: {}",
                endpoint.name, status
            );
            ProbeResult::failure_with_status(
                endpoint.url.clone(),
                status,
                latency_ms,
                error_msg,
                endpoint.is_critical,
            )
        }
    }

    /// Format error message for logging and results
    fn format_error(error: &reqwest::Error) -> String {
        if error.is_timeout() {
            "Connection timeout".to_string()
        } else if error.is_connect() {
            "Connection refused or failed".to_string()
        } else if error.is_request() {
            format!("Request error: {}", error)
        } else {
            format!("Error: {}", error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_config_default() {
        let config = ProbeConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_probe_config_fast() {
        let config = ProbeConfig::fast();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.connect_timeout, Duration::from_secs(3));
    }

    #[test]
    fn test_probe_config_slow() {
        let config = ProbeConfig::slow();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.connect_timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_probe_config_custom() {
        let config = ProbeConfig::new(Duration::from_secs(20), Duration::from_secs(10));
        assert_eq!(config.timeout, Duration::from_secs(20));
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_probe_result_success() {
        let result = ProbeResult::success(
            "https://example.com".to_string(),
            150.5,
            200,
            true,
        );

        assert!(result.success);
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.latency_ms, Some(150.5));
        assert_eq!(result.status_code, Some(200));
        assert!(result.error.is_none());
        assert!(result.is_critical);
    }

    #[test]
    fn test_probe_result_failure() {
        let result = ProbeResult::failure(
            "https://example.com".to_string(),
            "Connection timeout".to_string(),
            false,
        );

        assert!(!result.success);
        assert_eq!(result.url, "https://example.com");
        assert!(result.latency_ms.is_none());
        assert!(result.status_code.is_none());
        assert_eq!(result.error, Some("Connection timeout".to_string()));
        assert!(!result.is_critical);
    }

    #[test]
    fn test_probe_result_failure_with_status() {
        let result = ProbeResult::failure_with_status(
            "https://example.com".to_string(),
            500,
            200.0,
            "Server error".to_string(),
            true,
        );

        assert!(!result.success);
        assert_eq!(result.status_code, Some(500));
        assert_eq!(result.latency_ms, Some(200.0));
        assert!(result.error.is_some());
    }

    #[test]
    fn test_prober_creation() {
        let prober = HttpProber::with_default_config();
        assert_eq!(prober.config().timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_prober_with_custom_config() {
        let config = ProbeConfig::fast();
        let prober = HttpProber::new(config);
        assert_eq!(prober.config().timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_probe_result_serialization() {
        let result = ProbeResult::success(
            "https://test.com".to_string(),
            100.0,
            204,
            true,
        );

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"latency_ms\":100.0"));
        assert!(json.contains("\"status_code\":204"));
    }

    // Integration tests that require network access
    // Run with: cargo test -p isolate-app testing -- --ignored

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_probe_direct_success() {
        let prober = HttpProber::with_default_config();
        let endpoint = TestEndpoint::critical(
            "https://www.google.com/generate_204",
            "Google",
        );

        let result = prober.probe_direct(&endpoint).await;

        // This should succeed on most networks
        println!("Result: {:?}", result);
        // Note: We don't assert success because network conditions vary
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_probe_direct_timeout() {
        let config = ProbeConfig::new(
            Duration::from_millis(100),
            Duration::from_millis(50),
        );
        let prober = HttpProber::new(config);
        let endpoint = TestEndpoint::critical(
            "https://10.255.255.1/", // Non-routable IP
            "Timeout Test",
        );

        let result = prober.probe_direct(&endpoint).await;

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    #[ignore = "Requires network access and SOCKS proxy"]
    async fn test_probe_via_socks() {
        let prober = HttpProber::with_default_config();
        let endpoint = TestEndpoint::critical(
            "https://www.google.com/generate_204",
            "Google",
        );

        // This will fail unless there's a SOCKS proxy on port 1080
        let result = prober.probe_via_socks(&endpoint, 1080).await;
        println!("SOCKS probe result: {:?}", result);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_probe_all_sequential() {
        let prober = HttpProber::new(ProbeConfig::fast());
        let endpoints = vec![
            TestEndpoint::critical("https://www.google.com/generate_204", "Google"),
            TestEndpoint::optional("https://httpbin.org/status/200", "HTTPBin"),
        ];

        let results = prober.probe_all(&endpoints, None).await;

        assert_eq!(results.len(), 2);
        println!("Sequential results: {:?}", results);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_probe_all_concurrent() {
        let prober = HttpProber::new(ProbeConfig::fast());
        let endpoints = vec![
            TestEndpoint::critical("https://www.google.com/generate_204", "Google"),
            TestEndpoint::optional("https://httpbin.org/status/200", "HTTPBin"),
        ];

        let results = prober.probe_all_concurrent(&endpoints, None).await;

        assert_eq!(results.len(), 2);
        println!("Concurrent results: {:?}", results);
    }
}

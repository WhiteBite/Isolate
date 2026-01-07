//! Unified Service Checker - checks service availability
//!
//! Provides parallel endpoint checking with caching support.
//! This module consolidates checker functionality from services and plugins.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::core::retry::{with_retry, RetryConfig};

#[derive(Error, Debug)]
pub enum CheckerError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Check failed: {0}")]
    CheckFailed(String),

    #[error("Timeout")]
    Timeout,

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// HTTP method for endpoint checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    GET,
    HEAD,
    POST,
}

impl HttpMethod {
    /// Parse from string (case-insensitive)
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "HEAD" => HttpMethod::HEAD,
            "POST" => HttpMethod::POST,
            _ => HttpMethod::GET,
        }
    }
}

/// Generic endpoint definition for checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// URL to check
    pub url: String,
    /// Human-readable name for this endpoint
    pub name: String,
    /// HTTP method to use
    #[serde(default)]
    pub method: HttpMethod,
    /// Expected status codes (empty = any 2xx/3xx)
    #[serde(default)]
    pub expected_status: Vec<u16>,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

impl Endpoint {
    /// Create a simple endpoint with defaults
    pub fn new(url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            name: name.into(),
            method: HttpMethod::GET,
            expected_status: Vec::new(),
            timeout_ms: default_timeout(),
        }
    }

    /// Set HTTP method
    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Result of checking a single endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Whether the endpoint is accessible
    pub accessible: bool,
    /// Response latency in milliseconds
    pub latency_ms: Option<u64>,
    /// HTTP status code (if applicable)
    pub status_code: Option<u16>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Endpoint URL that was checked
    pub url: String,
    /// Endpoint name
    pub name: String,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CheckResult {
    /// Create a successful result
    pub fn success(url: impl Into<String>, name: impl Into<String>, latency_ms: u64, status_code: u16) -> Self {
        Self {
            accessible: true,
            latency_ms: Some(latency_ms),
            status_code: Some(status_code),
            error: None,
            url: url.into(),
            name: name.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a failed result
    pub fn failure(url: impl Into<String>, name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            accessible: false,
            latency_ms: None,
            status_code: None,
            error: Some(error.into()),
            url: url.into(),
            name: name.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a timeout result
    pub fn timeout(url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            accessible: false,
            latency_ms: None,
            status_code: None,
            error: Some("Request timed out".to_string()),
            url: url.into(),
            name: name.into(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Aggregated status for a service (all endpoints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service ID
    pub service_id: String,
    /// Service name
    pub service_name: String,
    /// Overall accessibility (any endpoint succeeded)
    pub accessible: bool,
    /// Individual endpoint results
    pub results: Vec<CheckResult>,
    /// Average latency of successful checks (ms)
    pub avg_latency_ms: Option<u64>,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether this result came from cache
    pub from_cache: bool,
    /// Error message (if all checks failed)
    pub error: Option<String>,
}

impl ServiceStatus {
    /// Create from individual check results
    pub fn from_results(
        service_id: impl Into<String>,
        service_name: impl Into<String>,
        results: Vec<CheckResult>,
    ) -> Self {
        let successful: Vec<_> = results.iter().filter(|r| r.accessible).collect();
        let success_count = successful.len();
        let total_count = results.len();

        let avg_latency_ms = if success_count > 0 {
            let total_latency: u64 = successful.iter().filter_map(|r| r.latency_ms).sum();
            Some(total_latency / success_count as u64)
        } else {
            None
        };

        let success_rate = if total_count > 0 {
            success_count as f64 / total_count as f64
        } else {
            0.0
        };

        let error = if success_count == 0 && !results.is_empty() {
            results.first().and_then(|r| r.error.clone())
        } else {
            None
        };

        Self {
            service_id: service_id.into(),
            service_name: service_name.into(),
            accessible: success_count > 0,
            results,
            avg_latency_ms,
            success_rate,
            timestamp: chrono::Utc::now(),
            from_cache: false,
            error,
        }
    }

    /// Mark as from cache
    pub fn with_cache_flag(mut self, from_cache: bool) -> Self {
        self.from_cache = from_cache;
        self
    }
}

/// Cached check result with expiration
struct CachedStatus {
    status: ServiceStatus,
    expires_at: Instant,
}

/// HTTP client wrapper for endpoint checking
pub struct EndpointChecker {
    client: reqwest::Client,
    /// Retry configuration for failed requests
    retry_config: RetryConfig,
}

impl EndpointChecker {
    /// Create a new endpoint checker
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Isolate/1.0")
            .build()
            .unwrap_or_default();

        Self { 
            client,
            retry_config: RetryConfig::network(),
        }
    }

    /// Create with custom retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Check a single endpoint (with retry for transient failures)
    pub async fn check(&self, endpoint: &Endpoint) -> CheckResult {
        let start = Instant::now();
        let url = endpoint.url.clone();
        let name = endpoint.name.clone();
        let timeout_ms = endpoint.timeout_ms;
        let method = endpoint.method.clone();
        let expected_status = endpoint.expected_status.clone();

        // Use retry for network operations
        let result = with_retry(
            self.retry_config.clone(),
            &format!("check_endpoint:{}", &name),
            || {
                let client = self.client.clone();
                let url = url.clone();
                let method = method.clone();
                let expected_status = expected_status.clone();
                
                async move {
                    let request = match method {
                        HttpMethod::GET => client.get(&url),
                        HttpMethod::HEAD => client.head(&url),
                        HttpMethod::POST => client.post(&url),
                    };

                    let request = request.timeout(Duration::from_millis(timeout_ms));

                    match request.send().await {
                        Ok(response) => {
                            let status = response.status().as_u16();

                            // Check if status is acceptable
                            let is_success = if expected_status.is_empty() {
                                response.status().is_success() || response.status().is_redirection()
                            } else {
                                expected_status.contains(&status)
                            };

                            if is_success {
                                Ok((status, true, None))
                            } else {
                                // Non-retryable: wrong status code
                                Ok((status, false, Some(format!("Unexpected status: {}", status))))
                            }
                        }
                        Err(e) => {
                            if e.is_timeout() {
                                Err("Request timed out".to_string())
                            } else if e.is_connect() {
                                Err(format!("Connection failed: {}", e))
                            } else {
                                Err(e.to_string())
                            }
                        }
                    }
                }
            },
        )
        .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((status, true, _)) => CheckResult::success(&endpoint.url, &endpoint.name, latency_ms, status),
            Ok((status, false, error)) => CheckResult {
                accessible: false,
                latency_ms: Some(latency_ms),
                status_code: Some(status),
                error,
                url: endpoint.url.clone(),
                name: endpoint.name.clone(),
                timestamp: chrono::Utc::now(),
            },
            Err(e) => {
                if e.contains("timed out") || e.contains("timeout") {
                    CheckResult::timeout(&endpoint.url, &endpoint.name)
                } else {
                    CheckResult::failure(&endpoint.url, &endpoint.name, e)
                }
            }
        }
    }

    /// Check multiple endpoints in parallel
    pub async fn check_all(&self, endpoints: &[Endpoint]) -> Vec<CheckResult> {
        if endpoints.is_empty() {
            return Vec::new();
        }

        let futures: Vec<_> = endpoints.iter().map(|ep| self.check(ep)).collect();
        futures::future::join_all(futures).await
    }
}

impl Default for EndpointChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Concrete ServiceChecker for services::registry::ServiceRegistry
// ============================================================================

/// Service Checker with caching support
/// 
/// Works with services::registry::ServiceRegistry directly.
pub struct ServiceChecker {
    /// HTTP client for making requests
    checker: EndpointChecker,
    /// Service registry reference
    registry: Arc<crate::services::registry::ServiceRegistry>,
    /// Cache of recent check results
    cache: RwLock<HashMap<String, CachedStatus>>,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

impl ServiceChecker {
    /// Create a new service checker
    pub fn new(registry: Arc<crate::services::registry::ServiceRegistry>) -> Self {
        Self::with_cache_ttl(registry, 300) // 5 minutes default
    }

    /// Create with custom cache TTL
    pub fn with_cache_ttl(registry: Arc<crate::services::registry::ServiceRegistry>, cache_ttl_secs: u64) -> Self {
        Self {
            checker: EndpointChecker::new(),
            registry,
            cache: RwLock::new(HashMap::new()),
            cache_ttl_secs,
        }
    }

    /// Convert registry service to checker endpoint format
    fn convert_service(service: &crate::services::registry::Service) -> Vec<Endpoint> {
        service.endpoints.iter().map(|ep| {
            Endpoint {
                url: ep.url.clone(),
                name: ep.name.clone(),
                method: match ep.method {
                    crate::services::registry::HttpMethod::GET => HttpMethod::GET,
                    crate::services::registry::HttpMethod::HEAD => HttpMethod::HEAD,
                    crate::services::registry::HttpMethod::POST => HttpMethod::POST,
                },
                expected_status: ep.expected_status.clone(),
                timeout_ms: ep.timeout_ms,
            }
        }).collect()
    }

    /// Check a service by ID (with caching)
    pub async fn check_service(&self, service_id: &str) -> Result<ServiceStatus, CheckerError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(service_id) {
                if cached.expires_at > Instant::now() {
                    debug!(service_id = %service_id, "Returning cached status");
                    return Ok(cached.status.clone().with_cache_flag(true));
                }
            }
        }

        // Get service from registry
        let service = self
            .registry
            .get(service_id)
            .await
            .ok_or_else(|| CheckerError::ServiceNotFound(service_id.to_string()))?;

        // Convert and check all endpoints in parallel
        let endpoints = Self::convert_service(&service);
        let results = self.checker.check_all(&endpoints).await;
        let status = ServiceStatus::from_results(&service.id, &service.name, results);

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                service_id.to_string(),
                CachedStatus {
                    status: status.clone(),
                    expires_at: Instant::now() + Duration::from_secs(self.cache_ttl_secs),
                },
            );
        }

        info!(
            service_id = %service_id,
            accessible = status.accessible,
            success_rate = status.success_rate,
            "Service check completed"
        );

        Ok(status)
    }

    /// Check a service without using cache
    pub async fn check_service_fresh(&self, service_id: &str) -> Result<ServiceStatus, CheckerError> {
        // Invalidate cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(service_id);
        }

        self.check_service(service_id).await
    }

    /// Check all registered services
    pub async fn check_all_services(&self) -> Vec<ServiceStatus> {
        let services = self.registry.get_all().await;

        if services.is_empty() {
            return Vec::new();
        }

        // Check all services in parallel
        let futures: Vec<_> = services
            .iter()
            .map(|s| self.check_service(&s.id))
            .collect();

        let results = futures::future::join_all(futures).await;
        results.into_iter().filter_map(|r| r.ok()).collect()
    }

    /// Get cached status for a service (if available)
    pub async fn get_cached_status(&self, service_id: &str) -> Option<ServiceStatus> {
        let cache = self.cache.read().await;
        cache.get(service_id).and_then(|cached| {
            if cached.expires_at > Instant::now() {
                Some(cached.status.clone().with_cache_flag(true))
            } else {
                None
            }
        })
    }

    /// Clear all cached results
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Service checker cache cleared");
    }

    /// Clear expired cache entries
    pub async fn cleanup_expired_cache(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        cache.retain(|_, v| v.expires_at > now);
    }

    /// Get the service registry
    pub fn registry(&self) -> &Arc<crate::services::registry::ServiceRegistry> {
        &self.registry
    }
}

// ============================================================================
// Standalone endpoint checking (for plugins without registry)
// ============================================================================

/// Check endpoints without a registry (standalone mode)
pub async fn check_endpoints(endpoints: &[Endpoint]) -> ServiceStatus {
    let checker = EndpointChecker::new();
    let results = checker.check_all(endpoints).await;
    ServiceStatus::from_results("", "", results)
}

/// Check endpoints from plugin manifest format
pub async fn check_plugin_endpoints(endpoints: &[crate::plugins::manifest::ServiceEndpoint]) -> ServiceStatus {
    let converted: Vec<Endpoint> = endpoints
        .iter()
        .map(|ep| Endpoint {
            url: ep.url.clone(),
            name: ep.name.clone(),
            method: HttpMethod::from_str_loose(&ep.method),
            expected_status: Vec::new(),
            timeout_ms: default_timeout(),
        })
        .collect();

    check_endpoints(&converted).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_success() {
        let result = CheckResult::success("https://example.com", "Test", 100, 200);
        assert!(result.accessible);
        assert_eq!(result.latency_ms, Some(100));
        assert_eq!(result.status_code, Some(200));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_check_result_failure() {
        let result = CheckResult::failure("https://example.com", "Test", "Connection refused");
        assert!(!result.accessible);
        assert!(result.latency_ms.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_service_status_from_results() {
        let results = vec![
            CheckResult::success("https://a.com", "A", 100, 200),
            CheckResult::success("https://b.com", "B", 200, 200),
            CheckResult::failure("https://c.com", "C", "Error"),
        ];

        let status = ServiceStatus::from_results("test", "Test Service", results);

        assert!(status.accessible);
        assert_eq!(status.results.len(), 3);
        assert!((status.success_rate - 0.666).abs() < 0.01);
        assert_eq!(status.avg_latency_ms, Some(150)); // (100 + 200) / 2
    }

    #[test]
    fn test_service_status_all_failed() {
        let results = vec![
            CheckResult::failure("https://a.com", "A", "Error 1"),
            CheckResult::failure("https://b.com", "B", "Error 2"),
        ];

        let status = ServiceStatus::from_results("test", "Test", results);

        assert!(!status.accessible);
        assert_eq!(status.success_rate, 0.0);
        assert!(status.avg_latency_ms.is_none());
        assert!(status.error.is_some());
    }

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str_loose("GET"), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str_loose("get"), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str_loose("HEAD"), HttpMethod::HEAD);
        assert_eq!(HttpMethod::from_str_loose("head"), HttpMethod::HEAD);
        assert_eq!(HttpMethod::from_str_loose("POST"), HttpMethod::POST);
        assert_eq!(HttpMethod::from_str_loose("unknown"), HttpMethod::GET);
    }

    #[test]
    fn test_endpoint_builder() {
        let endpoint = Endpoint::new("https://example.com", "Test")
            .with_method(HttpMethod::HEAD)
            .with_timeout(3000);

        assert_eq!(endpoint.url, "https://example.com");
        assert_eq!(endpoint.name, "Test");
        assert_eq!(endpoint.method, HttpMethod::HEAD);
        assert_eq!(endpoint.timeout_ms, 3000);
    }
}

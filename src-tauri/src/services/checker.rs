//! Service Checker - checks service availability
//!
//! Provides parallel endpoint checking with caching support.

use super::registry::{HttpMethod, Service, ServiceEndpoint, ServiceRegistry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

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
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CheckResult {
    /// Create a successful result
    pub fn success(url: impl Into<String>, latency_ms: u64, status_code: u16) -> Self {
        Self {
            accessible: true,
            latency_ms: Some(latency_ms),
            status_code: Some(status_code),
            error: None,
            url: url.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a failed result
    pub fn failure(url: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            accessible: false,
            latency_ms: None,
            status_code: None,
            error: Some(error.into()),
            url: url.into(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a timeout result
    pub fn timeout(url: impl Into<String>) -> Self {
        Self {
            accessible: false,
            latency_ms: None,
            status_code: None,
            error: Some("Request timed out".to_string()),
            url: url.into(),
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

        Self {
            service_id: service_id.into(),
            service_name: service_name.into(),
            accessible: success_count > 0,
            results,
            avg_latency_ms,
            success_rate,
            timestamp: chrono::Utc::now(),
            from_cache: false,
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

/// Service Checker - checks service availability with caching
pub struct ServiceChecker {
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Service registry reference
    registry: Arc<ServiceRegistry>,
    /// Cache of recent check results
    cache: RwLock<HashMap<String, CachedStatus>>,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

impl ServiceChecker {
    /// Create a new service checker
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self::with_cache_ttl(registry, 300) // 5 minutes default
    }

    /// Create with custom cache TTL
    pub fn with_cache_ttl(registry: Arc<ServiceRegistry>, cache_ttl_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Isolate/1.0")
            .build()
            .unwrap_or_default();

        Self {
            client,
            registry,
            cache: RwLock::new(HashMap::new()),
            cache_ttl_secs,
        }
    }

    /// Check a single endpoint
    async fn check_endpoint(&self, endpoint: &ServiceEndpoint) -> CheckResult {
        let start = Instant::now();

        let request = match endpoint.method {
            HttpMethod::GET => self.client.get(&endpoint.url),
            HttpMethod::HEAD => self.client.head(&endpoint.url),
            HttpMethod::POST => self.client.post(&endpoint.url),
        };

        let request = request.timeout(Duration::from_millis(endpoint.timeout_ms));

        match request.send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let status = response.status().as_u16();

                // Check if status is acceptable
                let is_success = if endpoint.expected_status.is_empty() {
                    response.status().is_success() || response.status().is_redirection()
                } else {
                    endpoint.expected_status.contains(&status)
                };

                if is_success {
                    CheckResult::success(&endpoint.url, latency_ms, status)
                } else {
                    CheckResult {
                        accessible: false,
                        latency_ms: Some(latency_ms),
                        status_code: Some(status),
                        error: Some(format!("Unexpected status: {}", status)),
                        url: endpoint.url.clone(),
                        timestamp: chrono::Utc::now(),
                    }
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    CheckResult::timeout(&endpoint.url)
                } else if e.is_connect() {
                    CheckResult::failure(&endpoint.url, format!("Connection failed: {}", e))
                } else {
                    CheckResult::failure(&endpoint.url, e.to_string())
                }
            }
        }
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

        // Check all endpoints in parallel
        let status = self.check_service_endpoints(&service).await;

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

    /// Check all endpoints of a service in parallel
    async fn check_service_endpoints(&self, service: &Service) -> ServiceStatus {
        if service.endpoints.is_empty() {
            return ServiceStatus::from_results(&service.id, &service.name, Vec::new());
        }

        // Check all endpoints in parallel using tokio::join!
        let futures: Vec<_> = service
            .endpoints
            .iter()
            .map(|ep| self.check_endpoint(ep))
            .collect();

        let results = futures::future::join_all(futures).await;

        ServiceStatus::from_results(&service.id, &service.name, results)
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
    pub fn registry(&self) -> &Arc<ServiceRegistry> {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_success() {
        let result = CheckResult::success("https://example.com", 100, 200);
        assert!(result.accessible);
        assert_eq!(result.latency_ms, Some(100));
        assert_eq!(result.status_code, Some(200));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_check_result_failure() {
        let result = CheckResult::failure("https://example.com", "Connection refused");
        assert!(!result.accessible);
        assert!(result.latency_ms.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_service_status_from_results() {
        let results = vec![
            CheckResult::success("https://a.com", 100, 200),
            CheckResult::success("https://b.com", 200, 200),
            CheckResult::failure("https://c.com", "Error"),
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
            CheckResult::failure("https://a.com", "Error 1"),
            CheckResult::failure("https://b.com", "Error 2"),
        ];

        let status = ServiceStatus::from_results("test", "Test", results);
        
        assert!(!status.accessible);
        assert_eq!(status.success_rate, 0.0);
        assert!(status.avg_latency_ms.is_none());
    }

    #[tokio::test]
    async fn test_checker_with_registry() {
        let registry = Arc::new(ServiceRegistry::new());
        registry.register_builtin_services().await;

        let checker = ServiceChecker::new(registry);
        
        // Just verify it doesn't panic
        let services = checker.registry().get_all().await;
        assert!(!services.is_empty());
    }
}

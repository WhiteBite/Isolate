//! Test endpoint registry for strategy validation
//!
//! Manages a collection of test endpoints used to verify DPI bypass strategies.
//! Endpoints can be marked as critical (must work) or optional.
//!
//! NOTE: This module is part of the testing infrastructure for strategy validation.

// Public API for test endpoint management
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::core::models::Service;

/// A test endpoint for strategy validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEndpoint {
    /// URL to test (should return quickly, e.g., /generate_204)
    pub url: String,
    /// Human-readable name for the endpoint
    pub name: String,
    /// Whether this endpoint must succeed for strategy to be viable
    pub is_critical: bool,
}

impl TestEndpoint {
    /// Create a new test endpoint
    pub fn new(url: impl Into<String>, name: impl Into<String>, is_critical: bool) -> Self {
        Self {
            url: url.into(),
            name: name.into(),
            is_critical,
        }
    }

    /// Create a critical endpoint (must succeed)
    pub fn critical(url: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(url, name, true)
    }

    /// Create an optional endpoint
    pub fn optional(url: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(url, name, false)
    }
}

/// Registry of test endpoints
#[derive(Debug, Clone)]
pub struct EndpointRegistry {
    endpoints: Vec<TestEndpoint>,
}

impl Default for EndpointRegistry {
    /// Creates registry with default test endpoints for common blocked services
    fn default() -> Self {
        Self {
            endpoints: vec![
                TestEndpoint {
                    url: "https://www.youtube.com/generate_204".into(),
                    name: "YouTube".into(),
                    is_critical: true,
                },
                TestEndpoint {
                    url: "https://discord.com/api/v10/gateway".into(),
                    name: "Discord".into(),
                    is_critical: true,
                },
                TestEndpoint {
                    url: "https://api.telegram.org/".into(),
                    name: "Telegram".into(),
                    is_critical: false,
                },
            ],
        }
    }
}

impl EndpointRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }

    /// Create registry from a list of services
    ///
    /// Extracts test URLs from services and creates endpoints.
    /// Services marked as critical will have critical endpoints.
    pub fn from_services(services: &[Service]) -> Self {
        let endpoints = services
            .iter()
            .filter_map(|service| {
                service.get_test_url().map(|url| TestEndpoint {
                    url,
                    name: service.name.clone(),
                    is_critical: service.critical,
                })
            })
            .collect();

        Self { endpoints }
    }

    /// Get all endpoints
    pub fn get_all(&self) -> &[TestEndpoint] {
        &self.endpoints
    }

    /// Get only critical endpoints
    pub fn get_critical(&self) -> Vec<&TestEndpoint> {
        self.endpoints.iter().filter(|e| e.is_critical).collect()
    }

    /// Get only optional (non-critical) endpoints
    pub fn get_optional(&self) -> Vec<&TestEndpoint> {
        self.endpoints.iter().filter(|e| !e.is_critical).collect()
    }

    /// Add an endpoint to the registry
    pub fn add(&mut self, endpoint: TestEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Add multiple endpoints
    pub fn add_all(&mut self, endpoints: impl IntoIterator<Item = TestEndpoint>) {
        self.endpoints.extend(endpoints);
    }

    /// Remove all endpoints
    pub fn clear(&mut self) {
        self.endpoints.clear();
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    /// Get number of endpoints
    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    /// Get number of critical endpoints
    pub fn critical_count(&self) -> usize {
        self.endpoints.iter().filter(|e| e.is_critical).count()
    }

    /// Find endpoint by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&TestEndpoint> {
        let name_lower = name.to_lowercase();
        self.endpoints
            .iter()
            .find(|e| e.name.to_lowercase() == name_lower)
    }

    /// Merge with another registry (adds all endpoints from other)
    pub fn merge(&mut self, other: &EndpointRegistry) {
        self.endpoints.extend(other.endpoints.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_new() {
        let endpoint = TestEndpoint::new("https://example.com", "Example", true);
        assert_eq!(endpoint.url, "https://example.com");
        assert_eq!(endpoint.name, "Example");
        assert!(endpoint.is_critical);
    }

    #[test]
    fn test_endpoint_critical() {
        let endpoint = TestEndpoint::critical("https://youtube.com", "YouTube");
        assert!(endpoint.is_critical);
    }

    #[test]
    fn test_endpoint_optional() {
        let endpoint = TestEndpoint::optional("https://telegram.org", "Telegram");
        assert!(!endpoint.is_critical);
    }

    #[test]
    fn test_registry_default() {
        let registry = EndpointRegistry::default();
        assert!(!registry.is_empty());
        assert!(registry.len() >= 3);

        // Check default endpoints exist
        assert!(registry.find_by_name("YouTube").is_some());
        assert!(registry.find_by_name("Discord").is_some());
        assert!(registry.find_by_name("Telegram").is_some());

        // YouTube and Discord should be critical
        let youtube = registry.find_by_name("YouTube").unwrap();
        assert!(youtube.is_critical);

        let discord = registry.find_by_name("Discord").unwrap();
        assert!(discord.is_critical);

        // Telegram should be optional
        let telegram = registry.find_by_name("Telegram").unwrap();
        assert!(!telegram.is_critical);
    }

    #[test]
    fn test_registry_new_empty() {
        let registry = EndpointRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_add() {
        let mut registry = EndpointRegistry::new();
        registry.add(TestEndpoint::critical("https://test.com", "Test"));

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_add_all() {
        let mut registry = EndpointRegistry::new();
        registry.add_all(vec![
            TestEndpoint::critical("https://a.com", "A"),
            TestEndpoint::optional("https://b.com", "B"),
        ]);

        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_get_critical() {
        let registry = EndpointRegistry::default();
        let critical = registry.get_critical();

        assert!(!critical.is_empty());
        for endpoint in critical {
            assert!(endpoint.is_critical);
        }
    }

    #[test]
    fn test_registry_get_optional() {
        let registry = EndpointRegistry::default();
        let optional = registry.get_optional();

        for endpoint in optional {
            assert!(!endpoint.is_critical);
        }
    }

    #[test]
    fn test_registry_critical_count() {
        let mut registry = EndpointRegistry::new();
        registry.add(TestEndpoint::critical("https://a.com", "A"));
        registry.add(TestEndpoint::critical("https://b.com", "B"));
        registry.add(TestEndpoint::optional("https://c.com", "C"));

        assert_eq!(registry.critical_count(), 2);
    }

    #[test]
    fn test_registry_find_by_name() {
        let registry = EndpointRegistry::default();

        // Case-insensitive search
        assert!(registry.find_by_name("youtube").is_some());
        assert!(registry.find_by_name("YOUTUBE").is_some());
        assert!(registry.find_by_name("YouTube").is_some());

        // Non-existent
        assert!(registry.find_by_name("NonExistent").is_none());
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = EndpointRegistry::default();
        assert!(!registry.is_empty());

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_merge() {
        let mut registry1 = EndpointRegistry::new();
        registry1.add(TestEndpoint::critical("https://a.com", "A"));

        let mut registry2 = EndpointRegistry::new();
        registry2.add(TestEndpoint::optional("https://b.com", "B"));

        registry1.merge(&registry2);
        assert_eq!(registry1.len(), 2);
    }

    #[test]
    fn test_registry_from_services() {
        use crate::core::models::{Service, TestDefinition};

        let services = vec![
            Service {
                id: "youtube".to_string(),
                name: "YouTube".to_string(),
                enabled_by_default: true,
                critical: true,
                tests: vec![TestDefinition::HttpsGet {
                    url: "https://www.youtube.com/generate_204".to_string(),
                    timeout_ms: 5000,
                    expected_status: vec![204],
                    min_body_size: None,
                }],
                test_url: None,
            },
            Service {
                id: "discord".to_string(),
                name: "Discord".to_string(),
                enabled_by_default: true,
                critical: true,
                tests: vec![],
                test_url: Some("https://discord.com/api/v10/gateway".to_string()),
            },
            Service {
                id: "telegram".to_string(),
                name: "Telegram".to_string(),
                enabled_by_default: true,
                critical: false,
                tests: vec![],
                test_url: Some("https://api.telegram.org/".to_string()),
            },
            // Service without test URL should be skipped
            Service {
                id: "no_url".to_string(),
                name: "No URL".to_string(),
                enabled_by_default: true,
                critical: false,
                tests: vec![TestDefinition::TcpConnect {
                    host: "example.com".to_string(),
                    port: 443,
                    timeout_ms: 5000,
                }],
                test_url: None,
            },
        ];

        let registry = EndpointRegistry::from_services(&services);

        // Should have 3 endpoints (one service has no URL)
        assert_eq!(registry.len(), 3);

        // Check critical flags are preserved
        let youtube = registry.find_by_name("YouTube").unwrap();
        assert!(youtube.is_critical);

        let telegram = registry.find_by_name("Telegram").unwrap();
        assert!(!telegram.is_critical);
    }

    #[test]
    fn test_endpoint_serialization() {
        let endpoint = TestEndpoint::critical("https://test.com", "Test");
        let json = serde_json::to_string(&endpoint).unwrap();
        let deserialized: TestEndpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(endpoint.url, deserialized.url);
        assert_eq!(endpoint.name, deserialized.name);
        assert_eq!(endpoint.is_critical, deserialized.is_critical);
    }
}

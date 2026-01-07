//! Service-related models

use serde::{Deserialize, Serialize};

/// Test type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TestDefinition {
    HttpsGet {
        url: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
        #[serde(default)]
        expected_status: Vec<u16>,
        min_body_size: Option<usize>,
    },
    HttpsHead {
        url: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
    WebSocket {
        url: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
    TcpConnect {
        host: String,
        port: u16,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
    Dns {
        domain: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u32,
    },
}

fn default_timeout() -> u32 {
    5000
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled_by_default: bool,
    #[serde(default)]
    pub critical: bool,
    pub tests: Vec<TestDefinition>,
    /// Simple test URL for quick connectivity checks
    #[serde(default)]
    pub test_url: Option<String>,
}

impl Service {
    /// Get the test URL for this service
    /// Falls back to extracting URL from first HTTPS test if test_url is not set
    pub fn get_test_url(&self) -> Option<String> {
        if let Some(ref url) = self.test_url {
            return Some(url.clone());
        }
        
        // Try to extract from tests
        for test in &self.tests {
            match test {
                TestDefinition::HttpsGet { url, .. } => return Some(url.clone()),
                TestDefinition::HttpsHead { url, .. } => return Some(url.clone()),
                TestDefinition::WebSocket { url, .. } => return Some(url.clone()),
                _ => continue,
            }
        }
        
        None
    }
}

fn default_true() -> bool {
    true
}

/// Service with enabled state for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWithState {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub critical: bool,
}

/// Aggregated service test summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTestSummary {
    pub service_id: String,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub errors: Vec<super::ErrorType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_get_test_url() {
        // Test with explicit test_url
        let service_with_url = Service {
            id: "test".to_string(),
            name: "Test Service".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![],
            test_url: Some("https://example.com".to_string()),
        };
        assert_eq!(service_with_url.get_test_url(), Some("https://example.com".to_string()));

        // Test extracting from HttpsGet test
        let service_with_test = Service {
            id: "test2".to_string(),
            name: "Test Service 2".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![
                TestDefinition::HttpsGet {
                    url: "https://test.com/api".to_string(),
                    timeout_ms: 5000,
                    expected_status: vec![200],
                    min_body_size: None,
                },
            ],
            test_url: None,
        };
        assert_eq!(service_with_test.get_test_url(), Some("https://test.com/api".to_string()));

        // Test with no URL available
        let service_no_url = Service {
            id: "test3".to_string(),
            name: "Test Service 3".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![
                TestDefinition::TcpConnect {
                    host: "example.com".to_string(),
                    port: 443,
                    timeout_ms: 5000,
                },
            ],
            test_url: None,
        };
        assert_eq!(service_no_url.get_test_url(), None);
    }

    #[test]
    fn test_test_definition_variants() {
        let tests = vec![
            TestDefinition::HttpsGet {
                url: "https://example.com".to_string(),
                timeout_ms: 5000,
                expected_status: vec![200, 201],
                min_body_size: Some(100),
            },
            TestDefinition::HttpsHead {
                url: "https://example.com".to_string(),
                timeout_ms: 3000,
            },
            TestDefinition::WebSocket {
                url: "wss://example.com/ws".to_string(),
                timeout_ms: 5000,
            },
            TestDefinition::TcpConnect {
                host: "example.com".to_string(),
                port: 443,
                timeout_ms: 5000,
            },
            TestDefinition::Dns {
                domain: "example.com".to_string(),
                timeout_ms: 2000,
            },
        ];

        for test in tests {
            let json = serde_json::to_string(&test).unwrap();
            let _deserialized: TestDefinition = serde_json::from_str(&json).unwrap();
        }
    }
}

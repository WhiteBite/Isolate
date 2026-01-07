//! Diagnostic and result models

use serde::{Deserialize, Serialize};
use super::StrategyFamily;

/// Error type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    Dns,
    Tcp,
    Tls,
    Http,
    Timeout,
    Unknown,
}

/// Single test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub error_type: Option<ErrorType>,
    pub error_message: Option<String>,
}

/// DPI block type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DpiKind {
    DnsBlock,
    SniTlsBlock,
    IpBlock,
    #[default]
    NoBlock,
    Unknown,
}

/// DPI diagnostic profile
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DpiProfile {
    pub kind: DpiKind,
    pub details: Option<String>,
    pub candidate_families: Vec<StrategyFamily>,
}

/// Diagnostic result for frontend
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticResult {
    pub profile: DpiProfile,
    pub tested_services: Vec<String>,
    pub blocked_services: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_serialization() {
        let error_types = vec![
            (ErrorType::Dns, "\"dns\""),
            (ErrorType::Tcp, "\"tcp\""),
            (ErrorType::Tls, "\"tls\""),
            (ErrorType::Http, "\"http\""),
            (ErrorType::Timeout, "\"timeout\""),
            (ErrorType::Unknown, "\"unknown\""),
        ];

        for (error_type, expected_json) in error_types {
            let json = serde_json::to_string(&error_type).unwrap();
            assert_eq!(json, expected_json);
            
            let deserialized: ErrorType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, error_type);
        }
    }

    #[test]
    fn test_error_type_equality() {
        assert_eq!(ErrorType::Dns, ErrorType::Dns);
        assert_ne!(ErrorType::Dns, ErrorType::Tcp);
        assert_ne!(ErrorType::Timeout, ErrorType::Unknown);
    }

    #[test]
    fn test_error_type_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(ErrorType::Dns);
        set.insert(ErrorType::Tcp);
        set.insert(ErrorType::Dns); // duplicate
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&ErrorType::Dns));
        assert!(set.contains(&ErrorType::Tcp));
    }

    #[test]
    fn test_dpi_kind_default() {
        let kind: DpiKind = Default::default();
        assert_eq!(kind, DpiKind::NoBlock);
    }

    #[test]
    fn test_test_result_success() {
        let result = TestResult {
            test_id: "test-1".to_string(),
            success: true,
            latency_ms: Some(150),
            error_type: None,
            error_message: None,
        };
        
        assert!(result.success);
        assert_eq!(result.latency_ms, Some(150));
        assert!(result.error_type.is_none());
    }

    #[test]
    fn test_test_result_failure() {
        let result = TestResult {
            test_id: "test-2".to_string(),
            success: false,
            latency_ms: None,
            error_type: Some(ErrorType::Timeout),
            error_message: Some("Connection timed out".to_string()),
        };
        
        assert!(!result.success);
        assert!(result.latency_ms.is_none());
        assert_eq!(result.error_type, Some(ErrorType::Timeout));
    }

    #[test]
    fn test_diagnostic_result_default() {
        let result = DiagnosticResult::default();
        
        assert_eq!(result.profile.kind, DpiKind::NoBlock);
        assert!(result.profile.details.is_none());
        assert!(result.profile.candidate_families.is_empty());
        assert!(result.tested_services.is_empty());
        assert!(result.blocked_services.is_empty());
    }

    #[test]
    fn test_dpi_profile_default() {
        let profile = DpiProfile::default();
        
        assert_eq!(profile.kind, DpiKind::NoBlock);
        assert!(profile.details.is_none());
        assert!(profile.candidate_families.is_empty());
    }
}

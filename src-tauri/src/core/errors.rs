//! Error types for Isolate
//!
//! Unified error handling for the entire application.
//! All errors are serializable for Tauri IPC communication.

use serde::Serialize;
use thiserror::Error;

/// Main error type for Isolate application.
/// 
/// Implements `Serialize` for Tauri command responses.
/// Use `From` implementations for automatic conversion from common error types.
#[derive(Error, Debug)]
pub enum IsolateError {
    // ==================== Configuration ====================
    
    #[error("Configuration error: {0}")]
    Config(String),

    // ==================== Strategy ====================
    
    #[error("Strategy error: {0}")]
    Strategy(String),
    
    #[error("Strategy not found: {0}")]
    StrategyNotFound(String),
    
    #[error("Strategy timeout after {0}ms")]
    StrategyTimeout(u32),
    
    #[error("No working strategy found")]
    NoStrategyFound,

    // ==================== Process ====================
    
    #[error("Process error: {0}")]
    Process(String),

    #[error("WinDivert driver not loaded")]
    DriverNotLoaded,

    // ==================== Network ====================
    
    #[error("Network error: {0}")]
    Network(String),

    #[error("HTTP error: {0}")]
    Http(String),

    // ==================== IO & Storage ====================
    
    #[error("IO error: {0}")]
    Io(String),

    #[error("Storage error: {0}")]
    Storage(String),

    // ==================== Parsing ====================
    
    #[error("YAML parse error: {0}")]
    Yaml(String),

    #[error("JSON error: {0}")]
    Json(String),

    // ==================== Database ====================
    
    #[error("Database error: {0}")]
    Database(String),

    // ==================== Testing ====================
    
    #[error("Test failed: {0}")]
    TestFailed(String),

    // ==================== Validation ====================
    
    #[error("Validation error: {0}")]
    Validation(String),

    // ==================== System ====================
    
    #[error("Requires administrator privileges")]
    RequiresAdmin,

    #[error("System proxy error: {0}")]
    SystemProxy(String),

    // ==================== Tauri ====================
    
    #[error("Tauri error: {0}")]
    Tauri(String),

    // ==================== Control Flow ====================
    
    #[error("Optimization cancelled")]
    Cancelled,

    // ==================== Other ====================
    
    #[error("{0}")]
    Other(String),
}

// ==================== Serialize for Tauri IPC ====================

impl Serialize for IsolateError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as a structured object for better frontend handling
        use serde::ser::SerializeStruct;
        
        let kind = self.kind();
        let message = self.to_string();
        
        let mut state = serializer.serialize_struct("IsolateError", 2)?;
        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &message)?;
        state.end()
    }
}

// ==================== From implementations ====================

impl From<std::io::Error> for IsolateError {
    fn from(err: std::io::Error) -> Self {
        IsolateError::Io(err.to_string())
    }
}

impl From<serde_yaml::Error> for IsolateError {
    fn from(err: serde_yaml::Error) -> Self {
        IsolateError::Yaml(err.to_string())
    }
}

impl From<serde_json::Error> for IsolateError {
    fn from(err: serde_json::Error) -> Self {
        IsolateError::Json(err.to_string())
    }
}

impl From<reqwest::Error> for IsolateError {
    fn from(err: reqwest::Error) -> Self {
        IsolateError::Http(err.to_string())
    }
}

impl From<rusqlite::Error> for IsolateError {
    fn from(err: rusqlite::Error) -> Self {
        IsolateError::Database(err.to_string())
    }
}

impl From<anyhow::Error> for IsolateError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to IsolateError first
        match err.downcast::<IsolateError>() {
            Ok(isolate_err) => isolate_err,
            Err(err) => IsolateError::Other(err.to_string()),
        }
    }
}

impl From<String> for IsolateError {
    fn from(msg: String) -> Self {
        IsolateError::Other(msg)
    }
}

impl From<&str> for IsolateError {
    fn from(msg: &str) -> Self {
        IsolateError::Other(msg.to_string())
    }
}

/// Type alias for Result with IsolateError
pub type Result<T> = std::result::Result<T, IsolateError>;

// ==================== Helper constructors ====================

impl IsolateError {
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        IsolateError::Config(msg.into())
    }
    
    /// Create a strategy error
    pub fn strategy(msg: impl Into<String>) -> Self {
        IsolateError::Strategy(msg.into())
    }
    
    /// Create a process error
    pub fn process(msg: impl Into<String>) -> Self {
        IsolateError::Process(msg.into())
    }
    
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        IsolateError::Network(msg.into())
    }
    
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        IsolateError::Validation(msg.into())
    }
    
    /// Create a Tauri error
    pub fn tauri(msg: impl Into<String>) -> Self {
        IsolateError::Tauri(msg.into())
    }
    
    /// Create an other/generic error
    pub fn other(msg: impl Into<String>) -> Self {
        IsolateError::Other(msg.into())
    }
    
    /// Get the error kind as a string (useful for frontend)
    pub fn kind(&self) -> &'static str {
        match self {
            IsolateError::Config(_) => "config",
            IsolateError::Strategy(_) => "strategy",
            IsolateError::StrategyNotFound(_) => "strategy_not_found",
            IsolateError::StrategyTimeout(_) => "strategy_timeout",
            IsolateError::NoStrategyFound => "no_strategy_found",
            IsolateError::Process(_) => "process",
            IsolateError::DriverNotLoaded => "driver_not_loaded",
            IsolateError::Network(_) => "network",
            IsolateError::Http(_) => "http",
            IsolateError::Io(_) => "io",
            IsolateError::Storage(_) => "storage",
            IsolateError::Yaml(_) => "yaml",
            IsolateError::Json(_) => "json",
            IsolateError::Database(_) => "database",
            IsolateError::TestFailed(_) => "test_failed",
            IsolateError::Validation(_) => "validation",
            IsolateError::RequiresAdmin => "requires_admin",
            IsolateError::SystemProxy(_) => "system_proxy",
            IsolateError::Tauri(_) => "tauri",
            IsolateError::Cancelled => "cancelled",
            IsolateError::Other(_) => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Display trait tests ====================

    #[test]
    fn test_config_error_display() {
        let err = IsolateError::Config("invalid format".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid format");
    }

    #[test]
    fn test_strategy_error_display() {
        let err = IsolateError::Strategy("failed to apply".to_string());
        assert_eq!(err.to_string(), "Strategy error: failed to apply");
    }

    #[test]
    fn test_strategy_not_found_display() {
        let err = IsolateError::StrategyNotFound("youtube-v1".to_string());
        assert_eq!(err.to_string(), "Strategy not found: youtube-v1");
    }

    #[test]
    fn test_process_error_display() {
        let err = IsolateError::Process("failed to start winws".to_string());
        assert_eq!(err.to_string(), "Process error: failed to start winws");
    }

    #[test]
    fn test_driver_not_loaded_display() {
        let err = IsolateError::DriverNotLoaded;
        assert_eq!(err.to_string(), "WinDivert driver not loaded");
    }

    #[test]
    fn test_strategy_timeout_display() {
        let err = IsolateError::StrategyTimeout(5000);
        assert_eq!(err.to_string(), "Strategy timeout after 5000ms");
    }

    #[test]
    fn test_network_error_display() {
        let err = IsolateError::Network("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");
    }

    #[test]
    fn test_test_failed_display() {
        let err = IsolateError::TestFailed("youtube.com unreachable".to_string());
        assert_eq!(err.to_string(), "Test failed: youtube.com unreachable");
    }

    #[test]
    fn test_storage_error_display() {
        let err = IsolateError::Storage("disk full".to_string());
        assert_eq!(err.to_string(), "Storage error: disk full");
    }

    #[test]
    fn test_validation_error_display() {
        let err = IsolateError::Validation("invalid port number".to_string());
        assert_eq!(err.to_string(), "Validation error: invalid port number");
    }

    #[test]
    fn test_requires_admin_display() {
        let err = IsolateError::RequiresAdmin;
        assert_eq!(err.to_string(), "Requires administrator privileges");
    }

    #[test]
    fn test_cancelled_display() {
        let err = IsolateError::Cancelled;
        assert_eq!(err.to_string(), "Optimization cancelled");
    }

    #[test]
    fn test_no_strategy_found_display() {
        let err = IsolateError::NoStrategyFound;
        assert_eq!(err.to_string(), "No working strategy found");
    }

    #[test]
    fn test_system_proxy_error_display() {
        let err = IsolateError::SystemProxy("failed to set proxy".to_string());
        assert_eq!(err.to_string(), "System proxy error: failed to set proxy");
    }

    #[test]
    fn test_tauri_error_display() {
        let err = IsolateError::Tauri("window not found".to_string());
        assert_eq!(err.to_string(), "Tauri error: window not found");
    }

    #[test]
    fn test_other_error_display() {
        let err = IsolateError::Other("unexpected error".to_string());
        assert_eq!(err.to_string(), "unexpected error");
    }

    // ==================== From trait tests ====================

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: IsolateError = io_err.into();
        assert!(matches!(err, IsolateError::Io(_)));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_from_serde_yaml_error() {
        let yaml_result: std::result::Result<String, serde_yaml::Error> =
            serde_yaml::from_str("invalid: yaml: content: [");
        let yaml_err = yaml_result.unwrap_err();
        let err: IsolateError = yaml_err.into();
        assert!(matches!(err, IsolateError::Yaml(_)));
        assert!(err.to_string().starts_with("YAML parse error:"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_result: std::result::Result<String, serde_json::Error> =
            serde_json::from_str("{ invalid json }");
        let json_err = json_result.unwrap_err();
        let err: IsolateError = json_err.into();
        assert!(matches!(err, IsolateError::Json(_)));
        assert!(err.to_string().starts_with("JSON error:"));
    }

    #[test]
    fn test_from_string() {
        let err: IsolateError = "some error".into();
        assert!(matches!(err, IsolateError::Other(_)));
        assert_eq!(err.to_string(), "some error");
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("anyhow error message");
        let err: IsolateError = anyhow_err.into();
        assert!(matches!(err, IsolateError::Other(_)));
        assert!(err.to_string().contains("anyhow error message"));
    }

    // ==================== Serialize tests ====================

    #[test]
    fn test_serialize_config_error() {
        let err = IsolateError::Config("test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"config\""));
        assert!(json.contains("\"message\":\"test\""));
    }

    #[test]
    fn test_serialize_unit_variant() {
        let err = IsolateError::RequiresAdmin;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"requires_admin\""));
    }

    // ==================== Result alias tests ====================

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(IsolateError::Cancelled);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IsolateError::Cancelled));
    }

    // ==================== Helper constructor tests ====================

    #[test]
    fn test_helper_constructors() {
        assert!(matches!(IsolateError::config("test"), IsolateError::Config(_)));
        assert!(matches!(IsolateError::strategy("test"), IsolateError::Strategy(_)));
        assert!(matches!(IsolateError::process("test"), IsolateError::Process(_)));
        assert!(matches!(IsolateError::network("test"), IsolateError::Network(_)));
        assert!(matches!(IsolateError::validation("test"), IsolateError::Validation(_)));
        assert!(matches!(IsolateError::tauri("test"), IsolateError::Tauri(_)));
        assert!(matches!(IsolateError::other("test"), IsolateError::Other(_)));
    }

    #[test]
    fn test_kind_method() {
        assert_eq!(IsolateError::Config("".into()).kind(), "config");
        assert_eq!(IsolateError::Strategy("".into()).kind(), "strategy");
        assert_eq!(IsolateError::RequiresAdmin.kind(), "requires_admin");
        assert_eq!(IsolateError::Cancelled.kind(), "cancelled");
    }

    // ==================== Error variant creation tests ====================

    #[test]
    fn test_all_string_variants_creation() {
        let errors = vec![
            IsolateError::Config("test".into()),
            IsolateError::Strategy("test".into()),
            IsolateError::StrategyNotFound("test".into()),
            IsolateError::Process("test".into()),
            IsolateError::Network("test".into()),
            IsolateError::Http("test".into()),
            IsolateError::Io("test".into()),
            IsolateError::Storage("test".into()),
            IsolateError::Yaml("test".into()),
            IsolateError::Json("test".into()),
            IsolateError::Database("test".into()),
            IsolateError::TestFailed("test".into()),
            IsolateError::Validation("test".into()),
            IsolateError::SystemProxy("test".into()),
            IsolateError::Tauri("test".into()),
            IsolateError::Other("test".into()),
        ];

        for err in errors {
            let _ = err.to_string();
            let _ = serde_json::to_string(&err).unwrap();
        }
    }

    #[test]
    fn test_all_unit_variants_creation() {
        let errors = vec![
            IsolateError::DriverNotLoaded,
            IsolateError::RequiresAdmin,
            IsolateError::Cancelled,
            IsolateError::NoStrategyFound,
        ];

        for err in errors {
            let _ = err.to_string();
            let _ = serde_json::to_string(&err).unwrap();
        }
    }

    #[test]
    fn test_timeout_variant_with_different_values() {
        let err1 = IsolateError::StrategyTimeout(0);
        assert_eq!(err1.to_string(), "Strategy timeout after 0ms");

        let err2 = IsolateError::StrategyTimeout(1000);
        assert_eq!(err2.to_string(), "Strategy timeout after 1000ms");

        let err3 = IsolateError::StrategyTimeout(u32::MAX);
        assert_eq!(
            err3.to_string(),
            format!("Strategy timeout after {}ms", u32::MAX)
        );
    }

    // ==================== Debug trait tests ====================

    #[test]
    fn test_debug_trait() {
        let err = IsolateError::Config("test error".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("test error"));
    }

    // ==================== Error propagation tests ====================

    #[test]
    fn test_question_mark_operator_with_io_error() {
        fn inner() -> Result<()> {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
            Err(io_err)?
        }

        let result = inner();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IsolateError::Io(_)));
    }

    #[test]
    fn test_question_mark_operator_with_json_error() {
        fn inner() -> Result<String> {
            let value: String = serde_json::from_str("not valid json")?;
            Ok(value)
        }

        let result = inner();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IsolateError::Json(_)));
    }

    #[test]
    fn test_question_mark_operator_with_yaml_error() {
        fn inner() -> Result<String> {
            let value: String = serde_yaml::from_str("invalid: yaml: [")?;
            Ok(value)
        }

        let result = inner();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IsolateError::Yaml(_)));
    }
}

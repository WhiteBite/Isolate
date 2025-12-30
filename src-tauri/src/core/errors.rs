//! Error types for Isolate

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IsolateError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Strategy not found: {0}")]
    StrategyNotFound(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("WinDivert driver not loaded")]
    DriverNotLoaded,

    #[error("Strategy timeout after {0}ms")]
    StrategyTimeout(u32),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Test failed: {0}")]
    TestFailed(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Requires administrator privileges")]
    RequiresAdmin,

    #[error("Optimization cancelled")]
    Cancelled,

    #[error("No working strategy found")]
    NoStrategyFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("System proxy error: {0}")]
    SystemProxy(String),
}

pub type Result<T> = std::result::Result<T, IsolateError>;

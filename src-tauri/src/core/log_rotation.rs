//! Log rotation for orchestra/optimization logs
//!
//! Keeps only the last N log entries to prevent unbounded growth.
//!
//! Note: This module provides public API for log rotation that may be used
//! by external consumers or future features.

#![allow(dead_code)] // Public API for future use

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Default maximum log entries
pub const DEFAULT_MAX_LOGS: usize = 1000;

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Rotating log buffer
pub struct RotatingLog {
    entries: RwLock<VecDeque<LogEntry>>,
    max_entries: usize,
}

impl RotatingLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(max_entries)),
            max_entries,
        }
    }
    
    /// Add a log entry, removing oldest if at capacity
    pub async fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.write().await;
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }
        entries.push_back(entry);
    }
    
    /// Add a simple message
    pub async fn log(&self, level: LogLevel, source: &str, message: &str) {
        self.push(LogEntry {
            timestamp: Utc::now(),
            level,
            source: source.to_string(),
            message: message.to_string(),
        }).await;
    }
    
    /// Log debug message
    pub async fn debug(&self, source: &str, message: &str) {
        self.log(LogLevel::Debug, source, message).await;
    }
    
    /// Log info message
    pub async fn info(&self, source: &str, message: &str) {
        self.log(LogLevel::Info, source, message).await;
    }
    
    /// Log warning message
    pub async fn warn(&self, source: &str, message: &str) {
        self.log(LogLevel::Warn, source, message).await;
    }
    
    /// Log error message
    pub async fn error(&self, source: &str, message: &str) {
        self.log(LogLevel::Error, source, message).await;
    }
    
    /// Get all entries
    pub async fn get_all(&self) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries.iter().cloned().collect()
    }
    
    /// Get last N entries
    pub async fn get_last(&self, n: usize) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries.iter().rev().take(n).rev().cloned().collect()
    }
    
    /// Get entries by level
    pub async fn get_by_level(&self, level: LogLevel) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries.iter().filter(|e| e.level == level).cloned().collect()
    }
    
    /// Get entries by source
    pub async fn get_by_source(&self, source: &str) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries.iter().filter(|e| e.source == source).cloned().collect()
    }
    
    /// Get entries since a specific timestamp
    pub async fn get_since(&self, since: DateTime<Utc>) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries.iter().filter(|e| e.timestamp >= since).cloned().collect()
    }
    
    /// Clear all entries
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }
    
    /// Get current count
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }
    
    /// Check if empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    /// Get max entries capacity
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
}

impl Default for RotatingLog {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_LOGS)
    }
}

pub type SharedRotatingLog = Arc<RotatingLog>;

/// Creates a shared rotating log with specified capacity
pub fn create_rotating_log(max_entries: usize) -> SharedRotatingLog {
    Arc::new(RotatingLog::new(max_entries))
}

/// Creates a shared rotating log with default capacity
pub fn create_default_rotating_log() -> SharedRotatingLog {
    Arc::new(RotatingLog::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rotation() {
        let log = RotatingLog::new(3);
        
        log.log(LogLevel::Info, "test", "msg1").await;
        log.log(LogLevel::Info, "test", "msg2").await;
        log.log(LogLevel::Info, "test", "msg3").await;
        log.log(LogLevel::Info, "test", "msg4").await;
        
        let entries = log.get_all().await;
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].message, "msg2"); // msg1 was rotated out
        assert_eq!(entries[2].message, "msg4");
    }
    
    #[tokio::test]
    async fn test_get_last() {
        let log = RotatingLog::new(10);
        
        for i in 0..5 {
            log.log(LogLevel::Info, "test", &format!("msg{}", i)).await;
        }
        
        let last2 = log.get_last(2).await;
        assert_eq!(last2.len(), 2);
        assert_eq!(last2[0].message, "msg3");
        assert_eq!(last2[1].message, "msg4");
    }
    
    #[tokio::test]
    async fn test_filter_by_level() {
        let log = RotatingLog::new(10);
        
        log.log(LogLevel::Info, "test", "info1").await;
        log.log(LogLevel::Error, "test", "error1").await;
        log.log(LogLevel::Info, "test", "info2").await;
        
        let errors = log.get_by_level(LogLevel::Error).await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "error1");
    }
    
    #[tokio::test]
    async fn test_filter_by_source() {
        let log = RotatingLog::new(10);
        
        log.log(LogLevel::Info, "optimizer", "opt1").await;
        log.log(LogLevel::Info, "monitor", "mon1").await;
        log.log(LogLevel::Info, "optimizer", "opt2").await;
        
        let opt_logs = log.get_by_source("optimizer").await;
        assert_eq!(opt_logs.len(), 2);
    }
    
    #[tokio::test]
    async fn test_clear() {
        let log = RotatingLog::new(10);
        
        log.log(LogLevel::Info, "test", "msg").await;
        assert!(!log.is_empty().await);
        
        log.clear().await;
        assert!(log.is_empty().await);
    }
    
    #[tokio::test]
    async fn test_convenience_methods() {
        let log = RotatingLog::new(10);
        
        log.debug("src", "debug msg").await;
        log.info("src", "info msg").await;
        log.warn("src", "warn msg").await;
        log.error("src", "error msg").await;
        
        let entries = log.get_all().await;
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].level, LogLevel::Debug);
        assert_eq!(entries[1].level, LogLevel::Info);
        assert_eq!(entries[2].level, LogLevel::Warn);
        assert_eq!(entries[3].level, LogLevel::Error);
    }
    
    #[tokio::test]
    async fn test_default_capacity() {
        let log = RotatingLog::default();
        assert_eq!(log.max_entries(), DEFAULT_MAX_LOGS);
    }
    
    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Warn), "WARN");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }
}

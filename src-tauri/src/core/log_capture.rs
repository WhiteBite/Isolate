//! Log capture system for Isolate
//!
//! Captures tracing logs and stores them in memory for frontend display.
//! Supports filtering by level, module, and search text.
//!
//! NOTE: LogCaptureLayer is prepared for future log streaming to frontend.

// Public API for log capture
#![allow(dead_code)]

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing_subscriber::Layer;

use crate::core::models::LogEntry;

/// Maximum number of log entries to keep in memory
const MAX_LOG_ENTRIES: usize = 1000;

/// Global log buffer
static LOG_BUFFER: once_cell::sync::Lazy<Arc<RwLock<VecDeque<LogEntry>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))));

/// Get the global log buffer
pub fn get_log_buffer() -> Arc<RwLock<VecDeque<LogEntry>>> {
    LOG_BUFFER.clone()
}

/// Add a log entry to the buffer
pub fn add_log_entry(entry: LogEntry) {
    let mut buffer = LOG_BUFFER.write();
    if buffer.len() >= MAX_LOG_ENTRIES {
        buffer.pop_front();
    }
    buffer.push_back(entry);
}

/// Get all log entries
pub fn get_all_logs() -> Vec<LogEntry> {
    let buffer = LOG_BUFFER.read();
    buffer.iter().cloned().collect()
}

/// Get filtered log entries
pub fn get_filtered_logs(
    level: Option<&str>,
    module: Option<&str>,
    search: Option<&str>,
) -> Vec<LogEntry> {
    let buffer = LOG_BUFFER.read();
    
    buffer
        .iter()
        .filter(|entry| {
            // Filter by level
            if let Some(lvl) = level {
                let entry_level = entry.level.to_lowercase();
                let filter_level = lvl.to_lowercase();
                
                // Level hierarchy: error > warn > info > debug > trace
                let passes_level = match filter_level.as_str() {
                    "error" => entry_level == "error",
                    "warn" => entry_level == "error" || entry_level == "warn",
                    "info" => entry_level == "error" || entry_level == "warn" || entry_level == "info",
                    "debug" => entry_level != "trace",
                    _ => true,
                };
                
                if !passes_level {
                    return false;
                }
            }
            
            // Filter by module
            if let Some(mod_filter) = module {
                if !entry.module.to_lowercase().contains(&mod_filter.to_lowercase()) {
                    return false;
                }
            }
            
            // Filter by search text
            if let Some(search_text) = search {
                let search_lower = search_text.to_lowercase();
                if !entry.message.to_lowercase().contains(&search_lower)
                    && !entry.module.to_lowercase().contains(&search_lower)
                {
                    return false;
                }
            }
            
            true
        })
        .cloned()
        .collect()
}

/// Clear all log entries
pub fn clear_logs() {
    let mut buffer = LOG_BUFFER.write();
    buffer.clear();
}

/// Export logs to string
pub fn export_logs_to_string() -> String {
    let buffer = LOG_BUFFER.read();
    
    buffer
        .iter()
        .map(|entry| {
            format!(
                "[{}] {} [{}] {}",
                entry.timestamp, entry.level, entry.module, entry.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Custom tracing layer that captures logs to the buffer
pub struct LogCaptureLayer {
    app_handle: Option<tauri::AppHandle>,
}

impl LogCaptureLayer {
    pub fn new() -> Self {
        Self { app_handle: None }
    }
    
    pub fn with_app_handle(app_handle: tauri::AppHandle) -> Self {
        Self {
            app_handle: Some(app_handle),
        }
    }
}

impl<S> Layer<S> for LogCaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Extract event data
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let module = metadata.module_path().unwrap_or("unknown").to_string();
        
        // Extract message from event fields
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.message;
        
        // Create timestamp
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        
        let entry = LogEntry {
            timestamp,
            level: level.clone(),
            module: module.clone(),
            message: message.clone(),
        };
        
        // Add to buffer
        add_log_entry(entry.clone());
        
        // Emit to frontend if app handle is available
        if let Some(ref app_handle) = self.app_handle {
            use tauri::Emitter;
            let _ = app_handle.emit("log:entry", &entry);
        }
    }
}

/// Visitor to extract message from tracing event
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove surrounding quotes if present
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        } else if self.message.is_empty() {
            // Fallback: use first field as message
            self.message = format!("{}: {:?}", field.name(), value);
        }
    }
    
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else if self.message.is_empty() {
            self.message = format!("{}: {}", field.name(), value);
        }
    }
}

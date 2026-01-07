//! Log management commands

use tracing::info;

use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::models::LogEntry;

/// Log filter parameters
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LogFilter {
    pub level: Option<String>,
    pub module: Option<String>,
    pub search: Option<String>,
}

/// Get recent logs with optional filtering
#[tauri::command]
pub async fn get_logs(filter: Option<LogFilter>) -> Result<Vec<LogEntry>, String> {
    info!("Getting logs with filter: {:?}", filter);
    
    let logs = match filter {
        Some(f) => crate::core::log_capture::get_filtered_logs(
            f.level.as_deref(),
            f.module.as_deref(),
            f.search.as_deref(),
        ),
        None => crate::core::log_capture::get_all_logs(),
    };
    
    Ok(logs)
}

/// Clear all logs
#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    info!("Clearing logs");
    crate::core::log_capture::clear_logs();
    Ok(())
}

/// Export logs to file
#[tauri::command]
pub async fn export_logs() -> Result<String, IsolateError> {
    info!("Exporting logs");
    
    let logs_content = crate::core::log_capture::export_logs_to_string();
    
    // Get logs directory
    let logs_dir = crate::core::paths::get_logs_dir();
    tokio::fs::create_dir_all(&logs_dir)
        .await
        .io_context("Failed to create logs directory")?;
    
    // Create filename with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("isolate_logs_{}.txt", timestamp);
    let filepath = logs_dir.join(&filename);
    
    // Write logs
    tokio::fs::write(&filepath, logs_content)
        .await
        .io_context("Failed to write logs")?;
    
    info!(path = %filepath.display(), "Logs exported");
    Ok(filepath.display().to_string())
}

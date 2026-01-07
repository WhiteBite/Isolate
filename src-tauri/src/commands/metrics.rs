//! Strategy Metrics Tauri commands

use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::errors::IsolateError;
use crate::core::strategy_metrics::{
    get_current_metrics, get_metrics_history, get_snapshots_history, get_aggregated_stats,
    reset_metrics, clear_metrics_history, export_metrics_csv,
    take_snapshot, MetricsHistoryEntry, MetricsSnapshot, StrategyMetrics,
    AggregatedStats, HistoryPeriod,
};
use crate::state::AppState;

#[tauri::command]
pub async fn get_strategy_metrics(
    _state: State<'_, Arc<AppState>>,
) -> Result<Option<StrategyMetrics>, IsolateError> {
    info!("Getting current strategy metrics");
    Ok(get_current_metrics().await)
}

#[tauri::command]
pub async fn get_strategy_metrics_history(
    _state: State<'_, Arc<AppState>>,
    hours: Option<u32>,
) -> Result<Vec<MetricsHistoryEntry>, IsolateError> {
    let hours = hours.unwrap_or(24);
    info!(hours, "Getting strategy metrics history");
    Ok(get_metrics_history(hours).await)
}

#[tauri::command]
pub async fn get_metrics_snapshots(
    _state: State<'_, Arc<AppState>>,
    period: String,
) -> Result<Vec<MetricsSnapshot>, IsolateError> {
    let history_period = parse_period(&period)?;
    info!(period = %period, "Getting metrics snapshots");
    Ok(get_snapshots_history(history_period).await)
}

#[tauri::command]
pub async fn get_aggregated_metrics(
    _state: State<'_, Arc<AppState>>,
    period: String,
) -> Result<Option<AggregatedStats>, IsolateError> {
    let history_period = parse_period(&period)?;
    info!(period = %period, "Getting aggregated metrics");
    Ok(get_aggregated_stats(history_period).await)
}

#[tauri::command]
pub async fn take_metrics_snapshot(
    _state: State<'_, Arc<AppState>>,
) -> Result<Option<MetricsSnapshot>, IsolateError> {
    info!("Taking manual metrics snapshot");
    Ok(take_snapshot().await)
}

#[tauri::command]
pub async fn reset_strategy_metrics(
    _state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Resetting current strategy metrics");
    reset_metrics().await;
    Ok(())
}

#[tauri::command]
pub async fn clear_strategy_metrics_history(
    _state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Clearing metrics history");
    clear_metrics_history().await;
    Ok(())
}

#[tauri::command]
pub async fn export_metrics_csv_string(
    _state: State<'_, Arc<AppState>>,
    period: String,
) -> Result<String, IsolateError> {
    let history_period = parse_period(&period)?;
    info!(period = %period, "Exporting metrics to CSV string");
    export_metrics_csv(history_period)
        .await
        .map_err(|e| IsolateError::Io(e.to_string()))
}

#[tauri::command]
pub async fn export_metrics_to_csv_file(
    _state: State<'_, Arc<AppState>>,
    path: String,
    period: String,
) -> Result<(), IsolateError> {
    let history_period = parse_period(&period)?;
    let file_path = PathBuf::from(&path);
    info!(path = %path, period = %period, "Exporting metrics to CSV file");
    let csv_content = export_metrics_csv(history_period)
        .await
        .map_err(|e| IsolateError::Io(e.to_string()))?;
    tokio::fs::write(file_path, csv_content)
        .await
        .map_err(|e| IsolateError::Io(e.to_string()))
}

fn parse_period(period: &str) -> Result<HistoryPeriod, IsolateError> {
    match period.to_lowercase().as_str() {
        "1h" | "hour" => Ok(HistoryPeriod::Hour1),
        "24h" | "day" => Ok(HistoryPeriod::Hours24),
        "7d" | "week" => Ok(HistoryPeriod::Days7),
        _ => period
            .trim_end_matches('h')
            .parse::<u32>()
            .map(HistoryPeriod::Custom)
            .map_err(|_| {
                IsolateError::Validation(format!(
                    "Invalid period: {}. Use 1h, 24h, 7d, or number of hours",
                    period
                ))
            }),
    }
}

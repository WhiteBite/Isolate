//! Strategy Prewarming Tauri Commands
//!
//! Команды для предзапуска стратегий в фоне.

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::core::errors::IsolateError;
use crate::core::strategy_prewarming::{get_prewarming_manager, PrewarmedStrategyInfo};
use crate::state::AppState;

/// Предзапускает стратегию для быстрого переключения
///
/// # Arguments
/// * `strategy_id` - ID стратегии для prewarming
///
/// # Returns
/// Информация о prewarmed стратегии
#[tauri::command]
pub async fn prewarm_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<PrewarmedStrategyInfo, IsolateError> {
    info!(strategy_id = %strategy_id, "Prewarming strategy via command");

    // Загружаем стратегию
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await?;

    // Получаем менеджер и выполняем prewarm
    let manager = get_prewarming_manager();
    manager.prewarm_strategy(strategy).await
}

/// Получает список всех prewarmed стратегий
///
/// # Returns
/// Список информации о prewarmed стратегиях
#[tauri::command]
pub async fn get_prewarmed_strategies() -> Result<Vec<PrewarmedStrategyInfo>, IsolateError> {
    let manager = get_prewarming_manager();
    Ok(manager.get_all_prewarmed().await)
}

/// Очищает все prewarmed стратегии
#[tauri::command]
pub async fn clear_prewarmed() -> Result<(), IsolateError> {
    info!("Clearing all prewarmed strategies via command");
    let manager = get_prewarming_manager();
    manager.clear_all().await;
    Ok(())
}

/// Проверяет, prewarmed ли стратегия
///
/// # Arguments
/// * `strategy_id` - ID стратегии
///
/// # Returns
/// true если стратегия prewarmed и готова
#[tauri::command]
pub async fn is_strategy_prewarmed(strategy_id: String) -> Result<bool, IsolateError> {
    let manager = get_prewarming_manager();
    Ok(manager.is_prewarmed(&strategy_id).await)
}

/// Удаляет конкретную prewarmed стратегию
///
/// # Arguments
/// * `strategy_id` - ID стратегии для удаления
///
/// # Returns
/// true если стратегия была удалена
#[tauri::command]
pub async fn remove_prewarmed(strategy_id: String) -> Result<bool, IsolateError> {
    info!(strategy_id = %strategy_id, "Removing prewarmed strategy via command");
    let manager = get_prewarming_manager();
    Ok(manager.remove(&strategy_id).await)
}

/// Очищает истёкшие prewarmed стратегии
///
/// # Returns
/// Количество удалённых стратегий
#[tauri::command]
pub async fn cleanup_prewarmed() -> Result<usize, IsolateError> {
    let manager = get_prewarming_manager();
    Ok(manager.cleanup_expired().await)
}

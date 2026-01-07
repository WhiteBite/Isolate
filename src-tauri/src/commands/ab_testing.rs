//! A/B Testing Tauri Commands
//!
//! Команды для запуска и управления A/B тестами стратегий.

use std::sync::Arc;
use tauri::{State, Window};
use tracing::{info, warn};

use crate::core::ab_testing::{
    ABTest, ABTestManager, ABTestProgress, ABTestResult, ABTestSummary,
    create_ab_test_manager_with_storage,
};
use crate::core::errors::IsolateError;
use crate::state::AppState;
use super::rate_limiter;

/// Глобальный менеджер A/B тестов (lazy init)
static AB_TEST_MANAGER: tokio::sync::OnceCell<Arc<ABTestManager>> = tokio::sync::OnceCell::const_new();

/// Получает или создаёт менеджер A/B тестов
async fn get_ab_manager(state: &Arc<AppState>) -> Arc<ABTestManager> {
    AB_TEST_MANAGER
        .get_or_init(|| async {
            let manager = create_ab_test_manager_with_storage(
                state.strategy_engine.clone(),
                state.storage.clone(),
            );
            // Загружаем сохранённые результаты
            if let Err(e) = manager.load_saved_results().await {
                warn!(error = %e, "Failed to load saved A/B test results");
            }
            manager
        })
        .await
        .clone()
}

/// Запускает A/B тест двух стратегий
///
/// # Arguments
/// * `strategy_a_id` - ID первой стратегии
/// * `strategy_b_id` - ID второй стратегии
/// * `service_id` - ID сервиса для тестирования
/// * `iterations` - Количество итераций для каждой стратегии
///
/// # Returns
/// ID созданного теста
#[tauri::command]
pub async fn start_ab_test(
    window: Window,
    state: State<'_, Arc<AppState>>,
    strategy_a_id: String,
    strategy_b_id: String,
    service_id: String,
    iterations: u32,
) -> Result<String, IsolateError> {
    use tauri::Emitter;
    
    // Rate limit: max once per 10 seconds
    rate_limiter::check_rate_limit("start_ab_test", 10)?;

    info!(
        strategy_a = %strategy_a_id,
        strategy_b = %strategy_b_id,
        service = %service_id,
        iterations,
        "Starting A/B test"
    );

    // Загружаем стратегии
    let strategy_a = state
        .config_manager
        .load_strategy_by_id(&strategy_a_id)
        .await?;
    let strategy_b = state
        .config_manager
        .load_strategy_by_id(&strategy_b_id)
        .await?;

    // Загружаем сервис
    let services = state.config_manager.load_services().await?;
    let service = services
        .get(&service_id)
        .ok_or_else(|| IsolateError::Config(format!("Service '{}' not found", service_id)))?
        .clone();

    // Валидация
    if iterations == 0 || iterations > 20 {
        return Err(IsolateError::Config(
            "Iterations must be between 1 and 20".to_string(),
        ));
    }

    if strategy_a_id == strategy_b_id {
        return Err(IsolateError::Config(
            "Cannot compare strategy with itself".to_string(),
        ));
    }

    // Получаем менеджер
    let manager = get_ab_manager(&state).await;

    // Запускаем тест
    let test_id = manager
        .start_test(&strategy_a, &strategy_b, &service, iterations)
        .await?;

    // Запускаем выполнение теста в фоне
    let manager_clone = manager.clone();
    let test_id_clone = test_id.clone();
    let window_clone = window.clone();

    tokio::spawn(async move {
        // Отправляем начальный прогресс
        let _ = window_clone.emit("ab_test:started", &test_id_clone);

        // Запускаем тест
        match manager_clone
            .run_test(&test_id_clone, &strategy_a, &strategy_b, &service, iterations)
            .await
        {
            Ok(result) => {
                info!(test_id = %test_id_clone, "A/B test completed successfully");
                let _ = window_clone.emit("ab_test:completed", &result);
            }
            Err(e) => {
                warn!(test_id = %test_id_clone, error = %e, "A/B test failed");
                let _ = window_clone.emit("ab_test:error", &e.to_string());
            }
        }
    });

    Ok(test_id)
}

/// Получает статус A/B теста
#[tauri::command]
pub async fn get_ab_test_status(
    state: State<'_, Arc<AppState>>,
    test_id: String,
) -> Result<Option<ABTest>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.get_test_status(&test_id).await)
}

/// Получает прогресс A/B теста
#[tauri::command]
pub async fn get_ab_test_progress(
    state: State<'_, Arc<AppState>>,
    test_id: String,
) -> Result<Option<ABTestProgress>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.get_test_progress(&test_id).await)
}

/// Получает результаты A/B теста
#[tauri::command]
pub async fn get_ab_test_results(
    state: State<'_, Arc<AppState>>,
    test_id: String,
) -> Result<Option<ABTestResult>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.get_test_results(&test_id).await)
}

/// Отменяет A/B тест
#[tauri::command]
pub async fn cancel_ab_test(
    state: State<'_, Arc<AppState>>,
    test_id: String,
) -> Result<(), IsolateError> {
    info!(test_id = %test_id, "Cancelling A/B test");
    let manager = get_ab_manager(&state).await;
    manager.cancel_test(&test_id).await
}

/// Получает список активных A/B тестов
#[tauri::command]
pub async fn get_active_ab_tests(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ABTest>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.get_active_tests().await)
}

/// Получает список всех A/B тестов (активных и завершённых)
#[tauri::command]
pub async fn list_ab_tests(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ABTestSummary>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.list_all_tests().await)
}

/// Удаляет A/B тест по ID
#[tauri::command]
pub async fn delete_ab_test(
    state: State<'_, Arc<AppState>>,
    test_id: String,
) -> Result<bool, IsolateError> {
    info!(test_id = %test_id, "Deleting A/B test");
    let manager = get_ab_manager(&state).await;
    manager.delete_test(&test_id).await
}

/// Получает все результаты A/B тестов
#[tauri::command]
pub async fn get_all_ab_test_results(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ABTestResult>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.get_all_results().await)
}

/// Сравнивает две стратегии на основе исторических данных
#[tauri::command]
pub async fn compare_strategies(
    state: State<'_, Arc<AppState>>,
    strategy_a_id: String,
    strategy_b_id: String,
) -> Result<Option<ABTestResult>, IsolateError> {
    let manager = get_ab_manager(&state).await;
    Ok(manager.compare_strategies_from_history(&strategy_a_id, &strategy_b_id).await)
}

/// Очищает все результаты A/B тестов
#[tauri::command]
pub async fn clear_ab_test_results(
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Clearing all A/B test results");
    let manager = get_ab_manager(&state).await;
    manager.clear_all_results().await
}

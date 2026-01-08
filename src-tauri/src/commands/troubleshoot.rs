//! Troubleshooter commands - диагностика и автоматический подбор стратегий

use std::sync::Arc;
use tauri::{State, Window, Emitter};
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

use crate::core::errors::IsolateError;
use crate::state::AppState;

// ============================================================================
// Types
// ============================================================================

/// Проблема сервиса для диагностики
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProblem {
    pub service_id: String,
    pub service_name: String,
    pub category: String, // video, social, gaming, other
}

/// Прогресс тестирования стратегии
#[derive(Debug, Clone, Serialize)]
pub struct TroubleshootProgress {
    pub strategy_id: String,
    pub strategy_name: String,
    pub status: String, // waiting, testing, success, failed
    pub progress: u8,   // 0-100
    pub latency_ms: Option<u32>,
}

/// Результат тестирования одной стратегии
#[derive(Debug, Clone, Serialize)]
pub struct TroubleshootStrategyResult {
    pub strategy_id: String,
    pub strategy_name: String,
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub error: Option<String>,
}

/// Финальный результат troubleshoot
#[derive(Debug, Clone, Serialize)]
pub struct TroubleshootResult {
    pub service_id: String,
    pub strategies_tested: Vec<TroubleshootStrategyResult>,
    pub best_strategy_id: Option<String>,
    pub best_strategy_name: Option<String>,
    pub best_latency_ms: Option<u32>,
}

// ============================================================================
// Commands
// ============================================================================

/// Запустить диагностику для сервиса
/// 
/// Тестирует все подходящие стратегии последовательно и находит лучшую.
/// Эмитит события:
/// - troubleshoot:progress - прогресс каждой стратегии
/// - troubleshoot:strategy_result - результат каждой стратегии
/// - troubleshoot:complete - финальный результат
#[tauri::command]
pub async fn troubleshoot_service(
    window: Window,
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<TroubleshootResult, IsolateError> {
    info!(service_id = %service_id, "Starting troubleshoot for service");
    
    // Загружаем сервис
    let services = state.config_manager.load_services().await?;
    let service = services.get(&service_id)
        .ok_or_else(|| IsolateError::Config(format!("Service not found: {}", service_id)))?;
    
    // Загружаем все стратегии (HashMap<String, Strategy>)
    let strategies_map = state.config_manager.load_strategies().await?;
    
    // Фильтруем стратегии, поддерживающие глобальный режим
    let testable_strategies: Vec<_> = strategies_map
        .values()
        .filter(|s| s.mode_capabilities.supports_global)
        .take(6) // Максимум 6 стратегий для теста
        .collect();
    
    let total = testable_strategies.len();
    let mut results: Vec<TroubleshootStrategyResult> = Vec::new();
    let mut best_strategy: Option<(String, String, u32)> = None; // (id, name, latency)
    
    for (index, strategy) in testable_strategies.iter().enumerate() {
        // Эмитим прогресс - начало тестирования
        let _ = window.emit("troubleshoot:progress", TroubleshootProgress {
            strategy_id: strategy.id.clone(),
            strategy_name: strategy.name.clone(),
            status: "testing".to_string(),
            progress: 0,
            latency_ms: None,
        });
        
        // Запускаем стратегию
        let start_result = state.strategy_engine.start_global(strategy).await;
        
        if let Err(e) = start_result {
            warn!(strategy_id = %strategy.id, error = %e, "Failed to start strategy");
            
            let result = TroubleshootStrategyResult {
                strategy_id: strategy.id.clone(),
                strategy_name: strategy.name.clone(),
                success: false,
                latency_ms: None,
                error: Some(format!("Failed to start: {}", e)),
            };
            
            let _ = window.emit("troubleshoot:strategy_result", &result);
            results.push(result);
            continue;
        }
        
        // Ждём применения стратегии
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Эмитим прогресс - 50%
        let _ = window.emit("troubleshoot:progress", TroubleshootProgress {
            strategy_id: strategy.id.clone(),
            strategy_name: strategy.name.clone(),
            status: "testing".to_string(),
            progress: 50,
            latency_ms: None,
        });
        
        // Тестируем сервис - используем get_test_url() из Service
        let test_url = service.get_test_url()
            .unwrap_or_else(|| format!("https://{}.com", service.id));
        
        let test_result = test_url_direct(&test_url, 5).await;
        
        // Останавливаем стратегию
        let _ = state.strategy_engine.stop_global().await;
        
        // Ждём перед следующей стратегией (предотвращение BSOD)
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        let (success, latency_ms, error) = match test_result {
            Ok(latency) => {
                // Обновляем лучшую стратегию
                if best_strategy.is_none() || latency < best_strategy.as_ref().unwrap().2 {
                    best_strategy = Some((strategy.id.clone(), strategy.name.clone(), latency));
                }
                (true, Some(latency), None)
            }
            Err(e) => (false, None, Some(e.to_string())),
        };
        
        let result = TroubleshootStrategyResult {
            strategy_id: strategy.id.clone(),
            strategy_name: strategy.name.clone(),
            success,
            latency_ms,
            error,
        };
        
        // Эмитим результат стратегии
        let _ = window.emit("troubleshoot:strategy_result", &result);
        let _ = window.emit("troubleshoot:progress", TroubleshootProgress {
            strategy_id: strategy.id.clone(),
            strategy_name: strategy.name.clone(),
            status: if success { "success" } else { "failed" }.to_string(),
            progress: 100,
            latency_ms,
        });
        
        results.push(result);
        
        info!(
            strategy_id = %strategy.id,
            success,
            latency_ms = ?latency_ms,
            progress = format!("{}/{}", index + 1, total),
            "Strategy test completed"
        );
    }
    
    let final_result = TroubleshootResult {
        service_id: service_id.clone(),
        strategies_tested: results,
        best_strategy_id: best_strategy.as_ref().map(|(id, _, _)| id.clone()),
        best_strategy_name: best_strategy.as_ref().map(|(_, name, _)| name.clone()),
        best_latency_ms: best_strategy.as_ref().map(|(_, _, latency)| *latency),
    };
    
    // Эмитим финальный результат
    let _ = window.emit("troubleshoot:complete", &final_result);
    
    info!(
        service_id = %service_id,
        best_strategy = ?final_result.best_strategy_name,
        "Troubleshoot completed"
    );
    
    Ok(final_result)
}

/// Ключ для хранения привязки service → strategy
const SERVICE_STRATEGY_BINDING_PREFIX: &str = "service_strategy_binding:";

/// Применить результат troubleshoot - установить лучшую стратегию для сервиса
#[tauri::command]
pub async fn apply_troubleshoot_result(
    state: State<'_, Arc<AppState>>,
    service_id: String,
    strategy_id: String,
) -> Result<(), IsolateError> {
    info!(
        service_id = %service_id,
        strategy_id = %strategy_id,
        "Applying troubleshoot result"
    );
    
    // Загружаем стратегию
    let strategy = state.config_manager.load_strategy_by_id(&strategy_id).await?;
    
    // Запускаем стратегию глобально
    state.strategy_engine.start_global(&strategy).await?;
    
    // Сохраняем привязку service → strategy в storage (legacy)
    let binding_key = format!("{}{}", SERVICE_STRATEGY_BINDING_PREFIX, service_id);
    state.storage.set_setting(&binding_key, &strategy_id).await
        .map_err(|e| IsolateError::Storage(format!("Failed to save service-strategy binding: {}", e)))?;
    
    // Интеграция с Library: создаём/обновляем правило для сервиса
    if let Err(e) = update_library_rule_for_service(&state, &service_id, &strategy_id).await {
        // Логируем ошибку, но не прерываем основной flow
        // Стратегия уже применена, привязка сохранена
        warn!(
            service_id = %service_id,
            strategy_id = %strategy_id,
            error = %e,
            "Failed to update Library rule (non-critical)"
        );
    }
    
    info!(
        service_id = %service_id,
        strategy_id = %strategy_id,
        "Service-strategy binding saved successfully"
    );
    
    Ok(())
}

/// Получить сохранённую привязку стратегии для сервиса
#[tauri::command]
pub async fn get_service_strategy_binding(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<Option<String>, IsolateError> {
    let binding_key = format!("{}{}", SERVICE_STRATEGY_BINDING_PREFIX, service_id);
    let strategy_id: Option<String> = state.storage.get_setting(&binding_key).await
        .map_err(|e| IsolateError::Storage(format!("Failed to get service-strategy binding: {}", e)))?;
    
    Ok(strategy_id)
}

/// Получить список проблем для выбора (сервисы с категориями)
#[tauri::command]
pub async fn get_troubleshoot_problems(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ServiceProblem>, IsolateError> {
    let services = state.config_manager.load_services().await?;
    
    let problems: Vec<ServiceProblem> = services
        .values()
        .map(|s| {
            let category = match s.id.as_str() {
                "youtube" | "twitch" | "vimeo" => "video",
                "discord" | "telegram" | "whatsapp" => "social", 
                "twitter" | "instagram" | "facebook" => "social",
                "steam" | "epicgames" | "gog" => "gaming",
                _ => "other",
            };
            
            ServiceProblem {
                service_id: s.id.clone(),
                service_name: s.name.clone(),
                category: category.to_string(),
            }
        })
        .collect();
    
    Ok(problems)
}

// ============================================================================
// Helpers
// ============================================================================

/// Обновляет правило в Library для сервиса после успешного troubleshoot
/// 
/// Создаёт новое правило если его нет, или обновляет существующее.
async fn update_library_rule_for_service(
    state: &Arc<AppState>,
    service_id: &str,
    strategy_id: &str,
) -> Result<(), IsolateError> {
    use crate::commands::library::{LibraryRule, AccessMethod};
    use uuid::Uuid;
    use chrono::Utc;
    
    info!(
        service_id = %service_id,
        strategy_id = %strategy_id,
        "Updating Library rule for service"
    );
    
    // Загружаем сервис для получения домена
    let services = state.config_manager.load_services().await?;
    let service = services.get(service_id)
        .ok_or_else(|| IsolateError::Config(format!("Service not found: {}", service_id)))?;
    
    // Извлекаем домен из test_url или используем service_id
    let pattern = if let Some(test_url) = service.get_test_url() {
        // Парсим URL и извлекаем домен
        if let Ok(url) = url::Url::parse(&test_url) {
            if let Some(host) = url.host_str() {
                format!("*.{}", host)
            } else {
                format!("*.{}.com", service_id)
            }
        } else {
            format!("*.{}.com", service_id)
        }
    } else {
        // Fallback: используем service_id
        format!("*.{}.com", service_id)
    };
    
    // Создаём новое правило
    let now = Utc::now().timestamp();
    let rule = LibraryRule {
        id: Uuid::new_v4().to_string(),
        service_id: Some(service_id.to_string()),
        pattern,
        method: AccessMethod::Zapret,
        is_enabled: true,
        strategy_id: Some(strategy_id.to_string()),
        priority: 100, // Высокий приоритет для troubleshoot результатов
        created_at: now,
        updated_at: now,
    };
    
    // TODO: Сохранить правило в БД
    // Пока просто логируем создание правила
    info!(
        rule_id = %rule.id,
        service_id = %service_id,
        strategy_id = %strategy_id,
        pattern = %rule.pattern,
        "Library rule created (mock - not persisted yet)"
    );
    
    Ok(())
}

/// Тестирует URL напрямую
async fn test_url_direct(url: &str, timeout_secs: u64) -> Result<u32, IsolateError> {
    let start = std::time::Instant::now();
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| IsolateError::Network(e.to_string()))?;
    
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| IsolateError::Network(e.to_string()))?;
    
    if !response.status().is_success() && response.status().as_u16() >= 400 {
        return Err(IsolateError::Network(format!("HTTP {}", response.status())));
    }
    
    Ok(start.elapsed().as_millis() as u32)
}

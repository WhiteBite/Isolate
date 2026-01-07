//! Strategy Prewarming Module
//!
//! Предзапуск стратегий в фоне для быстрого переключения.
//! Позволяет подготовить стратегию заранее, чтобы при активации
//! не тратить время на загрузку и валидацию.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::strategy::Strategy;

/// Время жизни prewarmed стратегии по умолчанию (5 минут)
const DEFAULT_TTL_SECS: u64 = 300;

/// Состояние prewarmed стратегии
#[derive(Debug, Clone)]
pub struct PrewarmedStrategy {
    /// Загруженная стратегия
    pub strategy: Strategy,
    /// Время создания записи
    pub created_at: Instant,
    /// Время жизни (TTL)
    pub ttl: Duration,
    /// Статус готовности
    pub ready: bool,
    /// Ошибка при подготовке (если есть)
    pub error: Option<String>,
}

impl PrewarmedStrategy {
    /// Создаёт новую prewarmed стратегию
    pub fn new(strategy: Strategy, ttl: Duration) -> Self {
        Self {
            strategy,
            created_at: Instant::now(),
            ttl,
            ready: true,
            error: None,
        }
    }

    /// Создаёт prewarmed стратегию с ошибкой
    pub fn with_error(strategy: Strategy, error: String, ttl: Duration) -> Self {
        Self {
            strategy,
            created_at: Instant::now(),
            ttl,
            ready: false,
            error: Some(error),
        }
    }

    /// Проверяет, истёк ли TTL
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Возвращает оставшееся время жизни
    pub fn remaining_ttl(&self) -> Duration {
        let elapsed = self.created_at.elapsed();
        if elapsed >= self.ttl {
            Duration::ZERO
        } else {
            self.ttl - elapsed
        }
    }
}

/// Информация о prewarmed стратегии для сериализации
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrewarmedStrategyInfo {
    /// ID стратегии
    pub strategy_id: String,
    /// Название стратегии
    pub strategy_name: String,
    /// Готова ли стратегия
    pub ready: bool,
    /// Ошибка (если есть)
    pub error: Option<String>,
    /// Оставшееся время жизни в секундах
    pub remaining_ttl_secs: u64,
}

/// Менеджер prewarming стратегий
pub struct PrewarmingManager {
    /// Хранилище prewarmed стратегий
    prewarmed: RwLock<HashMap<String, PrewarmedStrategy>>,
    /// TTL по умолчанию
    default_ttl: Duration,
}

impl PrewarmingManager {
    /// Создаёт новый менеджер с TTL по умолчанию
    pub fn new() -> Self {
        Self {
            prewarmed: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        }
    }

    /// Создаёт менеджер с кастомным TTL
    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            prewarmed: RwLock::new(HashMap::new()),
            default_ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Подготавливает стратегию для быстрого запуска
    ///
    /// Загружает и валидирует стратегию, сохраняя её в кэше.
    /// Если стратегия уже prewarmed и не истекла — возвращает её.
    pub async fn prewarm_strategy(&self, strategy: Strategy) -> Result<PrewarmedStrategyInfo> {
        let strategy_id = strategy.id.clone();
        let strategy_name = strategy.name.clone();

        // Проверяем, есть ли уже prewarmed версия
        {
            let cache = self.prewarmed.read().await;
            if let Some(existing) = cache.get(&strategy_id) {
                if !existing.is_expired() {
                    debug!(strategy_id = %strategy_id, "Strategy already prewarmed");
                    return Ok(PrewarmedStrategyInfo {
                        strategy_id,
                        strategy_name,
                        ready: existing.ready,
                        error: existing.error.clone(),
                        remaining_ttl_secs: existing.remaining_ttl().as_secs(),
                    });
                }
            }
        }

        info!(strategy_id = %strategy_id, "Prewarming strategy");

        // Валидируем стратегию
        let validation_result = self.validate_strategy(&strategy).await;

        let prewarmed = match validation_result {
            Ok(()) => PrewarmedStrategy::new(strategy, self.default_ttl),
            Err(e) => {
                warn!(strategy_id = %strategy_id, error = %e, "Strategy validation failed during prewarm");
                PrewarmedStrategy::with_error(strategy, e.to_string(), self.default_ttl)
            }
        };

        let info = PrewarmedStrategyInfo {
            strategy_id: prewarmed.strategy.id.clone(),
            strategy_name: prewarmed.strategy.name.clone(),
            ready: prewarmed.ready,
            error: prewarmed.error.clone(),
            remaining_ttl_secs: prewarmed.remaining_ttl().as_secs(),
        };

        // Сохраняем в кэш
        {
            let mut cache = self.prewarmed.write().await;
            cache.insert(prewarmed.strategy.id.clone(), prewarmed);
        }

        Ok(info)
    }

    /// Получает prewarmed стратегию по ID
    ///
    /// Возвращает None если стратегия не найдена или истекла.
    pub async fn get_prewarmed(&self, strategy_id: &str) -> Option<PrewarmedStrategy> {
        let cache = self.prewarmed.read().await;
        
        if let Some(prewarmed) = cache.get(strategy_id) {
            if !prewarmed.is_expired() {
                debug!(strategy_id = %strategy_id, "Retrieved prewarmed strategy");
                return Some(prewarmed.clone());
            }
            debug!(strategy_id = %strategy_id, "Prewarmed strategy expired");
        }
        
        None
    }

    /// Получает список всех prewarmed стратегий
    pub async fn get_all_prewarmed(&self) -> Vec<PrewarmedStrategyInfo> {
        let cache = self.prewarmed.read().await;
        
        cache
            .values()
            .filter(|p| !p.is_expired())
            .map(|p| PrewarmedStrategyInfo {
                strategy_id: p.strategy.id.clone(),
                strategy_name: p.strategy.name.clone(),
                ready: p.ready,
                error: p.error.clone(),
                remaining_ttl_secs: p.remaining_ttl().as_secs(),
            })
            .collect()
    }

    /// Очищает все prewarmed стратегии
    pub async fn clear_all(&self) {
        let mut cache = self.prewarmed.write().await;
        let count = cache.len();
        cache.clear();
        info!(count, "Cleared all prewarmed strategies");
    }

    /// Очищает истёкшие prewarmed стратегии
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.prewarmed.write().await;
        let before = cache.len();
        cache.retain(|_, v| !v.is_expired());
        let removed = before - cache.len();
        
        if removed > 0 {
            debug!(removed, "Cleaned up expired prewarmed strategies");
        }
        
        removed
    }

    /// Удаляет конкретную prewarmed стратегию
    pub async fn remove(&self, strategy_id: &str) -> bool {
        let mut cache = self.prewarmed.write().await;
        let removed = cache.remove(strategy_id).is_some();
        
        if removed {
            debug!(strategy_id = %strategy_id, "Removed prewarmed strategy");
        }
        
        removed
    }

    /// Проверяет, есть ли стратегия в кэше (и не истекла)
    pub async fn is_prewarmed(&self, strategy_id: &str) -> bool {
        let cache = self.prewarmed.read().await;
        cache
            .get(strategy_id)
            .map(|p| !p.is_expired() && p.ready)
            .unwrap_or(false)
    }

    /// Возвращает количество prewarmed стратегий (не истёкших)
    pub async fn count(&self) -> usize {
        let cache = self.prewarmed.read().await;
        cache.values().filter(|p| !p.is_expired()).count()
    }

    /// Валидирует стратегию перед prewarm
    async fn validate_strategy(&self, strategy: &Strategy) -> Result<()> {
        // Проверяем наличие хотя бы одного шаблона запуска
        if strategy.socks_template.is_none() && strategy.global_template.is_none() {
            return Err(IsolateError::Validation(
                "Strategy has no launch templates".to_string(),
            ));
        }

        // Проверяем ID
        if strategy.id.is_empty() {
            return Err(IsolateError::Validation(
                "Strategy ID cannot be empty".to_string(),
            ));
        }

        // Проверяем имя
        if strategy.name.is_empty() {
            return Err(IsolateError::Validation(
                "Strategy name cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for PrewarmingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный экземпляр PrewarmingManager
static PREWARMING_MANAGER: std::sync::OnceLock<Arc<PrewarmingManager>> =
    std::sync::OnceLock::new();

/// Получает глобальный PrewarmingManager
pub fn get_prewarming_manager() -> Arc<PrewarmingManager> {
    PREWARMING_MANAGER
        .get_or_init(|| Arc::new(PrewarmingManager::new()))
        .clone()
}

/// Инициализирует глобальный PrewarmingManager с кастомным TTL
pub fn init_prewarming_manager(ttl_secs: u64) -> Arc<PrewarmingManager> {
    PREWARMING_MANAGER
        .get_or_init(|| Arc::new(PrewarmingManager::with_ttl(ttl_secs)))
        .clone()
}

// ==================== Unit Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::strategy::{
        LaunchTemplate, ModeCapabilities, StrategyEngine, StrategyFamily, StrategyRequirements,
    };

    fn create_test_strategy(id: &str, name: &str) -> Strategy {
        Strategy {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test strategy".to_string(),
            family: StrategyFamily::DnsBypass,
            engine: StrategyEngine::Zapret,
            mode_capabilities: ModeCapabilities {
                supports_socks: true,
                supports_global: true,
            },
            socks_template: Some(LaunchTemplate {
                binary: "winws.exe".to_string(),
                args: vec!["--test".to_string()],
                env: std::collections::HashMap::new(),
                log_file: None,
                requires_admin: false,
            }),
            global_template: None,
            requirements: StrategyRequirements::default(),
            weight_hint: 100,
            services: vec!["youtube".to_string()],
        }
    }

    #[tokio::test]
    async fn test_prewarm_strategy() {
        let manager = PrewarmingManager::new();
        let strategy = create_test_strategy("test-1", "Test Strategy 1");

        let info = manager.prewarm_strategy(strategy).await.unwrap();

        assert_eq!(info.strategy_id, "test-1");
        assert_eq!(info.strategy_name, "Test Strategy 1");
        assert!(info.ready);
        assert!(info.error.is_none());
        assert!(info.remaining_ttl_secs > 0);
    }

    #[tokio::test]
    async fn test_get_prewarmed() {
        let manager = PrewarmingManager::new();
        let strategy = create_test_strategy("test-2", "Test Strategy 2");

        manager.prewarm_strategy(strategy.clone()).await.unwrap();

        let prewarmed = manager.get_prewarmed("test-2").await;
        assert!(prewarmed.is_some());

        let prewarmed = prewarmed.unwrap();
        assert_eq!(prewarmed.strategy.id, "test-2");
        assert!(prewarmed.ready);
    }

    #[tokio::test]
    async fn test_get_prewarmed_not_found() {
        let manager = PrewarmingManager::new();

        let prewarmed = manager.get_prewarmed("non-existent").await;
        assert!(prewarmed.is_none());
    }

    #[tokio::test]
    async fn test_is_prewarmed() {
        let manager = PrewarmingManager::new();
        let strategy = create_test_strategy("test-3", "Test Strategy 3");

        assert!(!manager.is_prewarmed("test-3").await);

        manager.prewarm_strategy(strategy).await.unwrap();

        assert!(manager.is_prewarmed("test-3").await);
    }

    #[tokio::test]
    async fn test_clear_all() {
        let manager = PrewarmingManager::new();

        manager
            .prewarm_strategy(create_test_strategy("test-4", "Test 4"))
            .await
            .unwrap();
        manager
            .prewarm_strategy(create_test_strategy("test-5", "Test 5"))
            .await
            .unwrap();

        assert_eq!(manager.count().await, 2);

        manager.clear_all().await;

        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_remove() {
        let manager = PrewarmingManager::new();
        let strategy = create_test_strategy("test-6", "Test Strategy 6");

        manager.prewarm_strategy(strategy).await.unwrap();
        assert!(manager.is_prewarmed("test-6").await);

        let removed = manager.remove("test-6").await;
        assert!(removed);
        assert!(!manager.is_prewarmed("test-6").await);

        // Повторное удаление возвращает false
        let removed_again = manager.remove("test-6").await;
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_get_all_prewarmed() {
        let manager = PrewarmingManager::new();

        manager
            .prewarm_strategy(create_test_strategy("test-7", "Test 7"))
            .await
            .unwrap();
        manager
            .prewarm_strategy(create_test_strategy("test-8", "Test 8"))
            .await
            .unwrap();

        let all = manager.get_all_prewarmed().await;
        assert_eq!(all.len(), 2);

        let ids: Vec<_> = all.iter().map(|p| p.strategy_id.as_str()).collect();
        assert!(ids.contains(&"test-7"));
        assert!(ids.contains(&"test-8"));
    }

    #[tokio::test]
    async fn test_validation_no_templates() {
        let manager = PrewarmingManager::new();

        let mut strategy = create_test_strategy("test-9", "Test 9");
        strategy.socks_template = None;
        strategy.global_template = None;

        let info = manager.prewarm_strategy(strategy).await.unwrap();

        assert!(!info.ready);
        assert!(info.error.is_some());
        assert!(info.error.unwrap().contains("no launch templates"));
    }

    #[tokio::test]
    async fn test_validation_empty_id() {
        let manager = PrewarmingManager::new();

        let strategy = create_test_strategy("", "Test");

        let info = manager.prewarm_strategy(strategy).await.unwrap();

        assert!(!info.ready);
        assert!(info.error.is_some());
        assert!(info.error.unwrap().contains("ID cannot be empty"));
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        // Создаём менеджер с очень коротким TTL (1 секунда)
        let manager = PrewarmingManager::with_ttl(1);
        let strategy = create_test_strategy("test-10", "Test 10");

        manager.prewarm_strategy(strategy).await.unwrap();
        assert!(manager.is_prewarmed("test-10").await);

        // Ждём истечения TTL
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Стратегия должна быть истёкшей
        assert!(!manager.is_prewarmed("test-10").await);
        assert!(manager.get_prewarmed("test-10").await.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = PrewarmingManager::with_ttl(1);

        manager
            .prewarm_strategy(create_test_strategy("test-11", "Test 11"))
            .await
            .unwrap();

        // Ждём истечения
        tokio::time::sleep(Duration::from_secs(2)).await;

        let removed = manager.cleanup_expired().await;
        assert_eq!(removed, 1);
        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_prewarm_already_cached() {
        let manager = PrewarmingManager::new();
        let strategy = create_test_strategy("test-12", "Test 12");

        // Первый prewarm
        let info1 = manager.prewarm_strategy(strategy.clone()).await.unwrap();

        // Второй prewarm той же стратегии — должен вернуть кэшированную
        let info2 = manager.prewarm_strategy(strategy).await.unwrap();

        assert_eq!(info1.strategy_id, info2.strategy_id);
        // TTL второго вызова должен быть меньше или равен первому
        assert!(info2.remaining_ttl_secs <= info1.remaining_ttl_secs);
    }
}

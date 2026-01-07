//! Blocked Strategies - механизм автоматической блокировки неработающих стратегий
//!
//! Стратегия блокируется после N последовательных неудач и автоматически
//! разблокируется через заданный интервал времени.
//!
//! # Использование
//! ```rust,ignore
//! let manager = BlockedStrategiesManager::new();
//!
//! // Регистрация неудачи
//! manager.record_failure("strategy-1", "Connection timeout").await;
//!
//! // Проверка блокировки
//! if manager.is_blocked("strategy-1").await {
//!     // Пропустить стратегию
//! }
//!
//! // Ручная разблокировка
//! manager.unblock_strategy("strategy-1").await;
//! ```

#![allow(dead_code)] // Public blocked strategies API

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::errors::Result;
use crate::core::storage::Storage;

// ============================================================================
// Constants
// ============================================================================

/// Количество последовательных неудач для автоблокировки
const DEFAULT_FAILURE_THRESHOLD: u32 = 3;

/// Время автоматической разблокировки (1 час)
const DEFAULT_UNBLOCK_DURATION: Duration = Duration::from_secs(3600);

/// Ключ для сохранения в Storage
const STORAGE_KEY: &str = "blocked_strategies";

// ============================================================================
// Types
// ============================================================================

/// Информация о заблокированной стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedStrategy {
    /// ID стратегии
    pub strategy_id: String,
    /// Время блокировки (Unix timestamp в секундах)
    pub blocked_at: u64,
    /// Причина блокировки
    pub reason: String,
    /// Количество неудач до блокировки
    pub failure_count: u32,
}

impl BlockedStrategy {
    /// Создаёт новую запись о заблокированной стратегии
    pub fn new(strategy_id: String, reason: String, failure_count: u32) -> Self {
        Self {
            strategy_id,
            blocked_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            reason,
            failure_count,
        }
    }

    /// Проверяет, истёк ли срок блокировки
    pub fn is_expired(&self, unblock_duration: Duration) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now.saturating_sub(self.blocked_at) >= unblock_duration.as_secs()
    }
}

/// Счётчик неудач для стратегии (до блокировки)
#[derive(Debug, Clone)]
struct FailureCounter {
    /// Количество последовательных неудач
    count: u32,
    /// Последняя причина неудачи
    last_reason: String,
    /// Время последней неудачи
    last_failure_at: Instant,
}

impl FailureCounter {
    fn new(reason: String) -> Self {
        Self {
            count: 1,
            last_reason: reason,
            last_failure_at: Instant::now(),
        }
    }

    fn increment(&mut self, reason: String) {
        self.count += 1;
        self.last_reason = reason;
        self.last_failure_at = Instant::now();
    }

    fn reset(&mut self) {
        self.count = 0;
        self.last_reason.clear();
    }
}

/// Конфигурация менеджера блокировок
#[derive(Debug, Clone)]
pub struct BlockedStrategiesConfig {
    /// Порог неудач для автоблокировки
    pub failure_threshold: u32,
    /// Время автоматической разблокировки
    pub unblock_duration: Duration,
}

impl Default for BlockedStrategiesConfig {
    fn default() -> Self {
        Self {
            failure_threshold: DEFAULT_FAILURE_THRESHOLD,
            unblock_duration: DEFAULT_UNBLOCK_DURATION,
        }
    }
}

// ============================================================================
// BlockedStrategiesManager
// ============================================================================

/// Менеджер блокировки стратегий
///
/// Thread-safe менеджер для отслеживания неудачных стратегий
/// и их автоматической блокировки/разблокировки.
pub struct BlockedStrategiesManager {
    /// Заблокированные стратегии (strategy_id -> BlockedStrategy)
    blocked: RwLock<HashMap<String, BlockedStrategy>>,
    /// Счётчики неудач для незаблокированных стратегий
    failure_counters: RwLock<HashMap<String, FailureCounter>>,
    /// Конфигурация
    config: BlockedStrategiesConfig,
    /// Storage для persistence (опционально)
    storage: Option<Arc<Storage>>,
}

impl BlockedStrategiesManager {
    /// Создаёт новый менеджер с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            blocked: RwLock::new(HashMap::new()),
            failure_counters: RwLock::new(HashMap::new()),
            config: BlockedStrategiesConfig::default(),
            storage: None,
        }
    }

    /// Создаёт менеджер с кастомной конфигурацией
    pub fn with_config(config: BlockedStrategiesConfig) -> Self {
        Self {
            blocked: RwLock::new(HashMap::new()),
            failure_counters: RwLock::new(HashMap::new()),
            config,
            storage: None,
        }
    }

    /// Создаёт менеджер с persistence через Storage
    pub fn with_storage(storage: Arc<Storage>) -> Self {
        Self {
            blocked: RwLock::new(HashMap::new()),
            failure_counters: RwLock::new(HashMap::new()),
            config: BlockedStrategiesConfig::default(),
            storage: Some(storage),
        }
    }

    /// Создаёт менеджер с конфигурацией и Storage
    pub fn with_config_and_storage(config: BlockedStrategiesConfig, storage: Arc<Storage>) -> Self {
        Self {
            blocked: RwLock::new(HashMap::new()),
            failure_counters: RwLock::new(HashMap::new()),
            config,
            storage: Some(storage),
        }
    }

    /// Загружает состояние из Storage
    pub async fn load_from_storage(&self) -> Result<()> {
        let Some(storage) = &self.storage else {
            return Ok(());
        };

        let blocked_list: Option<Vec<BlockedStrategy>> = storage
            .get_setting(STORAGE_KEY)
            .await?;

        if let Some(list) = blocked_list {
            let mut blocked = self.blocked.write().await;
            
            // Фильтруем истёкшие блокировки при загрузке
            for entry in list {
                if !entry.is_expired(self.config.unblock_duration) {
                    debug!(
                        strategy_id = %entry.strategy_id,
                        "Loaded blocked strategy from storage"
                    );
                    blocked.insert(entry.strategy_id.clone(), entry);
                } else {
                    debug!(
                        strategy_id = %entry.strategy_id,
                        "Skipped expired blocked strategy"
                    );
                }
            }

            info!(
                count = blocked.len(),
                "Loaded blocked strategies from storage"
            );
        }

        Ok(())
    }

    /// Сохраняет состояние в Storage
    async fn save_to_storage(&self) -> Result<()> {
        let Some(storage) = &self.storage else {
            return Ok(());
        };

        let blocked = self.blocked.read().await;
        let list: Vec<BlockedStrategy> = blocked.values().cloned().collect();
        
        storage.set_setting(STORAGE_KEY, &list).await?;
        
        debug!(count = list.len(), "Saved blocked strategies to storage");
        Ok(())
    }

    /// Регистрирует неудачу стратегии
    ///
    /// Если количество последовательных неудач достигает порога,
    /// стратегия автоматически блокируется.
    ///
    /// # Arguments
    /// * `strategy_id` - ID стратегии
    /// * `reason` - Причина неудачи
    ///
    /// # Returns
    /// * `true` - стратегия была заблокирована в результате этой неудачи
    /// * `false` - стратегия ещё не заблокирована
    pub async fn record_failure(&self, strategy_id: &str, reason: &str) -> bool {
        // Проверяем, не заблокирована ли уже
        {
            let blocked = self.blocked.read().await;
            if blocked.contains_key(strategy_id) {
                debug!(
                    strategy_id,
                    "Strategy already blocked, ignoring failure"
                );
                return false;
            }
        }

        // Обновляем счётчик неудач
        let should_block = {
            let mut counters = self.failure_counters.write().await;
            
            let counter = counters
                .entry(strategy_id.to_string())
                .and_modify(|c| c.increment(reason.to_string()))
                .or_insert_with(|| FailureCounter::new(reason.to_string()));

            debug!(
                strategy_id,
                failure_count = counter.count,
                threshold = self.config.failure_threshold,
                "Recorded strategy failure"
            );

            counter.count >= self.config.failure_threshold
        };

        // Блокируем если достигнут порог
        if should_block {
            self.block_strategy(strategy_id, reason).await;
            
            // Очищаем счётчик
            let mut counters = self.failure_counters.write().await;
            counters.remove(strategy_id);
            
            return true;
        }

        false
    }

    /// Регистрирует успех стратегии (сбрасывает счётчик неудач)
    pub async fn record_success(&self, strategy_id: &str) {
        let mut counters = self.failure_counters.write().await;
        
        if let Some(counter) = counters.get_mut(strategy_id) {
            if counter.count > 0 {
                debug!(
                    strategy_id,
                    previous_failures = counter.count,
                    "Reset failure counter on success"
                );
                counter.reset();
            }
        }
    }

    /// Блокирует стратегию вручную
    pub async fn block_strategy(&self, strategy_id: &str, reason: &str) {
        let failure_count = {
            let counters = self.failure_counters.read().await;
            counters.get(strategy_id).map(|c| c.count).unwrap_or(0)
        };

        let entry = BlockedStrategy::new(
            strategy_id.to_string(),
            reason.to_string(),
            failure_count.max(1),
        );

        {
            let mut blocked = self.blocked.write().await;
            blocked.insert(strategy_id.to_string(), entry.clone());
        }

        warn!(
            strategy_id,
            reason,
            failure_count = entry.failure_count,
            "Strategy blocked"
        );

        // Сохраняем в Storage
        if let Err(e) = self.save_to_storage().await {
            warn!(
                error = %e,
                "Failed to save blocked strategies to storage"
            );
        }
    }

    /// Разблокирует стратегию вручную
    pub async fn unblock_strategy(&self, strategy_id: &str) -> bool {
        let removed = {
            let mut blocked = self.blocked.write().await;
            blocked.remove(strategy_id).is_some()
        };

        if removed {
            // Также сбрасываем счётчик неудач
            {
                let mut counters = self.failure_counters.write().await;
                counters.remove(strategy_id);
            }

            info!(strategy_id, "Strategy unblocked");

            // Сохраняем в Storage
            if let Err(e) = self.save_to_storage().await {
                warn!(
                    error = %e,
                    "Failed to save blocked strategies to storage"
                );
            }
        }

        removed
    }

    /// Проверяет, заблокирована ли стратегия
    ///
    /// Автоматически разблокирует стратегию, если истёк срок блокировки.
    pub async fn is_blocked(&self, strategy_id: &str) -> bool {
        // Сначала проверяем без записи
        {
            let blocked = self.blocked.read().await;
            
            if let Some(entry) = blocked.get(strategy_id) {
                if !entry.is_expired(self.config.unblock_duration) {
                    return true;
                }
            } else {
                return false;
            }
        }

        // Если блокировка истекла, разблокируем
        self.auto_unblock_expired(strategy_id).await;
        false
    }

    /// Автоматически разблокирует истёкшую стратегию
    async fn auto_unblock_expired(&self, strategy_id: &str) {
        let should_unblock = {
            let blocked = self.blocked.read().await;
            blocked
                .get(strategy_id)
                .map(|e| e.is_expired(self.config.unblock_duration))
                .unwrap_or(false)
        };

        if should_unblock {
            let mut blocked = self.blocked.write().await;
            if let Some(entry) = blocked.remove(strategy_id) {
                info!(
                    strategy_id,
                    blocked_at = entry.blocked_at,
                    "Strategy auto-unblocked after timeout"
                );
            }

            // Сохраняем в Storage (вне lock)
            drop(blocked);
            if let Err(e) = self.save_to_storage().await {
                warn!(
                    error = %e,
                    "Failed to save blocked strategies to storage"
                );
            }
        }
    }

    /// Возвращает список всех заблокированных стратегий
    ///
    /// Автоматически очищает истёкшие блокировки.
    pub async fn get_blocked_list(&self) -> Vec<BlockedStrategy> {
        // Очищаем истёкшие
        self.cleanup_expired().await;

        let blocked = self.blocked.read().await;
        blocked.values().cloned().collect()
    }

    /// Очищает все истёкшие блокировки
    pub async fn cleanup_expired(&self) {
        let expired_ids: Vec<String> = {
            let blocked = self.blocked.read().await;
            blocked
                .iter()
                .filter(|(_, entry)| entry.is_expired(self.config.unblock_duration))
                .map(|(id, _)| id.clone())
                .collect()
        };

        if expired_ids.is_empty() {
            return;
        }

        {
            let mut blocked = self.blocked.write().await;
            for id in &expired_ids {
                if let Some(entry) = blocked.remove(id) {
                    info!(
                        strategy_id = %id,
                        blocked_at = entry.blocked_at,
                        "Strategy auto-unblocked (cleanup)"
                    );
                }
            }
        }

        // Сохраняем в Storage
        if let Err(e) = self.save_to_storage().await {
            warn!(
                error = %e,
                "Failed to save blocked strategies to storage"
            );
        }
    }

    /// Возвращает количество заблокированных стратегий
    pub async fn blocked_count(&self) -> usize {
        let blocked = self.blocked.read().await;
        blocked.len()
    }

    /// Возвращает информацию о блокировке стратегии
    pub async fn get_blocked_info(&self, strategy_id: &str) -> Option<BlockedStrategy> {
        let blocked = self.blocked.read().await;
        blocked.get(strategy_id).cloned()
    }

    /// Возвращает текущий счётчик неудач для стратегии
    pub async fn get_failure_count(&self, strategy_id: &str) -> u32 {
        let counters = self.failure_counters.read().await;
        counters.get(strategy_id).map(|c| c.count).unwrap_or(0)
    }

    /// Сбрасывает все блокировки и счётчики
    pub async fn reset_all(&self) {
        {
            let mut blocked = self.blocked.write().await;
            blocked.clear();
        }
        {
            let mut counters = self.failure_counters.write().await;
            counters.clear();
        }

        info!("All blocked strategies and failure counters reset");

        // Сохраняем в Storage
        if let Err(e) = self.save_to_storage().await {
            warn!(
                error = %e,
                "Failed to save blocked strategies to storage"
            );
        }
    }
}

impl Default for BlockedStrategiesManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_failure_blocks_after_threshold() {
        let manager = BlockedStrategiesManager::new();

        // Первые две неудачи не блокируют
        assert!(!manager.record_failure("s1", "error 1").await);
        assert!(!manager.is_blocked("s1").await);
        
        assert!(!manager.record_failure("s1", "error 2").await);
        assert!(!manager.is_blocked("s1").await);

        // Третья неудача блокирует
        assert!(manager.record_failure("s1", "error 3").await);
        assert!(manager.is_blocked("s1").await);
    }

    #[tokio::test]
    async fn test_success_resets_failure_counter() {
        let manager = BlockedStrategiesManager::new();

        // Две неудачи
        manager.record_failure("s1", "error 1").await;
        manager.record_failure("s1", "error 2").await;
        assert_eq!(manager.get_failure_count("s1").await, 2);

        // Успех сбрасывает счётчик
        manager.record_success("s1").await;
        assert_eq!(manager.get_failure_count("s1").await, 0);

        // Снова нужно 3 неудачи для блокировки
        manager.record_failure("s1", "error 1").await;
        manager.record_failure("s1", "error 2").await;
        assert!(!manager.is_blocked("s1").await);
    }

    #[tokio::test]
    async fn test_manual_block_unblock() {
        let manager = BlockedStrategiesManager::new();

        // Ручная блокировка
        manager.block_strategy("s1", "manual block").await;
        assert!(manager.is_blocked("s1").await);

        // Ручная разблокировка
        assert!(manager.unblock_strategy("s1").await);
        assert!(!manager.is_blocked("s1").await);

        // Повторная разблокировка возвращает false
        assert!(!manager.unblock_strategy("s1").await);
    }

    #[tokio::test]
    async fn test_get_blocked_list() {
        let manager = BlockedStrategiesManager::new();

        manager.block_strategy("s1", "reason 1").await;
        manager.block_strategy("s2", "reason 2").await;

        let list = manager.get_blocked_list().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_auto_unblock_after_timeout() {
        let config = BlockedStrategiesConfig {
            failure_threshold: 3,
            unblock_duration: Duration::from_secs(1), // 1 секунда для теста
        };
        let manager = BlockedStrategiesManager::with_config(config);

        manager.block_strategy("s1", "test").await;
        assert!(manager.is_blocked("s1").await);

        // Ждём истечения таймаута (чуть больше 1 секунды)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Должна автоматически разблокироваться
        assert!(!manager.is_blocked("s1").await);
    }

    #[tokio::test]
    async fn test_custom_threshold() {
        let config = BlockedStrategiesConfig {
            failure_threshold: 5,
            unblock_duration: DEFAULT_UNBLOCK_DURATION,
        };
        let manager = BlockedStrategiesManager::with_config(config);

        // 4 неудачи не блокируют
        for i in 1..=4 {
            assert!(!manager.record_failure("s1", &format!("error {}", i)).await);
        }
        assert!(!manager.is_blocked("s1").await);

        // 5-я блокирует
        assert!(manager.record_failure("s1", "error 5").await);
        assert!(manager.is_blocked("s1").await);
    }

    #[tokio::test]
    async fn test_reset_all() {
        let manager = BlockedStrategiesManager::new();

        manager.block_strategy("s1", "reason").await;
        manager.record_failure("s2", "error").await;

        manager.reset_all().await;

        assert!(!manager.is_blocked("s1").await);
        assert_eq!(manager.get_failure_count("s2").await, 0);
        assert_eq!(manager.blocked_count().await, 0);
    }

    #[tokio::test]
    async fn test_blocked_strategy_info() {
        let manager = BlockedStrategiesManager::new();

        manager.block_strategy("s1", "test reason").await;

        let info = manager.get_blocked_info("s1").await;
        assert!(info.is_some());
        
        let info = info.unwrap();
        assert_eq!(info.strategy_id, "s1");
        assert_eq!(info.reason, "test reason");
    }
}

//! Auto Failover - автоматическое переключение на backup стратегию при сбое
//!
//! Модуль отслеживает сбои стратегий и автоматически переключается на backup
//! стратегию после заданного количества последовательных неудач.
//!
//! ## Конфигурация
//! - `primary_strategy_id` - ID основной стратегии
//! - `backup_strategy_ids` - список backup стратегий в порядке приоритета
//! - `max_failures` - количество сбоев до переключения (default: 3)
//! - `cooldown_secs` - время ожидания перед попыткой вернуться на primary (default: 300)
//! - `enabled` - включен ли failover
//!
//! ## Использование
//! ```rust
//! let failover = AutoFailover::new(config);
//! 
//! // При сбое стратегии
//! failover.record_failure("strategy-1", "Connection timeout").await;
//! 
//! // Проверка необходимости переключения
//! if failover.should_failover("strategy-1").await {
//!     let backup = failover.get_next_backup_strategy("strategy-1").await;
//!     // Применить backup стратегию
//! }
//! 
//! // При успехе
//! failover.record_success("strategy-1").await;
//! 
//! // Попытка вернуться на primary после cooldown
//! if let Some(primary) = failover.try_restore_primary().await {
//!     // Применить primary стратегию
//! }
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use chrono::{DateTime, Utc};

// ============================================================================
// Configuration
// ============================================================================

/// Конфигурация Auto Failover
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailoverConfig {
    /// ID основной (primary) стратегии
    #[serde(default)]
    pub primary_strategy_id: Option<String>,
    
    /// Список backup стратегий в порядке приоритета
    /// Если пустой, используются learned strategies из истории
    #[serde(default)]
    pub backup_strategy_ids: Vec<String>,
    
    /// Максимальное количество сбоев до переключения
    #[serde(default = "default_max_failures")]
    pub max_failures: u32,
    
    /// Время ожидания (cooldown) в секундах перед попыткой вернуться на primary
    #[serde(default = "default_cooldown_secs")]
    pub cooldown_secs: u64,
    
    /// Включен ли auto failover
    #[serde(default)]
    pub enabled: bool,
}

fn default_max_failures() -> u32 {
    3
}

fn default_cooldown_secs() -> u64 {
    300 // 5 минут
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            primary_strategy_id: None,
            backup_strategy_ids: Vec::new(),
            max_failures: default_max_failures(),
            cooldown_secs: default_cooldown_secs(),
            enabled: false,
        }
    }
}

// Backward compatibility alias
impl FailoverConfig {
    /// Alias for backup_strategy_ids (backward compatibility)
    pub fn backup_strategies(&self) -> &Vec<String> {
        &self.backup_strategy_ids
    }
}

// ============================================================================
// State
// ============================================================================

/// Состояние failover для конкретной стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailoverState {
    /// ID текущей активной стратегии
    pub current_strategy_id: String,
    
    /// Текущее количество последовательных сбоев
    pub failure_count: u32,
    
    /// Время последнего сбоя (ISO 8601)
    pub last_failure: Option<DateTime<Utc>>,
    
    /// Находимся ли на backup стратегии
    pub is_on_backup: bool,
    
    /// Причина последнего сбоя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_failure_reason: Option<String>,
    
    /// Индекс текущей backup стратегии (для циклического перебора)
    #[serde(default)]
    pub backup_index: usize,
    
    /// Стратегии, которые уже были попробованы в текущем цикле failover
    #[serde(default)]
    pub tried_strategies: Vec<String>,
    
    /// Время переключения на backup (для отслеживания cooldown)
    #[serde(skip)]
    pub switched_to_backup_at: Option<Instant>,
}

impl Default for FailoverState {
    fn default() -> Self {
        Self {
            current_strategy_id: String::new(),
            failure_count: 0,
            last_failure: None,
            is_on_backup: false,
            last_failure_reason: None,
            backup_index: 0,
            tried_strategies: Vec::new(),
            switched_to_backup_at: None,
        }
    }
}

impl FailoverState {
    /// Создаёт новое состояние для стратегии
    pub fn new(strategy_id: &str) -> Self {
        Self {
            current_strategy_id: strategy_id.to_string(),
            ..Default::default()
        }
    }
}

/// Статус failover для отображения в UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailoverStatus {
    /// Включен ли auto failover
    pub enabled: bool,
    
    /// Текущее количество сбоев
    pub failure_count: u32,
    
    /// Максимальное количество сбоев до переключения
    pub max_failures: u32,
    
    /// ID текущей стратегии
    pub current_strategy: Option<String>,
    
    /// ID primary стратегии
    pub primary_strategy: Option<String>,
    
    /// ID следующей backup стратегии (если есть)
    pub next_backup: Option<String>,
    
    /// Находимся ли на backup стратегии
    pub is_on_backup: bool,
    
    /// Время до окончания cooldown (в секундах)
    pub cooldown_remaining: Option<u64>,
    
    /// Причина последнего сбоя
    pub last_failure_reason: Option<String>,
    
    /// Время последнего сбоя (ISO 8601)
    pub last_failure: Option<DateTime<Utc>>,
}

// ============================================================================
// Auto Failover Manager
// ============================================================================

/// Менеджер автоматического переключения стратегий
pub struct AutoFailover {
    /// Конфигурация
    config: RwLock<FailoverConfig>,
    
    /// Состояние для каждой стратегии
    states: RwLock<HashMap<String, FailoverState>>,
    
    /// Текущая активная стратегия
    current_strategy: RwLock<Option<String>>,
    
    /// Primary стратегия (для восстановления)
    primary_strategy: RwLock<Option<String>>,
    
    /// Находимся ли на backup стратегии
    is_on_backup: RwLock<bool>,
    
    /// Время переключения на backup
    switched_to_backup_at: RwLock<Option<Instant>>,
    
    /// Включен ли failover
    enabled: RwLock<bool>,
    
    /// Список learned strategies (успешно работавших ранее)
    learned_strategies: RwLock<Vec<String>>,
}

impl AutoFailover {
    /// Создаёт новый экземпляр с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self::with_config(FailoverConfig::default())
    }
    
    /// Создаёт новый экземпляр с заданной конфигурацией
    pub fn with_config(config: FailoverConfig) -> Self {
        let enabled = config.enabled;
        let primary = config.primary_strategy_id.clone();
        
        Self {
            config: RwLock::new(config),
            states: RwLock::new(HashMap::new()),
            current_strategy: RwLock::new(None),
            primary_strategy: RwLock::new(primary),
            is_on_backup: RwLock::new(false),
            switched_to_backup_at: RwLock::new(None),
            enabled: RwLock::new(enabled),
            learned_strategies: RwLock::new(Vec::new()),
        }
    }
    
    // ========================================================================
    // Configuration
    // ========================================================================
    
    /// Включает/выключает auto failover
    pub async fn set_enabled(&self, enabled: bool) {
        let mut flag = self.enabled.write().await;
        let was_enabled = *flag;
        *flag = enabled;
        
        if was_enabled != enabled {
            info!(enabled, "Auto failover status changed");
            
            // При включении сбрасываем состояния
            if enabled {
                self.reset_all_states().await;
            }
        }
    }
    
    /// Проверяет, включен ли auto failover
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
    
    /// Обновляет конфигурацию
    pub async fn update_config(&self, config: FailoverConfig) {
        let mut current = self.config.write().await;
        *current = config;
        debug!("Failover config updated");
    }
    
    /// Получает текущую конфигурацию
    pub async fn get_config(&self) -> FailoverConfig {
        self.config.read().await.clone()
    }
    
    /// Устанавливает текущую стратегию
    pub async fn set_current_strategy(&self, strategy_id: Option<String>) {
        let mut current = self.current_strategy.write().await;
        *current = strategy_id.clone();
        
        if let Some(id) = strategy_id {
            debug!(strategy_id = %id, "Current strategy set");
        }
    }
    
    /// Получает текущую стратегию
    pub async fn get_current_strategy(&self) -> Option<String> {
        self.current_strategy.read().await.clone()
    }
    
    /// Устанавливает primary стратегию
    pub async fn set_primary_strategy(&self, strategy_id: Option<String>) {
        let mut primary = self.primary_strategy.write().await;
        *primary = strategy_id.clone();
        
        // Также обновляем конфиг
        let mut config = self.config.write().await;
        config.primary_strategy_id = strategy_id.clone();
        
        if let Some(id) = strategy_id {
            debug!(strategy_id = %id, "Primary strategy set");
        }
    }
    
    /// Получает primary стратегию
    pub async fn get_primary_strategy(&self) -> Option<String> {
        self.primary_strategy.read().await.clone()
    }
    
    /// Проверяет, находимся ли на backup стратегии
    pub async fn is_on_backup(&self) -> bool {
        *self.is_on_backup.read().await
    }
    
    // ========================================================================
    // Failure Tracking
    // ========================================================================
    
    /// Записывает сбой стратегии
    /// 
    /// Возвращает `true`, если достигнут порог для failover
    pub async fn record_failure(&self, strategy_id: &str, reason: &str) -> bool {
        if !self.is_enabled().await {
            return false;
        }
        
        let config = self.config.read().await;
        let max_failures = config.max_failures;
        drop(config);
        
        let mut states = self.states.write().await;
        let state = states.entry(strategy_id.to_string()).or_insert_with(|| {
            FailoverState::new(strategy_id)
        });
        
        state.failure_count += 1;
        state.last_failure = Some(Utc::now());
        state.last_failure_reason = Some(reason.to_string());
        
        let should_failover = state.failure_count >= max_failures;
        
        warn!(
            strategy_id,
            failure_count = state.failure_count,
            max_failures,
            reason,
            should_failover,
            "Strategy failure recorded"
        );
        
        should_failover
    }
    
    /// Записывает успех стратегии (сбрасывает счётчик сбоев)
    pub async fn record_success(&self, strategy_id: &str) {
        // Сбрасываем состояние сбоев
        {
            let mut states = self.states.write().await;
            if let Some(state) = states.get_mut(strategy_id) {
                if state.failure_count > 0 {
                    debug!(
                        strategy_id,
                        previous_failures = state.failure_count,
                        "Strategy success - resetting failure count"
                    );
                }
                state.failure_count = 0;
                state.last_failure = None;
                state.last_failure_reason = None;
                state.tried_strategies.clear();
            }
        }
        
        // Добавляем в learned strategies если ещё нет
        self.add_learned_strategy(strategy_id).await;
    }
    
    /// Попытка вернуться на primary стратегию после cooldown
    /// 
    /// Возвращает ID primary стратегии, если:
    /// - Мы на backup стратегии
    /// - Прошло достаточно времени (cooldown)
    /// - Primary стратегия задана
    /// 
    /// Возвращает `None` если условия не выполнены
    pub async fn try_restore_primary(&self) -> Option<String> {
        if !self.is_enabled().await {
            return None;
        }
        
        // Проверяем, что мы на backup
        if !*self.is_on_backup.read().await {
            return None;
        }
        
        // Проверяем cooldown
        let config = self.config.read().await;
        let cooldown_secs = config.cooldown_secs;
        let primary = config.primary_strategy_id.clone();
        drop(config);
        
        let primary_id = primary?;
        
        // Проверяем время с момента переключения на backup
        let switched_at = self.switched_to_backup_at.read().await;
        if let Some(switched) = *switched_at {
            let elapsed = switched.elapsed();
            if elapsed < Duration::from_secs(cooldown_secs) {
                debug!(
                    cooldown_remaining = (cooldown_secs - elapsed.as_secs()),
                    "Restore primary blocked by cooldown"
                );
                return None;
            }
        }
        
        info!(
            primary_strategy = %primary_id,
            "Cooldown expired, attempting to restore primary strategy"
        );
        
        // Сбрасываем состояние backup
        {
            let mut is_backup = self.is_on_backup.write().await;
            *is_backup = false;
        }
        {
            let mut switched = self.switched_to_backup_at.write().await;
            *switched = None;
        }
        
        // Сбрасываем счётчик сбоев для primary
        self.reset_strategy_state(&primary_id).await;
        
        Some(primary_id)
    }
    
    /// Принудительное восстановление primary стратегии (игнорирует cooldown)
    pub async fn force_restore_primary(&self) -> Option<String> {
        let primary = self.get_primary_strategy().await?;
        
        info!(
            primary_strategy = %primary,
            "Force restoring primary strategy"
        );
        
        // Сбрасываем состояние backup
        {
            let mut is_backup = self.is_on_backup.write().await;
            *is_backup = false;
        }
        {
            let mut switched = self.switched_to_backup_at.write().await;
            *switched = None;
        }
        
        // Сбрасываем счётчик сбоев для primary
        self.reset_strategy_state(&primary).await;
        
        Some(primary)
    }
    
    /// Добавляет стратегию в список learned (успешно работавших)
    pub async fn add_learned_strategy(&self, strategy_id: &str) {
        let mut learned = self.learned_strategies.write().await;
        if !learned.contains(&strategy_id.to_string()) {
            learned.push(strategy_id.to_string());
            debug!(strategy_id, "Added to learned strategies");
        }
    }
    
    /// Получает список learned strategies
    pub async fn get_learned_strategies(&self) -> Vec<String> {
        self.learned_strategies.read().await.clone()
    }
    
    /// Устанавливает список learned strategies
    pub async fn set_learned_strategies(&self, strategies: Vec<String>) {
        let mut learned = self.learned_strategies.write().await;
        *learned = strategies;
    }
    
    // ========================================================================
    // Failover Logic
    // ========================================================================
    
    /// Проверяет, нужно ли переключиться на backup стратегию
    pub async fn should_failover(&self, strategy_id: &str) -> bool {
        if !self.is_enabled().await {
            return false;
        }
        
        let config = self.config.read().await;
        let max_failures = config.max_failures;
        drop(config);
        
        let states = self.states.read().await;
        let state = match states.get(strategy_id) {
            Some(s) => s,
            None => return false,
        };
        
        // Проверяем количество сбоев
        state.failure_count >= max_failures
    }
    
    /// Получает следующую backup стратегию
    /// 
    /// Возвращает `None` если нет доступных backup стратегий
    pub async fn get_next_backup_strategy(&self, current_strategy_id: &str) -> Option<String> {
        let config = self.config.read().await;
        let backup_strategies = config.backup_strategy_ids.clone();
        drop(config);
        
        // Получаем состояние текущей стратегии
        let mut states = self.states.write().await;
        let state = states.entry(current_strategy_id.to_string()).or_insert_with(|| {
            FailoverState::new(current_strategy_id)
        });
        
        // Определяем список кандидатов
        let candidates: Vec<String> = if backup_strategies.is_empty() {
            // Используем learned strategies
            let learned = self.learned_strategies.read().await;
            learned.iter()
                .filter(|s| *s != current_strategy_id)
                .filter(|s| !state.tried_strategies.contains(s))
                .cloned()
                .collect()
        } else {
            // Используем заданный список backup стратегий
            backup_strategies.iter()
                .filter(|s| *s != current_strategy_id)
                .filter(|s| !state.tried_strategies.contains(s))
                .cloned()
                .collect()
        };
        
        if candidates.is_empty() {
            // Все стратегии уже попробованы, сбрасываем список
            info!(
                current_strategy_id,
                "All backup strategies tried, resetting cycle"
            );
            state.tried_strategies.clear();
            state.backup_index = 0;
            return None;
        }
        
        // Выбираем следующую стратегию
        let index = state.backup_index % candidates.len();
        let next_strategy = candidates.get(index).cloned();
        
        if let Some(ref strategy) = next_strategy {
            state.tried_strategies.push(strategy.clone());
            state.backup_index += 1;
            state.is_on_backup = true;
            state.switched_to_backup_at = Some(Instant::now());
            
            // Обновляем глобальное состояние
            {
                let mut is_backup = self.is_on_backup.write().await;
                *is_backup = true;
            }
            {
                let mut switched = self.switched_to_backup_at.write().await;
                *switched = Some(Instant::now());
            }
            
            info!(
                current_strategy_id,
                next_strategy = %strategy,
                tried_count = state.tried_strategies.len(),
                "Selected backup strategy"
            );
        }
        
        next_strategy
    }
    
    /// Выполняет ручное переключение на backup стратегию
    /// 
    /// Возвращает ID backup стратегии или None если нет доступных
    pub async fn trigger_manual_failover(&self) -> Option<String> {
        let current = self.get_current_strategy().await?;
        
        info!(current_strategy = %current, "Manual failover triggered");
        
        // Сохраняем текущую как primary если ещё не задана
        {
            let primary = self.primary_strategy.read().await;
            if primary.is_none() {
                drop(primary);
                self.set_primary_strategy(Some(current.clone())).await;
            }
        }
        
        // Записываем "сбой" для текущей стратегии
        let config = self.config.read().await;
        let max_failures = config.max_failures;
        drop(config);
        
        {
            let mut states = self.states.write().await;
            let state = states.entry(current.clone()).or_insert_with(|| {
                FailoverState::new(&current)
            });
            state.failure_count = max_failures; // Форсируем failover
            state.last_failure = Some(Utc::now());
            state.last_failure_reason = Some("Manual failover".to_string());
        }
        
        self.get_next_backup_strategy(&current).await
    }
    
    // ========================================================================
    // Status
    // ========================================================================
    
    /// Получает текущий статус failover
    pub async fn get_status(&self) -> FailoverStatus {
        let enabled = self.is_enabled().await;
        let config = self.config.read().await;
        let current_strategy = self.get_current_strategy().await;
        let primary_strategy = config.primary_strategy_id.clone();
        let is_on_backup = *self.is_on_backup.read().await;
        
        let (failure_count, cooldown_remaining, last_failure_reason, last_failure, next_backup) = 
            if let Some(ref strategy_id) = current_strategy {
                let states = self.states.read().await;
                if let Some(state) = states.get(strategy_id) {
                    // Вычисляем cooldown для восстановления primary
                    let cooldown = if is_on_backup {
                        let switched = self.switched_to_backup_at.read().await;
                        switched.map(|t| {
                            let elapsed = t.elapsed().as_secs();
                            if elapsed < config.cooldown_secs {
                                config.cooldown_secs - elapsed
                            } else {
                                0
                            }
                        })
                    } else {
                        None
                    };
                    
                    // Получаем следующую backup стратегию без изменения состояния
                    let backup = if state.failure_count >= config.max_failures {
                        let backup_strategies = &config.backup_strategy_ids;
                        let learned = self.learned_strategies.read().await;
                        
                        let candidates: Vec<&String> = if backup_strategies.is_empty() {
                            learned.iter()
                                .filter(|s| *s != strategy_id)
                                .filter(|s| !state.tried_strategies.contains(*s))
                                .collect()
                        } else {
                            backup_strategies.iter()
                                .filter(|s| *s != strategy_id)
                                .filter(|s| !state.tried_strategies.contains(*s))
                                .collect()
                        };
                        
                        candidates.first().map(|s| (*s).clone())
                    } else {
                        None
                    };
                    
                    (
                        state.failure_count,
                        cooldown,
                        state.last_failure_reason.clone(),
                        state.last_failure,
                        backup,
                    )
                } else {
                    (0, None, None, None, None)
                }
            } else {
                (0, None, None, None, None)
            };
        
        FailoverStatus {
            enabled,
            failure_count,
            max_failures: config.max_failures,
            current_strategy,
            primary_strategy,
            next_backup,
            is_on_backup,
            cooldown_remaining,
            last_failure_reason,
            last_failure,
        }
    }
    
    /// Получает текущее состояние failover (сериализуемое)
    pub async fn get_failover_state(&self) -> Option<FailoverState> {
        let current = self.get_current_strategy().await?;
        let states = self.states.read().await;
        states.get(&current).cloned()
    }
    
    /// Получает состояние для конкретной стратегии
    pub async fn get_strategy_state(&self, strategy_id: &str) -> Option<FailoverState> {
        let states = self.states.read().await;
        states.get(strategy_id).cloned()
    }
    
    /// Сбрасывает состояние для конкретной стратегии
    pub async fn reset_strategy_state(&self, strategy_id: &str) {
        let mut states = self.states.write().await;
        states.remove(strategy_id);
        debug!(strategy_id, "Failover state reset");
    }
    
    /// Сбрасывает все состояния
    pub async fn reset_all_states(&self) {
        let mut states = self.states.write().await;
        states.clear();
        debug!("All failover states reset");
    }
}

impl Default for AutoFailover {
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
    async fn test_failover_disabled_by_default() {
        let failover = AutoFailover::new();
        assert!(!failover.is_enabled().await);
    }

    #[tokio::test]
    async fn test_record_failure_when_disabled() {
        let failover = AutoFailover::new();
        let should_failover = failover.record_failure("test-strategy", "test error").await;
        assert!(!should_failover);
    }

    #[tokio::test]
    async fn test_record_failure_triggers_failover() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("test-strategy".to_string()),
            backup_strategy_ids: vec!["backup-1".to_string()],
            max_failures: 2,
            cooldown_secs: 0,
            enabled: true,
        });
        
        // Первый сбой - не должен триггерить failover
        let should_failover = failover.record_failure("test-strategy", "error 1").await;
        assert!(!should_failover);
        
        // Второй сбой - должен триггерить failover
        let should_failover = failover.record_failure("test-strategy", "error 2").await;
        assert!(should_failover);
    }

    #[tokio::test]
    async fn test_record_success_resets_failures() {
        let failover = AutoFailover::with_config(FailoverConfig {
            enabled: true,
            ..Default::default()
        });
        
        failover.record_failure("test-strategy", "error").await;
        failover.record_failure("test-strategy", "error").await;
        
        let state = failover.get_strategy_state("test-strategy").await.unwrap();
        assert_eq!(state.failure_count, 2);
        
        failover.record_success("test-strategy").await;
        
        let state = failover.get_strategy_state("test-strategy").await.unwrap();
        assert_eq!(state.failure_count, 0);
    }

    #[tokio::test]
    async fn test_get_next_backup_strategy() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("main-strategy".to_string()),
            backup_strategy_ids: vec![
                "backup-1".to_string(),
                "backup-2".to_string(),
            ],
            max_failures: 1,
            cooldown_secs: 0,
            enabled: true,
        });
        
        let backup = failover.get_next_backup_strategy("main-strategy").await;
        assert_eq!(backup, Some("backup-1".to_string()));
        
        let backup = failover.get_next_backup_strategy("main-strategy").await;
        assert_eq!(backup, Some("backup-2".to_string()));
        
        // Все попробованы - должен вернуть None и сбросить цикл
        let backup = failover.get_next_backup_strategy("main-strategy").await;
        assert!(backup.is_none());
    }

    #[tokio::test]
    async fn test_learned_strategies() {
        let failover = AutoFailover::with_config(FailoverConfig {
            enabled: true,
            ..Default::default()
        });
        
        failover.add_learned_strategy("strategy-1").await;
        failover.add_learned_strategy("strategy-2").await;
        failover.add_learned_strategy("strategy-1").await; // Дубликат
        
        let learned = failover.get_learned_strategies().await;
        assert_eq!(learned.len(), 2);
        assert!(learned.contains(&"strategy-1".to_string()));
        assert!(learned.contains(&"strategy-2".to_string()));
    }

    #[tokio::test]
    async fn test_failover_status() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("main-strategy".to_string()),
            backup_strategy_ids: vec!["backup-1".to_string()],
            max_failures: 3,
            cooldown_secs: 60,
            enabled: true,
        });
        failover.set_current_strategy(Some("main-strategy".to_string())).await;
        
        let status = failover.get_status().await;
        assert!(status.enabled);
        assert_eq!(status.failure_count, 0);
        assert_eq!(status.max_failures, 3);
        assert_eq!(status.current_strategy, Some("main-strategy".to_string()));
        assert_eq!(status.primary_strategy, Some("main-strategy".to_string()));
        assert!(!status.is_on_backup);
    }

    #[tokio::test]
    async fn test_manual_failover() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("main-strategy".to_string()),
            backup_strategy_ids: vec!["backup-1".to_string()],
            max_failures: 3,
            cooldown_secs: 0,
            enabled: true,
        });
        failover.set_current_strategy(Some("main-strategy".to_string())).await;
        
        let backup = failover.trigger_manual_failover().await;
        assert_eq!(backup, Some("backup-1".to_string()));
        assert!(failover.is_on_backup().await);
    }

    #[tokio::test]
    async fn test_try_restore_primary() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("main-strategy".to_string()),
            backup_strategy_ids: vec!["backup-1".to_string()],
            max_failures: 1,
            cooldown_secs: 0, // Нулевой cooldown для теста
            enabled: true,
        });
        failover.set_current_strategy(Some("main-strategy".to_string())).await;
        
        // Переключаемся на backup
        let backup = failover.trigger_manual_failover().await;
        assert_eq!(backup, Some("backup-1".to_string()));
        assert!(failover.is_on_backup().await);
        
        // Пытаемся восстановить primary (cooldown = 0, должно сработать)
        let primary = failover.try_restore_primary().await;
        assert_eq!(primary, Some("main-strategy".to_string()));
        assert!(!failover.is_on_backup().await);
    }

    #[tokio::test]
    async fn test_force_restore_primary() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("main-strategy".to_string()),
            backup_strategy_ids: vec!["backup-1".to_string()],
            max_failures: 1,
            cooldown_secs: 3600, // Большой cooldown
            enabled: true,
        });
        failover.set_current_strategy(Some("main-strategy".to_string())).await;
        
        // Переключаемся на backup
        failover.trigger_manual_failover().await;
        assert!(failover.is_on_backup().await);
        
        // Принудительно восстанавливаем primary (игнорируя cooldown)
        let primary = failover.force_restore_primary().await;
        assert_eq!(primary, Some("main-strategy".to_string()));
        assert!(!failover.is_on_backup().await);
    }

    #[tokio::test]
    async fn test_is_on_backup_state() {
        let failover = AutoFailover::with_config(FailoverConfig {
            primary_strategy_id: Some("main-strategy".to_string()),
            backup_strategy_ids: vec!["backup-1".to_string()],
            max_failures: 1,
            cooldown_secs: 300,
            enabled: true,
        });
        failover.set_current_strategy(Some("main-strategy".to_string())).await;
        
        // Изначально не на backup
        assert!(!failover.is_on_backup().await);
        
        // После failover - на backup
        failover.get_next_backup_strategy("main-strategy").await;
        assert!(failover.is_on_backup().await);
        
        // После force restore - не на backup
        failover.force_restore_primary().await;
        assert!(!failover.is_on_backup().await);
    }
}

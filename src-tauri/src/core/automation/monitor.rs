//! Domain Monitor - непрерывный мониторинг доменов
//!
//! Рефакторинг из orchestra.rs. Выполняет:
//! - Circular перебор стратегий для каждого домена
//! - Тестирование через HttpProber
//! - Lock стратегий после N успехов
//! - Unlock после M неудач
//! - Emit событий для UI

#![allow(dead_code)] // Public domain monitor API

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use crate::core::errors::Result;
use crate::core::managers::{BlockedStrategiesManager, LockedStrategiesManager, Protocol, StrategyHistoryManager};
use crate::core::models::Strategy;
use crate::core::testing::{HttpProber, ProbeConfig, TestEndpoint};
use serde::{Deserialize, Serialize};

use super::events::{AutomationEvent, DomainStatus};

// ============================================================================
// Configuration
// ============================================================================

/// Конфигурация мониторинга
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorConfig {
    /// Количество успехов для LOCK (default: 3)
    pub lock_threshold: u32,
    /// Количество failures для UNLOCK (default: 2)
    pub unlock_threshold: u32,
    /// Таймаут теста в миллисекундах
    #[serde(with = "duration_millis")]
    pub test_timeout: Duration,
    /// Пауза между циклами в миллисекундах
    #[serde(with = "duration_millis")]
    pub cycle_delay: Duration,
    /// Пауза между тестами доменов в миллисекундах
    #[serde(with = "duration_millis")]
    pub domain_delay: Duration,
    /// Минимум байт для успеха
    pub min_bytes_success: u64,
}

/// Сериализация Duration как миллисекунды
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            lock_threshold: 3,
            unlock_threshold: 2,
            test_timeout: Duration::from_secs(5),
            cycle_delay: Duration::from_secs(1),
            domain_delay: Duration::from_millis(500),
            min_bytes_success: 2048,
        }
    }
}

impl MonitorConfig {
    /// Создаёт конфигурацию с кастомными порогами
    pub fn with_thresholds(lock_threshold: u32, unlock_threshold: u32) -> Self {
        Self {
            lock_threshold,
            unlock_threshold,
            ..Default::default()
        }
    }

    /// Устанавливает таймаут теста
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.test_timeout = timeout;
        self
    }

    /// Устанавливает минимум байт для успеха
    pub fn with_min_bytes(mut self, min_bytes: u64) -> Self {
        self.min_bytes_success = min_bytes;
        self
    }
}

// ============================================================================
// Domain State
// ============================================================================

/// Состояние домена в мониторинге
#[derive(Debug, Clone)]
struct DomainState {
    /// Текущая стратегия
    strategy_id: String,
    /// Количество успехов подряд
    successes: u32,
    /// Количество неудач подряд
    failures: u32,
    /// Текущий статус
    status: DomainStatus,
    /// Индекс текущей стратегии (для circular перебора)
    strategy_index: usize,
}

impl DomainState {
    fn new(strategy_id: String) -> Self {
        Self {
            strategy_id,
            successes: 0,
            failures: 0,
            status: DomainStatus::Testing,
            strategy_index: 0,
        }
    }
}

// ============================================================================
// DomainMonitor
// ============================================================================

/// Мониторинг доменов с автоматическим перебором стратегий
pub struct DomainMonitor {
    /// HTTP prober для тестирования
    prober: HttpProber,
    /// Менеджер залоченных стратегий
    locked_manager: Arc<LockedStrategiesManager>,
    /// Менеджер заблокированных стратегий
    blocked_manager: Arc<BlockedStrategiesManager>,
    /// Менеджер истории
    history_manager: Arc<StrategyHistoryManager>,
    /// Конфигурация
    config: MonitorConfig,
    /// Флаг работы
    running: AtomicBool,
    /// Состояния доменов
    domain_states: RwLock<HashMap<String, DomainState>>,
    /// Канал событий
    event_tx: broadcast::Sender<AutomationEvent>,
}

impl DomainMonitor {
    /// Создаёт новый монитор
    pub fn new(
        locked_manager: Arc<LockedStrategiesManager>,
        blocked_manager: Arc<BlockedStrategiesManager>,
        history_manager: Arc<StrategyHistoryManager>,
        config: MonitorConfig,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(100);

        let prober_config = ProbeConfig::new(config.test_timeout, config.test_timeout / 2);

        Self {
            prober: HttpProber::new(prober_config),
            locked_manager,
            blocked_manager,
            history_manager,
            config,
            running: AtomicBool::new(false),
            domain_states: RwLock::new(HashMap::new()),
            event_tx,
        }
    }

    /// Создаёт монитор с конфигурацией по умолчанию
    pub fn with_default_config(
        locked_manager: Arc<LockedStrategiesManager>,
        blocked_manager: Arc<BlockedStrategiesManager>,
        history_manager: Arc<StrategyHistoryManager>,
    ) -> Self {
        Self::new(
            locked_manager,
            blocked_manager,
            history_manager,
            MonitorConfig::default(),
        )
    }

    /// Подписывается на события
    pub fn subscribe(&self) -> broadcast::Receiver<AutomationEvent> {
        self.event_tx.subscribe()
    }

    /// Запускает мониторинг для списка доменов
    pub async fn start(&self, domains: &[String], strategies: &[Strategy]) -> Result<()> {
        // Проверяем, не запущен ли уже
        if self.running.load(Ordering::SeqCst) {
            warn!("Monitor is already running");
            return Ok(());
        }

        // Устанавливаем флаг работы
        self.running.store(true, Ordering::SeqCst);

        info!(domains = ?domains, "Starting domain monitor");

        // Emit событие старта
        let _ = self.event_tx.send(AutomationEvent::MonitorStarted {
            domains: domains.to_vec(),
        });

        // Инициализируем состояния доменов
        {
            let mut states = self.domain_states.write().await;
            states.clear();
            for domain in domains {
                if let Some(strategy) = strategies.first() {
                    states.insert(domain.clone(), DomainState::new(strategy.id.clone()));
                }
            }
        }

        // Основной цикл мониторинга
        while self.running.load(Ordering::SeqCst) {
            let mut all_locked = true;

            for domain in domains {
                // Проверяем, не остановлен ли монитор
                if !self.running.load(Ordering::SeqCst) {
                    break;
                }

                // Пропускаем залоченные домены
                if self.is_domain_locked(domain).await {
                    debug!(domain = %domain, "Skipping locked domain");
                    continue;
                }

                all_locked = false;

                // Получаем следующую стратегию
                if let Some(strategy) = self.get_next_strategy(domain, strategies).await {
                    debug!(
                        domain = %domain,
                        strategy_id = %strategy.id,
                        "Testing domain with strategy"
                    );

                    // Тестируем
                    let success = self.test_domain(domain, &strategy).await;

                    // Обрабатываем результат
                    if success {
                        self.on_success(domain, &strategy.id).await;
                    } else {
                        self.on_failure(domain, &strategy.id).await;
                    }

                    // Пауза между доменами
                    tokio::time::sleep(self.config.domain_delay).await;
                }
            }

            // Если все домены залочены, завершаем
            if all_locked {
                info!("All domains are locked, stopping monitor");
                break;
            }

            // Пауза между циклами
            tokio::time::sleep(self.config.cycle_delay).await;
        }

        // Сбрасываем флаг работы
        self.running.store(false, Ordering::SeqCst);

        // Emit событие остановки
        let _ = self.event_tx.send(AutomationEvent::MonitorStopped);

        info!("Domain monitor stopped");
        Ok(())
    }

    /// Останавливает мониторинг
    pub async fn stop(&self) {
        info!("Stopping domain monitor");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Проверяет, работает ли монитор
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Получает статус домена
    pub async fn get_domain_status(&self, domain: &str) -> DomainStatus {
        let states = self.domain_states.read().await;
        states
            .get(domain)
            .map(|s| s.status.clone())
            .unwrap_or(DomainStatus::Unknown)
    }

    /// Получает все статусы доменов
    pub async fn get_all_statuses(&self) -> HashMap<String, DomainStatus> {
        let states = self.domain_states.read().await;
        states
            .iter()
            .map(|(k, v)| (k.clone(), v.status.clone()))
            .collect()
    }

    /// Сбрасывает состояние домена
    pub async fn reset_domain(&self, domain: &str) {
        let mut states = self.domain_states.write().await;
        if let Some(state) = states.get_mut(domain) {
            state.successes = 0;
            state.failures = 0;
            state.status = DomainStatus::Testing;
            info!(domain = %domain, "Domain state reset");
        }
    }

    /// Сбрасывает все состояния
    pub async fn reset_all(&self) {
        let mut states = self.domain_states.write().await;
        states.clear();
        info!("All domain states reset");
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Проверяет, залочен ли домен
    async fn is_domain_locked(&self, domain: &str) -> bool {
        let states = self.domain_states.read().await;
        states
            .get(domain)
            .map(|s| s.status == DomainStatus::Locked)
            .unwrap_or(false)
    }

    /// Получает следующую стратегию для домена (circular)
    async fn get_next_strategy<'a>(
        &self,
        domain: &str,
        strategies: &'a [Strategy],
    ) -> Option<&'a Strategy> {
        if strategies.is_empty() {
            return None;
        }

        let mut states = self.domain_states.write().await;
        let state = states
            .entry(domain.to_string())
            .or_insert_with(|| DomainState::new(strategies[0].id.clone()));

        // Фильтруем заблокированные стратегии
        let available: Vec<_> = strategies
            .iter()
            .enumerate()
            .filter(|(_, _s)| {
                // Синхронная проверка невозможна, пропускаем blocked check здесь
                // В реальном коде нужно предварительно отфильтровать
                true
            })
            .collect();

        if available.is_empty() {
            return None;
        }

        // Circular перебор
        let idx = state.strategy_index % available.len();
        state.strategy_index = (state.strategy_index + 1) % available.len();

        let (_, strategy) = available[idx];
        state.strategy_id = strategy.id.clone();

        Some(strategy)
    }

    /// Тестирует домен
    async fn test_domain(&self, domain: &str, _strategy: &Strategy) -> bool {
        let url = format!("https://{}/", domain);
        let endpoint = TestEndpoint::critical(&url, domain);

        let result = self.prober.probe_direct(&endpoint).await;

        // Критерий успеха: успешный запрос
        result.success
    }

    /// Обрабатывает успешный тест
    async fn on_success(&self, domain: &str, strategy_id: &str) {
        let should_lock = {
            let mut states = self.domain_states.write().await;
            let state = states
                .entry(domain.to_string())
                .or_insert_with(|| DomainState::new(strategy_id.to_string()));

            // Если стратегия изменилась, сбрасываем счётчики
            if state.strategy_id != strategy_id {
                debug!(
                    domain = %domain,
                    old_strategy = %state.strategy_id,
                    new_strategy = %strategy_id,
                    "Strategy changed, resetting counters"
                );
                state.strategy_id = strategy_id.to_string();
                state.successes = 0;
                state.failures = 0;
                state.status = DomainStatus::Testing;
            }

            state.successes += 1;
            state.failures = 0; // Сбрасываем failures при успехе

            info!(
                domain = %domain,
                strategy_id = %strategy_id,
                successes = state.successes,
                "Test SUCCESS"
            );

            // LOCK после N успехов
            if state.successes >= self.config.lock_threshold
                && state.status != DomainStatus::Locked
            {
                state.status = DomainStatus::Locked;
                true
            } else {
                false
            }
        };

        // Записываем в историю
        if let Err(e) = self.history_manager.record_success(domain, strategy_id).await {
            warn!(error = %e, "Failed to record success in history");
        }

        // Lock в менеджере и emit событие
        if should_lock {
            info!("🔒 LOCKED: {} -> {}", domain, strategy_id);

            if let Err(e) = self
                .locked_manager
                .lock(domain, strategy_id, Protocol::Tls)
                .await
            {
                warn!(error = %e, "Failed to lock strategy in manager");
            }

            let _ = self.event_tx.send(AutomationEvent::DomainLocked {
                domain: domain.to_string(),
                strategy_id: strategy_id.to_string(),
                protocol: "tls".to_string(),
            });
        }
    }

    /// Обрабатывает неудачный тест
    async fn on_failure(&self, domain: &str, strategy_id: &str) {
        let (should_unlock, should_block) = {
            let mut states = self.domain_states.write().await;
            let state = states
                .entry(domain.to_string())
                .or_insert_with(|| DomainState::new(strategy_id.to_string()));

            // Если стратегия изменилась, сбрасываем счётчики
            if state.strategy_id != strategy_id {
                state.strategy_id = strategy_id.to_string();
                state.successes = 0;
                state.failures = 0;
                state.status = DomainStatus::Testing;
            }

            state.failures += 1;

            warn!(
                domain = %domain,
                strategy_id = %strategy_id,
                failures = state.failures,
                "Test FAILURE"
            );

            let was_locked = state.status == DomainStatus::Locked;

            // UNLOCK после M неудач (если был заблокирован)
            let should_unlock =
                state.failures >= self.config.unlock_threshold && was_locked;

            if should_unlock {
                state.status = DomainStatus::Testing;
                state.successes = 0;
            }

            // Помечаем как Failed если много неудач подряд
            let should_block = state.failures >= self.config.unlock_threshold * 2;
            if should_block {
                state.status = DomainStatus::Failed;
            }

            (should_unlock, should_block)
        };

        // Записываем в историю
        if let Err(e) = self.history_manager.record_failure(domain, strategy_id).await {
            warn!(error = %e, "Failed to record failure in history");
        }

        // Unlock в менеджере и emit событие
        if should_unlock {
            info!("🔓 UNLOCKED: {} (was {})", domain, strategy_id);

            if let Err(e) = self.locked_manager.unlock(domain, Protocol::Tls).await {
                warn!(error = %e, "Failed to unlock strategy in manager");
            }

            let _ = self.event_tx.send(AutomationEvent::DomainUnlocked {
                domain: domain.to_string(),
                protocol: "tls".to_string(),
            });
        }

        // Block стратегию если слишком много неудач
        if should_block {
            warn!("❌ FAILED: {} with strategy {}", domain, strategy_id);

            if let Err(e) = self.blocked_manager.block(domain, strategy_id).await {
                warn!(error = %e, "Failed to block strategy");
            }

            let _ = self.event_tx.send(AutomationEvent::StrategyBlocked {
                domain: domain.to_string(),
                strategy_id: strategy_id.to_string(),
                reason: "Too many consecutive failures".to_string(),
            });
        }
    }
}

// ============================================================================
// Thread-safe wrapper
// ============================================================================

/// Thread-safe обёртка для DomainMonitor
pub type SharedDomainMonitor = Arc<DomainMonitor>;

/// Создаёт shared экземпляр монитора
pub fn create_monitor(
    locked_manager: Arc<LockedStrategiesManager>,
    blocked_manager: Arc<BlockedStrategiesManager>,
    history_manager: Arc<StrategyHistoryManager>,
    config: MonitorConfig,
) -> SharedDomainMonitor {
    Arc::new(DomainMonitor::new(
        locked_manager,
        blocked_manager,
        history_manager,
        config,
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.lock_threshold, 3);
        assert_eq!(config.unlock_threshold, 2);
        assert_eq!(config.test_timeout, Duration::from_secs(5));
        assert_eq!(config.min_bytes_success, 2048);
    }

    #[test]
    fn test_monitor_config_builder() {
        let config = MonitorConfig::with_thresholds(5, 3)
            .with_timeout(Duration::from_secs(10))
            .with_min_bytes(4096);

        assert_eq!(config.lock_threshold, 5);
        assert_eq!(config.unlock_threshold, 3);
        assert_eq!(config.test_timeout, Duration::from_secs(10));
        assert_eq!(config.min_bytes_success, 4096);
    }

    #[test]
    fn test_domain_state_new() {
        let state = DomainState::new("strategy-1".to_string());
        assert_eq!(state.strategy_id, "strategy-1");
        assert_eq!(state.successes, 0);
        assert_eq!(state.failures, 0);
        assert_eq!(state.status, DomainStatus::Testing);
        assert_eq!(state.strategy_index, 0);
    }

    #[test]
    fn test_domain_status_equality() {
        assert_eq!(DomainStatus::Testing, DomainStatus::Testing);
        assert_eq!(DomainStatus::Locked, DomainStatus::Locked);
        assert_ne!(DomainStatus::Testing, DomainStatus::Locked);
    }
}

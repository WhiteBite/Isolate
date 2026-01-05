//! Orchestra Engine - автоматический перебор стратегий для доменов
//!
//! Концепция:
//! - Автоматический перебор стратегий для каждого домена
//! - Детекция успеха/неудачи на основе полученных байт и latency
//! - Фиксация (LOCK) рабочих стратегий после N успехов
//! - Разблокировка (UNLOCK) после M неудач
//!
//! ## Использование
//!
//! ```rust,ignore
//! use crate::core::orchestra::{Orchestra, OrchestraConfig};
//!
//! let config = OrchestraConfig::default();
//! let orchestra = Orchestra::new(strategies, config);
//!
//! // Запуск автоматического перебора
//! orchestra.start(&["youtube.com", "discord.com"]).await?;
//!
//! // Получение заблокированных стратегий
//! let locked = orchestra.get_locked_strategies().await;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::strategy_loader::JsonStrategy;

// ============================================================================
// Data Structures
// ============================================================================

/// Статус стратегии для домена
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DomainStatus {
    /// Сейчас тестируется
    Testing,
    /// Зафиксирована (3+ успехов)
    Locked,
    /// Не работает
    Failed,
    /// Ещё не тестировалась
    Unknown,
}

impl Default for DomainStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Информация о заблокированной стратегии для домена
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedStrategy {
    /// ID стратегии
    pub strategy_id: String,
    /// Количество успешных тестов
    pub successes: u32,
    /// Количество неудачных тестов
    pub failures: u32,
    /// Время фиксации стратегии
    pub locked_at: Option<DateTime<Utc>>,
    /// Текущий статус
    pub status: DomainStatus,
}

impl LockedStrategy {
    /// Создаёт новую запись для домена
    fn new(strategy_id: String) -> Self {
        Self {
            strategy_id,
            successes: 0,
            failures: 0,
            locked_at: None,
            status: DomainStatus::Testing,
        }
    }
}

/// Результат теста домена
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Тестируемый домен
    pub domain: String,
    /// ID использованной стратегии
    pub strategy_id: String,
    /// Успешен ли тест
    pub success: bool,
    /// Задержка в миллисекундах
    pub latency_ms: Option<u64>,
    /// Количество полученных байт
    pub bytes_received: u64,
    /// Сообщение об ошибке (если есть)
    pub error: Option<String>,
}

impl TestResult {
    /// Создаёт успешный результат
    pub fn success(domain: String, strategy_id: String, latency_ms: u64, bytes_received: u64) -> Self {
        Self {
            domain,
            strategy_id,
            success: true,
            latency_ms: Some(latency_ms),
            bytes_received,
            error: None,
        }
    }

    /// Создаёт неудачный результат
    pub fn failure(domain: String, strategy_id: String, error: String) -> Self {
        Self {
            domain,
            strategy_id,
            success: false,
            latency_ms: None,
            bytes_received: 0,
            error: Some(error),
        }
    }
}

/// Конфигурация Orchestra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestraConfig {
    /// Количество успехов для LOCK (default: 3)
    pub lock_threshold: u32,
    /// Количество failures для UNLOCK (default: 2)
    pub unlock_threshold: u32,
    /// Таймаут теста в миллисекундах (default: 5000)
    pub test_timeout_ms: u64,
    /// Минимум байт для успеха (default: 2048)
    pub min_bytes_success: u64,
    /// Пауза между циклами в миллисекундах (default: 1000)
    pub cycle_delay_ms: u64,
    /// Пауза между тестами доменов в миллисекундах (default: 500)
    pub domain_delay_ms: u64,
}

impl Default for OrchestraConfig {
    fn default() -> Self {
        Self {
            lock_threshold: 3,
            unlock_threshold: 2,
            test_timeout_ms: 5000,
            min_bytes_success: 2048,
            cycle_delay_ms: 1000,
            domain_delay_ms: 500,
        }
    }
}

impl OrchestraConfig {
    /// Создаёт конфигурацию с кастомными порогами
    pub fn with_thresholds(lock_threshold: u32, unlock_threshold: u32) -> Self {
        Self {
            lock_threshold,
            unlock_threshold,
            ..Default::default()
        }
    }

    /// Устанавливает таймаут теста
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.test_timeout_ms = timeout_ms;
        self
    }

    /// Устанавливает минимум байт для успеха
    pub fn with_min_bytes(mut self, min_bytes: u64) -> Self {
        self.min_bytes_success = min_bytes;
        self
    }
}

// ============================================================================
// Orchestra Engine
// ============================================================================

/// Основной движок Orchestra для автоматического перебора стратегий
pub struct Orchestra {
    /// Список доступных стратегий
    strategies: Vec<JsonStrategy>,
    /// Карта заблокированных стратегий по доменам
    domain_locks: Arc<RwLock<HashMap<String, LockedStrategy>>>,
    /// Текущий индекс стратегии для circular перебора (per domain)
    domain_indices: Arc<RwLock<HashMap<String, usize>>>,
    /// Конфигурация
    config: OrchestraConfig,
    /// Флаг работы
    running: Arc<RwLock<bool>>,
}

impl Orchestra {
    /// Создаёт новый экземпляр Orchestra
    ///
    /// # Arguments
    /// * `strategies` - Список стратегий для перебора
    /// * `config` - Конфигурация Orchestra
    pub fn new(strategies: Vec<JsonStrategy>, config: OrchestraConfig) -> Self {
        info!(
            strategies_count = strategies.len(),
            lock_threshold = config.lock_threshold,
            unlock_threshold = config.unlock_threshold,
            "Creating Orchestra engine"
        );

        Self {
            strategies,
            domain_locks: Arc::new(RwLock::new(HashMap::new())),
            domain_indices: Arc::new(RwLock::new(HashMap::new())),
            config,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Создаёт Orchestra с конфигурацией по умолчанию
    pub fn with_default_config(strategies: Vec<JsonStrategy>) -> Self {
        Self::new(strategies, OrchestraConfig::default())
    }

    /// Запускает автоматический перебор для списка доменов
    ///
    /// Выполняет circular перебор стратегий для каждого домена,
    /// пропуская уже заблокированные домены.
    ///
    /// # Arguments
    /// * `domains` - Список доменов для тестирования
    pub async fn start(&self, domains: &[&str]) -> Result<()> {
        // Проверяем, не запущен ли уже
        {
            let running = self.running.read().await;
            if *running {
                warn!("Orchestra is already running");
                return Ok(());
            }
        }

        // Устанавливаем флаг работы
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        info!(domains = ?domains, "Starting Orchestra");

        // Основной цикл перебора
        while *self.running.read().await {
            let mut all_locked = true;

            for domain in domains {
                // Проверяем, не остановлен ли Orchestra
                if !*self.running.read().await {
                    break;
                }

                // Пропускаем заблокированные домены
                if self.is_domain_locked(domain).await {
                    debug!(domain = %domain, "Skipping locked domain");
                    continue;
                }

                all_locked = false;

                // Получаем следующую стратегию (circular)
                if let Some(strategy) = self.get_next_strategy(domain).await {
                    debug!(
                        domain = %domain,
                        strategy_id = %strategy.id,
                        "Testing domain with strategy"
                    );

                    // Тестируем
                    let result = self.test_domain(domain, strategy).await;

                    // Обрабатываем результат
                    if result.success {
                        self.on_success(&result).await;
                    } else {
                        self.on_failure(&result).await;
                    }

                    // Пауза между доменами
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        self.config.domain_delay_ms,
                    ))
                    .await;
                }
            }

            // Если все домены заблокированы, завершаем
            if all_locked {
                info!("All domains are locked, stopping Orchestra");
                break;
            }

            // Пауза между циклами
            tokio::time::sleep(tokio::time::Duration::from_millis(self.config.cycle_delay_ms))
                .await;
        }

        // Сбрасываем флаг работы
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        info!("Orchestra stopped");
        Ok(())
    }

    /// Останавливает перебор
    pub async fn stop(&self) {
        info!("Stopping Orchestra");
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Проверяет, работает ли Orchestra
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Обрабатывает успешный тест
    ///
    /// Увеличивает счётчик успехов и блокирует стратегию
    /// после достижения порога `lock_threshold`.
    pub async fn on_success(&self, result: &TestResult) {
        let mut locks = self.domain_locks.write().await;

        let entry = locks
            .entry(result.domain.clone())
            .or_insert_with(|| LockedStrategy::new(result.strategy_id.clone()));

        // Если стратегия изменилась, сбрасываем счётчики
        if entry.strategy_id != result.strategy_id {
            debug!(
                domain = %result.domain,
                old_strategy = %entry.strategy_id,
                new_strategy = %result.strategy_id,
                "Strategy changed, resetting counters"
            );
            entry.strategy_id = result.strategy_id.clone();
            entry.successes = 0;
            entry.failures = 0;
            entry.status = DomainStatus::Testing;
        }

        entry.successes += 1;
        entry.failures = 0; // Сбрасываем failures при успехе

        info!(
            domain = %result.domain,
            strategy_id = %result.strategy_id,
            successes = entry.successes,
            latency_ms = ?result.latency_ms,
            bytes = result.bytes_received,
            "Test SUCCESS"
        );

        // LOCK после N успехов
        if entry.successes >= self.config.lock_threshold && entry.status != DomainStatus::Locked {
            entry.status = DomainStatus::Locked;
            entry.locked_at = Some(Utc::now());
            info!(
                "🔒 LOCKED: {} -> {}",
                result.domain, result.strategy_id
            );
        }
    }

    /// Обрабатывает неудачный тест
    ///
    /// Увеличивает счётчик неудач и разблокирует стратегию
    /// после достижения порога `unlock_threshold`.
    pub async fn on_failure(&self, result: &TestResult) {
        let mut locks = self.domain_locks.write().await;

        let entry = locks
            .entry(result.domain.clone())
            .or_insert_with(|| LockedStrategy::new(result.strategy_id.clone()));

        // Если стратегия изменилась, сбрасываем счётчики
        if entry.strategy_id != result.strategy_id {
            entry.strategy_id = result.strategy_id.clone();
            entry.successes = 0;
            entry.failures = 0;
            entry.status = DomainStatus::Testing;
        }

        entry.failures += 1;

        warn!(
            domain = %result.domain,
            strategy_id = %result.strategy_id,
            failures = entry.failures,
            error = ?result.error,
            "Test FAILURE"
        );

        // UNLOCK после M неудач (если был заблокирован)
        if entry.failures >= self.config.unlock_threshold && entry.status == DomainStatus::Locked {
            entry.status = DomainStatus::Testing;
            entry.locked_at = None;
            entry.successes = 0;
            info!(
                "🔓 UNLOCKED: {} (was {})",
                result.domain, result.strategy_id
            );
        }

        // Помечаем как Failed если много неудач подряд
        if entry.failures >= self.config.unlock_threshold * 2 {
            entry.status = DomainStatus::Failed;
            warn!(
                "❌ FAILED: {} with strategy {}",
                result.domain, result.strategy_id
            );
        }
    }

    /// Получает следующую стратегию для домена (circular)
    ///
    /// Возвращает следующую стратегию в круговом порядке.
    /// Каждый домен имеет свой индекс для независимого перебора.
    pub async fn get_next_strategy(&self, domain: &str) -> Option<&JsonStrategy> {
        if self.strategies.is_empty() {
            return None;
        }

        let mut indices = self.domain_indices.write().await;
        let index = indices.entry(domain.to_string()).or_insert(0);

        let strategy = self.strategies.get(*index);

        // Переходим к следующей стратегии (circular)
        *index = (*index + 1) % self.strategies.len();

        strategy
    }

    /// Получает текущую стратегию для домена без инкремента
    pub async fn get_current_strategy(&self, domain: &str) -> Option<&JsonStrategy> {
        if self.strategies.is_empty() {
            return None;
        }

        let indices = self.domain_indices.read().await;
        let index = indices.get(domain).copied().unwrap_or(0);

        self.strategies.get(index)
    }

    /// Получает все заблокированные стратегии
    pub async fn get_locked_strategies(&self) -> HashMap<String, LockedStrategy> {
        self.domain_locks.read().await.clone()
    }

    /// Получает информацию о стратегии для конкретного домена
    pub async fn get_domain_info(&self, domain: &str) -> Option<LockedStrategy> {
        self.domain_locks.read().await.get(domain).cloned()
    }

    /// Проверяет, заблокирован ли домен
    pub async fn is_domain_locked(&self, domain: &str) -> bool {
        let locks = self.domain_locks.read().await;
        locks
            .get(domain)
            .map(|l| l.status == DomainStatus::Locked)
            .unwrap_or(false)
    }

    /// Сбрасывает блокировку для домена
    pub async fn unlock_domain(&self, domain: &str) {
        let mut locks = self.domain_locks.write().await;
        if let Some(entry) = locks.get_mut(domain) {
            entry.status = DomainStatus::Testing;
            entry.locked_at = None;
            entry.successes = 0;
            entry.failures = 0;
            info!("🔓 Manually unlocked: {}", domain);
        }
    }

    /// Сбрасывает все блокировки
    pub async fn reset_all(&self) {
        let mut locks = self.domain_locks.write().await;
        locks.clear();

        let mut indices = self.domain_indices.write().await;
        indices.clear();

        info!("Orchestra reset: all locks and indices cleared");
    }

    /// Тестирует один домен с указанной стратегией
    ///
    /// Выполняет HTTP-запрос к домену и определяет успех на основе:
    /// - Количества полученных байт (>= min_bytes_success)
    /// - Времени ответа (< test_timeout_ms)
    async fn test_domain(&self, domain: &str, strategy: &JsonStrategy) -> TestResult {
        let start = Instant::now();

        // TODO: Интеграция с реальным тестированием через strategy_engine
        // Пока используем заглушку для демонстрации логики

        // Симуляция теста (в реальности здесь будет HTTP-запрос через прокси)
        let test_result = self.perform_http_test(domain, strategy).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match test_result {
            Ok(bytes_received) => {
                // Проверяем критерии успеха
                let success = bytes_received >= self.config.min_bytes_success
                    && latency_ms < self.config.test_timeout_ms;

                if success {
                    TestResult::success(
                        domain.to_string(),
                        strategy.id.clone(),
                        latency_ms,
                        bytes_received,
                    )
                } else {
                    TestResult::failure(
                        domain.to_string(),
                        strategy.id.clone(),
                        format!(
                            "Insufficient response: {} bytes in {}ms",
                            bytes_received, latency_ms
                        ),
                    )
                }
            }
            Err(e) => TestResult::failure(domain.to_string(), strategy.id.clone(), e.to_string()),
        }
    }

    /// Выполняет HTTP-тест домена
    ///
    /// TODO: Реализовать реальное тестирование через strategy_engine
    async fn perform_http_test(
        &self,
        domain: &str,
        strategy: &JsonStrategy,
    ) -> Result<u64> {
        // Заглушка для демонстрации
        // В реальной реализации здесь будет:
        // 1. Запуск стратегии через strategy_engine
        // 2. HTTP-запрос через SOCKS-прокси или напрямую
        // 3. Измерение latency и bytes_received

        debug!(
            domain = %domain,
            strategy_id = %strategy.id,
            "Performing HTTP test (stub)"
        );

        // Симуляция задержки
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Возвращаем случайный результат для демонстрации
        // В реальности здесь будут реальные данные
        Ok(4096)
    }
}

// ============================================================================
// Thread-safe wrapper
// ============================================================================

/// Thread-safe обёртка для Orchestra
pub type SharedOrchestra = Arc<Orchestra>;

/// Создаёт shared экземпляр Orchestra
pub fn create_orchestra(strategies: Vec<JsonStrategy>, config: OrchestraConfig) -> SharedOrchestra {
    Arc::new(Orchestra::new(strategies, config))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::strategy_loader::{StrategyCategory, StrategyPorts, StrategyProfile};

    fn create_test_strategy(id: &str) -> JsonStrategy {
        JsonStrategy {
            id: id.to_string(),
            name: format!("Test Strategy {}", id),
            description: "Test strategy".to_string(),
            category: StrategyCategory::YouTube,
            family: "zapret".to_string(),
            author: None,
            label: None,
            ports: StrategyPorts::default(),
            profiles: vec![StrategyProfile {
                filter: "tcp".to_string(),
                hostlist: None,
                hostlist_exclude: None,
                hostlist_domains: None,
                ipset: None,
                ipset_exclude: None,
                l7: None,
                ip_id: None,
                desync: "fake".to_string(),
                repeats: None,
                split_seqovl: None,
                split_pos: None,
                split_seqovl_pattern: None,
                fooling: None,
                fake_tls: None,
                fake_quic: None,
                fake_tls_mod: None,
                fake_wireguard: None,
                fake_dht: None,
                fake_unknown_udp: None,
                fake_tcp_mod: None,
                fake_syndata: None,
                ttl: None,
                ttl6: None,
                autottl: None,
                badseq_increment: None,
                badack_increment: None,
                ts_increment: None,
                cutoff: None,
                hostfakesplit_mod: None,
                hostfakesplit_midhost: None,
                fakedsplit_mod: None,
                wsize: None,
                wssize: None,
                wssize_cutoff: None,
                filter_l3: None,
                filter_ssid: None,
                nlm_filter: None,
                dup: None,
                dup_replace: None,
                dup_ttl: None,
                dup_autottl: None,
                dup_fooling: None,
                dup_start: None,
                dup_cutoff: None,
                orig_ttl: None,
                orig_autottl: None,
                orig_tcp_flags_set: None,
                orig_tcp_flags_unset: None,
                orig_mod_start: None,
                orig_mod_cutoff: None,
            }],
        }
    }

    #[test]
    fn test_orchestra_config_default() {
        let config = OrchestraConfig::default();
        assert_eq!(config.lock_threshold, 3);
        assert_eq!(config.unlock_threshold, 2);
        assert_eq!(config.test_timeout_ms, 5000);
        assert_eq!(config.min_bytes_success, 2048);
    }

    #[test]
    fn test_orchestra_config_builder() {
        let config = OrchestraConfig::with_thresholds(5, 3)
            .with_timeout(10000)
            .with_min_bytes(4096);

        assert_eq!(config.lock_threshold, 5);
        assert_eq!(config.unlock_threshold, 3);
        assert_eq!(config.test_timeout_ms, 10000);
        assert_eq!(config.min_bytes_success, 4096);
    }

    #[test]
    fn test_domain_status_default() {
        let status = DomainStatus::default();
        assert_eq!(status, DomainStatus::Unknown);
    }

    #[test]
    fn test_test_result_success() {
        let result = TestResult::success(
            "youtube.com".to_string(),
            "strategy-1".to_string(),
            150,
            4096,
        );

        assert!(result.success);
        assert_eq!(result.domain, "youtube.com");
        assert_eq!(result.strategy_id, "strategy-1");
        assert_eq!(result.latency_ms, Some(150));
        assert_eq!(result.bytes_received, 4096);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_test_result_failure() {
        let result = TestResult::failure(
            "youtube.com".to_string(),
            "strategy-1".to_string(),
            "Connection timeout".to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.domain, "youtube.com");
        assert_eq!(result.error, Some("Connection timeout".to_string()));
        assert_eq!(result.bytes_received, 0);
    }

    #[tokio::test]
    async fn test_orchestra_creation() {
        let strategies = vec![
            create_test_strategy("s1"),
            create_test_strategy("s2"),
        ];
        let config = OrchestraConfig::default();
        let orchestra = Orchestra::new(strategies, config);

        assert!(!orchestra.is_running().await);
        assert!(orchestra.get_locked_strategies().await.is_empty());
    }

    #[tokio::test]
    async fn test_get_next_strategy_circular() {
        let strategies = vec![
            create_test_strategy("s1"),
            create_test_strategy("s2"),
            create_test_strategy("s3"),
        ];
        let orchestra = Orchestra::with_default_config(strategies);

        // First cycle
        let s1 = orchestra.get_next_strategy("test.com").await.unwrap();
        assert_eq!(s1.id, "s1");

        let s2 = orchestra.get_next_strategy("test.com").await.unwrap();
        assert_eq!(s2.id, "s2");

        let s3 = orchestra.get_next_strategy("test.com").await.unwrap();
        assert_eq!(s3.id, "s3");

        // Circular - back to s1
        let s1_again = orchestra.get_next_strategy("test.com").await.unwrap();
        assert_eq!(s1_again.id, "s1");
    }

    #[tokio::test]
    async fn test_on_success_locks_after_threshold() {
        let strategies = vec![create_test_strategy("s1")];
        let config = OrchestraConfig::with_thresholds(3, 2);
        let orchestra = Orchestra::new(strategies, config);

        let result = TestResult::success(
            "youtube.com".to_string(),
            "s1".to_string(),
            100,
            4096,
        );

        // First success
        orchestra.on_success(&result).await;
        assert!(!orchestra.is_domain_locked("youtube.com").await);

        // Second success
        orchestra.on_success(&result).await;
        assert!(!orchestra.is_domain_locked("youtube.com").await);

        // Third success - should lock
        orchestra.on_success(&result).await;
        assert!(orchestra.is_domain_locked("youtube.com").await);

        let info = orchestra.get_domain_info("youtube.com").await.unwrap();
        assert_eq!(info.status, DomainStatus::Locked);
        assert_eq!(info.successes, 3);
        assert!(info.locked_at.is_some());
    }

    #[tokio::test]
    async fn test_on_failure_unlocks_after_threshold() {
        let strategies = vec![create_test_strategy("s1")];
        let config = OrchestraConfig::with_thresholds(2, 2);
        let orchestra = Orchestra::new(strategies, config);

        // Lock the domain first
        let success = TestResult::success("youtube.com".to_string(), "s1".to_string(), 100, 4096);
        orchestra.on_success(&success).await;
        orchestra.on_success(&success).await;
        assert!(orchestra.is_domain_locked("youtube.com").await);

        // Now fail
        let failure = TestResult::failure(
            "youtube.com".to_string(),
            "s1".to_string(),
            "Timeout".to_string(),
        );

        orchestra.on_failure(&failure).await;
        assert!(orchestra.is_domain_locked("youtube.com").await); // Still locked

        orchestra.on_failure(&failure).await;
        assert!(!orchestra.is_domain_locked("youtube.com").await); // Unlocked
    }

    #[tokio::test]
    async fn test_manual_unlock() {
        let strategies = vec![create_test_strategy("s1")];
        let config = OrchestraConfig::with_thresholds(1, 2);
        let orchestra = Orchestra::new(strategies, config);

        let result = TestResult::success("youtube.com".to_string(), "s1".to_string(), 100, 4096);
        orchestra.on_success(&result).await;
        assert!(orchestra.is_domain_locked("youtube.com").await);

        orchestra.unlock_domain("youtube.com").await;
        assert!(!orchestra.is_domain_locked("youtube.com").await);
    }

    #[tokio::test]
    async fn test_reset_all() {
        let strategies = vec![create_test_strategy("s1")];
        let config = OrchestraConfig::with_thresholds(1, 2);
        let orchestra = Orchestra::new(strategies, config);

        let result = TestResult::success("youtube.com".to_string(), "s1".to_string(), 100, 4096);
        orchestra.on_success(&result).await;

        // Advance index
        orchestra.get_next_strategy("youtube.com").await;

        orchestra.reset_all().await;

        assert!(orchestra.get_locked_strategies().await.is_empty());
        // Index should be reset to 0
        let s = orchestra.get_next_strategy("youtube.com").await.unwrap();
        assert_eq!(s.id, "s1");
    }
}

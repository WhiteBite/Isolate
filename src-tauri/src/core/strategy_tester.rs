//! Strategy Tester - тестирование стратегий через DPI-симулятор
//!
//! Запускает стратегию и проверяет её эффективность через DPI-симулятор
//! в Hyper-V VM. Тестирование происходит через SSH в VM.
//!
//! ## Интеграция с Strategy Engine
//! Тестер использует `SharedStrategyEngine` для запуска/остановки стратегий
//! во время тестирования. Полный цикл:
//! 1. Сброс статистики DPI
//! 2. Проверка блокировки БЕЗ стратегии
//! 3. Запуск стратегии через engine
//! 4. Ожидание применения (2 сек)
//! 5. Проверка доступности СО стратегией
//! 6. Остановка стратегии

use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::Strategy;
use crate::core::strategy_engine::SharedStrategyEngine;

/// Задержка после запуска стратегии для применения (мс)
const STRATEGY_APPLY_DELAY_MS: u64 = 2000;

/// Конфигурация DPI-симулятора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpiSimulatorConfig {
    /// SSH хост (user@host)
    pub ssh_host: String,
    /// API URL симулятора (через SSH туннель)
    pub api_url: String,
    /// Таймаут теста в секундах
    pub test_timeout_secs: u32,
    /// Домен для тестирования
    pub test_domain: String,
}

impl Default for DpiSimulatorConfig {
    fn default() -> Self {
        Self {
            // WindhawkTest VM - Windows test client behind DPI
            ssh_host: "VM-test@192.168.100.20".to_string(),
            // DPI-Simulator API (via SSH tunnel to 192.168.100.10)
            api_url: "http://localhost:8888".to_string(),
            test_timeout_secs: 10,
            test_domain: "youtube.com".to_string(),
        }
    }
}

/// Результат тестирования стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTestResult {
    pub strategy_id: String,
    pub success: bool,
    pub blocked_before: u32,
    pub blocked_after: u32,
    pub passed_after: u32,
    pub latency_ms: Option<u32>,
    pub error: Option<String>,
}

/// Статистика DPI-симулятора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpiStats {
    pub total_packets: u32,
    pub blocked_sni: u32,
    pub blocked_http: u32,
    pub blocked_quic: u32,
    pub passed: u32,
}

/// Тестер стратегий через DPI-симулятор
pub struct StrategyTester {
    config: DpiSimulatorConfig,
    /// Опциональная ссылка на движок стратегий
    engine: Option<SharedStrategyEngine>,
}

impl StrategyTester {
    /// Создаёт новый тестер с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: DpiSimulatorConfig::default(),
            engine: None,
        }
    }

    /// Создаёт тестер с кастомной конфигурацией
    pub fn with_config(config: DpiSimulatorConfig) -> Self {
        Self { 
            config,
            engine: None,
        }
    }

    /// Создаёт тестер с движком стратегий
    pub fn with_engine(engine: SharedStrategyEngine) -> Self {
        Self {
            config: DpiSimulatorConfig::default(),
            engine: Some(engine),
        }
    }

    /// Создаёт тестер с конфигурацией и движком
    pub fn with_config_and_engine(config: DpiSimulatorConfig, engine: SharedStrategyEngine) -> Self {
        Self {
            config,
            engine: Some(engine),
        }
    }

    /// Устанавливает движок стратегий
    pub fn set_engine(&mut self, engine: SharedStrategyEngine) {
        self.engine = Some(engine);
    }

    /// Проверяет доступность DPI-симулятора
    pub async fn check_availability(&self) -> Result<bool> {
        let status = self.get_dpi_status().await?;
        Ok(status.contains("running"))
    }

    /// Получает статус DPI-симулятора
    async fn get_dpi_status(&self) -> Result<String> {
        let output = Command::new("curl")
            .args(["-s", &format!("{}/status", self.config.api_url)])
            .output()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to get DPI status: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Получает статистику DPI-симулятора
    pub async fn get_stats(&self) -> Result<DpiStats> {
        let output = Command::new("curl")
            .args(["-s", &format!("{}/stats", self.config.api_url)])
            .output()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to get DPI stats: {}", e)))?;

        let json = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&json)
            .map_err(|e| IsolateError::Config(format!("Failed to parse DPI stats: {}", e)))
    }

    /// Сбрасывает статистику DPI-симулятора
    pub async fn reset_stats(&self) -> Result<()> {
        Command::new("curl")
            .args([
                "-s",
                "-X", "POST",
                &format!("{}/reset-stats", self.config.api_url),
            ])
            .output()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to reset DPI stats: {}", e)))?;

        debug!("DPI stats reset");
        Ok(())
    }

    /// Устанавливает режим блокировки DPI
    pub async fn set_mode(&self, mode: &str) -> Result<()> {
        Command::new("curl")
            .args([
                "-s",
                "-X", "POST",
                "-H", "Content-Type: application/json",
                "-d", &format!(r#"{{"mode":"{}"}}"#, mode),
                &format!("{}/mode", self.config.api_url),
            ])
            .output()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to set DPI mode: {}", e)))?;

        info!(mode, "DPI mode set");
        Ok(())
    }

    /// Тестирует подключение к домену через VM (без стратегии)
    pub async fn test_connection(&self, domain: &str) -> Result<bool> {
        let result = self.run_curl_test(domain).await?;
        Ok(result.success)
    }

    /// Запускает curl тест через SSH в VM
    /// Поддерживает как Linux (curl), так и Windows (curl.exe) VM
    async fn run_curl_test(&self, domain: &str) -> Result<CurlTestResult> {
        let start = std::time::Instant::now();
        
        // Определяем команду curl в зависимости от целевой ОС
        // Windows VM использует curl.exe, Linux - curl
        let is_windows_vm = self.config.ssh_host.contains("VM-test");
        let curl_cmd = if is_windows_vm {
            format!(
                "curl.exe -s --connect-timeout {} -o NUL -w \"%%{{http_code}}\" https://{}",
                self.config.test_timeout_secs - 2,
                domain
            )
        } else {
            format!(
                "curl -s --connect-timeout {} -o /dev/null -w '%{{http_code}}' https://{}",
                self.config.test_timeout_secs - 2,
                domain
            )
        };
        
        let output = timeout(
            Duration::from_secs(self.config.test_timeout_secs as u64),
            Command::new("ssh")
                .args([
                    "-o", "BatchMode=yes",
                    "-o", "StrictHostKeyChecking=no",
                    &self.config.ssh_host,
                    &curl_cmd,
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| IsolateError::StrategyTimeout(5000))?
        .map_err(|e| IsolateError::Process(format!("SSH curl failed: {}", e)))?;

        let latency_ms = start.elapsed().as_millis() as u32;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Парсим HTTP код
        let http_code: u16 = stdout.parse().unwrap_or(0);
        let success = http_code >= 200 && http_code < 400;

        debug!(
            domain,
            http_code,
            success,
            latency_ms,
            exit_code = output.status.code(),
            "Curl test completed"
        );

        Ok(CurlTestResult {
            success,
            http_code,
            latency_ms,
        })
    }

    /// Тестирует стратегию через DPI-симулятор
    ///
    /// Алгоритм:
    /// 1. Сбросить статистику DPI
    /// 2. Проверить что домен блокируется (без стратегии)
    /// 3. Запустить стратегию
    /// 4. Проверить что домен доступен (со стратегией)
    /// 5. Остановить стратегию
    /// 6. Вернуть результат
    pub async fn test_strategy(&self, strategy: &Strategy) -> Result<StrategyTestResult> {
        // Если есть engine - используем полный цикл тестирования
        if let Some(ref engine) = self.engine {
            return self.test_strategy_with_engine_internal(strategy, engine).await;
        }

        // Fallback: тестирование без engine (только проверка DPI)
        self.test_strategy_without_engine(strategy).await
    }

    /// Тестирует стратегию с использованием переданного движка
    ///
    /// Полный цикл тестирования:
    /// 1. Сброс статистики DPI
    /// 2. Проверка блокировки БЕЗ стратегии
    /// 3. Запуск стратегии через engine.start_global()
    /// 4. Ожидание 2 секунды для применения
    /// 5. Проверка доступности СО стратегией
    /// 6. Остановка стратегии через engine.stop_global()
    /// 7. Возврат результата
    pub async fn test_strategy_with_engine(
        &self,
        strategy: &Strategy,
        engine: &SharedStrategyEngine,
    ) -> Result<StrategyTestResult> {
        self.test_strategy_with_engine_internal(strategy, engine).await
    }

    /// Внутренняя реализация тестирования с engine
    async fn test_strategy_with_engine_internal(
        &self,
        strategy: &Strategy,
        engine: &SharedStrategyEngine,
    ) -> Result<StrategyTestResult> {
        info!(
            strategy_id = %strategy.id,
            strategy_name = %strategy.name,
            "Starting strategy test with engine"
        );

        // Проверяем поддержку глобального режима
        if !strategy.mode_capabilities.supports_global {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' does not support GLOBAL mode required for testing",
                strategy.id
            )));
        }

        // 1. Сбросить статистику DPI
        if let Err(e) = self.reset_stats().await {
            warn!(error = %e, "Failed to reset DPI stats, continuing anyway");
        }

        // 2. Проверить блокировку без стратегии
        let stats_before = self.get_stats().await.unwrap_or_else(|_| DpiStats {
            total_packets: 0,
            blocked_sni: 0,
            blocked_http: 0,
            blocked_quic: 0,
            passed: 0,
        });
        let blocked_before = stats_before.blocked_sni;

        let test_without = self.run_curl_test(&self.config.test_domain).await?;
        
        if test_without.success {
            warn!(
                strategy_id = %strategy.id,
                domain = %self.config.test_domain,
                "Domain is accessible without strategy - DPI may not be blocking"
            );
        }

        // Получаем статистику после теста без стратегии
        let stats_after_block = self.get_stats().await.unwrap_or(stats_before.clone());
        let blocked_without_strategy = stats_after_block.blocked_sni - blocked_before;

        debug!(
            blocked_before,
            blocked_after = stats_after_block.blocked_sni,
            blocked_diff = blocked_without_strategy,
            "Blocked packets before strategy"
        );

        // 3. Запустить стратегию через engine
        info!(
            strategy_id = %strategy.id,
            "Starting strategy via engine"
        );

        if let Err(e) = engine.start_global(strategy).await {
            error!(
                strategy_id = %strategy.id,
                error = %e,
                "Failed to start strategy"
            );
            return Ok(StrategyTestResult {
                strategy_id: strategy.id.clone(),
                success: false,
                blocked_before: blocked_without_strategy,
                blocked_after: 0,
                passed_after: 0,
                latency_ms: None,
                error: Some(format!("Failed to start strategy: {}", e)),
            });
        }

        // 4. Подождать 2 секунды для применения стратегии
        debug!(
            delay_ms = STRATEGY_APPLY_DELAY_MS,
            "Waiting for strategy to apply"
        );
        tokio::time::sleep(Duration::from_millis(STRATEGY_APPLY_DELAY_MS)).await;

        // 5. Сбросить статистику и проверить доступность со стратегией
        if let Err(e) = self.reset_stats().await {
            warn!(error = %e, "Failed to reset DPI stats before test with strategy");
        }

        let test_with = self.run_curl_test(&self.config.test_domain).await;
        let stats_after = self.get_stats().await.unwrap_or_else(|_| DpiStats {
            total_packets: 0,
            blocked_sni: 0,
            blocked_http: 0,
            blocked_quic: 0,
            passed: 0,
        });

        // 6. Остановить стратегию
        info!(
            strategy_id = %strategy.id,
            "Stopping strategy via engine"
        );

        if let Err(e) = engine.stop_global().await {
            warn!(
                strategy_id = %strategy.id,
                error = %e,
                "Failed to stop strategy cleanly"
            );
        }

        // 7. Формируем результат
        let (success, latency_ms, error_msg) = match test_with {
            Ok(result) => (
                result.success,
                Some(result.latency_ms),
                if result.success {
                    None
                } else {
                    Some(format!("HTTP {}", result.http_code))
                },
            ),
            Err(e) => (false, None, Some(e.to_string())),
        };

        let result = StrategyTestResult {
            strategy_id: strategy.id.clone(),
            success,
            blocked_before: blocked_without_strategy,
            blocked_after: stats_after.blocked_sni,
            passed_after: stats_after.passed,
            latency_ms,
            error: error_msg,
        };

        info!(
            strategy_id = %strategy.id,
            success = result.success,
            blocked_before = result.blocked_before,
            blocked_after = result.blocked_after,
            passed_after = result.passed_after,
            latency_ms = ?result.latency_ms,
            "Strategy test completed"
        );

        Ok(result)
    }

    /// Тестирование без engine (legacy, только проверка DPI)
    async fn test_strategy_without_engine(&self, strategy: &Strategy) -> Result<StrategyTestResult> {
        info!(
            strategy_id = %strategy.id,
            strategy_name = %strategy.name,
            "Starting strategy test (without engine - limited functionality)"
        );

        // 1. Сбросить статистику
        self.reset_stats().await?;

        // 2. Проверить блокировку без стратегии
        let stats_before = self.get_stats().await?;
        let blocked_before = stats_before.blocked_sni;

        let test_without = self.run_curl_test(&self.config.test_domain).await?;
        
        if test_without.success {
            warn!(
                strategy_id = %strategy.id,
                "Domain is not blocked without strategy - DPI may not be working"
            );
        }

        // Получаем статистику после теста без стратегии
        let stats_after_block = self.get_stats().await?;
        let blocked_after_test = stats_after_block.blocked_sni;

        debug!(
            blocked_before,
            blocked_after_test,
            "Blocked packets before strategy"
        );

        // Без engine мы не можем запустить стратегию
        // Возвращаем результат только с информацией о блокировке
        warn!(
            strategy_id = %strategy.id,
            "No engine available - cannot test strategy effectiveness"
        );

        // Для теста - просто проверяем ещё раз
        self.reset_stats().await?;
        let test_with = self.run_curl_test(&self.config.test_domain).await?;
        let stats_after = self.get_stats().await?;

        let result = StrategyTestResult {
            strategy_id: strategy.id.clone(),
            success: test_with.success,
            blocked_before: blocked_after_test - blocked_before,
            blocked_after: stats_after.blocked_sni,
            passed_after: stats_after.passed,
            latency_ms: Some(test_with.latency_ms),
            error: if test_with.success {
                None
            } else {
                Some(format!("HTTP {} (no engine - strategy not applied)", test_with.http_code))
            },
        };

        info!(
            strategy_id = %strategy.id,
            success = result.success,
            blocked_before = result.blocked_before,
            blocked_after = result.blocked_after,
            "Strategy test completed (without engine)"
        );

        Ok(result)
    }
}

impl Default for StrategyTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Результат curl теста
#[derive(Debug)]
struct CurlTestResult {
    success: bool,
    http_code: u16,
    latency_ms: u32,
}

/// Создаёт тестер с движком стратегий (удобная функция)
pub fn create_tester_with_engine(engine: SharedStrategyEngine) -> StrategyTester {
    StrategyTester::with_engine(engine)
}

/// Создаёт тестер с конфигурацией и движком (удобная функция)
pub fn create_tester(config: DpiSimulatorConfig, engine: SharedStrategyEngine) -> StrategyTester {
    StrategyTester::with_config_and_engine(config, engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DpiSimulatorConfig::default();
        assert_eq!(config.ssh_host, "VM-test@192.168.100.20");
        assert_eq!(config.api_url, "http://localhost:8888");
        assert_eq!(config.test_timeout_secs, 10);
        assert_eq!(config.test_domain, "youtube.com");
    }
}

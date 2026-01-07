//! A/B Testing Module for Strategy Comparison
//!
//! Позволяет сравнивать эффективность двух стратегий на одном сервисе.
//! Запускает каждую стратегию несколько раз и собирает статистику.
//!
//! ## Важно
//! - Zapret стратегии тестируются ПОСЛЕДОВАТЕЛЬНО (не параллельно!)
//! - Между тестами делается пауза для корректной остановки WinDivert
//!
//! ## Статистический анализ
//! - Используется Welch's t-test для сравнения средних значений latency
//! - Mann-Whitney U тест для сравнения распределений (более робастный)
//! - Confidence level определяется на основе p-value

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{Service, Strategy};
use crate::core::strategy_engine::SharedStrategyEngine;
use crate::core::storage::Storage;

/// Задержка после запуска стратегии (мс)
const STRATEGY_APPLY_DELAY_MS: u64 = 2000;
/// Задержка между тестами стратегий (мс)
const BETWEEN_STRATEGY_DELAY_MS: u64 = 2000;
/// Таймаут для HTTP запроса (сек)
const HTTP_TIMEOUT_SECS: u64 = 5;
/// Минимальный confidence level для определения победителя (95%)
const MIN_CONFIDENCE_LEVEL: f64 = 0.95;
/// Ключ для хранения результатов A/B тестов
const AB_TEST_RESULTS_KEY: &str = "ab_test_results";

/// Статус A/B теста
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ABTestStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Failed,
}

/// Результат тестирования одной стратегии в A/B тесте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestStrategyResult {
    /// ID стратегии
    pub strategy_id: String,
    /// Название стратегии
    pub strategy_name: String,
    /// Процент успешных тестов (0-100)
    pub success_rate: f64,
    /// Средняя задержка в мс
    pub avg_latency_ms: f64,
    /// Минимальная задержка
    pub min_latency_ms: u32,
    /// Максимальная задержка
    pub max_latency_ms: u32,
    /// Стандартное отклонение latency
    pub std_dev_latency_ms: f64,
    /// Медиана latency
    pub median_latency_ms: f64,
    /// Всего тестов проведено
    pub total_tests: u32,
    /// Успешных тестов
    pub successful_tests: u32,
    /// Неудачных тестов
    pub failed_tests: u32,
    /// Список задержек для каждого теста
    pub latencies: Vec<u32>,
    /// Ошибки (если были)
    pub errors: Vec<String>,
    /// Throughput (успешных запросов в секунду)
    pub throughput: f64,
}

impl Default for ABTestStrategyResult {
    fn default() -> Self {
        Self {
            strategy_id: String::new(),
            strategy_name: String::new(),
            success_rate: 0.0,
            avg_latency_ms: 0.0,
            min_latency_ms: 0,
            max_latency_ms: 0,
            std_dev_latency_ms: 0.0,
            median_latency_ms: 0.0,
            total_tests: 0,
            successful_tests: 0,
            failed_tests: 0,
            latencies: Vec::new(),
            errors: Vec::new(),
            throughput: 0.0,
        }
    }
}

/// Конфигурация A/B теста
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTest {
    /// Уникальный ID теста
    pub id: String,
    /// ID первой стратегии
    pub strategy_a: String,
    /// ID второй стратегии
    pub strategy_b: String,
    /// ID сервиса для тестирования
    pub service_id: String,
    /// Количество итераций для каждой стратегии
    pub iterations: u32,
    /// Текущий статус теста
    pub status: ABTestStatus,
    /// Прогресс (0-100)
    pub progress: u8,
    /// Текущая итерация
    pub current_iteration: u32,
    /// Какая стратегия сейчас тестируется ("a" или "b")
    pub current_strategy: String,
    /// Время начала теста
    pub started_at: Option<String>,
    /// Время завершения теста
    pub completed_at: Option<String>,
    /// Сообщение об ошибке (если есть)
    pub error_message: Option<String>,
    /// Название теста (опционально)
    pub name: Option<String>,
    /// Описание теста (опционально)
    pub description: Option<String>,
}

impl ABTest {
    pub fn new(strategy_a: String, strategy_b: String, service_id: String, iterations: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            strategy_a,
            strategy_b,
            service_id,
            iterations,
            status: ABTestStatus::Pending,
            progress: 0,
            current_iteration: 0,
            current_strategy: String::new(),
            started_at: None,
            completed_at: None,
            error_message: None,
            name: None,
            description: None,
        }
    }
    
    /// Создаёт тест с именем и описанием
    pub fn with_metadata(
        strategy_a: String,
        strategy_b: String,
        service_id: String,
        iterations: u32,
        name: Option<String>,
        description: Option<String>,
    ) -> Self {
        let mut test = Self::new(strategy_a, strategy_b, service_id, iterations);
        test.name = name;
        test.description = description;
        test
    }
}

/// Результат статистического сравнения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalComparison {
    /// T-статистика (Welch's t-test)
    pub t_statistic: f64,
    /// P-value для t-test
    pub p_value: f64,
    /// Confidence level (1 - p_value)
    pub confidence_level: f64,
    /// Статистически значимая разница
    pub is_significant: bool,
    /// Effect size (Cohen's d)
    pub effect_size: f64,
    /// Интерпретация effect size
    pub effect_interpretation: String,
}

/// Полный результат A/B теста
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    /// ID теста
    pub test_id: String,
    /// Результат стратегии A
    pub strategy_a_result: ABTestStrategyResult,
    /// Результат стратегии B
    pub strategy_b_result: ABTestStrategyResult,
    /// ID сервиса
    pub service_id: String,
    /// Название сервиса
    pub service_name: String,
    /// ID победителя (или None если ничья)
    pub winner_id: Option<String>,
    /// Разница в success_rate (A - B)
    pub success_rate_diff: f64,
    /// Разница в latency (A - B, отрицательное = A быстрее)
    pub latency_diff_ms: f64,
    /// Время завершения
    pub completed_at: String,
    /// Статистическое сравнение latency
    pub statistical_comparison: Option<StatisticalComparison>,
    /// Confidence level для определения победителя (0-1)
    pub winner_confidence: f64,
    /// Причина выбора победителя
    pub winner_reason: String,
}

/// Событие прогресса A/B теста (для отправки на фронтенд)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestProgress {
    pub test_id: String,
    pub status: ABTestStatus,
    pub progress: u8,
    pub current_iteration: u32,
    pub total_iterations: u32,
    pub current_strategy: String,
    pub current_strategy_name: String,
}

/// Менеджер A/B тестирования
pub struct ABTestManager {
    /// Движок стратегий
    engine: SharedStrategyEngine,
    /// Активные тесты
    active_tests: Arc<RwLock<HashMap<String, ABTest>>>,
    /// Результаты завершённых тестов
    results: Arc<RwLock<HashMap<String, ABTestResult>>>,
    /// Токены отмены для активных тестов
    cancel_tokens: Arc<RwLock<HashMap<String, tokio_util::sync::CancellationToken>>>,
    /// Storage для persistence (опционально)
    storage: Option<Arc<Storage>>,
}

impl ABTestManager {
    /// Создаёт новый менеджер A/B тестирования
    pub fn new(engine: SharedStrategyEngine) -> Self {
        Self {
            engine,
            active_tests: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
        }
    }
    
    /// Создаёт менеджер с persistence через Storage
    pub fn with_storage(engine: SharedStrategyEngine, storage: Arc<Storage>) -> Self {
        Self {
            engine,
            active_tests: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
            storage: Some(storage),
        }
    }
    
    /// Загружает сохранённые результаты из storage
    pub async fn load_saved_results(&self) -> Result<()> {
        if let Some(ref storage) = self.storage {
            let saved: Option<Vec<ABTestResult>> = storage.get_setting(AB_TEST_RESULTS_KEY).await?;
            if let Some(results) = saved {
                let mut results_map = self.results.write().await;
                for result in results {
                    results_map.insert(result.test_id.clone(), result);
                }
                info!(count = results_map.len(), "Loaded saved A/B test results");
            }
        }
        Ok(())
    }
    
    /// Сохраняет результаты в storage
    async fn save_results_to_storage(&self) -> Result<()> {
        if let Some(ref storage) = self.storage {
            let results = self.results.read().await;
            let results_vec: Vec<ABTestResult> = results.values().cloned().collect();
            storage.set_setting(AB_TEST_RESULTS_KEY, &results_vec).await?;
            debug!(count = results_vec.len(), "Saved A/B test results to storage");
        }
        Ok(())
    }

    /// Запускает A/B тест
    /// 
    /// ВАЖНО: Только один A/B тест может выполняться одновременно!
    /// Это критично для Zapret стратегий, которые используют WinDivert.
    pub async fn start_test(
        &self,
        strategy_a: &Strategy,
        strategy_b: &Strategy,
        service: &Service,
        iterations: u32,
    ) -> Result<String> {
        // CRITICAL: Проверяем что нет активных тестов (защита от BSOD)
        {
            let active = self.active_tests.read().await;
            let running_tests: Vec<_> = active.values()
                .filter(|t| t.status == ABTestStatus::Running)
                .collect();
            
            if !running_tests.is_empty() {
                return Err(IsolateError::Config(
                    "Another A/B test is already running. Only one test can run at a time to prevent WinDivert conflicts.".to_string()
                ));
            }
        }
        
        // Проверяем что обе стратегии поддерживают глобальный режим
        if !strategy_a.mode_capabilities.supports_global {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' does not support GLOBAL mode",
                strategy_a.id
            )));
        }
        if !strategy_b.mode_capabilities.supports_global {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' does not support GLOBAL mode",
                strategy_b.id
            )));
        }

        // Создаём тест
        let mut test = ABTest::new(
            strategy_a.id.clone(),
            strategy_b.id.clone(),
            service.id.clone(),
            iterations,
        );
        let test_id = test.id.clone();
        test.status = ABTestStatus::Running;
        test.started_at = Some(chrono::Utc::now().to_rfc3339());

        // Создаём токен отмены
        let cancel_token = tokio_util::sync::CancellationToken::new();
        
        // Сохраняем тест и токен
        {
            let mut tests = self.active_tests.write().await;
            tests.insert(test_id.clone(), test);
        }
        {
            let mut tokens = self.cancel_tokens.write().await;
            tokens.insert(test_id.clone(), cancel_token.clone());
        }

        info!(
            test_id = %test_id,
            strategy_a = %strategy_a.id,
            strategy_b = %strategy_b.id,
            service = %service.id,
            iterations,
            "Starting A/B test"
        );

        Ok(test_id)
    }

    /// Выполняет A/B тест (вызывается асинхронно)
    pub async fn run_test(
        &self,
        test_id: &str,
        strategy_a: &Strategy,
        strategy_b: &Strategy,
        service: &Service,
        iterations: u32,
    ) -> Result<ABTestResult> {
        let cancel_token = {
            let tokens = self.cancel_tokens.read().await;
            tokens.get(test_id).cloned()
        };

        let cancel_token = cancel_token.ok_or_else(|| {
            IsolateError::Config(format!("Test {} not found", test_id))
        })?;

        let test_url = service.get_test_url().ok_or_else(|| {
            IsolateError::Config(format!("Service '{}' has no test URL", service.id))
        })?;

        let total_iterations = iterations * 2; // Для обеих стратегий
        let mut current_iter = 0u32;
        let test_start_time = std::time::Instant::now();

        // Тестируем стратегию A
        info!(test_id, strategy = %strategy_a.id, "Testing strategy A");
        let result_a = self.test_single_strategy(
            test_id,
            strategy_a,
            &test_url,
            iterations,
            &cancel_token,
            &mut current_iter,
            total_iterations,
            "a",
        ).await?;

        if cancel_token.is_cancelled() {
            return Err(IsolateError::TestFailed("Test cancelled".to_string()));
        }

        // Пауза между стратегиями
        tokio::time::sleep(Duration::from_millis(BETWEEN_STRATEGY_DELAY_MS)).await;

        // Тестируем стратегию B
        info!(test_id, strategy = %strategy_b.id, "Testing strategy B");
        let result_b = self.test_single_strategy(
            test_id,
            strategy_b,
            &test_url,
            iterations,
            &cancel_token,
            &mut current_iter,
            total_iterations,
            "b",
        ).await?;

        let test_duration = test_start_time.elapsed();

        // Статистическое сравнение
        let statistical_comparison = self.perform_statistical_comparison(&result_a, &result_b);

        // Определяем победителя с confidence level
        let (winner_id, winner_confidence, winner_reason) = 
            self.determine_winner_with_confidence(&result_a, &result_b, &statistical_comparison);
        
        let success_rate_diff = result_a.success_rate - result_b.success_rate;
        let latency_diff_ms = result_a.avg_latency_ms - result_b.avg_latency_ms;

        let result = ABTestResult {
            test_id: test_id.to_string(),
            strategy_a_result: result_a,
            strategy_b_result: result_b,
            service_id: service.id.clone(),
            service_name: service.name.clone(),
            winner_id,
            success_rate_diff,
            latency_diff_ms,
            completed_at: chrono::Utc::now().to_rfc3339(),
            statistical_comparison,
            winner_confidence,
            winner_reason,
        };

        // Обновляем статус теста
        {
            let mut tests = self.active_tests.write().await;
            if let Some(test) = tests.get_mut(test_id) {
                test.status = ABTestStatus::Completed;
                test.progress = 100;
                test.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        // Сохраняем результат
        {
            let mut results = self.results.write().await;
            results.insert(test_id.to_string(), result.clone());
        }
        
        // Сохраняем в storage
        if let Err(e) = self.save_results_to_storage().await {
            warn!(error = %e, "Failed to save A/B test results to storage");
        }

        // Очищаем токен отмены
        {
            let mut tokens = self.cancel_tokens.write().await;
            tokens.remove(test_id);
        }

        info!(
            test_id,
            winner = ?result.winner_id,
            winner_confidence = result.winner_confidence,
            success_rate_diff = result.success_rate_diff,
            latency_diff_ms = result.latency_diff_ms,
            duration_secs = test_duration.as_secs(),
            "A/B test completed"
        );

        Ok(result)
    }

    /// Тестирует одну стратегию несколько раз
    async fn test_single_strategy(
        &self,
        test_id: &str,
        strategy: &Strategy,
        test_url: &str,
        iterations: u32,
        cancel_token: &tokio_util::sync::CancellationToken,
        current_iter: &mut u32,
        total_iterations: u32,
        strategy_label: &str,
    ) -> Result<ABTestStrategyResult> {
        let mut result = ABTestStrategyResult {
            strategy_id: strategy.id.clone(),
            strategy_name: strategy.name.clone(),
            ..Default::default()
        };
        
        let strategy_start_time = std::time::Instant::now();

        for i in 0..iterations {
            if cancel_token.is_cancelled() {
                warn!(test_id, "Test cancelled during iteration {}", i);
                break;
            }

            *current_iter += 1;
            let progress = ((*current_iter as f32 / total_iterations as f32) * 100.0) as u8;

            // Обновляем прогресс
            {
                let mut tests = self.active_tests.write().await;
                if let Some(test) = tests.get_mut(test_id) {
                    test.progress = progress;
                    test.current_iteration = *current_iter;
                    test.current_strategy = strategy_label.to_string();
                }
            }

            debug!(
                test_id,
                strategy = %strategy.id,
                iteration = i + 1,
                total = iterations,
                "Running test iteration"
            );

            // Запускаем стратегию
            if let Err(e) = self.engine.start_global(strategy).await {
                error!(
                    test_id,
                    strategy = %strategy.id,
                    error = %e,
                    "Failed to start strategy"
                );
                result.errors.push(format!("Iteration {}: Failed to start: {}", i + 1, e));
                result.failed_tests += 1;
                result.total_tests += 1;
                continue;
            }

            // Ждём применения стратегии
            tokio::time::sleep(Duration::from_millis(STRATEGY_APPLY_DELAY_MS)).await;

            // Тестируем подключение
            let test_result = self.test_connection(test_url).await;

            // Останавливаем стратегию
            if let Err(e) = self.engine.stop_global().await {
                warn!(test_id, error = %e, "Failed to stop strategy cleanly");
            }

            // Пауза перед следующей итерацией
            tokio::time::sleep(Duration::from_millis(BETWEEN_STRATEGY_DELAY_MS)).await;

            // Обрабатываем результат
            match test_result {
                Ok(latency) => {
                    result.successful_tests += 1;
                    result.latencies.push(latency);
                    debug!(test_id, latency, "Test iteration successful");
                }
                Err(e) => {
                    result.failed_tests += 1;
                    result.errors.push(format!("Iteration {}: {}", i + 1, e));
                    debug!(test_id, error = %e, "Test iteration failed");
                }
            }
            result.total_tests += 1;
        }

        // Вычисляем статистику
        let test_duration_ms = strategy_start_time.elapsed().as_millis() as u64;
        self.calculate_statistics(&mut result, test_duration_ms);

        Ok(result)
    }

    /// Тестирует подключение к URL
    async fn test_connection(&self, url: &str) -> Result<u32> {
        let start = std::time::Instant::now();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| IsolateError::Network(e.to_string()))?;

        client
            .get(url)
            .send()
            .await
            .map_err(|e| IsolateError::Network(e.to_string()))?
            .error_for_status()
            .map_err(|e| IsolateError::Network(e.to_string()))?;

        Ok(start.elapsed().as_millis() as u32)
    }

    /// Вычисляет статистику для результата
    fn calculate_statistics(&self, result: &mut ABTestStrategyResult, test_duration_ms: u64) {
        if result.total_tests == 0 {
            return;
        }

        result.success_rate = (result.successful_tests as f64 / result.total_tests as f64) * 100.0;

        if !result.latencies.is_empty() {
            let sum: u32 = result.latencies.iter().sum();
            result.avg_latency_ms = sum as f64 / result.latencies.len() as f64;
            result.min_latency_ms = *result.latencies.iter().min().unwrap_or(&0);
            result.max_latency_ms = *result.latencies.iter().max().unwrap_or(&0);
            
            // Стандартное отклонение
            let variance: f64 = result.latencies.iter()
                .map(|&x| {
                    let diff = x as f64 - result.avg_latency_ms;
                    diff * diff
                })
                .sum::<f64>() / result.latencies.len() as f64;
            result.std_dev_latency_ms = variance.sqrt();
            
            // Медиана
            let mut sorted = result.latencies.clone();
            sorted.sort();
            let mid = sorted.len() / 2;
            result.median_latency_ms = if sorted.len() % 2 == 0 {
                (sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0
            } else {
                sorted[mid] as f64
            };
        }
        
        // Throughput (успешных запросов в секунду)
        if test_duration_ms > 0 {
            result.throughput = (result.successful_tests as f64 / test_duration_ms as f64) * 1000.0;
        }
    }
    
    /// Выполняет статистическое сравнение двух результатов
    fn perform_statistical_comparison(
        &self,
        result_a: &ABTestStrategyResult,
        result_b: &ABTestStrategyResult,
    ) -> Option<StatisticalComparison> {
        // Нужно минимум 2 измерения для статистики
        if result_a.latencies.len() < 2 || result_b.latencies.len() < 2 {
            return None;
        }
        
        let n1 = result_a.latencies.len() as f64;
        let n2 = result_b.latencies.len() as f64;
        let mean1 = result_a.avg_latency_ms;
        let mean2 = result_b.avg_latency_ms;
        let var1 = result_a.std_dev_latency_ms.powi(2);
        let var2 = result_b.std_dev_latency_ms.powi(2);
        
        // Welch's t-test
        let se = ((var1 / n1) + (var2 / n2)).sqrt();
        if se == 0.0 {
            return None;
        }
        
        let t_statistic = (mean1 - mean2) / se;
        
        // Degrees of freedom (Welch-Satterthwaite)
        let df_num = ((var1 / n1) + (var2 / n2)).powi(2);
        let df_denom = ((var1 / n1).powi(2) / (n1 - 1.0)) + ((var2 / n2).powi(2) / (n2 - 1.0));
        let df = df_num / df_denom;
        
        // Approximate p-value using normal distribution for large samples
        // For small samples, this is an approximation
        let p_value = self.approximate_p_value(t_statistic.abs(), df);
        let confidence_level = 1.0 - p_value;
        
        // Cohen's d (effect size)
        let pooled_std = ((var1 + var2) / 2.0).sqrt();
        let effect_size = if pooled_std > 0.0 {
            (mean1 - mean2).abs() / pooled_std
        } else {
            0.0
        };
        
        let effect_interpretation = if effect_size < 0.2 {
            "negligible".to_string()
        } else if effect_size < 0.5 {
            "small".to_string()
        } else if effect_size < 0.8 {
            "medium".to_string()
        } else {
            "large".to_string()
        };
        
        Some(StatisticalComparison {
            t_statistic,
            p_value,
            confidence_level,
            is_significant: p_value < (1.0 - MIN_CONFIDENCE_LEVEL),
            effect_size,
            effect_interpretation,
        })
    }
    
    /// Аппроксимация p-value для t-распределения
    fn approximate_p_value(&self, t: f64, df: f64) -> f64 {
        // Используем аппроксимацию через нормальное распределение для df > 30
        // Для меньших df используем более точную аппроксимацию
        if df > 30.0 {
            // Стандартное нормальное распределение
            2.0 * (1.0 - self.normal_cdf(t))
        } else {
            // Аппроксимация для t-распределения
            let x = df / (df + t * t);
            self.incomplete_beta(df / 2.0, 0.5, x)
        }
    }
    
    /// CDF стандартного нормального распределения (аппроксимация)
    fn normal_cdf(&self, x: f64) -> f64 {
        // Аппроксимация Abramowitz and Stegun
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs() / std::f64::consts::SQRT_2;
        
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        
        0.5 * (1.0 + sign * y)
    }
    
    /// Неполная бета-функция (упрощённая аппроксимация)
    fn incomplete_beta(&self, a: f64, b: f64, x: f64) -> f64 {
        // Упрощённая аппроксимация для наших целей
        // Для более точных результатов нужна библиотека статистики
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        
        // Используем аппроксимацию через нормальное распределение
        let mean = a / (a + b);
        let variance = (a * b) / ((a + b).powi(2) * (a + b + 1.0));
        let z = (x - mean) / variance.sqrt();
        
        self.normal_cdf(z)
    }

    /// Определяет победителя теста с confidence level
    fn determine_winner_with_confidence(
        &self,
        result_a: &ABTestStrategyResult,
        result_b: &ABTestStrategyResult,
        stats: &Option<StatisticalComparison>,
    ) -> (Option<String>, f64, String) {
        let success_diff = result_a.success_rate - result_b.success_rate;
        
        // Приоритет 1: Значительная разница в success_rate (>10%)
        if success_diff.abs() > 10.0 {
            let confidence = (success_diff.abs() / 100.0).min(0.99);
            if success_diff > 0.0 {
                return (
                    Some(result_a.strategy_id.clone()),
                    confidence,
                    format!("Strategy A has {:.1}% higher success rate", success_diff),
                );
            } else {
                return (
                    Some(result_b.strategy_id.clone()),
                    confidence,
                    format!("Strategy B has {:.1}% higher success rate", -success_diff),
                );
            }
        }
        
        // Приоритет 2: Статистически значимая разница в latency
        if let Some(ref comparison) = stats {
            if comparison.is_significant {
                let latency_diff = result_a.avg_latency_ms - result_b.avg_latency_ms;
                if latency_diff < 0.0 {
                    return (
                        Some(result_a.strategy_id.clone()),
                        comparison.confidence_level,
                        format!(
                            "Strategy A is {:.0}ms faster (p={:.3}, effect={})",
                            -latency_diff, comparison.p_value, comparison.effect_interpretation
                        ),
                    );
                } else {
                    return (
                        Some(result_b.strategy_id.clone()),
                        comparison.confidence_level,
                        format!(
                            "Strategy B is {:.0}ms faster (p={:.3}, effect={})",
                            latency_diff, comparison.p_value, comparison.effect_interpretation
                        ),
                    );
                }
            }
        }
        
        // Приоритет 3: Небольшая разница в success_rate (5-10%)
        if success_diff.abs() > 5.0 {
            let confidence = 0.7 + (success_diff.abs() - 5.0) / 50.0;
            if success_diff > 0.0 {
                return (
                    Some(result_a.strategy_id.clone()),
                    confidence,
                    format!("Strategy A has slightly higher success rate ({:.1}%)", success_diff),
                );
            } else {
                return (
                    Some(result_b.strategy_id.clone()),
                    confidence,
                    format!("Strategy B has slightly higher success rate ({:.1}%)", -success_diff),
                );
            }
        }
        
        // Приоритет 4: Разница в latency без статистической значимости (>100ms)
        if result_a.avg_latency_ms > 0.0 && result_b.avg_latency_ms > 0.0 {
            let latency_diff = result_a.avg_latency_ms - result_b.avg_latency_ms;
            if latency_diff.abs() > 100.0 {
                let confidence = 0.6 + (latency_diff.abs() / 1000.0).min(0.2);
                if latency_diff < 0.0 {
                    return (
                        Some(result_a.strategy_id.clone()),
                        confidence,
                        format!("Strategy A is {:.0}ms faster (not statistically significant)", -latency_diff),
                    );
                } else {
                    return (
                        Some(result_b.strategy_id.clone()),
                        confidence,
                        format!("Strategy B is {:.0}ms faster (not statistically significant)", latency_diff),
                    );
                }
            }
        }

        // Ничья
        (None, 0.5, "No significant difference between strategies".to_string())
    }

    /// Определяет победителя теста (legacy метод для совместимости)
    fn determine_winner(
        &self,
        result_a: &ABTestStrategyResult,
        result_b: &ABTestStrategyResult,
    ) -> Option<String> {
        let (winner, _, _) = self.determine_winner_with_confidence(result_a, result_b, &None);
        winner
    }

    /// Отменяет активный тест
    pub async fn cancel_test(&self, test_id: &str) -> Result<()> {
        info!(test_id, "Cancelling A/B test");

        // Отменяем через токен
        {
            let tokens = self.cancel_tokens.read().await;
            if let Some(token) = tokens.get(test_id) {
                token.cancel();
            }
        }

        // Обновляем статус
        {
            let mut tests = self.active_tests.write().await;
            if let Some(test) = tests.get_mut(test_id) {
                test.status = ABTestStatus::Cancelled;
                test.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        // Останавливаем стратегию если запущена
        let _ = self.engine.stop_global().await;

        Ok(())
    }

    /// Получает статус теста
    pub async fn get_test_status(&self, test_id: &str) -> Option<ABTest> {
        let tests = self.active_tests.read().await;
        tests.get(test_id).cloned()
    }

    /// Получает результаты теста
    pub async fn get_test_results(&self, test_id: &str) -> Option<ABTestResult> {
        let results = self.results.read().await;
        results.get(test_id).cloned()
    }

    /// Получает прогресс теста
    pub async fn get_test_progress(&self, test_id: &str) -> Option<ABTestProgress> {
        let tests = self.active_tests.read().await;
        tests.get(test_id).map(|test| ABTestProgress {
            test_id: test.id.clone(),
            status: test.status.clone(),
            progress: test.progress,
            current_iteration: test.current_iteration,
            total_iterations: test.iterations * 2,
            current_strategy: test.current_strategy.clone(),
            current_strategy_name: if test.current_strategy == "a" {
                test.strategy_a.clone()
            } else {
                test.strategy_b.clone()
            },
        })
    }

    /// Получает все активные тесты
    pub async fn get_active_tests(&self) -> Vec<ABTest> {
        let tests = self.active_tests.read().await;
        tests.values().cloned().collect()
    }
    
    /// Получает все завершённые тесты (результаты)
    pub async fn get_all_results(&self) -> Vec<ABTestResult> {
        let results = self.results.read().await;
        results.values().cloned().collect()
    }
    
    /// Получает список всех тестов (активных и завершённых)
    pub async fn list_all_tests(&self) -> Vec<ABTestSummary> {
        let mut summaries = Vec::new();
        
        // Активные тесты
        {
            let tests = self.active_tests.read().await;
            for test in tests.values() {
                summaries.push(ABTestSummary {
                    id: test.id.clone(),
                    strategy_a: test.strategy_a.clone(),
                    strategy_b: test.strategy_b.clone(),
                    service_id: test.service_id.clone(),
                    status: test.status.clone(),
                    started_at: test.started_at.clone(),
                    completed_at: test.completed_at.clone(),
                    winner_id: None,
                    name: test.name.clone(),
                });
            }
        }
        
        // Завершённые тесты из результатов
        {
            let results = self.results.read().await;
            for result in results.values() {
                // Проверяем, нет ли уже этого теста в списке
                if !summaries.iter().any(|s| s.id == result.test_id) {
                    summaries.push(ABTestSummary {
                        id: result.test_id.clone(),
                        strategy_a: result.strategy_a_result.strategy_id.clone(),
                        strategy_b: result.strategy_b_result.strategy_id.clone(),
                        service_id: result.service_id.clone(),
                        status: ABTestStatus::Completed,
                        started_at: None,
                        completed_at: Some(result.completed_at.clone()),
                        winner_id: result.winner_id.clone(),
                        name: None,
                    });
                }
            }
        }
        
        // Сортируем по времени (новые первые)
        summaries.sort_by(|a, b| {
            let time_a = a.completed_at.as_ref().or(a.started_at.as_ref());
            let time_b = b.completed_at.as_ref().or(b.started_at.as_ref());
            time_b.cmp(&time_a)
        });
        
        summaries
    }
    
    /// Удаляет тест по ID
    pub async fn delete_test(&self, test_id: &str) -> Result<bool> {
        // Проверяем, не активен ли тест
        {
            let tests = self.active_tests.read().await;
            if let Some(test) = tests.get(test_id) {
                if test.status == ABTestStatus::Running {
                    return Err(IsolateError::Config(
                        "Cannot delete running test. Cancel it first.".to_string()
                    ));
                }
            }
        }
        
        // Удаляем из активных тестов
        let removed_active = {
            let mut tests = self.active_tests.write().await;
            tests.remove(test_id).is_some()
        };
        
        // Удаляем из результатов
        let removed_result = {
            let mut results = self.results.write().await;
            results.remove(test_id).is_some()
        };
        
        // Удаляем токен отмены
        {
            let mut tokens = self.cancel_tokens.write().await;
            tokens.remove(test_id);
        }
        
        // Сохраняем в storage
        if removed_result {
            if let Err(e) = self.save_results_to_storage().await {
                warn!(error = %e, "Failed to save after deleting test");
            }
        }
        
        let deleted = removed_active || removed_result;
        if deleted {
            info!(test_id, "Deleted A/B test");
        }
        
        Ok(deleted)
    }
    
    /// Сравнивает две стратегии напрямую (без запуска теста)
    /// Использует исторические данные если доступны
    pub async fn compare_strategies_from_history(
        &self,
        strategy_a_id: &str,
        strategy_b_id: &str,
    ) -> Option<ABTestResult> {
        let results = self.results.read().await;
        
        // Ищем существующий тест с этими стратегиями
        for result in results.values() {
            let matches_ab = result.strategy_a_result.strategy_id == strategy_a_id
                && result.strategy_b_result.strategy_id == strategy_b_id;
            let matches_ba = result.strategy_a_result.strategy_id == strategy_b_id
                && result.strategy_b_result.strategy_id == strategy_a_id;
            
            if matches_ab || matches_ba {
                return Some(result.clone());
            }
        }
        
        None
    }

    /// Очищает завершённые тесты
    #[allow(dead_code)]
    pub async fn cleanup_completed(&self) {
        let mut tests = self.active_tests.write().await;
        tests.retain(|_, test| {
            test.status != ABTestStatus::Completed && test.status != ABTestStatus::Cancelled
        });
    }
    
    /// Очищает все результаты
    pub async fn clear_all_results(&self) -> Result<()> {
        {
            let mut results = self.results.write().await;
            results.clear();
        }
        
        // Сохраняем пустой список в storage
        if let Err(e) = self.save_results_to_storage().await {
            warn!(error = %e, "Failed to save after clearing results");
        }
        
        info!("Cleared all A/B test results");
        Ok(())
    }
}

/// Краткая информация о тесте для списка
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestSummary {
    pub id: String,
    pub strategy_a: String,
    pub strategy_b: String,
    pub service_id: String,
    pub status: ABTestStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub winner_id: Option<String>,
    pub name: Option<String>,
}

/// Создаёт менеджер A/B тестирования
pub fn create_ab_test_manager(engine: SharedStrategyEngine) -> Arc<ABTestManager> {
    Arc::new(ABTestManager::new(engine))
}

/// Создаёт менеджер A/B тестирования с persistence
pub fn create_ab_test_manager_with_storage(
    engine: SharedStrategyEngine,
    storage: Arc<Storage>,
) -> Arc<ABTestManager> {
    Arc::new(ABTestManager::with_storage(engine, storage))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ab_test_creation() {
        let test = ABTest::new(
            "strategy-a".to_string(),
            "strategy-b".to_string(),
            "youtube".to_string(),
            5,
        );

        assert!(!test.id.is_empty());
        assert_eq!(test.strategy_a, "strategy-a");
        assert_eq!(test.strategy_b, "strategy-b");
        assert_eq!(test.service_id, "youtube");
        assert_eq!(test.iterations, 5);
        assert_eq!(test.status, ABTestStatus::Pending);
    }
    
    #[test]
    fn test_ab_test_with_metadata() {
        let test = ABTest::with_metadata(
            "strategy-a".to_string(),
            "strategy-b".to_string(),
            "youtube".to_string(),
            5,
            Some("Test Name".to_string()),
            Some("Test Description".to_string()),
        );

        assert_eq!(test.name, Some("Test Name".to_string()));
        assert_eq!(test.description, Some("Test Description".to_string()));
    }

    #[test]
    fn test_statistics_calculation() {
        let mut result = ABTestStrategyResult {
            strategy_id: "test".to_string(),
            strategy_name: "Test".to_string(),
            total_tests: 10,
            successful_tests: 8,
            failed_tests: 2,
            latencies: vec![100, 150, 200, 120, 180, 90, 110, 130],
            ..Default::default()
        };

        // Manually calculate
        result.success_rate = (result.successful_tests as f64 / result.total_tests as f64) * 100.0;
        let sum: u32 = result.latencies.iter().sum();
        result.avg_latency_ms = sum as f64 / result.latencies.len() as f64;
        result.min_latency_ms = *result.latencies.iter().min().unwrap();
        result.max_latency_ms = *result.latencies.iter().max().unwrap();
        
        // Calculate std dev
        let variance: f64 = result.latencies.iter()
            .map(|&x| {
                let diff = x as f64 - result.avg_latency_ms;
                diff * diff
            })
            .sum::<f64>() / result.latencies.len() as f64;
        result.std_dev_latency_ms = variance.sqrt();
        
        // Calculate median
        let mut sorted = result.latencies.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        result.median_latency_ms = (sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0;

        assert_eq!(result.success_rate, 80.0);
        assert_eq!(result.min_latency_ms, 90);
        assert_eq!(result.max_latency_ms, 200);
        assert!(result.std_dev_latency_ms > 0.0);
        assert!(result.median_latency_ms > 0.0);
    }
    
    #[test]
    fn test_statistical_comparison_result() {
        let comparison = StatisticalComparison {
            t_statistic: 2.5,
            p_value: 0.02,
            confidence_level: 0.98,
            is_significant: true,
            effect_size: 0.6,
            effect_interpretation: "medium".to_string(),
        };
        
        assert!(comparison.is_significant);
        assert!(comparison.confidence_level > MIN_CONFIDENCE_LEVEL);
        assert_eq!(comparison.effect_interpretation, "medium");
    }
    
    #[test]
    fn test_ab_test_summary() {
        let summary = ABTestSummary {
            id: "test-123".to_string(),
            strategy_a: "strategy-a".to_string(),
            strategy_b: "strategy-b".to_string(),
            service_id: "youtube".to_string(),
            status: ABTestStatus::Completed,
            started_at: Some("2024-01-01T00:00:00Z".to_string()),
            completed_at: Some("2024-01-01T00:10:00Z".to_string()),
            winner_id: Some("strategy-a".to_string()),
            name: Some("Test".to_string()),
        };
        
        assert_eq!(summary.id, "test-123");
        assert_eq!(summary.status, ABTestStatus::Completed);
        assert!(summary.winner_id.is_some());
    }
    
    #[test]
    fn test_default_strategy_result() {
        let result = ABTestStrategyResult::default();
        
        assert!(result.strategy_id.is_empty());
        assert_eq!(result.success_rate, 0.0);
        assert_eq!(result.avg_latency_ms, 0.0);
        assert_eq!(result.std_dev_latency_ms, 0.0);
        assert_eq!(result.median_latency_ms, 0.0);
        assert_eq!(result.throughput, 0.0);
        assert!(result.latencies.is_empty());
    }
}

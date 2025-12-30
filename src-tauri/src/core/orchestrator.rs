//! Orchestrator - координатор процесса оптимизации
//!
//! Полный алгоритм:
//! 1. Проверка кэша
//! 2. DPI-диагностика
//! 3. Выбор кандидатов по family
//! 4. Параллельные SOCKS-тесты (только VLESS)
//! 5. Последовательные driver-тесты (Zapret)
//! 6. Выбор лучшей стратегии по score
//! 7. Применение в GLOBAL-режиме

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{
    DiagnosticResult, DpiKind, EnvInfo, Strategy, StrategyEngine as EngineType, StrategyFamily,
    StrategyScore,
};
use crate::core::storage::Storage;
use crate::core::strategy_engine::{LaunchMode, SharedStrategyEngine};

// ============================================================================
// Constants
// ============================================================================

/// Минимальный score для принятия стратегии
const MIN_ACCEPTABLE_SCORE: f64 = 0.6;
/// Таймаут теста стратегии в миллисекундах
const STRATEGY_TEST_TIMEOUT_MS: u64 = 10000;
/// Максимальное количество параллельных VLESS тестов
const MAX_PARALLEL_VLESS_TESTS: usize = 5;
/// Задержка между последовательными Zapret тестами
const ZAPRET_TEST_DELAY_MS: u64 = 2500;

// ============================================================================
// Progress Events
// ============================================================================

/// Этап оптимизации
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationStage {
    /// Инициализация
    Initializing,
    /// Проверка кэша
    CheckingCache,
    /// DPI-диагностика
    Diagnosing,
    /// Выбор кандидатов
    SelectingCandidates,
    /// Тестирование VLESS стратегий
    TestingVless,
    /// Тестирование Zapret стратегий
    TestingZapret,
    /// Выбор лучшей стратегии
    SelectingBest,
    /// Применение стратегии
    Applying,
    /// Завершено успешно
    Completed,
    /// Ошибка
    Failed,
    /// Отменено
    Cancelled,
}

/// Прогресс оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationProgress {
    /// Текущий этап
    pub stage: OptimizationStage,
    /// Процент выполнения (0-100)
    pub percent: u8,
    /// Текстовое сообщение
    pub message: String,
    /// Текущая тестируемая стратегия
    pub current_strategy: Option<String>,
    /// Количество протестированных стратегий
    pub tested_count: u32,
    /// Общее количество стратегий для тестирования
    pub total_count: u32,
    /// Лучший текущий score
    pub best_score: Option<f64>,
}

impl OptimizationProgress {
    fn new(stage: OptimizationStage, percent: u8, message: impl Into<String>) -> Self {
        Self {
            stage,
            percent,
            message: message.into(),
            current_strategy: None,
            tested_count: 0,
            total_count: 0,
            best_score: None,
        }
    }

    fn with_strategy(mut self, strategy: &str) -> Self {
        self.current_strategy = Some(strategy.to_string());
        self
    }

    fn with_counts(mut self, tested: u32, total: u32) -> Self {
        self.tested_count = tested;
        self.total_count = total;
        self
    }

    fn with_score(mut self, score: f64) -> Self {
        self.best_score = Some(score);
        self
    }
}

/// Результат оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Выбранная стратегия
    pub strategy_id: String,
    /// Название стратегии
    pub strategy_name: String,
    /// Score стратегии
    pub score: f64,
    /// Была ли стратегия из кэша
    pub from_cache: bool,
    /// Результаты всех тестов
    pub all_scores: Vec<StrategyScore>,
}

// ============================================================================
// Orchestrator
// ============================================================================

/// Координатор оптимизации
pub struct Orchestrator {
    /// Движок стратегий
    engine: SharedStrategyEngine,
    /// Хранилище
    storage: Arc<Storage>,
    /// Канал прогресса
    progress_tx: broadcast::Sender<OptimizationProgress>,
    /// Токен отмены
    cancel_token: RwLock<Option<CancellationToken>>,
}

impl Orchestrator {
    /// Создаёт новый оркестратор
    pub fn new(engine: SharedStrategyEngine, storage: Arc<Storage>) -> Self {
        let (progress_tx, _) = broadcast::channel(100);

        Self {
            engine,
            storage,
            progress_tx,
            cancel_token: RwLock::new(None),
        }
    }

    /// Подписывается на события прогресса
    pub fn subscribe_progress(&self) -> broadcast::Receiver<OptimizationProgress> {
        self.progress_tx.subscribe()
    }

    /// Запускает процесс оптимизации
    pub async fn optimize(
        &self,
        env_info: &EnvInfo,
        strategies: &[Strategy],
        services: &[String],
    ) -> Result<OptimizationResult> {
        // Создаём токен отмены
        let cancel_token = CancellationToken::new();
        {
            let mut token = self.cancel_token.write().await;
            *token = Some(cancel_token.clone());
        }

        let result = self
            .run_optimization(env_info, strategies, services, cancel_token.clone())
            .await;

        // Очищаем токен
        {
            let mut token = self.cancel_token.write().await;
            *token = None;
        }

        result
    }

    /// Отменяет текущую оптимизацию
    pub async fn cancel(&self) {
        let token = self.cancel_token.read().await;
        if let Some(t) = token.as_ref() {
            t.cancel();
            info!("Optimization cancelled by user");
        }
    }

    /// Проверяет, выполняется ли оптимизация
    pub async fn is_running(&self) -> bool {
        let token = self.cancel_token.read().await;
        token.is_some()
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Основной алгоритм оптимизации
    async fn run_optimization(
        &self,
        env_info: &EnvInfo,
        strategies: &[Strategy],
        services: &[String],
        cancel_token: CancellationToken,
    ) -> Result<OptimizationResult> {
        self.emit_progress(OptimizationProgress::new(
            OptimizationStage::Initializing,
            0,
            "Начинаем оптимизацию...",
        ));

        // 1. Проверка кэша
        self.check_cancelled(&cancel_token)?;
        self.emit_progress(OptimizationProgress::new(
            OptimizationStage::CheckingCache,
            5,
            "Проверяем кэш...",
        ));

        let env_key = env_info.cache_key();
        if let Some(cached) = self.check_cache(&env_key, strategies).await? {
            return Ok(cached);
        }

        // 2. DPI-диагностика
        self.check_cancelled(&cancel_token)?;
        self.emit_progress(OptimizationProgress::new(
            OptimizationStage::Diagnosing,
            10,
            "Анализируем тип блокировки...",
        ));

        let diagnostic = self.run_diagnostics(services).await?;
        debug!(diagnostic = ?diagnostic, "DPI diagnostic completed");

        // 3. Выбор кандидатов
        self.check_cancelled(&cancel_token)?;
        self.emit_progress(OptimizationProgress::new(
            OptimizationStage::SelectingCandidates,
            15,
            "Выбираем подходящие стратегии...",
        ));

        let candidates = self.select_candidates(strategies, &diagnostic);
        if candidates.is_empty() {
            self.emit_progress(OptimizationProgress::new(
                OptimizationStage::Failed,
                100,
                "Не найдено подходящих стратегий",
            ));
            return Err(IsolateError::NoStrategyFound);
        }

        info!(
            count = candidates.len(),
            "Selected {} candidate strategies",
            candidates.len()
        );

        // Разделяем на VLESS и Zapret
        let (vless_candidates, zapret_candidates): (Vec<_>, Vec<_>) = candidates
            .into_iter()
            .partition(|s| matches!(s.engine, EngineType::SingBox | EngineType::Xray));

        let mut all_scores: Vec<StrategyScore> = Vec::new();

        // 4. Параллельные VLESS тесты
        if !vless_candidates.is_empty() {
            self.check_cancelled(&cancel_token)?;
            self.emit_progress(
                OptimizationProgress::new(
                    OptimizationStage::TestingVless,
                    20,
                    "Тестируем VLESS стратегии...",
                )
                .with_counts(0, vless_candidates.len() as u32),
            );

            let vless_scores = self
                .test_vless_parallel(&vless_candidates, services, &cancel_token)
                .await?;
            all_scores.extend(vless_scores);
        }

        // 5. Последовательные Zapret тесты
        if !zapret_candidates.is_empty() {
            self.check_cancelled(&cancel_token)?;
            let base_percent = if vless_candidates.is_empty() { 20 } else { 50 };
            self.emit_progress(
                OptimizationProgress::new(
                    OptimizationStage::TestingZapret,
                    base_percent,
                    "Тестируем Zapret стратегии...",
                )
                .with_counts(0, zapret_candidates.len() as u32),
            );

            let zapret_scores = self
                .test_zapret_sequential(&zapret_candidates, services, &cancel_token)
                .await?;
            all_scores.extend(zapret_scores);
        }

        // 6. Выбор лучшей стратегии
        self.check_cancelled(&cancel_token)?;
        self.emit_progress(OptimizationProgress::new(
            OptimizationStage::SelectingBest,
            85,
            "Выбираем лучшую стратегию...",
        ));

        let best = self.select_best_strategy(&all_scores, strategies)?;

        // 7. Применение в GLOBAL-режиме
        self.check_cancelled(&cancel_token)?;
        self.emit_progress(
            OptimizationProgress::new(
                OptimizationStage::Applying,
                90,
                format!("Применяем стратегию {}...", best.strategy_name),
            )
            .with_strategy(&best.strategy_id),
        );

        let strategy = strategies
            .iter()
            .find(|s| s.id == best.strategy_id)
            .ok_or_else(|| IsolateError::StrategyNotFound(best.strategy_id.clone()))?;

        self.engine.start_global(strategy).await?;

        // Сохраняем в кэш
        self.storage
            .cache_strategy(&env_key, &best.strategy_id, best.score)?;

        self.emit_progress(
            OptimizationProgress::new(
                OptimizationStage::Completed,
                100,
                format!("Стратегия {} успешно применена", best.strategy_name),
            )
            .with_strategy(&best.strategy_id)
            .with_score(best.score),
        );

        info!(
            strategy_id = %best.strategy_id,
            score = best.score,
            "Optimization completed"
        );

        Ok(best)
    }

    /// Проверяет кэш на наличие рабочей стратегии
    async fn check_cache(
        &self,
        env_key: &str,
        strategies: &[Strategy],
    ) -> Result<Option<OptimizationResult>> {
        if let Some(cached) = self.storage.get_cached_strategy(env_key)? {
            // Проверяем, что стратегия всё ещё существует
            if let Some(strategy) = strategies.iter().find(|s| s.id == cached.strategy_id) {
                info!(
                    strategy_id = %cached.strategy_id,
                    score = cached.score,
                    "Using cached strategy"
                );

                self.emit_progress(
                    OptimizationProgress::new(
                        OptimizationStage::Completed,
                        100,
                        format!("Используем кэшированную стратегию {}", strategy.name),
                    )
                    .with_strategy(&strategy.id)
                    .with_score(cached.score),
                );

                // Применяем стратегию
                self.engine.start_global(strategy).await?;

                return Ok(Some(OptimizationResult {
                    strategy_id: cached.strategy_id,
                    strategy_name: strategy.name.clone(),
                    score: cached.score,
                    from_cache: true,
                    all_scores: vec![],
                }));
            }
        }

        Ok(None)
    }

    /// Запускает DPI-диагностику
    async fn run_diagnostics(&self, _services: &[String]) -> Result<DiagnosticResult> {
        // TODO: Реализовать полную диагностику
        // Пока возвращаем базовый профиль
        Ok(DiagnosticResult {
            profile: crate::core::models::DpiProfile {
                kind: DpiKind::Unknown,
                details: None,
                candidate_families: vec![
                    StrategyFamily::DnsBypass,
                    StrategyFamily::SniFrag,
                    StrategyFamily::TlsFrag,
                    StrategyFamily::Vless,
                ],
            },
            tested_services: vec![],
            blocked_services: vec![],
        })
    }

    /// Выбирает кандидатов на основе диагностики
    fn select_candidates<'a>(
        &self,
        strategies: &'a [Strategy],
        diagnostic: &DiagnosticResult,
    ) -> Vec<&'a Strategy> {
        let candidate_families = &diagnostic.profile.candidate_families;

        strategies
            .iter()
            .filter(|s| {
                // Фильтруем по family
                candidate_families.contains(&s.family)
            })
            .collect()
    }

    /// Тестирует VLESS стратегии параллельно
    async fn test_vless_parallel(
        &self,
        strategies: &[&Strategy],
        services: &[String],
        cancel_token: &CancellationToken,
    ) -> Result<Vec<StrategyScore>> {
        let mut scores = Vec::new();
        let total = strategies.len();

        // Разбиваем на чанки для ограничения параллелизма
        for (chunk_idx, chunk) in strategies.chunks(MAX_PARALLEL_VLESS_TESTS).enumerate() {
            self.check_cancelled(cancel_token)?;

            let mut handles = Vec::new();

            for strategy in chunk {
                let strategy_id = strategy.id.clone();
                let strategy_clone = (*strategy).clone();
                let services_clone = services.to_vec();
                let engine = self.engine.clone();

                let handle = tokio::spawn(async move {
                    test_strategy_socks(&engine, &strategy_clone, &services_clone).await
                });

                handles.push((strategy_id, handle));
            }

            // Собираем результаты
            for (strategy_id, handle) in handles {
                match handle.await {
                    Ok(Ok(score)) => {
                        debug!(strategy_id = %strategy_id, score = score.score, "VLESS test completed");
                        scores.push(score);
                    }
                    Ok(Err(e)) => {
                        warn!(strategy_id = %strategy_id, error = %e, "VLESS test failed");
                    }
                    Err(e) => {
                        warn!(strategy_id = %strategy_id, error = %e, "VLESS test panicked");
                    }
                }
            }

            // Обновляем прогресс
            let tested = (chunk_idx + 1) * MAX_PARALLEL_VLESS_TESTS;
            let percent = 20 + (tested * 30 / total) as u8;
            self.emit_progress(
                OptimizationProgress::new(
                    OptimizationStage::TestingVless,
                    percent.min(50),
                    format!("Протестировано {} из {} VLESS стратегий", tested.min(total), total),
                )
                .with_counts(tested.min(total) as u32, total as u32),
            );
        }

        Ok(scores)
    }

    /// Тестирует Zapret стратегии последовательно
    async fn test_zapret_sequential(
        &self,
        strategies: &[&Strategy],
        services: &[String],
        cancel_token: &CancellationToken,
    ) -> Result<Vec<StrategyScore>> {
        let mut scores = Vec::new();
        let total = strategies.len();

        for (idx, strategy) in strategies.iter().enumerate() {
            self.check_cancelled(cancel_token)?;

            // Обновляем прогресс
            let percent = 50 + (idx * 35 / total) as u8;
            self.emit_progress(
                OptimizationProgress::new(
                    OptimizationStage::TestingZapret,
                    percent.min(85),
                    format!("Тестируем {}...", strategy.name),
                )
                .with_strategy(&strategy.id)
                .with_counts(idx as u32, total as u32),
            );

            // Тестируем стратегию
            match test_strategy_socks(&self.engine, strategy, services).await {
                Ok(score) => {
                    debug!(
                        strategy_id = %strategy.id,
                        score = score.score,
                        "Zapret test completed"
                    );
                    scores.push(score);
                }
                Err(e) => {
                    warn!(strategy_id = %strategy.id, error = %e, "Zapret test failed");
                }
            }

            // Задержка между тестами (для стабильности WinDivert)
            if idx < strategies.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(ZAPRET_TEST_DELAY_MS)).await;
            }
        }

        Ok(scores)
    }

    /// Выбирает лучшую стратегию по score
    fn select_best_strategy(
        &self,
        scores: &[StrategyScore],
        strategies: &[Strategy],
    ) -> Result<OptimizationResult> {
        // Фильтруем по минимальному score
        let acceptable: Vec<_> = scores
            .iter()
            .filter(|s| s.score >= MIN_ACCEPTABLE_SCORE)
            .collect();

        if acceptable.is_empty() {
            // Если нет приемлемых, берём лучший из всех
            let best = scores
                .iter()
                .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
                .ok_or(IsolateError::NoStrategyFound)?;

            warn!(
                strategy_id = %best.strategy_id,
                score = best.score,
                "No strategy met minimum score, using best available"
            );

            let strategy = strategies
                .iter()
                .find(|s| s.id == best.strategy_id)
                .ok_or_else(|| IsolateError::StrategyNotFound(best.strategy_id.clone()))?;

            return Ok(OptimizationResult {
                strategy_id: best.strategy_id.clone(),
                strategy_name: strategy.name.clone(),
                score: best.score,
                from_cache: false,
                all_scores: scores.to_vec(),
            });
        }

        // Выбираем лучший по score
        let best = acceptable
            .iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .unwrap();

        let strategy = strategies
            .iter()
            .find(|s| s.id == best.strategy_id)
            .ok_or_else(|| IsolateError::StrategyNotFound(best.strategy_id.clone()))?;

        Ok(OptimizationResult {
            strategy_id: best.strategy_id.clone(),
            strategy_name: strategy.name.clone(),
            score: best.score,
            from_cache: false,
            all_scores: scores.to_vec(),
        })
    }

    /// Проверяет, была ли отмена
    fn check_cancelled(&self, token: &CancellationToken) -> Result<()> {
        if token.is_cancelled() {
            self.emit_progress(OptimizationProgress::new(
                OptimizationStage::Cancelled,
                0,
                "Оптимизация отменена",
            ));
            return Err(IsolateError::Cancelled);
        }
        Ok(())
    }

    /// Отправляет событие прогресса
    fn emit_progress(&self, progress: OptimizationProgress) {
        let _ = self.progress_tx.send(progress);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Тестирует стратегию в SOCKS-режиме
async fn test_strategy_socks(
    engine: &SharedStrategyEngine,
    strategy: &Strategy,
    _services: &[String],
) -> Result<StrategyScore> {
    // Запускаем стратегию в SOCKS-режиме
    let port = engine.start_socks(strategy).await?;

    // Даём время на инициализацию
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // TODO: Реализовать реальное тестирование через SOCKS-прокси
    // Пока возвращаем заглушку
    let score = StrategyScore {
        strategy_id: strategy.id.clone(),
        success_rate: 0.8,
        critical_success_rate: 1.0,
        latency_avg: 150.0,
        latency_jitter: 20.0,
        score: 0.75,
    };

    // Останавливаем стратегию
    engine.stop_socks(&strategy.id).await?;

    Ok(score)
}

// ============================================================================
// Thread-safe wrapper
// ============================================================================

/// Thread-safe обёртка для Orchestrator
pub type SharedOrchestrator = Arc<Orchestrator>;

/// Создаёт shared экземпляр оркестратора
pub fn create_orchestrator(
    engine: SharedStrategyEngine,
    storage: Arc<Storage>,
) -> SharedOrchestrator {
    Arc::new(Orchestrator::new(engine, storage))
}

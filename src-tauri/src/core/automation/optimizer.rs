//! Strategy Optimizer - одноразовая оптимизация стратегий
//!
//! Рефакторинг из orchestrator.rs. Выполняет полный цикл оптимизации:
//! 1. Проверка кэша
//! 2. DPI-диагностика
//! 3. Выбор кандидатов по family
//! 4. Параллельные SOCKS-тесты (VLESS)
//! 5. Последовательные driver-тесты (Zapret)
//! 6. Выбор лучшей стратегии по score
//! 7. Применение в нужном режиме

#![allow(dead_code)] // Public strategy optimizer API

use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::core::diagnostics;
use crate::core::errors::{IsolateError, Result};
use crate::core::managers::{BlockedStrategiesManager, StrategyCacheManager, StrategyHistoryManager};
use crate::core::models::{
    DiagnosticResult, EnvInfo, Service, Strategy, StrategyEngine as EngineType,
};
use crate::core::strategy_engine::SharedStrategyEngine;
use crate::core::testing::{EndpointRegistry, HttpProber, ProbeConfig, ScoreCalculator, StrategyScore, TestEndpoint};

use super::events::{OptimizationProgress, OptimizationResult, OptimizationStage};

// ============================================================================
// Constants
// ============================================================================

/// Минимальный score для принятия стратегии
const MIN_ACCEPTABLE_SCORE: f64 = 0.6;
/// Максимальное количество параллельных VLESS тестов
const MAX_PARALLEL_VLESS_TESTS: usize = 5;
/// Задержка между последовательными Zapret тестами (мс)
const ZAPRET_TEST_DELAY_MS: u64 = 2500;
/// Задержка инициализации стратегии (мс)
const STRATEGY_INIT_DELAY_MS: u64 = 500;
/// Задержка инициализации Zapret (мс)
const ZAPRET_INIT_DELAY_MS: u64 = 1000;

// ============================================================================
// StrategyOptimizer
// ============================================================================

/// Координатор одноразовой оптимизации стратегий
pub struct StrategyOptimizer {
    /// Движок стратегий
    engine: SharedStrategyEngine,
    /// HTTP prober для тестирования
    prober: HttpProber,
    /// Калькулятор score
    scorer: ScoreCalculator,
    /// Реестр тестовых endpoints
    endpoint_registry: EndpointRegistry,
    /// Менеджер кэша стратегий
    cache_manager: Arc<StrategyCacheManager>,
    /// Менеджер заблокированных стратегий
    blocked_manager: Arc<BlockedStrategiesManager>,
    /// Менеджер истории стратегий
    history_manager: Arc<StrategyHistoryManager>,
    /// Канал прогресса
    progress_tx: broadcast::Sender<OptimizationProgress>,
    /// Токен отмены
    cancel_token: RwLock<Option<CancellationToken>>,
}

impl StrategyOptimizer {
    /// Создаёт новый оптимизатор
    pub fn new(
        engine: SharedStrategyEngine,
        cache_manager: Arc<StrategyCacheManager>,
        blocked_manager: Arc<BlockedStrategiesManager>,
        history_manager: Arc<StrategyHistoryManager>,
    ) -> Self {
        let (progress_tx, _) = broadcast::channel(100);

        Self {
            engine,
            prober: HttpProber::new(ProbeConfig::default()),
            scorer: ScoreCalculator::with_default_weights(),
            endpoint_registry: EndpointRegistry::default(),
            cache_manager,
            blocked_manager,
            history_manager,
            progress_tx,
            cancel_token: RwLock::new(None),
        }
    }

    /// Создаёт оптимизатор с кастомной конфигурацией prober
    pub fn with_prober_config(
        engine: SharedStrategyEngine,
        cache_manager: Arc<StrategyCacheManager>,
        blocked_manager: Arc<BlockedStrategiesManager>,
        history_manager: Arc<StrategyHistoryManager>,
        prober_config: ProbeConfig,
    ) -> Self {
        let (progress_tx, _) = broadcast::channel(100);

        Self {
            engine,
            prober: HttpProber::new(prober_config),
            scorer: ScoreCalculator::with_default_weights(),
            endpoint_registry: EndpointRegistry::default(),
            cache_manager,
            blocked_manager,
            history_manager,
            progress_tx,
            cancel_token: RwLock::new(None),
        }
    }

    /// Подписывается на события прогресса
    pub fn subscribe(&self) -> broadcast::Receiver<OptimizationProgress> {
        self.progress_tx.subscribe()
    }

    /// Запускает процесс оптимизации
    pub async fn optimize(
        &self,
        env_info: &EnvInfo,
        strategies: &[Strategy],
        services: &[Service],
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
        services: &[Service],
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

        let candidates = self.select_candidates(strategies, &diagnostic).await;
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
                .test_vless_parallel(&vless_candidates, &cancel_token)
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
                .test_zapret_sequential(&zapret_candidates, &cancel_token)
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

        // 7. Применение стратегии
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

        self.apply_strategy(strategy).await?;

        // Сохраняем в кэш
        if let Err(e) = self
            .cache_manager
            .set(&env_key, &best.strategy_id, best.score)
            .await
        {
            warn!(error = %e, "Failed to cache strategy");
        }

        // Записываем в историю
        if let Err(e) = self
            .history_manager
            .record_success("global", &best.strategy_id)
            .await
        {
            warn!(error = %e, "Failed to record history");
        }

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
        if let Some(cached) = self.cache_manager.get(env_key).await {
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
                self.apply_strategy(strategy).await?;

                return Ok(Some(OptimizationResult::from_cache(
                    cached.strategy_id,
                    strategy.name.clone(),
                    cached.score,
                )));
            }
        }

        Ok(None)
    }

    /// Запускает DPI-диагностику
    async fn run_diagnostics(&self, services: &[Service]) -> Result<DiagnosticResult> {
        info!("Starting DPI diagnostics for {} services", services.len());

        // Запускаем реальную диагностику
        let profile = diagnostics::diagnose().await?;

        // Определяем какие сервисы заблокированы
        let mut tested_services = Vec::new();
        let mut blocked_services = Vec::new();

        for service in services {
            tested_services.push(service.id.clone());

            if let Some(test_url) = service.get_test_url() {
                if let Some(domain) = extract_domain_from_url(&test_url) {
                    let service_profile = diagnostics::diagnose_domain(&domain).await?;

                    if service_profile.kind != crate::core::models::DpiKind::NoBlock {
                        blocked_services.push(service.id.clone());
                        debug!(
                            service_id = %service.id,
                            domain = %domain,
                            kind = ?service_profile.kind,
                            "Service blocked"
                        );
                    }
                }
            }
        }

        info!(
            dpi_kind = ?profile.kind,
            tested = tested_services.len(),
            blocked = blocked_services.len(),
            "DPI diagnostics complete"
        );

        Ok(DiagnosticResult {
            profile,
            tested_services,
            blocked_services,
        })
    }

    /// Выбирает кандидатов на основе диагностики и исключает заблокированные
    async fn select_candidates<'a>(
        &self,
        strategies: &'a [Strategy],
        diagnostic: &DiagnosticResult,
    ) -> Vec<&'a Strategy> {
        let candidate_families = &diagnostic.profile.candidate_families;

        let mut candidates = Vec::new();

        for strategy in strategies {
            // Фильтруем по family
            if !candidate_families.contains(&strategy.family) {
                continue;
            }

            // Проверяем, не заблокирована ли стратегия
            let is_blocked = self
                .blocked_manager
                .is_blocked("global", &strategy.id)
                .await;

            if is_blocked {
                debug!(
                    strategy_id = %strategy.id,
                    "Strategy is blocked, skipping"
                );
                continue;
            }

            candidates.push(strategy);
        }

        candidates
    }

    /// Тестирует VLESS стратегии параллельно
    async fn test_vless_parallel(
        &self,
        strategies: &[&Strategy],
        cancel_token: &CancellationToken,
    ) -> Result<Vec<StrategyScore>> {
        let mut scores = Vec::new();
        let total = strategies.len();
        
        // Используем Arc для shared данных вместо клонирования
        let endpoints: Arc<[TestEndpoint]> = self.endpoint_registry
            .get_critical()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
            .into();
        let prober_config = Arc::new(self.prober.config().clone());

        // Разбиваем на чанки для ограничения параллелизма
        for (chunk_idx, chunk) in strategies.chunks(MAX_PARALLEL_VLESS_TESTS).enumerate() {
            self.check_cancelled(cancel_token)?;

            let mut handles = Vec::new();

            for strategy in chunk {
                let strategy_id = strategy.id.clone();
                // Используем Arc для стратегии вместо полного клонирования
                let strategy_arc = Arc::new((*strategy).clone());
                let engine = self.engine.clone();
                let endpoints_ref = Arc::clone(&endpoints);
                let config_ref = Arc::clone(&prober_config);

                let handle = tokio::spawn(async move {
                    test_strategy_socks_arc(&engine, &strategy_arc, &endpoints_ref, &config_ref).await
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
                    format!(
                        "Протестировано {} из {} VLESS стратегий",
                        tested.min(total),
                        total
                    ),
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
        cancel_token: &CancellationToken,
    ) -> Result<Vec<StrategyScore>> {
        let mut scores = Vec::new();
        let total = strategies.len();
        
        // Используем Arc для shared endpoints
        let endpoints: Arc<[TestEndpoint]> = self.endpoint_registry
            .get_critical()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
            .into();

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

            // Тестируем стратегию в GLOBAL-режиме
            match test_strategy_global(&self.engine, strategy, &endpoints, &self.prober).await {
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
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
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

            return Ok(OptimizationResult::from_testing(
                best.strategy_id.clone(),
                strategy.name.clone(),
                best.score,
                scores.to_vec(),
            ));
        }

        // Выбираем лучший по score
        let best = acceptable
            .iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("acceptable is not empty, checked above");

        let strategy = strategies
            .iter()
            .find(|s| s.id == best.strategy_id)
            .ok_or_else(|| IsolateError::StrategyNotFound(best.strategy_id.clone()))?;

        Ok(OptimizationResult::from_testing(
            best.strategy_id.clone(),
            strategy.name.clone(),
            best.score,
            scores.to_vec(),
        ))
    }

    /// Применяет стратегию в нужном режиме
    async fn apply_strategy(&self, strategy: &Strategy) -> Result<()> {
        let is_vless = matches!(strategy.engine, EngineType::SingBox | EngineType::Xray);

        if is_vless {
            // VLESS применяем в SOCKS режиме
            let port = self.engine.start_socks(strategy).await?;
            info!(strategy_id = %strategy.id, port = port, "Applied VLESS strategy in SOCKS mode");
        } else {
            // Zapret применяем в GLOBAL режиме
            self.engine.start_global(strategy).await?;
            info!(strategy_id = %strategy.id, "Applied Zapret strategy in GLOBAL mode");
        }

        Ok(())
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

/// Тестирует стратегию в SOCKS-режиме (для VLESS) с Arc для оптимизации памяти
async fn test_strategy_socks_arc(
    engine: &SharedStrategyEngine,
    strategy: &Arc<Strategy>,
    endpoints: &Arc<[TestEndpoint]>,
    prober_config: &Arc<ProbeConfig>,
) -> Result<StrategyScore> {
    // Запускаем стратегию в SOCKS-режиме
    let port = engine.start_socks(strategy).await?;

    // Даём время на инициализацию
    tokio::time::sleep(tokio::time::Duration::from_millis(STRATEGY_INIT_DELAY_MS)).await;

    // Создаём prober с shared config
    let prober = HttpProber::new((**prober_config).clone());

    // Выполняем тесты через SOCKS прокси
    let results = prober.probe_all(endpoints, Some(port)).await;

    // Рассчитываем score
    let scorer = ScoreCalculator::with_default_weights();
    let score = scorer.calculate(&strategy.id, &results);

    // Останавливаем стратегию
    engine.stop_socks(&strategy.id).await?;

    info!(
        strategy_id = %strategy.id,
        score = score.score,
        success_rate = score.success_rate,
        latency_avg = score.latency_avg,
        "SOCKS strategy test completed"
    );

    Ok(score)
}

/// Тестирует стратегию в SOCKS-режиме (для VLESS)
async fn test_strategy_socks(
    engine: &SharedStrategyEngine,
    strategy: &Strategy,
    endpoints: &[TestEndpoint],
    prober: &HttpProber,
) -> Result<StrategyScore> {
    // Запускаем стратегию в SOCKS-режиме
    let port = engine.start_socks(strategy).await?;

    // Даём время на инициализацию
    tokio::time::sleep(tokio::time::Duration::from_millis(STRATEGY_INIT_DELAY_MS)).await;

    // Выполняем тесты через SOCKS прокси
    let results = prober.probe_all(endpoints, Some(port)).await;

    // Рассчитываем score
    let scorer = ScoreCalculator::with_default_weights();
    let score = scorer.calculate(&strategy.id, &results);

    // Останавливаем стратегию
    engine.stop_socks(&strategy.id).await?;

    info!(
        strategy_id = %strategy.id,
        score = score.score,
        success_rate = score.success_rate,
        latency_avg = score.latency_avg,
        "SOCKS strategy test completed"
    );

    Ok(score)
}

/// Тестирует стратегию в GLOBAL-режиме (для Zapret)
async fn test_strategy_global(
    engine: &SharedStrategyEngine,
    strategy: &Strategy,
    endpoints: &Arc<[TestEndpoint]>,
    prober: &HttpProber,
) -> Result<StrategyScore> {
    // Запускаем стратегию в GLOBAL-режиме
    engine.start_global(strategy).await?;

    // Даём время на инициализацию (Zapret/WinDivert требует больше времени)
    tokio::time::sleep(tokio::time::Duration::from_millis(ZAPRET_INIT_DELAY_MS)).await;

    // Выполняем тесты напрямую (трафик идёт через WinDivert)
    let results = prober.probe_all(endpoints, None).await;

    // Рассчитываем score
    let scorer = ScoreCalculator::with_default_weights();
    let score = scorer.calculate(&strategy.id, &results);

    // Останавливаем стратегию
    engine.stop_global().await?;

    info!(
        strategy_id = %strategy.id,
        score = score.score,
        success_rate = score.success_rate,
        latency_avg = score.latency_avg,
        "GLOBAL strategy test completed"
    );

    Ok(score)
}

/// Извлекает домен из URL
fn extract_domain_from_url(url: &str) -> Option<String> {
    // Убираем протокол
    let without_protocol = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("wss://"))
        .or_else(|| url.strip_prefix("ws://"))
        .unwrap_or(url);

    // Берём часть до первого / или ?
    let host_part = without_protocol
        .split('/')
        .next()
        .unwrap_or(without_protocol)
        .split('?')
        .next()
        .unwrap_or(without_protocol);

    // Убираем порт если есть
    let domain = if host_part.contains('[') {
        // IPv6 адрес в квадратных скобках
        host_part.to_string()
    } else {
        host_part
            .rsplit(':')
            .last()
            .unwrap_or(host_part)
            .to_string()
    };

    if domain.is_empty() {
        None
    } else {
        Some(domain)
    }
}

// ============================================================================
// Thread-safe wrapper
// ============================================================================

/// Thread-safe обёртка для StrategyOptimizer
pub type SharedStrategyOptimizer = Arc<StrategyOptimizer>;

/// Создаёт shared экземпляр оптимизатора
pub fn create_optimizer(
    engine: SharedStrategyEngine,
    cache_manager: Arc<StrategyCacheManager>,
    blocked_manager: Arc<BlockedStrategiesManager>,
    history_manager: Arc<StrategyHistoryManager>,
) -> SharedStrategyOptimizer {
    Arc::new(StrategyOptimizer::new(
        engine,
        cache_manager,
        blocked_manager,
        history_manager,
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_from_url() {
        // Basic URLs
        assert_eq!(
            extract_domain_from_url("https://www.youtube.com/watch?v=123"),
            Some("www.youtube.com".to_string())
        );
        assert_eq!(
            extract_domain_from_url("https://discord.com:443/api"),
            Some("discord.com".to_string())
        );
        assert_eq!(
            extract_domain_from_url("http://example.com/path"),
            Some("example.com".to_string())
        );

        // WebSocket URLs
        assert_eq!(
            extract_domain_from_url("wss://ws.example.com/socket"),
            Some("ws.example.com".to_string())
        );

        // Without protocol
        assert_eq!(
            extract_domain_from_url("example.com/path"),
            Some("example.com".to_string())
        );

        // Empty
        assert_eq!(extract_domain_from_url(""), None);
    }

    #[test]
    fn test_min_acceptable_score() {
        assert!((MIN_ACCEPTABLE_SCORE - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_PARALLEL_VLESS_TESTS, 5);
        assert_eq!(ZAPRET_TEST_DELAY_MS, 2500);
        assert_eq!(STRATEGY_INIT_DELAY_MS, 500);
        assert_eq!(ZAPRET_INIT_DELAY_MS, 1000);
    }
}

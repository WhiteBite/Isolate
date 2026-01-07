//! Strategy Metrics - сбор метрик производительности стратегий в реальном времени
//!
//! Модуль предоставляет функциональность для:
//! - Отслеживания uptime активной стратегии
//! - Подсчёта bytes sent/received (базовые метрики)
//! - Учёта количества соединений и ошибок
//! - Real-time метрики: connections_active, bytes_per_second, errors_per_minute
//! - Периодические снимки (snapshots) каждые 10 секунд
//! - Хранение истории с ring buffer на 1000 записей
//! - Агрегированная статистика за периоды (1h, 24h, 7d)
//! - Экспорт в CSV

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

// ============================================================================
// Constants
// ============================================================================

/// Максимальное количество записей в истории метрик
const MAX_HISTORY_ENTRIES: usize = 1000;

/// Интервал автоматического обновления uptime (секунды)
const UPTIME_UPDATE_INTERVAL_SECS: u64 = 1;

/// Интервал автоматического snapshot (секунды)
const SNAPSHOT_INTERVAL_SECS: u64 = 10;

/// Максимальный возраст данных для автоочистки (дни)
const MAX_DATA_AGE_DAYS: i64 = 7;

/// Размер окна для расчёта bytes_per_second (секунды)
const RATE_WINDOW_SECS: u64 = 5;

/// Размер окна для расчёта errors_per_minute (секунды)
const ERROR_RATE_WINDOW_SECS: u64 = 60;

// ============================================================================
// Types
// ============================================================================

/// Метрики производительности стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    /// ID активной стратегии
    pub strategy_id: String,
    /// Время запуска стратегии
    pub started_at: DateTime<Utc>,
    /// Время работы в секундах
    pub uptime_secs: u64,
    /// Отправлено байт
    pub bytes_sent: u64,
    /// Получено байт
    pub bytes_received: u64,
    /// Количество соединений (всего)
    pub connection_count: u32,
    /// Количество активных соединений (real-time)
    pub connections_active: u32,
    /// Количество ошибок
    pub error_count: u32,
    /// Последняя ошибка
    pub last_error: Option<String>,
    /// Время последнего обновления
    pub last_updated: DateTime<Utc>,
    /// Real-time: байт в секунду (отправка)
    pub bytes_sent_per_second: f64,
    /// Real-time: байт в секунду (получение)
    pub bytes_received_per_second: f64,
    /// Real-time: ошибок в минуту
    pub errors_per_minute: f64,
}

impl StrategyMetrics {
    /// Создаёт новые метрики для стратегии
    pub fn new(strategy_id: &str) -> Self {
        let now = Utc::now();
        Self {
            strategy_id: strategy_id.to_string(),
            started_at: now,
            uptime_secs: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connection_count: 0,
            connections_active: 0,
            error_count: 0,
            last_error: None,
            last_updated: now,
            bytes_sent_per_second: 0.0,
            bytes_received_per_second: 0.0,
            errors_per_minute: 0.0,
        }
    }

    /// Обновляет uptime на основе времени запуска
    pub fn update_uptime(&mut self) {
        let now = Utc::now();
        self.uptime_secs = (now - self.started_at).num_seconds().max(0) as u64;
        self.last_updated = now;
    }
}

/// Снимок метрик в определённый момент времени (для периодического сбора)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Время снимка
    pub timestamp: DateTime<Utc>,
    /// ID стратегии
    pub strategy_id: String,
    /// Uptime на момент снимка
    pub uptime_secs: u64,
    /// Bytes sent на момент снимка
    pub bytes_sent: u64,
    /// Bytes received на момент снимка
    pub bytes_received: u64,
    /// Количество соединений (всего)
    pub connection_count: u32,
    /// Активные соединения
    pub connections_active: u32,
    /// Количество ошибок
    pub error_count: u32,
    /// Bytes sent per second (средний за период)
    pub bytes_sent_per_second: f64,
    /// Bytes received per second (средний за период)
    pub bytes_received_per_second: f64,
    /// Errors per minute
    pub errors_per_minute: f64,
}

impl From<&StrategyMetrics> for MetricsSnapshot {
    fn from(metrics: &StrategyMetrics) -> Self {
        Self {
            timestamp: Utc::now(),
            strategy_id: metrics.strategy_id.clone(),
            uptime_secs: metrics.uptime_secs,
            bytes_sent: metrics.bytes_sent,
            bytes_received: metrics.bytes_received,
            connection_count: metrics.connection_count,
            connections_active: metrics.connections_active,
            error_count: metrics.error_count,
            bytes_sent_per_second: metrics.bytes_sent_per_second,
            bytes_received_per_second: metrics.bytes_received_per_second,
            errors_per_minute: metrics.errors_per_minute,
        }
    }
}

/// Агрегированная статистика за период
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    /// Начало периода
    pub period_start: DateTime<Utc>,
    /// Конец периода
    pub period_end: DateTime<Utc>,
    /// Общее время работы (секунды)
    pub total_uptime_secs: u64,
    /// Всего отправлено байт
    pub total_bytes_sent: u64,
    /// Всего получено байт
    pub total_bytes_received: u64,
    /// Всего соединений
    pub total_connections: u32,
    /// Всего ошибок
    pub total_errors: u32,
    /// Средняя скорость отправки (bytes/sec)
    pub avg_bytes_sent_per_second: f64,
    /// Средняя скорость получения (bytes/sec)
    pub avg_bytes_received_per_second: f64,
    /// Пиковая скорость отправки (bytes/sec)
    pub peak_bytes_sent_per_second: f64,
    /// Пиковая скорость получения (bytes/sec)
    pub peak_bytes_received_per_second: f64,
    /// Средний error rate (errors/min)
    pub avg_errors_per_minute: f64,
    /// Количество снимков в периоде
    pub snapshot_count: usize,
}

/// Период для запроса истории
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HistoryPeriod {
    /// Последний час
    Hour1,
    /// Последние 24 часа
    Hours24,
    /// Последние 7 дней
    Days7,
    /// Произвольное количество часов
    Custom(u32),
}

impl HistoryPeriod {
    /// Возвращает количество часов для периода
    pub fn hours(&self) -> u32 {
        match self {
            HistoryPeriod::Hour1 => 1,
            HistoryPeriod::Hours24 => 24,
            HistoryPeriod::Days7 => 24 * 7,
            HistoryPeriod::Custom(h) => *h,
        }
    }
}

/// Точка данных для расчёта rate (bytes/sec, errors/min)
#[derive(Debug, Clone)]
struct RateDataPoint {
    timestamp: DateTime<Utc>,
    bytes_sent: u64,
    bytes_received: u64,
    error_count: u32,
}

/// Запись истории метрик (снимок в определённый момент времени)
/// Сохраняется для обратной совместимости, но рекомендуется использовать MetricsSnapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsHistoryEntry {
    /// Время записи
    pub timestamp: DateTime<Utc>,
    /// ID стратегии
    pub strategy_id: String,
    /// Uptime на момент записи
    pub uptime_secs: u64,
    /// Bytes sent на момент записи
    pub bytes_sent: u64,
    /// Bytes received на момент записи
    pub bytes_received: u64,
    /// Количество соединений
    pub connection_count: u32,
    /// Количество ошибок
    pub error_count: u32,
}

impl From<&StrategyMetrics> for MetricsHistoryEntry {
    fn from(metrics: &StrategyMetrics) -> Self {
        Self {
            timestamp: Utc::now(),
            strategy_id: metrics.strategy_id.clone(),
            uptime_secs: metrics.uptime_secs,
            bytes_sent: metrics.bytes_sent,
            bytes_received: metrics.bytes_received,
            connection_count: metrics.connection_count,
            error_count: metrics.error_count,
        }
    }
}

impl From<&MetricsSnapshot> for MetricsHistoryEntry {
    fn from(snapshot: &MetricsSnapshot) -> Self {
        Self {
            timestamp: snapshot.timestamp,
            strategy_id: snapshot.strategy_id.clone(),
            uptime_secs: snapshot.uptime_secs,
            bytes_sent: snapshot.bytes_sent,
            bytes_received: snapshot.bytes_received,
            connection_count: snapshot.connection_count,
            error_count: snapshot.error_count,
        }
    }
}

/// История метрик с ring buffer
#[derive(Debug)]
pub struct MetricsHistory {
    /// Ring buffer снимков
    snapshots: VecDeque<MetricsSnapshot>,
    /// Максимальный размер
    max_size: usize,
}

impl MetricsHistory {
    /// Создаёт новую историю с заданным размером
    pub fn new(max_size: usize) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Добавляет снимок в историю
    pub fn push(&mut self, snapshot: MetricsSnapshot) {
        if self.snapshots.len() >= self.max_size {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    /// Возвращает снимки за указанный период
    pub fn get_since(&self, since: DateTime<Utc>) -> Vec<MetricsSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Возвращает все снимки
    pub fn get_all(&self) -> Vec<MetricsSnapshot> {
        self.snapshots.iter().cloned().collect()
    }

    /// Очищает историю
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Удаляет записи старше указанной даты
    pub fn cleanup_older_than(&mut self, cutoff: DateTime<Utc>) {
        self.snapshots.retain(|s| s.timestamp >= cutoff);
    }

    /// Возвращает количество записей
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Проверяет, пуста ли история
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Вычисляет агрегированную статистику за период
    pub fn get_aggregated_stats(&self, since: DateTime<Utc>) -> Option<AggregatedStats> {
        let snapshots: Vec<_> = self.snapshots
            .iter()
            .filter(|s| s.timestamp >= since)
            .collect();

        if snapshots.is_empty() {
            return None;
        }

        let period_start = snapshots.first().map(|s| s.timestamp).unwrap_or_else(Utc::now);
        let period_end = snapshots.last().map(|s| s.timestamp).unwrap_or_else(Utc::now);

        // Суммируем метрики
        let total_uptime_secs = snapshots.last().map(|s| s.uptime_secs).unwrap_or(0);
        let total_bytes_sent = snapshots.last().map(|s| s.bytes_sent).unwrap_or(0);
        let total_bytes_received = snapshots.last().map(|s| s.bytes_received).unwrap_or(0);
        let total_connections = snapshots.last().map(|s| s.connection_count).unwrap_or(0);
        let total_errors = snapshots.last().map(|s| s.error_count).unwrap_or(0);

        // Средние и пиковые значения
        let (sum_sent_rate, sum_recv_rate, sum_error_rate) = snapshots.iter().fold(
            (0.0, 0.0, 0.0),
            |(sent, recv, err), s| {
                (
                    sent + s.bytes_sent_per_second,
                    recv + s.bytes_received_per_second,
                    err + s.errors_per_minute,
                )
            },
        );

        let count = snapshots.len() as f64;
        let avg_bytes_sent_per_second = sum_sent_rate / count;
        let avg_bytes_received_per_second = sum_recv_rate / count;
        let avg_errors_per_minute = sum_error_rate / count;

        let peak_bytes_sent_per_second = snapshots
            .iter()
            .map(|s| s.bytes_sent_per_second)
            .fold(0.0_f64, |a, b| a.max(b));

        let peak_bytes_received_per_second = snapshots
            .iter()
            .map(|s| s.bytes_received_per_second)
            .fold(0.0_f64, |a, b| a.max(b));

        Some(AggregatedStats {
            period_start,
            period_end,
            total_uptime_secs,
            total_bytes_sent,
            total_bytes_received,
            total_connections,
            total_errors,
            avg_bytes_sent_per_second,
            avg_bytes_received_per_second,
            peak_bytes_sent_per_second,
            peak_bytes_received_per_second,
            avg_errors_per_minute,
            snapshot_count: snapshots.len(),
        })
    }
}

/// Внутреннее состояние коллектора метрик
struct MetricsState {
    /// Текущие метрики (None если стратегия не запущена)
    current: Option<StrategyMetrics>,
    /// История метрик (legacy формат для обратной совместимости)
    history: Vec<MetricsHistoryEntry>,
    /// Новая история с ring buffer
    snapshots: MetricsHistory,
    /// Флаг активности сбора
    collecting: bool,
    /// Данные для расчёта rate
    rate_data: VecDeque<RateDataPoint>,
    /// Handle для background task
    snapshot_task_running: bool,
    /// Cancellation token для graceful shutdown
    shutdown_token: Option<tokio_util::sync::CancellationToken>,
}

impl Default for MetricsState {
    fn default() -> Self {
        Self {
            current: None,
            history: Vec::new(),
            snapshots: MetricsHistory::new(MAX_HISTORY_ENTRIES),
            collecting: false,
            rate_data: VecDeque::with_capacity(120), // ~2 минуты данных
            snapshot_task_running: false,
            shutdown_token: None,
        }
    }
}

// ============================================================================
// MetricsCollector
// ============================================================================

/// Коллектор метрик стратегий
///
/// Thread-safe сервис для сбора и хранения метрик производительности.
/// Поддерживает одну активную стратегию в каждый момент времени.
/// 
/// Функциональность:
/// - Real-time метрики (connections_active, bytes_per_second, errors_per_minute)
/// - Периодические снимки каждые 10 секунд
/// - Ring buffer история на 1000 записей
/// - Агрегированная статистика за периоды
/// - Экспорт в CSV
pub struct MetricsCollector {
    state: Arc<RwLock<MetricsState>>,
}

impl MetricsCollector {
    /// Создаёт новый коллектор метрик
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MetricsState::default())),
        }
    }

    /// Начинает сбор метрик для стратегии
    ///
    /// Если уже идёт сбор для другой стратегии, он будет остановлен.
    /// Запускает background task для периодических снимков.
    pub async fn start_collection(&self, strategy_id: &str) {
        let mut state = self.state.write().await;

        // Сохраняем предыдущие метрики в историю если были
        if let Some(ref metrics) = state.current {
            let entry = MetricsHistoryEntry::from(metrics);
            let snapshot = MetricsSnapshot::from(metrics);
            debug!(
                strategy_id = %metrics.strategy_id,
                uptime_secs = metrics.uptime_secs,
                "Saved previous metrics to history"
            );
            Self::add_to_history(&mut state.history, entry);
            state.snapshots.push(snapshot);
        }

        // Создаём новые метрики
        state.current = Some(StrategyMetrics::new(strategy_id));
        state.collecting = true;
        state.rate_data.clear();

        info!(strategy_id, "Started metrics collection");

        // Запускаем background snapshot task если ещё не запущен
        if !state.snapshot_task_running {
            state.snapshot_task_running = true;
            
            // Создаём cancellation token для graceful shutdown
            let shutdown_token = tokio_util::sync::CancellationToken::new();
            state.shutdown_token = Some(shutdown_token.clone());
            
            let state_clone = self.state.clone();
            tokio::spawn(async move {
                Self::snapshot_background_task(state_clone, shutdown_token).await;
            });
        }
    }

    /// Background task для периодических снимков с graceful shutdown
    async fn snapshot_background_task(
        state: Arc<RwLock<MetricsState>>,
        shutdown_token: tokio_util::sync::CancellationToken,
    ) {
        let mut interval = interval(tokio::time::Duration::from_secs(SNAPSHOT_INTERVAL_SECS));
        
        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    debug!("Snapshot background task received shutdown signal");
                    break;
                }
                _ = interval.tick() => {
                    let mut state_guard = state.write().await;
                    
                    // Проверяем, активен ли сбор
                    if !state_guard.collecting {
                        state_guard.snapshot_task_running = false;
                        debug!("Snapshot background task stopped - collection inactive");
                        break;
                    }

                    // Обновляем rate метрики и делаем снимок
                    Self::update_rate_metrics(&mut state_guard);
                    
                    // Делаем снимок текущих метрик
                    if let Some(ref mut metrics) = state_guard.current {
                        metrics.update_uptime();
                        
                        let snapshot = MetricsSnapshot::from(&*metrics);
                        let strategy_id = metrics.strategy_id.clone();
                        let uptime_secs = metrics.uptime_secs;
                        let bytes_sent_per_second = metrics.bytes_sent_per_second;
                        
                        state_guard.snapshots.push(snapshot);
                        
                        debug!(
                            strategy_id = %strategy_id,
                            uptime_secs,
                            bytes_sent_per_second,
                            "Periodic snapshot taken"
                        );
                    }

                    // Автоочистка старых данных
                    let cutoff = Utc::now() - Duration::days(MAX_DATA_AGE_DAYS);
                    state_guard.snapshots.cleanup_older_than(cutoff);
                    state_guard.history.retain(|e| e.timestamp >= cutoff);
                }
            }
        }
        
        // Помечаем task как остановленный
        let mut state_guard = state.write().await;
        state_guard.snapshot_task_running = false;
        state_guard.shutdown_token = None;
        debug!("Snapshot background task exited cleanly");
    }

    /// Обновляет rate метрики на основе накопленных данных
    fn update_rate_metrics(state: &mut MetricsState) {
        if let Some(ref mut metrics) = state.current {
            let now = Utc::now();
            
            // Добавляем текущую точку данных
            state.rate_data.push_back(RateDataPoint {
                timestamp: now,
                bytes_sent: metrics.bytes_sent,
                bytes_received: metrics.bytes_received,
                error_count: metrics.error_count,
            });

            // Удаляем старые точки (старше ERROR_RATE_WINDOW_SECS)
            let cutoff = now - Duration::seconds(ERROR_RATE_WINDOW_SECS as i64);
            while let Some(front) = state.rate_data.front() {
                if front.timestamp < cutoff {
                    state.rate_data.pop_front();
                } else {
                    break;
                }
            }

            // Рассчитываем bytes per second (за последние RATE_WINDOW_SECS секунд)
            let rate_cutoff = now - Duration::seconds(RATE_WINDOW_SECS as i64);
            let rate_points: Vec<_> = state.rate_data
                .iter()
                .filter(|p| p.timestamp >= rate_cutoff)
                .collect();

            if rate_points.len() >= 2 {
                let first = rate_points.first().unwrap();
                let last = rate_points.last().unwrap();
                let duration_secs = (last.timestamp - first.timestamp).num_seconds().max(1) as f64;
                
                metrics.bytes_sent_per_second = 
                    (last.bytes_sent.saturating_sub(first.bytes_sent)) as f64 / duration_secs;
                metrics.bytes_received_per_second = 
                    (last.bytes_received.saturating_sub(first.bytes_received)) as f64 / duration_secs;
            }

            // Рассчитываем errors per minute (за последние ERROR_RATE_WINDOW_SECS секунд)
            if state.rate_data.len() >= 2 {
                let first = state.rate_data.front().unwrap();
                let last = state.rate_data.back().unwrap();
                let duration_mins = (last.timestamp - first.timestamp).num_seconds().max(1) as f64 / 60.0;
                
                metrics.errors_per_minute = 
                    (last.error_count.saturating_sub(first.error_count)) as f64 / duration_mins;
            }
        }
    }

    /// Останавливает сбор метрик
    ///
    /// Текущие метрики сохраняются в историю.
    /// Background task останавливается через cancellation token.
    pub async fn stop_collection(&self) {
        let mut state = self.state.write().await;

        // Отменяем background task через cancellation token
        if let Some(ref token) = state.shutdown_token {
            token.cancel();
            debug!("Sent shutdown signal to background task");
        }

        // Создаём entry до того как берём mutable borrow
        let entry_and_log = if let Some(ref mut metrics) = state.current {
            // Обновляем uptime перед сохранением
            metrics.update_uptime();

            let entry = MetricsHistoryEntry::from(&*metrics);
            let snapshot = MetricsSnapshot::from(&*metrics);

            Some((entry, snapshot, metrics.strategy_id.clone(), metrics.uptime_secs, metrics.bytes_sent, metrics.bytes_received))
        } else {
            None
        };

        if let Some((entry, snapshot, strategy_id, uptime_secs, bytes_sent, bytes_received)) = entry_and_log {
            Self::add_to_history(&mut state.history, entry);
            state.snapshots.push(snapshot);
            info!(
                strategy_id = %strategy_id,
                uptime_secs,
                bytes_sent,
                bytes_received,
                "Stopped metrics collection"
            );
        }

        state.current = None;
        state.collecting = false;
        state.rate_data.clear();
        state.shutdown_token = None;
    }

    /// Делает снимок текущих метрик
    pub async fn take_snapshot(&self) -> Option<MetricsSnapshot> {
        let mut state = self.state.write().await;
        
        // Сначала обновляем rate метрики
        Self::update_rate_metrics(&mut state);
        
        if let Some(ref mut metrics) = state.current {
            metrics.update_uptime();
            
            let snapshot = MetricsSnapshot::from(&*metrics);
            state.snapshots.push(snapshot.clone());
            
            Some(snapshot)
        } else {
            None
        }
    }

    /// Возвращает текущие метрики
    ///
    /// Автоматически обновляет uptime и rate метрики перед возвратом.
    pub async fn get_current_metrics(&self) -> Option<StrategyMetrics> {
        let mut state = self.state.write().await;

        // Сначала обновляем rate метрики
        Self::update_rate_metrics(&mut state);
        
        if let Some(ref mut metrics) = state.current {
            metrics.update_uptime();
            Some(metrics.clone())
        } else {
            None
        }
    }

    /// Проверяет, идёт ли сбор метрик
    pub async fn is_collecting(&self) -> bool {
        let state = self.state.read().await;
        state.collecting
    }

    /// Записывает ошибку
    pub async fn record_error(&self, error: &str) {
        let mut state = self.state.write().await;

        if let Some(ref mut metrics) = state.current {
            metrics.error_count += 1;
            metrics.last_error = Some(error.to_string());
            metrics.last_updated = Utc::now();

            debug!(
                strategy_id = %metrics.strategy_id,
                error_count = metrics.error_count,
                error,
                "Recorded error"
            );
        } else {
            warn!(error, "Attempted to record error but no active metrics collection");
        }
    }

    /// Увеличивает счётчик соединений
    pub async fn record_connection(&self) {
        let mut state = self.state.write().await;

        if let Some(ref mut metrics) = state.current {
            metrics.connection_count += 1;
            metrics.connections_active += 1;
            metrics.last_updated = Utc::now();
        }
    }

    /// Уменьшает счётчик активных соединений
    pub async fn record_connection_closed(&self) {
        let mut state = self.state.write().await;

        if let Some(ref mut metrics) = state.current {
            metrics.connections_active = metrics.connections_active.saturating_sub(1);
            metrics.last_updated = Utc::now();
        }
    }

    /// Добавляет bytes sent
    pub async fn add_bytes_sent(&self, bytes: u64) {
        let mut state = self.state.write().await;

        if let Some(ref mut metrics) = state.current {
            metrics.bytes_sent += bytes;
            metrics.last_updated = Utc::now();
        }
    }

    /// Добавляет bytes received
    pub async fn add_bytes_received(&self, bytes: u64) {
        let mut state = self.state.write().await;

        if let Some(ref mut metrics) = state.current {
            metrics.bytes_received += bytes;
            metrics.last_updated = Utc::now();
        }
    }

    /// Возвращает историю метрик за указанное количество часов (legacy формат)
    ///
    /// # Arguments
    /// * `hours` - Количество часов истории (0 = вся история)
    pub async fn get_history(&self, hours: u32) -> Vec<MetricsHistoryEntry> {
        let state = self.state.read().await;

        if hours == 0 {
            return state.history.clone();
        }

        let cutoff = Utc::now() - Duration::hours(hours as i64);
        state
            .history
            .iter()
            .filter(|entry| entry.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    /// Возвращает историю снимков за указанный период
    pub async fn get_snapshots(&self, period: HistoryPeriod) -> Vec<MetricsSnapshot> {
        let state = self.state.read().await;
        let since = Utc::now() - Duration::hours(period.hours() as i64);
        state.snapshots.get_since(since)
    }

    /// Возвращает агрегированную статистику за период
    pub async fn get_aggregated_stats(&self, period: HistoryPeriod) -> Option<AggregatedStats> {
        let state = self.state.read().await;
        let since = Utc::now() - Duration::hours(period.hours() as i64);
        state.snapshots.get_aggregated_stats(since)
    }

    /// Очищает историю метрик
    pub async fn clear_history(&self) {
        let mut state = self.state.write().await;
        state.history.clear();
        state.snapshots.clear();
        info!("Metrics history cleared");
    }

    /// Сбрасывает текущие метрики (без остановки сбора)
    pub async fn reset_current_metrics(&self) {
        let mut state = self.state.write().await;
        
        if let Some(ref mut metrics) = state.current {
            let strategy_id = metrics.strategy_id.clone();
            *metrics = StrategyMetrics::new(&strategy_id);
            state.rate_data.clear();
            info!(strategy_id, "Current metrics reset");
        }
    }

    /// Экспортирует метрики в CSV формат
    pub async fn export_to_csv(&self, period: HistoryPeriod) -> Result<String, std::io::Error> {
        let snapshots = self.get_snapshots(period).await;
        
        let mut csv = String::new();
        csv.push_str("timestamp,strategy_id,uptime_secs,bytes_sent,bytes_received,connection_count,connections_active,error_count,bytes_sent_per_second,bytes_received_per_second,errors_per_minute\n");
        
        for s in snapshots {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{:.2},{:.2},{:.2}\n",
                s.timestamp.to_rfc3339(),
                s.strategy_id,
                s.uptime_secs,
                s.bytes_sent,
                s.bytes_received,
                s.connection_count,
                s.connections_active,
                s.error_count,
                s.bytes_sent_per_second,
                s.bytes_received_per_second,
                s.errors_per_minute,
            ));
        }
        
        Ok(csv)
    }

    /// Экспортирует метрики в CSV файл
    pub async fn export_to_csv_file(&self, path: PathBuf, period: HistoryPeriod) -> Result<(), std::io::Error> {
        let csv = self.export_to_csv(period).await?;
        tokio::fs::write(path, csv).await
    }

    /// Добавляет запись в историю с ограничением размера
    fn add_to_history(history: &mut Vec<MetricsHistoryEntry>, entry: MetricsHistoryEntry) {
        history.push(entry);

        // Удаляем старые записи если превышен лимит
        if history.len() > MAX_HISTORY_ENTRIES {
            let excess = history.len() - MAX_HISTORY_ENTRIES;
            history.drain(0..excess);
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Global Instance
// ============================================================================

use std::sync::OnceLock;

/// Глобальный экземпляр коллектора метрик
static METRICS_COLLECTOR: OnceLock<Arc<MetricsCollector>> = OnceLock::new();

/// Возвращает глобальный коллектор метрик
pub fn get_metrics_collector() -> Arc<MetricsCollector> {
    METRICS_COLLECTOR
        .get_or_init(|| Arc::new(MetricsCollector::new()))
        .clone()
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Начинает сбор метрик для стратегии
pub async fn start_metrics_collection(strategy_id: &str) {
    get_metrics_collector().start_collection(strategy_id).await;
}

/// Останавливает сбор метрик
pub async fn stop_metrics_collection() {
    get_metrics_collector().stop_collection().await;
}

/// Возвращает текущие метрики
pub async fn get_current_metrics() -> Option<StrategyMetrics> {
    get_metrics_collector().get_current_metrics().await
}

/// Делает снимок текущих метрик
pub async fn take_snapshot() -> Option<MetricsSnapshot> {
    get_metrics_collector().take_snapshot().await
}

/// Записывает ошибку
pub async fn record_error(error: &str) {
    get_metrics_collector().record_error(error).await;
}

/// Записывает соединение
pub async fn record_connection() {
    get_metrics_collector().record_connection().await;
}

/// Записывает закрытие соединения
pub async fn record_connection_closed() {
    get_metrics_collector().record_connection_closed().await;
}

/// Добавляет bytes sent
pub async fn add_bytes_sent(bytes: u64) {
    get_metrics_collector().add_bytes_sent(bytes).await;
}

/// Добавляет bytes received
pub async fn add_bytes_received(bytes: u64) {
    get_metrics_collector().add_bytes_received(bytes).await;
}

/// Возвращает историю метрик (legacy формат)
pub async fn get_metrics_history(hours: u32) -> Vec<MetricsHistoryEntry> {
    get_metrics_collector().get_history(hours).await
}

/// Возвращает историю снимков за период
pub async fn get_snapshots_history(period: HistoryPeriod) -> Vec<MetricsSnapshot> {
    get_metrics_collector().get_snapshots(period).await
}

/// Возвращает агрегированную статистику за период
pub async fn get_aggregated_stats(period: HistoryPeriod) -> Option<AggregatedStats> {
    get_metrics_collector().get_aggregated_stats(period).await
}

/// Сбрасывает текущие метрики
pub async fn reset_metrics() {
    get_metrics_collector().reset_current_metrics().await;
}

/// Очищает всю историю метрик
pub async fn clear_metrics_history() {
    get_metrics_collector().clear_history().await;
}

/// Экспортирует метрики в CSV строку
pub async fn export_metrics_csv(period: HistoryPeriod) -> Result<String, std::io::Error> {
    get_metrics_collector().export_to_csv(period).await
}

/// Экспортирует метрики в CSV файл
pub async fn export_metrics_csv_file(path: PathBuf, period: HistoryPeriod) -> Result<(), std::io::Error> {
    get_metrics_collector().export_to_csv_file(path, period).await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection_lifecycle() {
        let collector = MetricsCollector::new();

        // Initially not collecting
        assert!(!collector.is_collecting().await);
        assert!(collector.get_current_metrics().await.is_none());

        // Start collection
        collector.start_collection("test-strategy").await;
        assert!(collector.is_collecting().await);

        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.strategy_id, "test-strategy");
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.connection_count, 0);
        assert_eq!(metrics.connections_active, 0);

        // Record some data
        collector.record_error("test error").await;
        collector.record_connection().await;
        collector.add_bytes_sent(1000).await;
        collector.add_bytes_received(2000).await;

        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.connection_count, 1);
        assert_eq!(metrics.connections_active, 1);
        assert_eq!(metrics.bytes_sent, 1000);
        assert_eq!(metrics.bytes_received, 2000);
        assert_eq!(metrics.last_error, Some("test error".to_string()));

        // Close connection
        collector.record_connection_closed().await;
        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.connections_active, 0);

        // Stop collection
        collector.stop_collection().await;
        assert!(!collector.is_collecting().await);
        assert!(collector.get_current_metrics().await.is_none());

        // Check history
        let history = collector.get_history(0).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].strategy_id, "test-strategy");
    }

    #[tokio::test]
    async fn test_metrics_history_limit() {
        let collector = MetricsCollector::new();

        // Create many entries
        for i in 0..1100 {
            collector.start_collection(&format!("strategy-{}", i)).await;
            collector.stop_collection().await;
        }

        let history = collector.get_history(0).await;
        assert!(history.len() <= MAX_HISTORY_ENTRIES);
    }

    #[tokio::test]
    async fn test_uptime_calculation() {
        let collector = MetricsCollector::new();
        collector.start_collection("test").await;

        // Wait a bit
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let metrics = collector.get_current_metrics().await.unwrap();
        // Uptime should be at least 0 (might be 0 if less than 1 second)
        assert!(metrics.uptime_secs >= 0);
    }

    #[tokio::test]
    async fn test_take_snapshot() {
        let collector = MetricsCollector::new();
        collector.start_collection("test-snapshot").await;
        
        collector.add_bytes_sent(500).await;
        collector.add_bytes_received(1000).await;
        
        let snapshot = collector.take_snapshot().await;
        assert!(snapshot.is_some());
        
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.strategy_id, "test-snapshot");
        assert_eq!(snapshot.bytes_sent, 500);
        assert_eq!(snapshot.bytes_received, 1000);
        
        collector.stop_collection().await;
    }

    #[tokio::test]
    async fn test_metrics_history_ring_buffer() {
        let mut history = MetricsHistory::new(5);
        
        for i in 0..10 {
            history.push(MetricsSnapshot {
                timestamp: Utc::now(),
                strategy_id: format!("strategy-{}", i),
                uptime_secs: i as u64,
                bytes_sent: 0,
                bytes_received: 0,
                connection_count: 0,
                connections_active: 0,
                error_count: 0,
                bytes_sent_per_second: 0.0,
                bytes_received_per_second: 0.0,
                errors_per_minute: 0.0,
            });
        }
        
        assert_eq!(history.len(), 5);
        let all = history.get_all();
        assert_eq!(all[0].strategy_id, "strategy-5");
        assert_eq!(all[4].strategy_id, "strategy-9");
    }

    #[tokio::test]
    async fn test_reset_metrics() {
        let collector = MetricsCollector::new();
        collector.start_collection("test-reset").await;
        
        collector.add_bytes_sent(1000).await;
        collector.record_error("error").await;
        
        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.bytes_sent, 1000);
        assert_eq!(metrics.error_count, 1);
        
        collector.reset_current_metrics().await;
        
        let metrics = collector.get_current_metrics().await.unwrap();
        assert_eq!(metrics.bytes_sent, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.strategy_id, "test-reset");
        
        collector.stop_collection().await;
    }

    #[tokio::test]
    async fn test_export_csv() {
        let collector = MetricsCollector::new();
        collector.start_collection("csv-test").await;
        
        collector.add_bytes_sent(100).await;
        collector.take_snapshot().await;
        
        let csv = collector.export_to_csv(HistoryPeriod::Hour1).await.unwrap();
        assert!(csv.contains("timestamp,strategy_id"));
        assert!(csv.contains("csv-test"));
        
        collector.stop_collection().await;
    }

    #[tokio::test]
    async fn test_aggregated_stats() {
        let mut history = MetricsHistory::new(100);
        
        // Add some snapshots
        for i in 0u32..5 {
            history.push(MetricsSnapshot {
                timestamp: Utc::now(),
                strategy_id: "test".to_string(),
                uptime_secs: ((i + 1) * 10) as u64,
                bytes_sent: ((i + 1) * 1000) as u64,
                bytes_received: ((i + 1) * 2000) as u64,
                connection_count: i + 1,
                connections_active: 1,
                error_count: i,
                bytes_sent_per_second: 100.0 * (i + 1) as f64,
                bytes_received_per_second: 200.0 * (i + 1) as f64,
                errors_per_minute: i as f64,
            });
        }
        
        let stats = history.get_aggregated_stats(Utc::now() - Duration::hours(1));
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert_eq!(stats.snapshot_count, 5);
        assert_eq!(stats.total_uptime_secs, 50);
        assert_eq!(stats.total_bytes_sent, 5000);
        assert_eq!(stats.total_bytes_received, 10000);
    }
}

//! Strategy Engine - управление запуском и остановкой стратегий
//!
//! Поддерживает два режима:
//! - SOCKS: локальный прокси на выделенном порту
//! - GLOBAL: глобальный перехват трафика через WinDivert
//!
//! КРИТИЧНО: Zapret стратегии запускаются ТОЛЬКО последовательно!
//! Используется nodpi_engine для управления winws процессами.
//!
//! ## Engine Mode
//! Движок поддерживает три режима работы:
//! - Real: реальный запуск winws/sing-box процессов
//! - Mock: симуляция для UI разработки (без реальных процессов)
//! - DpiTest: ожидает DPI Simulator для тестирования
//!

#![allow(dead_code)] // Public strategy engine API
//! ## Blocked Strategies
//! Движок автоматически блокирует стратегии после N последовательных неудач.
//! Заблокированные стратегии автоматически разблокируются через 1 час.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::core::blocked_strategies::{BlockedStrategiesManager, BlockedStrategy, BlockedStrategiesConfig};
use crate::core::errors::{IsolateError, Result};
use crate::core::models::{LaunchTemplate, Strategy, StrategyEngine as EngineType, WinDivertMode};
use crate::core::nodpi_engine::{
    self, build_winws_args_from_template_with_mode,
    get_binary_path_from_template,
    is_windivert_active, reset_windivert_flag,
    NoDpiHandle, WinDivertGuard,
    start_nodpi_from_strategy_with_extra_hostlist,
};
use crate::core::routing_converter::convert_routing_rules;
use crate::core::dynamic_hostlist::create_dynamic_hostlist;
use crate::core::storage::Storage;
use crate::core::strategy_metrics;

// ============================================================================
// Constants
// ============================================================================

/// Начальный порт для SOCKS-прокси
const SOCKS_PORT_START: u16 = 10800;
/// Максимальное количество одновременных SOCKS-портов
const MAX_SOCKS_PORTS: u16 = 100;
/// Таймаут graceful shutdown в миллисекундах
const SHUTDOWN_TIMEOUT_MS: u64 = 3000;
/// Задержка между запусками Zapret стратегий
const ZAPRET_LAUNCH_DELAY_MS: u64 = 2500;
/// Минимальная задержка для mock-симуляции (мс)
const MOCK_DELAY_MIN_MS: u64 = 500;
/// Максимальная задержка для mock-симуляции (мс)
const MOCK_DELAY_MAX_MS: u64 = 1500;
/// Вероятность успеха в mock-режиме (80%)
const MOCK_SUCCESS_RATE: f64 = 0.8;

// ============================================================================
// Types
// ============================================================================

/// Режим работы движка стратегий
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EngineMode {
    /// Реальный режим - запускает настоящие winws/sing-box процессы
    #[default]
    Real,
    /// Mock режим - симуляция для UI разработки без реальных процессов
    Mock,
    /// DPI Test режим - ожидает подключения DPI Simulator
    DpiTest,
}

impl std::fmt::Display for EngineMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineMode::Real => write!(f, "Real"),
            EngineMode::Mock => write!(f, "Mock"),
            EngineMode::DpiTest => write!(f, "DpiTest"),
        }
    }
}

/// Режим запуска стратегии
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LaunchMode {
    /// Локальный SOCKS-прокси
    Socks,
    /// Глобальный перехват трафика
    Global,
}

/// Информация о запущенном процессе
/// 
/// NOTE: Some fields are stored for debugging and future process management features.
#[derive(Debug)]
#[allow(dead_code)] // Fields used for debugging and future features
struct RunningProcess {
    child: Option<Child>,
    strategy_id: String,
    mode: LaunchMode,
    port: Option<u16>,
    engine: EngineType,
    /// RAII guard for WinDivert flag - automatically releases on drop
    windivert_guard: Option<WinDivertGuard>,
}

/// Информация о запущенной Zapret стратегии (через nodpi_engine)
/// 
/// NOTE: mode field is stored for debugging and future features.
#[allow(dead_code)] // Fields used for debugging and future features
struct RunningZapretStrategy {
    handle: NoDpiHandle,
    strategy_id: String,
    mode: LaunchMode,
}

/// Менеджер портов для SOCKS-прокси
/// 
/// Thread-safe port allocation with atomic check-and-reserve operations.
/// Uses Mutex internally for synchronization.
#[derive(Debug, Default)]
struct PortManager {
    allocated: HashMap<u16, String>, // port -> strategy_id
}

impl PortManager {
    /// Atomically checks if port is available and allocates it.
    /// This is the ONLY method that should be used for allocation.
    /// 
    /// # Arguments
    /// * `strategy_id` - ID of the strategy requesting the port
    /// 
    /// # Returns
    /// * `Ok(u16)` - Allocated port number
    /// * `Err(IsolateError)` - No free ports available
    fn allocate(&mut self, strategy_id: &str) -> Result<u16> {
        // Check if strategy already has a port allocated
        if let Some(existing_port) = self.get_port(strategy_id) {
            debug!(port = existing_port, strategy_id, "Strategy already has allocated port");
            return Ok(existing_port);
        }
        
        // Find and allocate first free port atomically
        for port in SOCKS_PORT_START..(SOCKS_PORT_START + MAX_SOCKS_PORTS) {
            if let std::collections::hash_map::Entry::Vacant(e) = self.allocated.entry(port) {
                e.insert(strategy_id.to_string());
                debug!(port, strategy_id, "Allocated SOCKS port");
                return Ok(port);
            }
        }
        Err(IsolateError::Process("No free SOCKS ports available".into()))
    }
    
    /// Atomically tries to allocate a specific port.
    /// 
    /// # Arguments
    /// * `port` - Specific port to allocate
    /// * `strategy_id` - ID of the strategy requesting the port
    /// 
    /// # Returns
    /// * `Ok(())` - Port allocated successfully
    /// * `Err(IsolateError)` - Port already in use
    #[allow(dead_code)]
    fn allocate_specific(&mut self, port: u16, strategy_id: &str) -> Result<()> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.allocated.entry(port) {
            e.insert(strategy_id.to_string());
            debug!(port, strategy_id, "Allocated specific SOCKS port");
            Ok(())
        } else {
            let existing = self.allocated.get(&port).map(|s| s.as_str()).unwrap_or("unknown");
            Err(IsolateError::Process(format!(
                "Port {} already allocated to strategy '{}'",
                port, existing
            )))
        }
    }

    /// Освобождает порт
    fn release(&mut self, port: u16) {
        if let Some(strategy_id) = self.allocated.remove(&port) {
            debug!(port, strategy_id, "Released SOCKS port");
        }
    }
    
    /// Освобождает все порты для стратегии
    fn release_by_strategy(&mut self, strategy_id: &str) {
        let ports_to_release: Vec<u16> = self.allocated
            .iter()
            .filter(|(_, sid)| *sid == strategy_id)
            .map(|(port, _)| *port)
            .collect();
        
        for port in ports_to_release {
            self.release(port);
        }
    }

    /// Проверяет, выделен ли порт
    fn is_allocated(&self, port: u16) -> bool {
        self.allocated.contains_key(&port)
    }

    /// Получает порт для стратегии
    fn get_port(&self, strategy_id: &str) -> Option<u16> {
        self.allocated
            .iter()
            .find(|(_, sid)| *sid == strategy_id)
            .map(|(port, _)| *port)
    }
    
    /// Получает количество выделенных портов
    #[allow(dead_code)]
    fn allocated_count(&self) -> usize {
        self.allocated.len()
    }
}

// ============================================================================
// Strategy Engine
// ============================================================================

/// Движок управления стратегиями
///
/// ## Lock Ordering (КРИТИЧНО для избежания deadlock'ов!)
///
/// При захвате нескольких lock'ов ВСЕГДА соблюдать порядок:
/// 1. `zapret_lock` — координация запуска Zapret стратегий
/// 2. `nodpi_engine::ZAPRET_LAUNCH_LOCK` — внутренний lock nodpi_engine (захватывается автоматически)
/// 3. `zapret_strategy` — состояние текущей Zapret стратегии
/// 4. `processes` — состояние не-Zapret процессов
/// 5. `port_manager` — выделение портов
/// 6. `global_strategy` — ID глобальной стратегии
///
/// ## Безопасность async операций внутри lock'ов
///
/// - `stop_zapret_strategy()` вызывается ДО захвата `zapret_lock` чтобы избежать
///   длительного удержания lock'а во время async shutdown
/// - Внутри `zapret_lock` выполняются только быстрые операции:
///   - Проверка состояния `zapret_strategy`
///   - Вызов `nodpi_engine::start_*` (который имеет свой внутренний lock)
///   - Сохранение handle в `zapret_strategy`
///
/// ## Blocked Strategies
///
/// Движок автоматически отслеживает неудачные запуски стратегий и блокирует
/// их после 3 последовательных неудач. Заблокированные стратегии автоматически
/// разблокируются через 1 час.
pub struct StrategyEngine {
    /// Режим работы движка
    mode: RwLock<EngineMode>,
    /// WinDivert operation mode
    windivert_mode: RwLock<WinDivertMode>,
    /// Запущенные процессы (не-Zapret)
    processes: RwLock<HashMap<String, RunningProcess>>,
    /// Запущенная Zapret стратегия (через nodpi_engine)
    zapret_strategy: RwLock<Option<RunningZapretStrategy>>,
    /// Менеджер портов
    port_manager: Mutex<PortManager>,
    /// Блокировка для координации запуска Zapret стратегий.
    /// 
    /// ВАЖНО: Этот lock координирует доступ к `zapret_strategy` и обеспечивает
    /// атомарность операции "проверить-и-запустить". Внутри lock'а вызывается
    /// `nodpi_engine::start_*`, который имеет свой внутренний `ZAPRET_LAUNCH_LOCK`
    /// для обеспечения задержки между запусками WinDivert.
    /// 
    /// Порядок захвата: zapret_lock → ZAPRET_LAUNCH_LOCK (внутри nodpi_engine)
    zapret_lock: Mutex<()>,
    /// Текущая глобальная стратегия
    global_strategy: RwLock<Option<String>>,
    /// Mock-состояние запущенных стратегий (для Mock режима)
    mock_running: RwLock<HashMap<String, MockStrategyState>>,
    /// Менеджер блокировки неработающих стратегий
    blocked_strategies: BlockedStrategiesManager,
}

/// Состояние mock-стратегии
/// 
/// NOTE: Fields are stored for debugging and future mock features.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for debugging and future features
struct MockStrategyState {
    strategy_id: String,
    mode: LaunchMode,
    port: Option<u16>,
    started_at: std::time::Instant,
}

impl StrategyEngine {
    /// Создаёт новый экземпляр движка
    pub fn new() -> Self {
        Self {
            mode: RwLock::new(EngineMode::Real),
            windivert_mode: RwLock::new(WinDivertMode::Normal),
            processes: RwLock::new(HashMap::new()),
            zapret_strategy: RwLock::new(None),
            port_manager: Mutex::new(PortManager::default()),
            zapret_lock: Mutex::new(()),
            global_strategy: RwLock::new(None),
            mock_running: RwLock::new(HashMap::new()),
            blocked_strategies: BlockedStrategiesManager::new(),
        }
    }

    /// Создаёт движок с кастомной конфигурацией блокировки стратегий
    pub fn with_blocked_config(config: BlockedStrategiesConfig) -> Self {
        Self {
            mode: RwLock::new(EngineMode::Real),
            windivert_mode: RwLock::new(WinDivertMode::Normal),
            processes: RwLock::new(HashMap::new()),
            zapret_strategy: RwLock::new(None),
            port_manager: Mutex::new(PortManager::default()),
            zapret_lock: Mutex::new(()),
            global_strategy: RwLock::new(None),
            mock_running: RwLock::new(HashMap::new()),
            blocked_strategies: BlockedStrategiesManager::with_config(config),
        }
    }

    /// Создаёт движок с persistence для блокировок через Storage
    pub fn with_storage(storage: Arc<Storage>) -> Self {
        Self {
            mode: RwLock::new(EngineMode::Real),
            windivert_mode: RwLock::new(WinDivertMode::Normal),
            processes: RwLock::new(HashMap::new()),
            zapret_strategy: RwLock::new(None),
            port_manager: Mutex::new(PortManager::default()),
            zapret_lock: Mutex::new(()),
            global_strategy: RwLock::new(None),
            mock_running: RwLock::new(HashMap::new()),
            blocked_strategies: BlockedStrategiesManager::with_storage(storage),
        }
    }

    /// Загружает состояние заблокированных стратегий из Storage
    pub async fn load_blocked_strategies(&self) -> Result<()> {
        self.blocked_strategies.load_from_storage().await
    }

    /// Устанавливает WinDivert operation mode
    pub async fn set_windivert_mode(&self, mode: WinDivertMode) {
        let old_mode = {
            let mut current = self.windivert_mode.write().await;
            let old = *current;
            *current = mode;
            old
        };

        if old_mode != mode {
            info!(
                old_mode = %old_mode,
                new_mode = %mode,
                "WinDivert mode changed"
            );
        }
    }

    /// Получает текущий WinDivert operation mode
    pub async fn get_windivert_mode(&self) -> WinDivertMode {
        *self.windivert_mode.read().await
    }

    /// Устанавливает режим работы движка
    pub async fn set_mode(&self, mode: EngineMode) {
        let old_mode = {
            let mut current = self.mode.write().await;
            let old = *current;
            *current = mode;
            old
        };

        if old_mode != mode {
            info!(
                old_mode = %old_mode,
                new_mode = %mode,
                "Engine mode changed"
            );

            // При смене режима очищаем mock-состояние
            if mode != EngineMode::Mock {
                let mut mock = self.mock_running.write().await;
                mock.clear();
            }
        }
    }

    /// Получает текущий режим работы движка
    pub async fn get_mode(&self) -> EngineMode {
        *self.mode.read().await
    }

    /// Проверяет, работает ли движок в mock-режиме
    async fn is_mock_mode(&self) -> bool {
        *self.mode.read().await == EngineMode::Mock
    }

    /// Запускает стратегию в SOCKS-режиме
    ///
    /// Возвращает выделенный порт для подключения.
    /// Возвращает ошибку, если стратегия заблокирована.
    pub async fn start_socks(&self, strategy: &Strategy) -> Result<u16> {
        // Проверяем, не заблокирована ли стратегия
        if self.blocked_strategies.is_blocked(&strategy.id).await {
            return Err(IsolateError::Strategy(format!(
                "Strategy '{}' is blocked due to consecutive failures",
                strategy.id
            )));
        }

        // Проверяем поддержку SOCKS-режима
        if !strategy.mode_capabilities.supports_socks {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' does not support SOCKS mode",
                strategy.id
            )));
        }

        let template = strategy.socks_template.as_ref().ok_or_else(|| {
            IsolateError::Config(format!(
                "Strategy '{}' has no SOCKS template",
                strategy.id
            ))
        })?;

        // Выделяем порт
        let port = {
            let mut pm = self.port_manager.lock().await;
            pm.allocate(&strategy.id)?
        };

        // Mock режим - симуляция без реальных процессов
        if self.is_mock_mode().await {
            let result = self.mock_start_strategy(strategy, LaunchMode::Socks, Some(port)).await;
            if result.is_err() {
                let mut pm = self.port_manager.lock().await;
                pm.release(port);
            }
            return result.map(|_| port);
        }

        // Для Zapret - последовательный запуск
        let result = if strategy.engine == EngineType::Zapret {
            self.start_zapret_socks(strategy, template, port).await
        } else {
            self.start_process(strategy, template, LaunchMode::Socks, Some(port))
                .await
        };

        // При ошибке освобождаем порт
        if result.is_err() {
            let mut pm = self.port_manager.lock().await;
            pm.release(port);
        }

        result.map(|_| port)
    }

    /// Запускает стратегию в глобальном режиме
    ///
    /// Возвращает ошибку, если стратегия заблокирована.
    pub async fn start_global(&self, strategy: &Strategy) -> Result<()> {
        // Проверяем, не заблокирована ли стратегия
        if self.blocked_strategies.is_blocked(&strategy.id).await {
            return Err(IsolateError::Strategy(format!(
                "Strategy '{}' is blocked due to consecutive failures",
                strategy.id
            )));
        }

        // Проверяем поддержку глобального режима
        if !strategy.mode_capabilities.supports_global {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' does not support GLOBAL mode",
                strategy.id
            )));
        }

        let template = strategy.global_template.as_ref().ok_or_else(|| {
            IsolateError::Config(format!(
                "Strategy '{}' has no GLOBAL template",
                strategy.id
            ))
        })?;

        // Mock режим - симуляция без реальных процессов
        if self.is_mock_mode().await {
            // Останавливаем текущую mock-глобальную стратегию
            self.mock_stop_global().await?;
            
            self.mock_start_strategy(strategy, LaunchMode::Global, None).await?;
            
            // Сохраняем ID глобальной стратегии
            {
                let mut global = self.global_strategy.write().await;
                *global = Some(strategy.id.clone());
            }
            
            info!(
                strategy_id = %strategy.id,
                mode = "Mock",
                "Started global strategy (mock)"
            );
            return Ok(());
        }

        // Проверяем права администратора
        if template.requires_admin && !is_admin() {
            return Err(IsolateError::RequiresAdmin);
        }

        // Останавливаем текущую глобальную стратегию
        self.stop_global().await?;

        // Для Zapret - используем nodpi_engine
        if strategy.engine == EngineType::Zapret {
            self.start_zapret_global_via_nodpi(strategy).await?;
        } else {
            self.start_process(strategy, template, LaunchMode::Global, None)
                .await?;
        }

        // Сохраняем ID глобальной стратегии
        {
            let mut global = self.global_strategy.write().await;
            *global = Some(strategy.id.clone());
        }

        // Начинаем сбор метрик
        strategy_metrics::start_metrics_collection(&strategy.id).await;

        info!(strategy_id = %strategy.id, "Started global strategy");
        Ok(())
    }

    /// Запускает стратегию в глобальном режиме с поддержкой routing rules
    ///
    /// Эта версия загружает routing rules из storage и создаёт dynamic hostlist
    /// для доменов с action "dpi-bypass".
    ///
    /// # Arguments
    /// * `strategy` - Стратегия для запуска
    /// * `storage` - Storage для загрузки routing rules
    ///
    /// # Returns
    /// * `Ok(())` - Стратегия запущена успешно
    /// * `Err(IsolateError)` - Ошибка запуска
    pub async fn start_global_with_routing(&self, strategy: &Strategy, storage: &Storage) -> Result<()> {
        // Проверяем, не заблокирована ли стратегия
        if self.blocked_strategies.is_blocked(&strategy.id).await {
            return Err(IsolateError::Strategy(format!(
                "Strategy '{}' is blocked due to consecutive failures",
                strategy.id
            )));
        }

        // Проверяем поддержку глобального режима
        if !strategy.mode_capabilities.supports_global {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' does not support GLOBAL mode",
                strategy.id
            )));
        }

        let template = strategy.global_template.as_ref().ok_or_else(|| {
            IsolateError::Config(format!(
                "Strategy '{}' has no GLOBAL template",
                strategy.id
            ))
        })?;

        // Mock режим - делегируем в обычный start_global
        if self.is_mock_mode().await {
            return self.start_global(strategy).await;
        }

        // Проверяем права администратора
        if template.requires_admin && !is_admin() {
            return Err(IsolateError::RequiresAdmin);
        }

        // Останавливаем текущую глобальную стратегию
        self.stop_global().await?;

        // Для Zapret - используем nodpi_engine с routing rules
        if strategy.engine == EngineType::Zapret {
            self.start_zapret_global_with_routing(strategy, storage).await?;
        } else {
            self.start_process(strategy, template, LaunchMode::Global, None)
                .await?;
        }

        // Сохраняем ID глобальной стратегии
        {
            let mut global = self.global_strategy.write().await;
            *global = Some(strategy.id.clone());
        }

        // Начинаем сбор метрик
        strategy_metrics::start_metrics_collection(&strategy.id).await;

        info!(strategy_id = %strategy.id, "Started global strategy with routing rules");
        Ok(())
    }

    /// Останавливает SOCKS-стратегию
    pub async fn stop_socks(&self, strategy_id: &str) -> Result<()> {
        let port = {
            let pm = self.port_manager.lock().await;
            pm.get_port(strategy_id)
        };

        // Mock режим
        if self.is_mock_mode().await {
            self.mock_stop_strategy(strategy_id).await?;
        } else {
            self.stop_process(strategy_id).await?;
        }

        if let Some(port) = port {
            let mut pm = self.port_manager.lock().await;
            pm.release(port);
        }

        Ok(())
    }

    /// Останавливает глобальную стратегию
    pub async fn stop_global(&self) -> Result<()> {
        // Mock режим
        if self.is_mock_mode().await {
            return self.mock_stop_global().await;
        }

        let strategy_id = {
            let global = self.global_strategy.read().await;
            global.clone()
        };

        if let Some(id) = strategy_id {
            // Проверяем, это Zapret стратегия или обычная
            let is_zapret = {
                let zapret = self.zapret_strategy.read().await;
                zapret.as_ref().map(|z| z.strategy_id == id).unwrap_or(false)
            };

            if is_zapret {
                self.stop_zapret_strategy().await?;
            } else {
                self.stop_process(&id).await?;
            }

            let mut global = self.global_strategy.write().await;
            *global = None;
            
            // Останавливаем сбор метрик
            strategy_metrics::stop_metrics_collection().await;
            
            info!(strategy_id = %id, "Stopped global strategy");
        }

        Ok(())
    }

    /// Останавливает все запущенные процессы
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all strategy processes");

        // Mock режим - просто очищаем состояние
        if self.is_mock_mode().await {
            {
                let mut mock = self.mock_running.write().await;
                mock.clear();
            }
            {
                let mut global = self.global_strategy.write().await;
                *global = None;
            }
            {
                let mut pm = self.port_manager.lock().await;
                *pm = PortManager::default();
            }
            info!("All mock strategies stopped");
            return Ok(());
        }

        // Останавливаем Zapret стратегию если запущена
        self.stop_zapret_strategy().await.ok();

        let strategy_ids: Vec<String> = {
            let processes = self.processes.read().await;
            processes.keys().cloned().collect()
        };

        for id in strategy_ids {
            if let Err(e) = self.stop_process(&id).await {
                warn!(strategy_id = %id, error = %e, "Failed to stop process during shutdown");
            }
        }

        // Очищаем порты
        {
            let mut pm = self.port_manager.lock().await;
            *pm = PortManager::default();
        }

        // Очищаем глобальную стратегию
        {
            let mut global = self.global_strategy.write().await;
            *global = None;
        }

        // Останавливаем сбор метрик
        strategy_metrics::stop_metrics_collection().await;

        // Сбрасываем флаг WinDivert на всякий случай
        reset_windivert_flag();

        Ok(())
    }

    /// Проверяет, запущена ли стратегия
    pub async fn is_running(&self, strategy_id: &str) -> bool {
        // Mock режим
        if self.is_mock_mode().await {
            let mock = self.mock_running.read().await;
            return mock.contains_key(strategy_id);
        }

        // Проверяем Zapret стратегию
        {
            let zapret = self.zapret_strategy.read().await;
            if let Some(ref z) = *zapret {
                if z.strategy_id == strategy_id {
                    return z.handle.is_running().await;
                }
            }
        }

        // Проверяем обычные процессы
        let processes = self.processes.read().await;
        processes.contains_key(strategy_id)
    }

    /// Проверяет, активен ли WinDivert (запущена ли Zapret стратегия)
    pub fn is_windivert_active(&self) -> bool {
        is_windivert_active()
    }

    /// Получает текущую глобальную стратегию
    pub async fn get_global_strategy(&self) -> Option<String> {
        let global = self.global_strategy.read().await;
        global.clone()
    }

    /// Получает порт SOCKS-стратегии
    pub async fn get_socks_port(&self, strategy_id: &str) -> Option<u16> {
        let pm = self.port_manager.lock().await;
        pm.get_port(strategy_id)
    }

    // ========================================================================
    // Blocked Strategies API
    // ========================================================================

    /// Проверяет, заблокирована ли стратегия
    ///
    /// Автоматически разблокирует стратегию, если истёк срок блокировки (1 час).
    pub async fn is_strategy_blocked(&self, strategy_id: &str) -> bool {
        self.blocked_strategies.is_blocked(strategy_id).await
    }

    /// Регистрирует неудачу стратегии
    ///
    /// После 3 последовательных неудач стратегия автоматически блокируется.
    /// Возвращает `true`, если стратегия была заблокирована в результате этой неудачи.
    pub async fn record_strategy_failure(&self, strategy_id: &str, reason: &str) -> bool {
        self.blocked_strategies.record_failure(strategy_id, reason).await
    }

    /// Регистрирует успех стратегии (сбрасывает счётчик неудач)
    pub async fn record_strategy_success(&self, strategy_id: &str) {
        self.blocked_strategies.record_success(strategy_id).await
    }

    /// Блокирует стратегию вручную
    pub async fn block_strategy(&self, strategy_id: &str, reason: &str) {
        self.blocked_strategies.block_strategy(strategy_id, reason).await
    }

    /// Разблокирует стратегию вручную
    pub async fn unblock_strategy(&self, strategy_id: &str) -> bool {
        self.blocked_strategies.unblock_strategy(strategy_id).await
    }

    /// Возвращает список всех заблокированных стратегий
    pub async fn get_blocked_strategies(&self) -> Vec<BlockedStrategy> {
        self.blocked_strategies.get_blocked_list().await
    }

    /// Возвращает информацию о блокировке стратегии
    pub async fn get_blocked_info(&self, strategy_id: &str) -> Option<BlockedStrategy> {
        self.blocked_strategies.get_blocked_info(strategy_id).await
    }

    /// Возвращает текущий счётчик неудач для стратегии
    pub async fn get_failure_count(&self, strategy_id: &str) -> u32 {
        self.blocked_strategies.get_failure_count(strategy_id).await
    }

    /// Сбрасывает все блокировки и счётчики неудач
    pub async fn reset_all_blocks(&self) {
        self.blocked_strategies.reset_all().await
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Запускает Zapret стратегию через nodpi_engine
    ///
    /// ## Lock Safety
    /// 
    /// Этот метод безопасен от deadlock'ов благодаря следующему порядку:
    /// 1. `stop_zapret_strategy()` вызывается ДО захвата lock'а (async cleanup)
    /// 2. `zapret_lock` захватывается для критической секции
    /// 3. Внутри lock'а: проверка состояния + вызов nodpi_engine (быстрые операции)
    /// 4. `nodpi_engine::ZAPRET_LAUNCH_LOCK` захватывается внутри nodpi_engine
    ///
    /// Порядок lock'ов: zapret_lock → ZAPRET_LAUNCH_LOCK (всегда одинаковый)
    async fn start_zapret_global_via_nodpi(&self, strategy: &Strategy) -> Result<()> {
        // SAFETY: Останавливаем предыдущую Zapret стратегию ДО захвата lock'а
        // чтобы избежать длительного удержания lock'а во время async shutdown
        self.stop_zapret_strategy().await.ok();

        // Захватываем блокировку только для критической секции запуска
        // Внутри lock'а выполняются только быстрые операции
        let _guard = self.zapret_lock.lock().await;

        debug!(
            strategy_id = %strategy.id,
            "Acquired Zapret lock for GLOBAL launch via nodpi_engine"
        );

        // Проверяем что никто не запустил стратегию пока мы останавливали
        {
            let zapret = self.zapret_strategy.read().await;
            if zapret.is_some() {
                return Err(IsolateError::Process(
                    "Another Zapret strategy was started concurrently".into()
                ));
            }
        }

        // Get current WinDivert mode
        let windivert_mode = self.get_windivert_mode().await;

        // Запускаем через nodpi_engine с WinDivert mode
        let handle = nodpi_engine::start_nodpi_from_strategy_with_mode(
            strategy,
            true,
            Some(windivert_mode),
        ).await?;

        // Сохраняем handle
        {
            let mut zapret = self.zapret_strategy.write().await;
            *zapret = Some(RunningZapretStrategy {
                handle,
                strategy_id: strategy.id.clone(),
                mode: LaunchMode::Global,
            });
        }

        info!(
            strategy_id = %strategy.id,
            windivert_mode = %windivert_mode,
            "Zapret strategy started via nodpi_engine"
        );

        Ok(())
    }

    /// Запускает Zapret стратегию через nodpi_engine с поддержкой routing rules
    ///
    /// Загружает routing rules из storage, конвертирует их и создаёт
    /// dynamic hostlist для доменов с action "dpi-bypass".
    ///
    /// ## Lock Safety
    /// 
    /// Аналогично `start_zapret_global_via_nodpi`:
    /// 1. `stop_zapret_strategy()` — ДО захвата lock'а
    /// 2. Загрузка routing rules — внутри lock'а (быстрая I/O операция)
    /// 3. `nodpi_engine::start_*` — внутри lock'а (захватывает свой ZAPRET_LAUNCH_LOCK)
    async fn start_zapret_global_with_routing(&self, strategy: &Strategy, storage: &Storage) -> Result<()> {
        // SAFETY: Останавливаем предыдущую Zapret стратегию ДО захвата lock'а
        self.stop_zapret_strategy().await.ok();

        // Захватываем блокировку только для критической секции запуска
        let _guard = self.zapret_lock.lock().await;

        debug!(
            strategy_id = %strategy.id,
            "Acquired Zapret lock for GLOBAL launch with routing rules"
        );

        // Проверяем что никто не запустил стратегию пока мы останавливали
        {
            let zapret = self.zapret_strategy.read().await;
            if zapret.is_some() {
                return Err(IsolateError::Process(
                    "Another Zapret strategy was started concurrently".into()
                ));
            }
        }

        // Get current WinDivert mode
        let windivert_mode = self.get_windivert_mode().await;

        // Load routing rules and create dynamic hostlist for dpi-bypass domains
        let extra_hostlist = match storage.get_routing_rules().await {
            Ok(rules) => {
                let converted = convert_routing_rules(&rules);
                
                if !converted.dpi_bypass_domains.is_empty() {
                    info!(
                        domains_count = converted.dpi_bypass_domains.len(),
                        "Creating dynamic hostlist for DPI bypass domains"
                    );
                    
                    match create_dynamic_hostlist(&converted.dpi_bypass_domains).await {
                        Ok(path) => {
                            info!(
                                path = %path.display(),
                                "Dynamic hostlist created"
                            );
                            Some(path)
                        }
                        Err(e) => {
                            warn!(
                                error = %e,
                                "Failed to create dynamic hostlist, continuing without it"
                            );
                            None
                        }
                    }
                } else {
                    debug!("No DPI bypass domains in routing rules");
                    None
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "Failed to load routing rules, continuing without dynamic hostlist"
                );
                None
            }
        };

        // Запускаем через nodpi_engine с WinDivert mode и extra hostlist
        let handle = start_nodpi_from_strategy_with_extra_hostlist(
            strategy,
            true,
            Some(windivert_mode),
            extra_hostlist.as_deref(),
        ).await?;

        // Сохраняем handle
        {
            let mut zapret = self.zapret_strategy.write().await;
            *zapret = Some(RunningZapretStrategy {
                handle,
                strategy_id: strategy.id.clone(),
                mode: LaunchMode::Global,
            });
        }

        info!(
            strategy_id = %strategy.id,
            windivert_mode = %windivert_mode,
            has_extra_hostlist = extra_hostlist.is_some(),
            "Zapret strategy started with routing rules"
        );

        Ok(())
    }

    /// Останавливает текущую Zapret стратегию
    async fn stop_zapret_strategy(&self) -> Result<()> {
        let mut zapret = self.zapret_strategy.write().await;

        if let Some(mut running) = zapret.take() {
            info!(
                strategy_id = %running.strategy_id,
                "Stopping Zapret strategy"
            );

            if let Err(e) = running.handle.stop().await {
                warn!(
                    strategy_id = %running.strategy_id,
                    error = %e,
                    "Error stopping Zapret strategy"
                );
            }

            info!(
                strategy_id = %running.strategy_id,
                "Zapret strategy stopped"
            );
        }

        Ok(())
    }

    /// Запускает Zapret в SOCKS-режиме с блокировкой (legacy, для совместимости)
    ///
    /// ## Lock Safety
    /// 
    /// Задержка `ZAPRET_LAUNCH_DELAY_MS` выполняется ДО захвата lock'а,
    /// чтобы не блокировать другие операции во время ожидания.
    async fn start_zapret_socks(
        &self,
        strategy: &Strategy,
        template: &LaunchTemplate,
        port: u16,
    ) -> Result<()> {
        // SAFETY: Задержка перед запуском (для стабильности WinDivert) — ДО захвата lock'а
        tokio::time::sleep(tokio::time::Duration::from_millis(ZAPRET_LAUNCH_DELAY_MS)).await;

        // Захватываем блокировку только для критической секции
        let _guard = self.zapret_lock.lock().await;

        debug!(
            strategy_id = %strategy.id,
            "Acquired Zapret lock for SOCKS launch"
        );

        // Acquire WinDivert guard using RAII pattern
        let windivert_guard = WinDivertGuard::acquire()?;

        self.start_process_with_guard(strategy, template, LaunchMode::Socks, Some(port), Some(windivert_guard))
            .await
    }

    /// Запускает процесс стратегии (для не-Zapret стратегий или legacy)
    async fn start_process(
        &self,
        strategy: &Strategy,
        template: &LaunchTemplate,
        mode: LaunchMode,
        port: Option<u16>,
    ) -> Result<()> {
        self.start_process_with_guard(strategy, template, mode, port, None).await
    }

    /// Запускает процесс стратегии с опциональным WinDivert guard
    async fn start_process_with_guard(
        &self,
        strategy: &Strategy,
        template: &LaunchTemplate,
        mode: LaunchMode,
        port: Option<u16>,
        windivert_guard: Option<WinDivertGuard>,
    ) -> Result<()> {
        // Get current WinDivert mode
        let windivert_mode = self.get_windivert_mode().await;

        // Для Zapret используем nodpi_engine для разрешения путей
        let (binary_path, args) = if strategy.engine == EngineType::Zapret {
            let binary = get_binary_path_from_template(template);
            // Use WinDivert mode when building args
            let resolved_args = build_winws_args_from_template_with_mode(template, Some(windivert_mode));
            let args = self.prepare_args(&resolved_args, port);
            (binary.display().to_string(), args)
        } else {
            let args = self.prepare_args(&template.args, port);
            (template.binary.clone(), args)
        };

        debug!(
            strategy_id = %strategy.id,
            binary = %binary_path,
            args = ?args,
            mode = ?mode,
            windivert_mode = %windivert_mode,
            "Starting strategy process"
        );

        // Запускаем процесс
        let mut cmd = Command::new(&binary_path);
        cmd.args(&args);

        // Добавляем переменные окружения
        for (key, value) in &template.env {
            let value = self.substitute_port(value, port);
            cmd.env(key, value);
        }

        // Скрываем окно консоли на Windows
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let child = cmd.spawn().map_err(|e| {
            IsolateError::Process(format!(
                "Failed to start '{}': {}",
                binary_path, e
            ))
        })?;

        // Сохраняем информацию о процессе
        let process = RunningProcess {
            child: Some(child),
            strategy_id: strategy.id.clone(),
            mode,
            port,
            engine: strategy.engine.clone(),
            windivert_guard,
        };

        // ATOMIC: Проверяем и вставляем в одной критической секции
        // Используем entry() API для атомарной операции check-and-insert
        {
            let mut processes = self.processes.write().await;
            use std::collections::hash_map::Entry;
            
            match processes.entry(strategy.id.clone()) {
                Entry::Vacant(e) => {
                    e.insert(process);
                }
                Entry::Occupied(_) => {
                    // Стратегия уже запущена - убиваем только что созданный процесс
                    // чтобы избежать утечки ресурсов
                    // CRITICAL FIX: Kill the child process before returning error
                    // to prevent resource leak
                    if let Some(mut child) = process.child {
                        let _ = child.start_kill();
                        // Don't await - just initiate kill and let OS clean up
                    }
                    // WinDivert guard will be dropped automatically (RAII)
                    return Err(IsolateError::Process(format!(
                        "Strategy '{}' is already running",
                        strategy.id
                    )));
                }
            }
        }

        info!(
            strategy_id = %strategy.id,
            mode = ?mode,
            port = ?port,
            windivert_mode = %windivert_mode,
            "Strategy process started"
        );

        Ok(())
    }

    /// Останавливает процесс стратегии с двухэтапным graceful shutdown
    ///
    /// ## Graceful Shutdown Process
    /// 1. Отправляем мягкий сигнал завершения (taskkill на Windows, SIGTERM на Unix)
    /// 2. Ждём завершения с таймаутом
    /// 3. Если процесс не завершился — принудительный kill
    async fn stop_process(&self, strategy_id: &str) -> Result<()> {
        let mut process = {
            let mut processes = self.processes.write().await;
            match processes.remove(strategy_id) {
                Some(p) => p,
                None => return Ok(()), // Процесс не найден - уже остановлен
            }
        };

        debug!(strategy_id, "Stopping strategy process");

        if let Some(ref mut child) = process.child {
            // Stage 1: Graceful termination signal
            #[cfg(windows)]
            {
                if let Some(pid) = child.id() {
                    debug!(strategy_id, pid, "Sending graceful terminate signal via taskkill");
                    // taskkill без /F отправляет WM_CLOSE, что позволяет процессу
                    // корректно завершиться и освободить ресурсы (включая WinDivert)
                    let _ = tokio::process::Command::new("taskkill")
                        .args(["/PID", &pid.to_string()])
                        .output()
                        .await;
                }
            }

            #[cfg(not(windows))]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Some(pid) = child.id() {
                    debug!(strategy_id, pid, "Sending SIGTERM");
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
            }

            // Stage 2: Wait for graceful shutdown with timeout
            let shutdown_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(SHUTDOWN_TIMEOUT_MS),
                child.wait(),
            )
            .await;

            match shutdown_result {
                Ok(Ok(status)) => {
                    debug!(strategy_id, ?status, "Process terminated gracefully");
                }
                Ok(Err(e)) => {
                    warn!(strategy_id, error = %e, "Error waiting for process, force killing");
                    // Stage 3: Force kill on error
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                }
                Err(_) => {
                    // Stage 3: Force kill on timeout
                    warn!(strategy_id, timeout_ms = SHUTDOWN_TIMEOUT_MS, "Graceful shutdown timeout, force killing");
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                }
            }
        }

        // WinDivert guard будет автоматически освобождён через Drop
        // когда process выйдет из scope

        info!(strategy_id, "Strategy process stopped");
        Ok(())
    }

    /// Подготавливает аргументы с подстановкой порта
    fn prepare_args(&self, args: &[String], port: Option<u16>) -> Vec<String> {
        args.iter()
            .map(|arg| self.substitute_port(arg, port))
            .collect()
    }

    /// Подставляет порт в строку
    fn substitute_port(&self, s: &str, port: Option<u16>) -> String {
        match port {
            Some(p) => s.replace("{{port}}", &p.to_string()),
            None => s.to_string(),
        }
    }

    // ========================================================================
    // Mock Methods
    // ========================================================================

    /// Симулирует запуск стратегии в mock-режиме
    async fn mock_start_strategy(
        &self,
        strategy: &Strategy,
        mode: LaunchMode,
        port: Option<u16>,
    ) -> Result<()> {
        // Симулируем задержку запуска (500-1500ms)
        let delay = {
            let mut rng = rand::rng();
            rng.random_range(MOCK_DELAY_MIN_MS..=MOCK_DELAY_MAX_MS)
        };

        info!(
            strategy_id = %strategy.id,
            mode = ?mode,
            port = ?port,
            delay_ms = delay,
            "[MOCK] Simulating strategy start"
        );

        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

        // ATOMIC: Проверяем и вставляем в одной критической секции
        // Используем entry() API для атомарной операции check-and-insert
        {
            let mut mock = self.mock_running.write().await;
            use std::collections::hash_map::Entry;
            
            match mock.entry(strategy.id.clone()) {
                Entry::Vacant(e) => {
                    e.insert(MockStrategyState {
                        strategy_id: strategy.id.clone(),
                        mode,
                        port,
                        started_at: std::time::Instant::now(),
                    });
                }
                Entry::Occupied(_) => {
                    return Err(IsolateError::Process(format!(
                        "Strategy '{}' is already running (mock)",
                        strategy.id
                    )));
                }
            }
        }

        info!(
            strategy_id = %strategy.id,
            mode = ?mode,
            port = ?port,
            "[MOCK] Strategy started successfully"
        );

        Ok(())
    }

    /// Останавливает mock-стратегию
    async fn mock_stop_strategy(&self, strategy_id: &str) -> Result<()> {
        let mut mock = self.mock_running.write().await;

        if let Some(state) = mock.remove(strategy_id) {
            let running_time = state.started_at.elapsed();
            info!(
                strategy_id = %strategy_id,
                running_time_ms = running_time.as_millis(),
                "[MOCK] Strategy stopped"
            );
        }

        Ok(())
    }

    /// Останавливает глобальную mock-стратегию
    async fn mock_stop_global(&self) -> Result<()> {
        let strategy_id = {
            let global = self.global_strategy.read().await;
            global.clone()
        };

        if let Some(id) = strategy_id {
            self.mock_stop_strategy(&id).await?;

            let mut global = self.global_strategy.write().await;
            *global = None;

            info!(strategy_id = %id, "[MOCK] Global strategy stopped");
        }

        Ok(())
    }

    /// Симулирует результат тестирования стратегии (80% успех, 20% fail)
    #[allow(dead_code)]
    pub async fn mock_test_result(&self) -> bool {
        if !self.is_mock_mode().await {
            warn!("mock_test_result called in non-mock mode");
            return false;
        }

        // Симулируем задержку тестирования
        let delay = {
            let mut rng = rand::rng();
            rng.random_range(MOCK_DELAY_MIN_MS..=MOCK_DELAY_MAX_MS)
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

        // 80% успех
        let success = {
            let mut rng = rand::rng();
            rng.random::<f64>() < MOCK_SUCCESS_RATE
        };

        info!(
            success = success,
            delay_ms = delay,
            "[MOCK] Test result simulated"
        );

        success
    }
}

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Проверяет, запущено ли приложение с правами администратора
#[cfg(windows)]
fn is_admin() -> bool {
    use std::mem;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = std::mem::zeroed();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = mem::zeroed();
        let mut size = mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        CloseHandle(token);

        result != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
fn is_admin() -> bool {
    // На Unix проверяем UID
    unsafe { libc::geteuid() == 0 }
}

// ============================================================================
// Thread-safe wrapper
// ============================================================================

/// Thread-safe обёртка для StrategyEngine
pub type SharedStrategyEngine = Arc<StrategyEngine>;

/// Создаёт shared экземпляр движка
pub fn create_engine() -> SharedStrategyEngine {
    Arc::new(StrategyEngine::new())
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{
        LaunchTemplate, ModeCapabilities, Strategy, StrategyEngine as EngineType,
        StrategyFamily, StrategyRequirements,
    };

    // ========================================================================
    // Helper Functions for Tests
    // ========================================================================

    /// Creates a test strategy with configurable capabilities
    fn create_test_strategy(
        id: &str,
        supports_socks: bool,
        supports_global: bool,
        engine: EngineType,
    ) -> Strategy {
        Strategy {
            id: id.to_string(),
            name: format!("Test Strategy {}", id),
            description: "Test strategy for unit tests".to_string(),
            family: StrategyFamily::DnsBypass,
            engine,
            mode_capabilities: ModeCapabilities {
                supports_socks,
                supports_global,
            },
            socks_template: if supports_socks {
                Some(LaunchTemplate {
                    binary: "test-binary.exe".to_string(),
                    args: vec!["--port".to_string(), "{{port}}".to_string()],
                    env: HashMap::new(),
                    log_file: None,
                    requires_admin: false,
                })
            } else {
                None
            },
            global_template: if supports_global {
                Some(LaunchTemplate {
                    binary: "test-binary.exe".to_string(),
                    args: vec!["--global".to_string()],
                    env: HashMap::new(),
                    log_file: None,
                    requires_admin: true,
                })
            } else {
                None
            },
            requirements: StrategyRequirements::default(),
            weight_hint: 0,
            services: vec!["youtube".to_string()],
        }
    }

    /// Creates a Zapret strategy for testing
    fn create_zapret_strategy(id: &str) -> Strategy {
        create_test_strategy(id, true, true, EngineType::Zapret)
    }

    /// Creates a SingBox strategy for testing
    fn create_singbox_strategy(id: &str) -> Strategy {
        create_test_strategy(id, true, false, EngineType::SingBox)
    }

    // ========================================================================
    // PortManager Tests
    // ========================================================================

    #[test]
    fn test_port_manager_allocate_first_port() {
        let mut pm = PortManager::default();
        let port = pm.allocate("strategy-1").unwrap();
        assert_eq!(port, SOCKS_PORT_START);
        assert!(pm.is_allocated(port));
    }

    #[test]
    fn test_port_manager_allocate_sequential_ports() {
        let mut pm = PortManager::default();
        
        let port1 = pm.allocate("strategy-1").unwrap();
        let port2 = pm.allocate("strategy-2").unwrap();
        let port3 = pm.allocate("strategy-3").unwrap();
        
        assert_eq!(port1, SOCKS_PORT_START);
        assert_eq!(port2, SOCKS_PORT_START + 1);
        assert_eq!(port3, SOCKS_PORT_START + 2);
    }

    #[test]
    fn test_port_manager_reuse_existing_allocation() {
        let mut pm = PortManager::default();
        
        let port1 = pm.allocate("strategy-1").unwrap();
        let port2 = pm.allocate("strategy-1").unwrap(); // Same strategy
        
        assert_eq!(port1, port2);
        assert_eq!(pm.allocated_count(), 1);
    }

    #[test]
    fn test_port_manager_release_port() {
        let mut pm = PortManager::default();
        
        let port = pm.allocate("strategy-1").unwrap();
        assert!(pm.is_allocated(port));
        
        pm.release(port);
        assert!(!pm.is_allocated(port));
    }

    #[test]
    fn test_port_manager_release_by_strategy() {
        let mut pm = PortManager::default();
        
        pm.allocate("strategy-1").unwrap();
        pm.allocate("strategy-2").unwrap();
        
        assert_eq!(pm.allocated_count(), 2);
        
        pm.release_by_strategy("strategy-1");
        
        assert_eq!(pm.allocated_count(), 1);
        assert!(pm.get_port("strategy-1").is_none());
        assert!(pm.get_port("strategy-2").is_some());
    }

    #[test]
    fn test_port_manager_get_port() {
        let mut pm = PortManager::default();
        
        let port = pm.allocate("strategy-1").unwrap();
        
        assert_eq!(pm.get_port("strategy-1"), Some(port));
        assert_eq!(pm.get_port("nonexistent"), None);
    }

    #[test]
    fn test_port_manager_allocate_specific_success() {
        let mut pm = PortManager::default();
        
        let specific_port = SOCKS_PORT_START + 50;
        pm.allocate_specific(specific_port, "strategy-1").unwrap();
        
        assert!(pm.is_allocated(specific_port));
        assert_eq!(pm.get_port("strategy-1"), Some(specific_port));
    }

    #[test]
    fn test_port_manager_allocate_specific_conflict() {
        let mut pm = PortManager::default();
        
        let port = pm.allocate("strategy-1").unwrap();
        let result = pm.allocate_specific(port, "strategy-2");
        
        assert!(result.is_err());
    }

    // ========================================================================
    // EngineMode Tests
    // ========================================================================

    #[test]
    fn test_engine_mode_default() {
        let mode = EngineMode::default();
        assert_eq!(mode, EngineMode::Real);
    }

    #[test]
    fn test_engine_mode_display() {
        assert_eq!(format!("{}", EngineMode::Real), "Real");
        assert_eq!(format!("{}", EngineMode::Mock), "Mock");
        assert_eq!(format!("{}", EngineMode::DpiTest), "DpiTest");
    }

    #[test]
    fn test_engine_mode_serialization() {
        let modes = vec![
            (EngineMode::Real, "\"Real\""),
            (EngineMode::Mock, "\"Mock\""),
            (EngineMode::DpiTest, "\"DpiTest\""),
        ];

        for (mode, expected) in modes {
            let json = serde_json::to_string(&mode).unwrap();
            assert_eq!(json, expected);
        }
    }

    // ========================================================================
    // LaunchMode Tests
    // ========================================================================

    #[test]
    fn test_launch_mode_equality() {
        assert_eq!(LaunchMode::Socks, LaunchMode::Socks);
        assert_eq!(LaunchMode::Global, LaunchMode::Global);
        assert_ne!(LaunchMode::Socks, LaunchMode::Global);
    }

    // ========================================================================
    // StrategyEngine Tests
    // ========================================================================

    #[tokio::test]
    async fn test_engine_new_default_mode() {
        let engine = StrategyEngine::new();
        assert_eq!(engine.get_mode().await, EngineMode::Real);
    }

    #[tokio::test]
    async fn test_engine_set_mode() {
        let engine = StrategyEngine::new();
        
        engine.set_mode(EngineMode::Mock).await;
        assert_eq!(engine.get_mode().await, EngineMode::Mock);
        
        engine.set_mode(EngineMode::DpiTest).await;
        assert_eq!(engine.get_mode().await, EngineMode::DpiTest);
    }

    #[tokio::test]
    async fn test_engine_windivert_mode_default() {
        let engine = StrategyEngine::new();
        assert_eq!(engine.get_windivert_mode().await, WinDivertMode::Normal);
    }

    #[tokio::test]
    async fn test_engine_set_windivert_mode() {
        let engine = StrategyEngine::new();
        
        engine.set_windivert_mode(WinDivertMode::AutoTTL).await;
        assert_eq!(engine.get_windivert_mode().await, WinDivertMode::AutoTTL);
        
        engine.set_windivert_mode(WinDivertMode::AutoHostlist).await;
        assert_eq!(engine.get_windivert_mode().await, WinDivertMode::AutoHostlist);
    }

    #[tokio::test]
    async fn test_engine_is_running_false_initially() {
        let engine = StrategyEngine::new();
        assert!(!engine.is_running("any-strategy").await);
    }

    #[tokio::test]
    async fn test_engine_get_global_strategy_none_initially() {
        let engine = StrategyEngine::new();
        assert!(engine.get_global_strategy().await.is_none());
    }

    #[tokio::test]
    async fn test_engine_get_socks_port_none_initially() {
        let engine = StrategyEngine::new();
        assert!(engine.get_socks_port("any-strategy").await.is_none());
    }

    // ========================================================================
    // Strategy Validation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_start_socks_unsupported_mode() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Strategy that doesn't support SOCKS
        let strategy = create_test_strategy("no-socks", false, true, EngineType::SingBox);
        
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
    }

    #[tokio::test]
    async fn test_start_global_unsupported_mode() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Strategy that doesn't support GLOBAL
        let strategy = create_test_strategy("no-global", true, false, EngineType::SingBox);
        
        let result = engine.start_global(&strategy).await;
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
    }

    #[tokio::test]
    async fn test_start_socks_no_template() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Strategy with supports_socks=true but no template
        let mut strategy = create_test_strategy("broken", true, false, EngineType::SingBox);
        strategy.socks_template = None;
        
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_global_no_template() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Strategy with supports_global=true but no template
        let mut strategy = create_test_strategy("broken", false, true, EngineType::SingBox);
        strategy.global_template = None;
        
        let result = engine.start_global(&strategy).await;
        assert!(result.is_err());
    }

    // ========================================================================
    // Mock Mode Tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_start_socks_allocates_port() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("mock-socks-1");
        
        let port = engine.start_socks(&strategy).await.unwrap();
        assert_eq!(port, SOCKS_PORT_START);
        assert!(engine.is_running(&strategy.id).await);
    }

    #[tokio::test]
    async fn test_mock_start_global_sets_global_strategy() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_test_strategy("mock-global-1", false, true, EngineType::SingBox);
        
        engine.start_global(&strategy).await.unwrap();
        
        assert_eq!(engine.get_global_strategy().await, Some(strategy.id.clone()));
        assert!(engine.is_running(&strategy.id).await);
    }

    #[tokio::test]
    async fn test_mock_stop_socks_releases_port() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("mock-stop-1");
        
        let port = engine.start_socks(&strategy).await.unwrap();
        assert!(engine.get_socks_port(&strategy.id).await.is_some());
        
        engine.stop_socks(&strategy.id).await.unwrap();
        
        assert!(!engine.is_running(&strategy.id).await);
        assert!(engine.get_socks_port(&strategy.id).await.is_none());
    }

    #[tokio::test]
    async fn test_mock_stop_global_clears_global_strategy() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_test_strategy("mock-global-stop", false, true, EngineType::SingBox);
        
        engine.start_global(&strategy).await.unwrap();
        assert!(engine.get_global_strategy().await.is_some());
        
        engine.stop_global().await.unwrap();
        
        assert!(engine.get_global_strategy().await.is_none());
    }

    #[tokio::test]
    async fn test_mock_shutdown_all_clears_everything() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Start multiple strategies
        let socks_strategy = create_singbox_strategy("shutdown-socks");
        let global_strategy = create_test_strategy("shutdown-global", false, true, EngineType::SingBox);
        
        engine.start_socks(&socks_strategy).await.unwrap();
        engine.start_global(&global_strategy).await.unwrap();
        
        assert!(engine.is_running(&socks_strategy.id).await);
        assert!(engine.is_running(&global_strategy.id).await);
        
        engine.shutdown_all().await.unwrap();
        
        assert!(!engine.is_running(&socks_strategy.id).await);
        assert!(!engine.is_running(&global_strategy.id).await);
        assert!(engine.get_global_strategy().await.is_none());
    }

    #[tokio::test]
    async fn test_mock_start_already_running_fails() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("already-running");
        
        engine.start_socks(&strategy).await.unwrap();
        
        // Try to start again
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_err());
    }

    // ========================================================================
    // Port Substitution Tests
    // ========================================================================

    #[test]
    fn test_substitute_port_with_port() {
        let engine = StrategyEngine::new();
        
        let result = engine.substitute_port("--port={{port}}", Some(12345));
        assert_eq!(result, "--port=12345");
    }

    #[test]
    fn test_substitute_port_without_port() {
        let engine = StrategyEngine::new();
        
        let result = engine.substitute_port("--port={{port}}", None);
        assert_eq!(result, "--port={{port}}");
    }

    #[test]
    fn test_substitute_port_no_placeholder() {
        let engine = StrategyEngine::new();
        
        let result = engine.substitute_port("--global", Some(12345));
        assert_eq!(result, "--global");
    }

    #[test]
    fn test_prepare_args_with_port() {
        let engine = StrategyEngine::new();
        
        let args = vec![
            "--listen".to_string(),
            "127.0.0.1:{{port}}".to_string(),
            "--verbose".to_string(),
        ];
        
        let result = engine.prepare_args(&args, Some(8080));
        
        assert_eq!(result, vec!["--listen", "127.0.0.1:8080", "--verbose"]);
    }

    // ========================================================================
    // Shared Engine Tests
    // ========================================================================

    #[test]
    fn test_create_engine_returns_arc() {
        let engine = create_engine();
        
        // Verify it's an Arc by cloning
        let engine2 = Arc::clone(&engine);
        
        // Both should point to the same engine
        assert!(Arc::ptr_eq(&engine, &engine2));
    }

    #[test]
    fn test_strategy_engine_default() {
        let engine = StrategyEngine::default();
        
        // Should be equivalent to new()
        assert!(!engine.is_windivert_active());
    }

    // ========================================================================
    // Additional PortManager Tests
    // ========================================================================

    #[test]
    fn test_port_manager_release_nonexistent_port() {
        let mut pm = PortManager::default();
        
        // Should not panic when releasing non-allocated port
        pm.release(SOCKS_PORT_START + 999);
        assert_eq!(pm.allocated_count(), 0);
    }

    #[test]
    fn test_port_manager_release_by_nonexistent_strategy() {
        let mut pm = PortManager::default();
        
        pm.allocate("strategy-1").unwrap();
        
        // Should not affect existing allocations
        pm.release_by_strategy("nonexistent");
        assert_eq!(pm.allocated_count(), 1);
    }

    #[test]
    fn test_port_manager_multiple_allocations_same_strategy() {
        let mut pm = PortManager::default();
        
        // Multiple calls for same strategy should return same port
        let port1 = pm.allocate("strategy-1").unwrap();
        let port2 = pm.allocate("strategy-1").unwrap();
        let port3 = pm.allocate("strategy-1").unwrap();
        
        assert_eq!(port1, port2);
        assert_eq!(port2, port3);
        assert_eq!(pm.allocated_count(), 1);
    }

    #[test]
    fn test_port_manager_reallocation_after_release() {
        let mut pm = PortManager::default();
        
        let port1 = pm.allocate("strategy-1").unwrap();
        pm.release(port1);
        
        // Should get the same port back after release
        let port2 = pm.allocate("strategy-2").unwrap();
        assert_eq!(port1, port2);
    }

    #[test]
    fn test_port_manager_is_allocated_false_for_unallocated() {
        let pm = PortManager::default();
        
        assert!(!pm.is_allocated(SOCKS_PORT_START));
        assert!(!pm.is_allocated(SOCKS_PORT_START + 50));
    }

    // ========================================================================
    // Strategy Creation Helper Tests
    // ========================================================================

    #[test]
    fn test_create_test_strategy_socks_only() {
        let strategy = create_test_strategy("test-1", true, false, EngineType::SingBox);
        
        assert_eq!(strategy.id, "test-1");
        assert!(strategy.mode_capabilities.supports_socks);
        assert!(!strategy.mode_capabilities.supports_global);
        assert!(strategy.socks_template.is_some());
        assert!(strategy.global_template.is_none());
    }

    #[test]
    fn test_create_test_strategy_global_only() {
        let strategy = create_test_strategy("test-2", false, true, EngineType::Zapret);
        
        assert_eq!(strategy.id, "test-2");
        assert!(!strategy.mode_capabilities.supports_socks);
        assert!(strategy.mode_capabilities.supports_global);
        assert!(strategy.socks_template.is_none());
        assert!(strategy.global_template.is_some());
    }

    #[test]
    fn test_create_test_strategy_both_modes() {
        let strategy = create_test_strategy("test-3", true, true, EngineType::Zapret);
        
        assert!(strategy.mode_capabilities.supports_socks);
        assert!(strategy.mode_capabilities.supports_global);
        assert!(strategy.socks_template.is_some());
        assert!(strategy.global_template.is_some());
    }

    #[test]
    fn test_create_zapret_strategy_has_both_modes() {
        let strategy = create_zapret_strategy("zapret-1");
        
        assert_eq!(strategy.engine, EngineType::Zapret);
        assert!(strategy.mode_capabilities.supports_socks);
        assert!(strategy.mode_capabilities.supports_global);
    }

    #[test]
    fn test_create_singbox_strategy_socks_only() {
        let strategy = create_singbox_strategy("singbox-1");
        
        assert_eq!(strategy.engine, EngineType::SingBox);
        assert!(strategy.mode_capabilities.supports_socks);
        assert!(!strategy.mode_capabilities.supports_global);
    }

    #[test]
    fn test_strategy_template_port_placeholder() {
        let strategy = create_test_strategy("test-port", true, false, EngineType::SingBox);
        
        let template = strategy.socks_template.unwrap();
        assert!(template.args.contains(&"{{port}}".to_string()));
    }

    #[test]
    fn test_strategy_global_template_requires_admin() {
        let strategy = create_test_strategy("test-admin", false, true, EngineType::Zapret);
        
        let template = strategy.global_template.unwrap();
        assert!(template.requires_admin);
    }

    // ========================================================================
    // Multiple Strategies Tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_multiple_socks_strategies() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy1 = create_singbox_strategy("multi-socks-1");
        let strategy2 = create_singbox_strategy("multi-socks-2");
        let strategy3 = create_singbox_strategy("multi-socks-3");
        
        let port1 = engine.start_socks(&strategy1).await.unwrap();
        let port2 = engine.start_socks(&strategy2).await.unwrap();
        let port3 = engine.start_socks(&strategy3).await.unwrap();
        
        // All ports should be different
        assert_ne!(port1, port2);
        assert_ne!(port2, port3);
        assert_ne!(port1, port3);
        
        // All strategies should be running
        assert!(engine.is_running(&strategy1.id).await);
        assert!(engine.is_running(&strategy2.id).await);
        assert!(engine.is_running(&strategy3.id).await);
    }

    #[tokio::test]
    async fn test_mock_stop_one_of_multiple_socks() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy1 = create_singbox_strategy("stop-one-1");
        let strategy2 = create_singbox_strategy("stop-one-2");
        
        engine.start_socks(&strategy1).await.unwrap();
        engine.start_socks(&strategy2).await.unwrap();
        
        // Stop only first strategy
        engine.stop_socks(&strategy1.id).await.unwrap();
        
        assert!(!engine.is_running(&strategy1.id).await);
        assert!(engine.is_running(&strategy2.id).await);
    }

    #[tokio::test]
    async fn test_mock_global_replaces_previous() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy1 = create_test_strategy("global-replace-1", false, true, EngineType::SingBox);
        let strategy2 = create_test_strategy("global-replace-2", false, true, EngineType::SingBox);
        
        engine.start_global(&strategy1).await.unwrap();
        assert_eq!(engine.get_global_strategy().await, Some(strategy1.id.clone()));
        
        // Starting second global should replace first
        engine.start_global(&strategy2).await.unwrap();
        assert_eq!(engine.get_global_strategy().await, Some(strategy2.id.clone()));
        
        // First should no longer be running
        assert!(!engine.is_running(&strategy1.id).await);
        assert!(engine.is_running(&strategy2.id).await);
    }

    // ========================================================================
    // Mode Switching Tests
    // ========================================================================

    #[tokio::test]
    async fn test_mode_switch_clears_mock_state() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("mode-switch-1");
        engine.start_socks(&strategy).await.unwrap();
        
        assert!(engine.is_running(&strategy.id).await);
        
        // Switch to Real mode should clear mock state
        engine.set_mode(EngineMode::Real).await;
        
        // Note: is_running checks mock_running in mock mode, processes in real mode
        // After switching to Real, mock state is cleared but we're now checking processes
        assert!(!engine.is_running(&strategy.id).await);
    }

    #[tokio::test]
    async fn test_mode_switch_back_to_mock() {
        let engine = StrategyEngine::new();
        
        engine.set_mode(EngineMode::Mock).await;
        engine.set_mode(EngineMode::Real).await;
        engine.set_mode(EngineMode::Mock).await;
        
        assert_eq!(engine.get_mode().await, EngineMode::Mock);
        
        // Should be able to start strategies again
        let strategy = create_singbox_strategy("mode-switch-back");
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dpi_test_mode() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::DpiTest).await;
        
        assert_eq!(engine.get_mode().await, EngineMode::DpiTest);
        
        // DpiTest mode should not be mock mode
        assert!(!engine.is_mock_mode().await);
    }

    // ========================================================================
    // WinDivert Mode Tests
    // ========================================================================

    #[tokio::test]
    async fn test_windivert_mode_all_variants() {
        let engine = StrategyEngine::new();
        
        let modes = vec![
            WinDivertMode::Normal,
            WinDivertMode::AutoTTL,
            WinDivertMode::AutoHostlist,
        ];
        
        for mode in modes {
            engine.set_windivert_mode(mode).await;
            assert_eq!(engine.get_windivert_mode().await, mode);
        }
    }

    // ========================================================================
    // Edge Cases Tests
    // ========================================================================

    #[tokio::test]
    async fn test_stop_nonexistent_socks() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Should not error when stopping non-running strategy
        let result = engine.stop_socks("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_global_when_none_running() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Should not error when no global strategy is running
        let result = engine.stop_global().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_all_when_empty() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        // Should not error when nothing is running
        let result = engine.shutdown_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_running_nonexistent_strategy() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        assert!(!engine.is_running("does-not-exist").await);
    }

    #[tokio::test]
    async fn test_get_socks_port_after_stop() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("port-after-stop");
        
        let port = engine.start_socks(&strategy).await.unwrap();
        assert!(engine.get_socks_port(&strategy.id).await.is_some());
        
        engine.stop_socks(&strategy.id).await.unwrap();
        
        // Port should be released after stop
        assert!(engine.get_socks_port(&strategy.id).await.is_none());
        
        // Port should be available for reuse
        let strategy2 = create_singbox_strategy("port-reuse");
        let port2 = engine.start_socks(&strategy2).await.unwrap();
        assert_eq!(port, port2);
    }

    // ========================================================================
    // Strategy Services Tests
    // ========================================================================

    #[test]
    fn test_strategy_has_services() {
        let strategy = create_test_strategy("services-test", true, true, EngineType::Zapret);
        
        assert!(!strategy.services.is_empty());
        assert!(strategy.services.contains(&"youtube".to_string()));
    }

    #[test]
    fn test_strategy_family() {
        let strategy = create_test_strategy("family-test", true, true, EngineType::Zapret);
        
        assert_eq!(strategy.family, StrategyFamily::DnsBypass);
    }

    // ========================================================================
    // Concurrent Access Tests (basic)
    // ========================================================================

    #[tokio::test]
    async fn test_concurrent_port_allocation() {
        let engine = Arc::new(StrategyEngine::new());
        engine.set_mode(EngineMode::Mock).await;
        
        let mut handles = vec![];
        
        for i in 0..5 {
            let engine_clone = Arc::clone(&engine);
            let handle = tokio::spawn(async move {
                let strategy = create_singbox_strategy(&format!("concurrent-{}", i));
                engine_clone.start_socks(&strategy).await
            });
            handles.push(handle);
        }
        
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        // All should succeed
        let ports: Vec<u16> = results
            .into_iter()
            .map(|r| r.unwrap().unwrap())
            .collect();
        
        // All ports should be unique
        let unique_ports: std::collections::HashSet<_> = ports.iter().collect();
        assert_eq!(unique_ports.len(), 5);
    }

    #[tokio::test]
    async fn test_concurrent_mode_read() {
        let engine = Arc::new(StrategyEngine::new());
        engine.set_mode(EngineMode::Mock).await;
        
        let mut handles = vec![];
        
        for _ in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = tokio::spawn(async move {
                engine_clone.get_mode().await
            });
            handles.push(handle);
        }
        
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        // All reads should return Mock
        for result in results {
            assert_eq!(result.unwrap(), EngineMode::Mock);
        }
    }

    // ========================================================================
    // Blocked Strategies Tests
    // ========================================================================

    #[tokio::test]
    async fn test_blocked_strategy_cannot_start_socks() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("blocked-socks");
        
        // Block the strategy
        engine.block_strategy(&strategy.id, "test block").await;
        
        // Try to start - should fail
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Strategy(_)));
    }

    #[tokio::test]
    async fn test_blocked_strategy_cannot_start_global() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_test_strategy("blocked-global", false, true, EngineType::SingBox);
        
        // Block the strategy
        engine.block_strategy(&strategy.id, "test block").await;
        
        // Try to start - should fail
        let result = engine.start_global(&strategy).await;
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Strategy(_)));
    }

    #[tokio::test]
    async fn test_unblocked_strategy_can_start() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("unblock-test");
        
        // Block then unblock
        engine.block_strategy(&strategy.id, "test block").await;
        assert!(engine.is_strategy_blocked(&strategy.id).await);
        
        engine.unblock_strategy(&strategy.id).await;
        assert!(!engine.is_strategy_blocked(&strategy.id).await);
        
        // Should be able to start now
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_failure_blocks_after_threshold() {
        let engine = StrategyEngine::new();
        
        // Record 3 failures
        assert!(!engine.record_strategy_failure("fail-test", "error 1").await);
        assert!(!engine.record_strategy_failure("fail-test", "error 2").await);
        assert!(engine.record_strategy_failure("fail-test", "error 3").await); // Should block
        
        assert!(engine.is_strategy_blocked("fail-test").await);
    }

    #[tokio::test]
    async fn test_record_success_resets_failure_counter() {
        let engine = StrategyEngine::new();
        
        // Record 2 failures
        engine.record_strategy_failure("success-reset", "error 1").await;
        engine.record_strategy_failure("success-reset", "error 2").await;
        assert_eq!(engine.get_failure_count("success-reset").await, 2);
        
        // Record success - should reset counter
        engine.record_strategy_success("success-reset").await;
        assert_eq!(engine.get_failure_count("success-reset").await, 0);
        
        // Need 3 more failures to block
        engine.record_strategy_failure("success-reset", "error 1").await;
        engine.record_strategy_failure("success-reset", "error 2").await;
        assert!(!engine.is_strategy_blocked("success-reset").await);
    }

    #[tokio::test]
    async fn test_get_blocked_strategies_list() {
        let engine = StrategyEngine::new();
        
        engine.block_strategy("blocked-1", "reason 1").await;
        engine.block_strategy("blocked-2", "reason 2").await;
        
        let blocked = engine.get_blocked_strategies().await;
        assert_eq!(blocked.len(), 2);
    }

    #[tokio::test]
    async fn test_get_blocked_info() {
        let engine = StrategyEngine::new();
        
        engine.block_strategy("info-test", "test reason").await;
        
        let info = engine.get_blocked_info("info-test").await;
        assert!(info.is_some());
        
        let info = info.unwrap();
        assert_eq!(info.strategy_id, "info-test");
        assert_eq!(info.reason, "test reason");
    }

    #[tokio::test]
    async fn test_reset_all_blocks() {
        let engine = StrategyEngine::new();
        
        engine.block_strategy("reset-1", "reason").await;
        engine.record_strategy_failure("reset-2", "error").await;
        
        engine.reset_all_blocks().await;
        
        assert!(!engine.is_strategy_blocked("reset-1").await);
        assert_eq!(engine.get_failure_count("reset-2").await, 0);
        assert!(engine.get_blocked_strategies().await.is_empty());
    }

    #[tokio::test]
    async fn test_blocked_strategy_in_mock_mode() {
        let engine = StrategyEngine::new();
        engine.set_mode(EngineMode::Mock).await;
        
        let strategy = create_singbox_strategy("mock-blocked");
        
        // Start successfully first
        let port = engine.start_socks(&strategy).await.unwrap();
        engine.stop_socks(&strategy.id).await.unwrap();
        
        // Block the strategy
        engine.block_strategy(&strategy.id, "mock test").await;
        
        // Should not be able to start again
        let result = engine.start_socks(&strategy).await;
        assert!(result.is_err());
    }
}

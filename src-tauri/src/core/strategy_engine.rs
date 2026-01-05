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

use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};
use rand::Rng;

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{LaunchTemplate, Strategy, StrategyEngine as EngineType};
use crate::core::nodpi_engine::{
    self, build_winws_args_from_template, get_binary_path_from_template,
    is_windivert_active, reset_windivert_flag,
    NoDpiHandle,
};

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
#[derive(Debug)]
struct RunningProcess {
    child: Option<Child>,
    strategy_id: String,
    mode: LaunchMode,
    port: Option<u16>,
    engine: EngineType,
}

/// Информация о запущенной Zapret стратегии (через nodpi_engine)
struct RunningZapretStrategy {
    handle: NoDpiHandle,
    strategy_id: String,
    mode: LaunchMode,
}

/// Менеджер портов для SOCKS-прокси
#[derive(Debug, Default)]
struct PortManager {
    allocated: HashMap<u16, String>, // port -> strategy_id
}

impl PortManager {
    /// Выделяет свободный порт для стратегии
    fn allocate(&mut self, strategy_id: &str) -> Result<u16> {
        for port in SOCKS_PORT_START..(SOCKS_PORT_START + MAX_SOCKS_PORTS) {
            if let std::collections::hash_map::Entry::Vacant(e) = self.allocated.entry(port) {
                e.insert(strategy_id.to_string());
                debug!(port, strategy_id, "Allocated SOCKS port");
                return Ok(port);
            }
        }
        Err(IsolateError::Process("No free SOCKS ports available".into()))
    }

    /// Освобождает порт
    fn release(&mut self, port: u16) {
        if let Some(strategy_id) = self.allocated.remove(&port) {
            debug!(port, strategy_id, "Released SOCKS port");
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
}

// ============================================================================
// Strategy Engine
// ============================================================================

/// Движок управления стратегиями
pub struct StrategyEngine {
    /// Режим работы движка
    mode: RwLock<EngineMode>,
    /// Запущенные процессы (не-Zapret)
    processes: RwLock<HashMap<String, RunningProcess>>,
    /// Запущенная Zapret стратегия (через nodpi_engine)
    zapret_strategy: RwLock<Option<RunningZapretStrategy>>,
    /// Менеджер портов
    port_manager: Mutex<PortManager>,
    /// Блокировка для последовательного запуска Zapret
    zapret_lock: Mutex<()>,
    /// Текущая глобальная стратегия
    global_strategy: RwLock<Option<String>>,
    /// Mock-состояние запущенных стратегий (для Mock режима)
    mock_running: RwLock<HashMap<String, MockStrategyState>>,
}

/// Состояние mock-стратегии
#[derive(Debug, Clone)]
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
            processes: RwLock::new(HashMap::new()),
            zapret_strategy: RwLock::new(None),
            port_manager: Mutex::new(PortManager::default()),
            zapret_lock: Mutex::new(()),
            global_strategy: RwLock::new(None),
            mock_running: RwLock::new(HashMap::new()),
        }
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
    /// Возвращает выделенный порт для подключения
    pub async fn start_socks(&self, strategy: &Strategy) -> Result<u16> {
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
    pub async fn start_global(&self, strategy: &Strategy) -> Result<()> {
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

        info!(strategy_id = %strategy.id, "Started global strategy");
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
    // Private Methods
    // ========================================================================

    /// Запускает Zapret стратегию через nodpi_engine
    async fn start_zapret_global_via_nodpi(&self, strategy: &Strategy) -> Result<()> {
        // Захватываем блокировку для последовательного запуска
        let _guard = self.zapret_lock.lock().await;

        debug!(
            strategy_id = %strategy.id,
            "Acquired Zapret lock for GLOBAL launch via nodpi_engine"
        );

        // Останавливаем предыдущую Zapret стратегию если есть
        self.stop_zapret_strategy().await.ok();

        // Запускаем через nodpi_engine
        let handle = nodpi_engine::start_nodpi_from_strategy(strategy, true).await?;

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
            "Zapret strategy started via nodpi_engine"
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
    async fn start_zapret_socks(
        &self,
        strategy: &Strategy,
        template: &LaunchTemplate,
        port: u16,
    ) -> Result<()> {
        // Захватываем блокировку для последовательного запуска
        let _guard = self.zapret_lock.lock().await;

        debug!(
            strategy_id = %strategy.id,
            "Acquired Zapret lock for SOCKS launch"
        );

        // Задержка перед запуском (для стабильности WinDivert)
        tokio::time::sleep(tokio::time::Duration::from_millis(ZAPRET_LAUNCH_DELAY_MS)).await;

        self.start_process(strategy, template, LaunchMode::Socks, Some(port))
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
        // Проверяем, не запущена ли уже стратегия
        {
            let processes = self.processes.read().await;
            if processes.contains_key(&strategy.id) {
                return Err(IsolateError::Process(format!(
                    "Strategy '{}' is already running",
                    strategy.id
                )));
            }
        }

        // Для Zapret используем nodpi_engine для разрешения путей
        let (binary_path, args) = if strategy.engine == EngineType::Zapret {
            let binary = get_binary_path_from_template(template);
            let resolved_args = build_winws_args_from_template(template);
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
        };

        {
            let mut processes = self.processes.write().await;
            processes.insert(strategy.id.clone(), process);
        }

        info!(
            strategy_id = %strategy.id,
            mode = ?mode,
            port = ?port,
            "Strategy process started"
        );

        Ok(())
    }

    /// Останавливает процесс стратегии
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
            // Пытаемся graceful shutdown через kill
            // На Windows это отправит TerminateProcess
            let _ = child.start_kill();

            // Ждём завершения с таймаутом
            let shutdown_result = tokio::time::timeout(
                tokio::time::Duration::from_millis(SHUTDOWN_TIMEOUT_MS),
                child.wait(),
            )
            .await;

            match shutdown_result {
                Ok(Ok(status)) => {
                    debug!(strategy_id, ?status, "Process exited gracefully");
                }
                Ok(Err(e)) => {
                    warn!(strategy_id, error = %e, "Error waiting for process");
                }
                Err(_) => {
                    // Таймаут - принудительное завершение
                    warn!(strategy_id, "Process did not exit gracefully, killing");
                    let _ = child.kill().await;
                }
            }
        }

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
        // Проверяем, не запущена ли уже стратегия
        {
            let mock = self.mock_running.read().await;
            if mock.contains_key(&strategy.id) {
                return Err(IsolateError::Process(format!(
                    "Strategy '{}' is already running (mock)",
                    strategy.id
                )));
            }
        }

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

        // Сохраняем mock-состояние
        {
            let mut mock = self.mock_running.write().await;
            mock.insert(
                strategy.id.clone(),
                MockStrategyState {
                    strategy_id: strategy.id.clone(),
                    mode,
                    port,
                    started_at: std::time::Instant::now(),
                },
            );
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

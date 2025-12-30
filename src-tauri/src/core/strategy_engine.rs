//! Strategy Engine - управление запуском и остановкой стратегий
//!
//! Поддерживает два режима:
//! - SOCKS: локальный прокси на выделенном порту
//! - GLOBAL: глобальный перехват трафика через WinDivert
//!
//! КРИТИЧНО: Zapret стратегии запускаются ТОЛЬКО последовательно!

use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{LaunchTemplate, Strategy, StrategyEngine as EngineType};

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

// ============================================================================
// Types
// ============================================================================

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
    child: Child,
    strategy_id: String,
    mode: LaunchMode,
    port: Option<u16>,
    engine: EngineType,
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
            if !self.allocated.contains_key(&port) {
                self.allocated.insert(port, strategy_id.to_string());
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
    /// Запущенные процессы
    processes: RwLock<HashMap<String, RunningProcess>>,
    /// Менеджер портов
    port_manager: Mutex<PortManager>,
    /// Блокировка для последовательного запуска Zapret
    zapret_lock: Mutex<()>,
    /// Текущая глобальная стратегия
    global_strategy: RwLock<Option<String>>,
}

impl StrategyEngine {
    /// Создаёт новый экземпляр движка
    pub fn new() -> Self {
        Self {
            processes: RwLock::new(HashMap::new()),
            port_manager: Mutex::new(PortManager::default()),
            zapret_lock: Mutex::new(()),
            global_strategy: RwLock::new(None),
        }
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

        // Проверяем права администратора
        if template.requires_admin && !is_admin() {
            return Err(IsolateError::RequiresAdmin);
        }

        // Останавливаем текущую глобальную стратегию
        self.stop_global().await?;

        // Для Zapret - последовательный запуск с блокировкой
        if strategy.engine == EngineType::Zapret {
            self.start_zapret_global(strategy, template).await?;
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

        self.stop_process(strategy_id).await?;

        if let Some(port) = port {
            let mut pm = self.port_manager.lock().await;
            pm.release(port);
        }

        Ok(())
    }

    /// Останавливает глобальную стратегию
    pub async fn stop_global(&self) -> Result<()> {
        let strategy_id = {
            let global = self.global_strategy.read().await;
            global.clone()
        };

        if let Some(id) = strategy_id {
            self.stop_process(&id).await?;
            let mut global = self.global_strategy.write().await;
            *global = None;
            info!(strategy_id = %id, "Stopped global strategy");
        }

        Ok(())
    }

    /// Останавливает все запущенные процессы
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all strategy processes");

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

        Ok(())
    }

    /// Проверяет, запущена ли стратегия
    pub async fn is_running(&self, strategy_id: &str) -> bool {
        let processes = self.processes.read().await;
        processes.contains_key(strategy_id)
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

    /// Запускает Zapret в SOCKS-режиме с блокировкой
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

    /// Запускает Zapret в глобальном режиме с блокировкой
    async fn start_zapret_global(
        &self,
        strategy: &Strategy,
        template: &LaunchTemplate,
    ) -> Result<()> {
        // Захватываем блокировку для последовательного запуска
        let _guard = self.zapret_lock.lock().await;

        debug!(
            strategy_id = %strategy.id,
            "Acquired Zapret lock for GLOBAL launch"
        );

        // Задержка перед запуском
        tokio::time::sleep(tokio::time::Duration::from_millis(ZAPRET_LAUNCH_DELAY_MS)).await;

        self.start_process(strategy, template, LaunchMode::Global, None)
            .await
    }

    /// Запускает процесс стратегии
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

        // Подготавливаем аргументы с подстановкой порта
        let args = self.prepare_args(&template.args, port);

        debug!(
            strategy_id = %strategy.id,
            binary = %template.binary,
            args = ?args,
            mode = ?mode,
            "Starting strategy process"
        );

        // Запускаем процесс
        let mut cmd = Command::new(&template.binary);
        cmd.args(&args);

        // Добавляем переменные окружения
        for (key, value) in &template.env {
            let value = self.substitute_port(value, port);
            cmd.env(key, value);
        }

        // Скрываем окно консоли на Windows
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let child = cmd.spawn().map_err(|e| {
            IsolateError::Process(format!(
                "Failed to start '{}': {}",
                template.binary, e
            ))
        })?;

        // Сохраняем информацию о процессе
        let process = RunningProcess {
            child,
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

        // Пытаемся graceful shutdown через kill
        // На Windows это отправит TerminateProcess
        let _ = process.child.start_kill();

        // Ждём завершения с таймаутом
        let shutdown_result = tokio::time::timeout(
            tokio::time::Duration::from_millis(SHUTDOWN_TIMEOUT_MS),
            process.child.wait(),
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
                let _ = process.child.kill().await;
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

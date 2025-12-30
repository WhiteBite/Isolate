//! Application state management for Isolate
//!
//! Централизованное хранение глобального состояния приложения.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::core::app_routing::AppRouter;
use crate::core::config::ConfigManager;
use crate::core::domain_routing::DomainRouter;
use crate::core::env_info::collect_env_info;
use crate::core::errors::{IsolateError, Result};
use crate::core::models::EnvInfo;
use crate::core::orchestrator::{create_orchestrator, SharedOrchestrator};
use crate::core::storage::Storage;
use crate::core::strategy_engine::{create_engine, SharedStrategyEngine};

// ============================================================================
// Constants
// ============================================================================

/// Имя директории приложения
const APP_DIR_NAME: &str = "Isolate";
/// Поддиректория для конфигов
const CONFIGS_DIR: &str = "configs";
/// Поддиректория для стратегий
const STRATEGIES_DIR: &str = "strategies";
/// Поддиректория для сервисов
const SERVICES_DIR: &str = "services";

// ============================================================================
// AppState
// ============================================================================

/// Глобальное состояние приложения
pub struct AppState {
    /// Оркестратор оптимизации
    pub orchestrator: SharedOrchestrator,
    /// Движок стратегий
    pub strategy_engine: SharedStrategyEngine,
    /// Менеджер конфигураций
    pub config_manager: Arc<ConfigManager>,
    /// SQLite хранилище
    pub storage: Arc<Storage>,
    /// Информация об окружении
    pub env_info: Arc<RwLock<EnvInfo>>,
    /// Domain-based routing manager
    pub domain_router: Arc<DomainRouter>,
    /// Application-based routing manager
    pub app_router: Arc<AppRouter>,
}

impl AppState {
    /// Создаёт и инициализирует глобальное состояние приложения
    pub async fn new() -> Result<Self> {
        info!("Initializing application state");

        // 1. Определяем пути к конфигам
        let (strategies_dir, services_dir) = Self::get_config_paths()?;
        debug!(
            strategies_dir = %strategies_dir.display(),
            services_dir = %services_dir.display(),
            "Config paths resolved"
        );

        // 2. Создаём ConfigManager
        let config_manager = Arc::new(ConfigManager::new(strategies_dir, services_dir));

        // 3. Загружаем стратегии и сервисы для валидации
        let strategies = config_manager.load_strategies().await?;
        let services = config_manager.load_services().await?;
        info!(
            strategies_count = strategies.len(),
            services_count = services.len(),
            "Configurations loaded"
        );

        // 4. Инициализируем Storage (SQLite)
        let storage = Arc::new(Storage::new()?);
        debug!("Storage initialized");

        // 5. Создаём StrategyEngine
        let strategy_engine = create_engine();
        debug!("Strategy engine created");

        // 6. Создаём Orchestrator
        let orchestrator = create_orchestrator(strategy_engine.clone(), storage.clone());
        debug!("Orchestrator created");

        // 7. Собираем EnvInfo
        let env_info = collect_env_info().await;
        info!(
            asn = ?env_info.asn,
            country = ?env_info.country,
            is_admin = env_info.is_admin,
            "Environment info collected"
        );

        // 8. Создаём роутеры
        let domain_router = Arc::new(DomainRouter::new(storage.clone()));
        let app_router = Arc::new(AppRouter::new(storage.clone()));
        debug!("Routing managers created");

        let state = Self {
            orchestrator,
            strategy_engine,
            config_manager,
            storage,
            env_info: Arc::new(RwLock::new(env_info)),
            domain_router,
            app_router,
        };

        info!("Application state initialized successfully");
        Ok(state)
    }

    /// Определяет пути к директориям конфигов
    ///
    /// В dev-режиме: `configs/strategies/` и `configs/services/`
    /// В prod-режиме: `%APPDATA%/Isolate/configs/strategies/` и `%APPDATA%/Isolate/configs/services/`
    fn get_config_paths() -> Result<(PathBuf, PathBuf)> {
        let base_dir = if cfg!(debug_assertions) {
            // Dev mode: используем локальную директорию configs/
            std::env::current_dir()
                .map_err(|e| IsolateError::Config(format!("Failed to get current dir: {}", e)))?
        } else {
            // Prod mode: используем %APPDATA%/Isolate/
            let app_data = std::env::var("APPDATA").map_err(|_| {
                IsolateError::Config("APPDATA environment variable not found".into())
            })?;
            PathBuf::from(app_data).join(APP_DIR_NAME)
        };

        let configs_dir = base_dir.join(CONFIGS_DIR);
        let strategies_dir = configs_dir.join(STRATEGIES_DIR);
        let services_dir = configs_dir.join(SERVICES_DIR);

        Ok((strategies_dir, services_dir))
    }

    /// Обновляет информацию об окружении
    pub async fn refresh_env_info(&self) {
        let new_env_info = collect_env_info().await;
        let mut env_info = self.env_info.write().await;
        *env_info = new_env_info;
        debug!("Environment info refreshed");
    }

    /// Получает копию текущей информации об окружении
    pub async fn get_env_info(&self) -> EnvInfo {
        let env_info = self.env_info.read().await;
        env_info.clone()
    }

    /// Перезагружает конфигурации стратегий и сервисов
    pub async fn reload_configs(&self) -> Result<()> {
        info!("Reloading configurations");
        let (strategies, services) = self.config_manager.reload().await?;
        info!(
            strategies_count = strategies.len(),
            services_count = services.len(),
            "Configurations reloaded"
        );
        Ok(())
    }

    /// Выполняет graceful shutdown всех компонентов
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down application state");

        // Останавливаем все запущенные стратегии
        self.strategy_engine.shutdown_all().await?;

        // Очищаем устаревший кэш
        let _ = self.storage.cleanup_expired_cache();

        info!("Application state shutdown complete");
        Ok(())
    }
}

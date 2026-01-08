//! Application state management for Isolate
//!
//! Централизованное хранение глобального состояния приложения.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::core::app_routing::AppRouter;
use crate::core::auto_failover::AutoFailover;
use crate::core::automation::{DomainMonitor, MonitorConfig, StrategyOptimizer};
use crate::core::config::ConfigManager;
use crate::core::domain_routing::DomainRouter;
use crate::core::env_info::collect_env_info;
use crate::core::errors::{IsolateError, Result};
use crate::core::event_bus::{create_event_bus, SharedEventBus};
use crate::core::managers::{
    BlockedStrategiesManager, LockedStrategiesManager, StrategyCacheManager, StrategyHistoryManager,
};
use crate::core::models::EnvInfo;
use crate::core::monitor::Monitor;
use crate::core::storage::Storage;
use crate::core::strategy_engine::{create_engine, SharedStrategyEngine};
use crate::core::telemetry::TelemetryService;
use crate::plugins::{create_hostlist_registry, create_plugin_manager, HostlistRegistry, PluginManager, StrategyRegistry};
use crate::services::{ServiceChecker, ServiceRegistry};

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
    /// Strategy health monitor
    pub monitor: Arc<Monitor>,
    /// Telemetry service (opt-in)
    pub telemetry: Arc<TelemetryService>,
    /// Plugins directory path
    pub plugins_dir: PathBuf,
    /// Service registry (new services system)
    pub service_registry: Arc<ServiceRegistry>,
    /// Service checker (new services system)
    pub service_checker: Arc<ServiceChecker>,
    /// Hostlist registry for plugin hostlists
    pub hostlist_registry: Arc<HostlistRegistry>,
    /// Strategy registry for plugin strategies
    pub strategy_registry: Arc<StrategyRegistry>,
    /// Plugin manager for hot reload
    pub plugin_manager: Arc<PluginManager>,
    /// Cancellation token for tests (per-session, not global)
    pub tests_cancel_token: Arc<RwLock<CancellationToken>>,
    /// Blocked strategies manager (user + default blocks)
    pub blocked_manager: Arc<BlockedStrategiesManager>,
    /// Locked strategies manager (per-domain locks)
    pub locked_manager: Arc<LockedStrategiesManager>,
    /// Strategy history manager (success/failure tracking)
    pub history_manager: Arc<StrategyHistoryManager>,
    /// Strategy cache manager (env_key -> strategy cache)
    pub cache_manager: Arc<StrategyCacheManager>,
    /// Strategy optimizer (one-time optimization)
    pub optimizer: Arc<StrategyOptimizer>,
    /// Domain monitor (continuous monitoring)
    pub domain_monitor: Arc<DomainMonitor>,
    /// Centralized event bus for pub/sub
    pub event_bus: SharedEventBus,
    /// Auto failover manager
    pub auto_failover: Arc<AutoFailover>,
}

impl AppState {
    /// Создаёт и инициализирует глобальное состояние приложения
    pub async fn new() -> Result<Self> {
        info!("Initializing application state");

        // 1. Определяем пути к конфигам
        info!("[AppState] Step 1: Resolving config paths...");
        let (strategies_dir, services_dir) = Self::get_config_paths()?;
        info!(
            strategies_dir = %strategies_dir.display(),
            services_dir = %services_dir.display(),
            "[AppState] Config paths resolved"
        );

        // 2. Создаём ConfigManager
        info!("[AppState] Step 2: Creating ConfigManager...");
        let config_manager = Arc::new(ConfigManager::new(strategies_dir, services_dir));

        // 3. Загружаем стратегии и сервисы для валидации
        info!("[AppState] Step 3: Loading strategies...");
        let strategies = config_manager.load_strategies().await?;
        info!("[AppState] Step 3b: Loading services...");
        let services = config_manager.load_services().await?;
        info!(
            strategies_count = strategies.len(),
            services_count = services.len(),
            "[AppState] Configurations loaded"
        );

        // 4. Инициализируем Storage (SQLite)
        info!("[AppState] Step 4: Initializing Storage (SQLite)...");
        let storage = Arc::new(Storage::new().await?);
        info!("[AppState] Storage initialized");

        // 4.5. Создаём менеджеры стратегий
        info!("[AppState] Step 4.5: Creating BlockedStrategiesManager...");
        let blocked_manager = Arc::new(
            BlockedStrategiesManager::new(storage.clone())
                .await
                .map_err(|e| IsolateError::Storage(format!("Failed to create blocked manager: {}", e)))?
        );
        info!("[AppState] Blocked strategies manager created");

        info!("[AppState] Step 4.6: Creating LockedStrategiesManager...");
        let locked_manager = Arc::new(
            LockedStrategiesManager::new(storage.clone(), blocked_manager.clone())
                .await
                .map_err(|e| IsolateError::Storage(format!("Failed to create locked manager: {}", e)))?
        );
        info!("[AppState] Locked strategies manager created");

        info!("[AppState] Step 4.7: Creating StrategyHistoryManager...");
        let history_manager = Arc::new(
            StrategyHistoryManager::new(storage.clone())
                .await
                .map_err(|e| IsolateError::Storage(format!("Failed to create history manager: {}", e)))?
        );
        info!("[AppState] Strategy history manager created");

        info!("[AppState] Step 4.8: Creating StrategyCacheManager...");
        let cache_manager = Arc::new(
            StrategyCacheManager::new(storage.clone())
                .await
                .map_err(|e| IsolateError::Storage(format!("Failed to create cache manager: {}", e)))?
        );
        info!("[AppState] Strategy cache manager created");

        // 5. Создаём StrategyEngine
        info!("[AppState] Step 5: Creating StrategyEngine...");
        let strategy_engine = create_engine();
        info!("[AppState] Strategy engine created");

        // 5.5. Создаём Monitor
        info!("[AppState] Step 5.5: Creating Monitor...");
        let monitor = Arc::new(Monitor::new(strategy_engine.clone()));
        info!("[AppState] Monitor created");

        // 5.6. Создаём TelemetryService
        info!("[AppState] Step 5.6: Creating TelemetryService...");
        let telemetry = Arc::new(TelemetryService::new());
        info!("[AppState] Telemetry service created");

        // 6.5. Создаём StrategyOptimizer
        info!("[AppState] Step 6.5: Creating StrategyOptimizer...");
        let optimizer = Arc::new(StrategyOptimizer::new(
            strategy_engine.clone(),
            cache_manager.clone(),
            blocked_manager.clone(),
            history_manager.clone(),
        ));
        info!("[AppState] Strategy optimizer created");

        // 6.6. Создаём DomainMonitor
        info!("[AppState] Step 6.6: Creating DomainMonitor...");
        let domain_monitor = Arc::new(DomainMonitor::new(
            locked_manager.clone(),
            blocked_manager.clone(),
            history_manager.clone(),
            MonitorConfig::default(),
        ));
        info!("[AppState] Domain monitor created");

        // 6.7. Создаём EventBus
        info!("[AppState] Step 6.7: Creating EventBus...");
        let event_bus = create_event_bus();
        info!("[AppState] Event bus created");

        // 6.8. Создаём AutoFailover
        info!("[AppState] Step 6.8: Creating AutoFailover...");
        let settings = storage.get_settings().await.unwrap_or_default();
        let auto_failover = Arc::new(AutoFailover::with_config(
            crate::core::auto_failover::FailoverConfig {
                max_failures: settings.failover_max_failures,
                cooldown_secs: settings.failover_cooldown_secs as u64,
                backup_strategy_ids: Vec::new(), // Will use learned strategies
                ..Default::default()
            }
        ));
        auto_failover.set_enabled(settings.auto_failover_enabled).await;
        info!("[AppState] Auto failover created");

        // 7. Собираем EnvInfo
        info!("[AppState] Step 7: Collecting EnvInfo (may take up to 5s for network info)...");
        let env_info = collect_env_info().await;
        info!(
            asn = ?env_info.asn,
            country = ?env_info.country,
            is_admin = env_info.is_admin,
            "[AppState] Environment info collected"
        );

        // 8. Создаём роутеры
        info!("[AppState] Step 8: Creating routers...");
        let domain_router = Arc::new(DomainRouter::new(storage.clone()));
        let app_router = Arc::new(AppRouter::new(storage.clone()));
        info!("[AppState] Routing managers created");

        // 9. Определяем путь к плагинам
        info!("[AppState] Step 9: Configuring plugins directory...");
        let plugins_dir = if cfg!(debug_assertions) {
            let current = std::env::current_dir()
                .map_err(|e| IsolateError::Config(format!("Failed to get current dir: {}", e)))?;
            if current.ends_with("src-tauri") {
                current.parent().map(|p| p.to_path_buf()).unwrap_or(current)
            } else {
                current
            }.join("plugins")
        } else {
            let app_data = std::env::var("APPDATA").map_err(|_| {
                IsolateError::Config("APPDATA environment variable not found".into())
            })?;
            PathBuf::from(app_data).join(APP_DIR_NAME).join("plugins")
        };
        
        // Создаём директорию плагинов если не существует
        if !tokio::fs::try_exists(&plugins_dir).await.unwrap_or(false) {
            tokio::fs::create_dir_all(&plugins_dir)
                .await
                .map_err(|e| IsolateError::Config(format!("Failed to create plugins dir: {}", e)))?;
        }
        info!(plugins_dir = %plugins_dir.display(), "[AppState] Plugins directory configured");

        // 10. Создаём ServiceRegistry и загружаем сервисы
        info!("[AppState] Step 10: Creating ServiceRegistry...");
        let service_registry = Arc::new(ServiceRegistry::new());
        
        // Load services from plugins FIRST (they have priority)
        if let Err(e) = service_registry.load_from_plugins(&plugins_dir).await {
            tracing::warn!(error = %e, "Failed to load services from plugins");
        }
        
        // Register built-in services (won't override existing from plugins)
        service_registry.register_builtin_services().await;
        info!("[AppState] Service registry initialized");

        // 11. Создаём ServiceChecker
        info!("[AppState] Step 11: Creating ServiceChecker...");
        let service_checker = Arc::new(ServiceChecker::new(service_registry.clone()));
        info!("[AppState] Service checker created");

        // 12. Создаём HostlistRegistry и загружаем hostlists из плагинов
        info!("[AppState] Step 12: Creating HostlistRegistry...");
        let hostlist_registry = create_hostlist_registry();
        if let Err(e) = Self::load_hostlists_from_plugins(&hostlist_registry, &plugins_dir).await {
            tracing::warn!(error = %e, "Failed to load hostlists from plugins");
        }
        info!("[AppState] Hostlist registry initialized");

        // 13. Создаём StrategyRegistry и загружаем стратегии из плагинов
        info!("[AppState] Step 13: Creating StrategyRegistry...");
        let strategy_registry = Arc::new(StrategyRegistry::new());
        if let Err(e) = Self::load_strategies_from_plugins(&strategy_registry, &plugins_dir).await {
            tracing::warn!(error = %e, "Failed to load strategies from plugins");
        }
        info!("[AppState] Strategy registry initialized");

        // 14. Создаём PluginManager для hot reload
        info!("[AppState] Step 14: Creating PluginManager...");
        let plugin_manager = create_plugin_manager(&plugins_dir);
        if let Err(e) = plugin_manager.init().await {
            tracing::warn!(error = %e, "Failed to initialize plugin manager");
        }
        info!("[AppState] Plugin manager initialized");

        let state = Self {
            strategy_engine,
            config_manager,
            storage,
            env_info: Arc::new(RwLock::new(env_info)),
            domain_router,
            app_router,
            monitor,
            telemetry,
            plugins_dir,
            service_registry,
            service_checker,
            hostlist_registry,
            strategy_registry,
            plugin_manager,
            tests_cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
            blocked_manager,
            locked_manager,
            history_manager,
            cache_manager,
            optimizer,
            domain_monitor,
            event_bus,
            auto_failover,
        };

        info!("Application state initialized successfully");
        Ok(state)
    }

    /// Определяет пути к директориям конфигов
    ///
    /// В dev-режиме: `configs/strategies/` и `configs/services/` (относительно корня проекта)
    /// В prod-режиме: `%APPDATA%/Isolate/configs/strategies/` и `%APPDATA%/Isolate/configs/services/`
    fn get_config_paths() -> Result<(PathBuf, PathBuf)> {
        let base_dir = if cfg!(debug_assertions) {
            // Dev mode: cargo запускается из src-tauri/, поэтому поднимаемся на уровень выше
            let current = std::env::current_dir()
                .map_err(|e| IsolateError::Config(format!("Failed to get current dir: {}", e)))?;
            
            // Если мы в src-tauri, поднимаемся на уровень выше
            if current.ends_with("src-tauri") {
                current.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or(current)
            } else {
                current
            }
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

        // Останавливаем monitor
        self.monitor.stop();

        // Останавливаем все запущенные стратегии
        self.strategy_engine.shutdown_all().await?;

        // Очищаем устаревший кэш
        let _ = self.storage.cleanup_expired_cache().await;

        info!("Application state shutdown complete");
        Ok(())
    }

    /// Загружает hostlists из плагинов
    async fn load_hostlists_from_plugins(
        registry: &Arc<HostlistRegistry>,
        plugins_dir: &PathBuf,
    ) -> Result<()> {
        use crate::plugins::{get_all_plugins_async, PluginType};

        let plugins = get_all_plugins_async(plugins_dir).await;
        let mut loaded_count = 0;

        for plugin_info in plugins {
            if !plugin_info.enabled || plugin_info.error.is_some() {
                continue;
            }

            let manifest = &plugin_info.manifest;
            let plugin_path = PathBuf::from(&plugin_info.path);

            // Load hostlists from contributes.hostlists (new format)
            for hostlist_def in &manifest.contributes.hostlists {
                match registry
                    .register(&manifest.id, plugin_path.clone(), hostlist_def.clone())
                    .await
                {
                    Ok(()) => {
                        loaded_count += 1;
                        debug!(
                            plugin_id = %manifest.id,
                            hostlist_id = %hostlist_def.id,
                            "Loaded hostlist from plugin"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            plugin_id = %manifest.id,
                            hostlist_id = %hostlist_def.id,
                            error = %e,
                            "Failed to load hostlist from plugin"
                        );
                    }
                }
            }

            // Also load from legacy hostlist field (for backward compatibility)
            if manifest.plugin_type == PluginType::HostlistProvider {
                if let Some(ref hostlist_def) = manifest.hostlist {
                    match registry
                        .register(&manifest.id, plugin_path.clone(), hostlist_def.clone())
                        .await
                    {
                        Ok(()) => {
                            loaded_count += 1;
                            debug!(
                                plugin_id = %manifest.id,
                                hostlist_id = %hostlist_def.id,
                                "Loaded legacy hostlist from plugin"
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                plugin_id = %manifest.id,
                                hostlist_id = %hostlist_def.id,
                                error = %e,
                                "Failed to load legacy hostlist from plugin"
                            );
                        }
                    }
                }
            }
        }

        info!(count = loaded_count, "Loaded hostlists from plugins");
        Ok(())
    }

    /// Загружает стратегии из плагинов
    async fn load_strategies_from_plugins(
        registry: &Arc<StrategyRegistry>,
        plugins_dir: &PathBuf,
    ) -> Result<()> {
        use crate::plugins::{
            get_all_plugins_async, PluginType, PluginStrategyDefinition, PluginStrategyConfig,
            StrategyFamily, StrategySource,
        };

        let plugins = get_all_plugins_async(plugins_dir).await;
        let mut loaded_count = 0;

        for plugin_info in plugins {
            if !plugin_info.enabled || plugin_info.error.is_some() {
                continue;
            }

            let manifest = &plugin_info.manifest;

            // Skip non-strategy plugins
            if manifest.plugin_type != PluginType::StrategyProvider {
                continue;
            }

            // Load strategies from contributes.strategies (new format)
            for strategy_def in &manifest.contributes.strategies {
                let plugin_strategy = PluginStrategyDefinition {
                    id: strategy_def.id.clone(),
                    name: strategy_def.name.clone(),
                    description: None,
                    family: StrategyFamily::from(strategy_def.family.as_str()),
                    engine: "winws".to_string(),
                    target_services: Vec::new(),
                    priority: 0,
                    config: PluginStrategyConfig::default(),
                    author: None,
                    label: None,
                    source_plugin: Some(manifest.id.clone()),
                };

                match registry
                    .register(
                        plugin_strategy,
                        StrategySource::Plugin {
                            plugin_id: manifest.id.clone(),
                        },
                    )
                    .await
                {
                    Ok(()) => {
                        loaded_count += 1;
                        debug!(
                            plugin_id = %manifest.id,
                            strategy_id = %strategy_def.id,
                            "Loaded strategy from plugin"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            plugin_id = %manifest.id,
                            strategy_id = %strategy_def.id,
                            error = %e,
                            "Failed to load strategy from plugin"
                        );
                    }
                }
            }

            // Also load from legacy strategy field (for backward compatibility)
            if let Some(ref strategy_def) = manifest.strategy {
                let plugin_strategy = PluginStrategyDefinition {
                    id: strategy_def.id.clone(),
                    name: strategy_def.name.clone(),
                    description: None,
                    family: StrategyFamily::from(strategy_def.family.as_str()),
                    engine: "winws".to_string(),
                    target_services: Vec::new(),
                    priority: 0,
                    config: PluginStrategyConfig::default(),
                    author: None,
                    label: None,
                    source_plugin: Some(manifest.id.clone()),
                };

                match registry
                    .register(
                        plugin_strategy,
                        StrategySource::Plugin {
                            plugin_id: manifest.id.clone(),
                        },
                    )
                    .await
                {
                    Ok(()) => {
                        loaded_count += 1;
                        debug!(
                            plugin_id = %manifest.id,
                            strategy_id = %strategy_def.id,
                            "Loaded legacy strategy from plugin"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            plugin_id = %manifest.id,
                            strategy_id = %strategy_def.id,
                            error = %e,
                            "Failed to load legacy strategy from plugin"
                        );
                    }
                }
            }
        }

        info!(count = loaded_count, "Loaded strategies from plugins");
        Ok(())
    }
}

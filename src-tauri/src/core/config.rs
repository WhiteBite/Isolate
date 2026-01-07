//! Configuration loader for Isolate
//!
//! Loads and validates YAML configs for strategies and services.
//!
//! NOTE: Some functions are prepared for future config management features.

// Public API for configuration loading
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{Service, Strategy};

/// Cached data with timestamp
#[derive(Debug)]
struct CachedData<T> {
    data: T,
    loaded_at: Instant,
}

/// Configuration manager for loading strategies and services
#[derive(Debug)]
pub struct ConfigManager {
    strategies_dir: PathBuf,
    services_dir: PathBuf,
    /// Cache for strategies
    strategies_cache: RwLock<Option<CachedData<HashMap<String, Strategy>>>>,
    /// Cache for services
    services_cache: RwLock<Option<CachedData<HashMap<String, Service>>>>,
}

impl ConfigManager {
    /// Cache TTL (60 seconds)
    const CACHE_TTL: Duration = Duration::from_secs(60);

    /// Create a new ConfigManager with specified directories
    pub fn new(strategies_dir: PathBuf, services_dir: PathBuf) -> Self {
        Self {
            strategies_dir,
            services_dir,
            strategies_cache: RwLock::new(None),
            services_cache: RwLock::new(None),
        }
    }

    /// Create ConfigManager with default paths relative to app data
    pub fn with_default_paths(base_dir: &Path) -> Self {
        Self {
            strategies_dir: base_dir.join("configs").join("strategies"),
            services_dir: base_dir.join("configs").join("services"),
            strategies_cache: RwLock::new(None),
            services_cache: RwLock::new(None),
        }
    }

    /// Load all strategies from the strategies directory (with caching)
    pub async fn load_strategies(&self) -> Result<HashMap<String, Strategy>> {
        // Check cache first
        {
            let cache = self.strategies_cache.read().await;
            if let Some(ref cached) = *cache {
                if cached.loaded_at.elapsed() < Self::CACHE_TTL {
                    debug!("Using cached strategies");
                    return Ok(cached.data.clone());
                }
            }
        }

        // Load from disk
        let strategies = self.load_strategies_from_disk().await?;

        // Update cache
        {
            let mut cache = self.strategies_cache.write().await;
            *cache = Some(CachedData {
                data: strategies.clone(),
                loaded_at: Instant::now(),
            });
        }

        Ok(strategies)
    }

    /// Load strategies from disk (internal, no caching)
    async fn load_strategies_from_disk(&self) -> Result<HashMap<String, Strategy>> {
        info!(dir = ?self.strategies_dir, "Loading strategies from disk");

        let mut strategies = HashMap::new();

        if !tokio::fs::try_exists(&self.strategies_dir).await.unwrap_or(false) {
            warn!(dir = ?self.strategies_dir, "Strategies directory does not exist");
            return Ok(strategies);
        }

        let mut entries = fs::read_dir(&self.strategies_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if !is_yaml_file(&path) {
                continue;
            }

            match self.load_strategy_file(&path).await {
                Ok(strategy) => {
                    debug!(id = %strategy.id, name = %strategy.name, "Loaded strategy");
                    strategies.insert(strategy.id.clone(), strategy);
                }
                Err(e) => {
                    error!(path = ?path, error = %e, "Failed to load strategy");
                }
            }
        }

        info!(count = strategies.len(), "Strategies loaded");
        Ok(strategies)
    }

    /// Load a single strategy from file
    async fn load_strategy_file(&self, path: &Path) -> Result<Strategy> {
        let content = fs::read_to_string(path).await?;
        let strategy: Strategy = serde_yaml::from_str(&content)?;

        self.validate_strategy(&strategy)?;

        Ok(strategy)
    }

    /// Validate strategy configuration
    fn validate_strategy(&self, strategy: &Strategy) -> Result<()> {
        if strategy.id.is_empty() {
            return Err(IsolateError::Config("Strategy ID cannot be empty".into()));
        }

        if strategy.name.is_empty() {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' has empty name",
                strategy.id
            )));
        }

        // Validate that at least one template is defined
        if strategy.socks_template.is_none() && strategy.global_template.is_none() {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' must have at least one launch template (socks or global)",
                strategy.id
            )));
        }

        // Validate mode capabilities match templates
        if strategy.mode_capabilities.supports_socks && strategy.socks_template.is_none() {
            warn!(
                id = %strategy.id,
                "Strategy supports SOCKS mode but has no socks_template"
            );
        }

        if strategy.mode_capabilities.supports_global && strategy.global_template.is_none() {
            warn!(
                id = %strategy.id,
                "Strategy supports global mode but has no global_template"
            );
        }

        // Validate launch templates
        if let Some(ref template) = strategy.socks_template {
            self.validate_launch_template(template, &strategy.id, "socks")?;
        }

        if let Some(ref template) = strategy.global_template {
            self.validate_launch_template(template, &strategy.id, "global")?;
        }

        Ok(())
    }

    /// Validate launch template
    fn validate_launch_template(
        &self,
        template: &crate::core::models::LaunchTemplate,
        strategy_id: &str,
        mode: &str,
    ) -> Result<()> {
        if template.binary.is_empty() {
            return Err(IsolateError::Config(format!(
                "Strategy '{}' {} template has empty binary path",
                strategy_id, mode
            )));
        }

        Ok(())
    }

    /// Load all services from the services directory (with caching)
    pub async fn load_services(&self) -> Result<HashMap<String, Service>> {
        // Check cache first
        {
            let cache = self.services_cache.read().await;
            if let Some(ref cached) = *cache {
                if cached.loaded_at.elapsed() < Self::CACHE_TTL {
                    debug!("Using cached services");
                    return Ok(cached.data.clone());
                }
            }
        }

        // Load from disk
        let services = self.load_services_from_disk().await?;

        // Update cache
        {
            let mut cache = self.services_cache.write().await;
            *cache = Some(CachedData {
                data: services.clone(),
                loaded_at: Instant::now(),
            });
        }

        Ok(services)
    }

    /// Load services from disk (internal, no caching)
    async fn load_services_from_disk(&self) -> Result<HashMap<String, Service>> {
        info!(dir = ?self.services_dir, "Loading services from disk");

        let mut services = HashMap::new();

        if !tokio::fs::try_exists(&self.services_dir).await.unwrap_or(false) {
            warn!(dir = ?self.services_dir, "Services directory does not exist");
            return Ok(services);
        }

        let mut entries = fs::read_dir(&self.services_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if !is_yaml_file(&path) {
                continue;
            }

            match self.load_service_file(&path).await {
                Ok(service) => {
                    debug!(id = %service.id, name = %service.name, "Loaded service");
                    services.insert(service.id.clone(), service);
                }
                Err(e) => {
                    error!(path = ?path, error = %e, "Failed to load service");
                }
            }
        }

        info!(count = services.len(), "Services loaded");
        Ok(services)
    }

    /// Load a single service from file
    async fn load_service_file(&self, path: &Path) -> Result<Service> {
        let content = fs::read_to_string(path).await?;
        let service: Service = serde_yaml::from_str(&content)?;

        self.validate_service(&service)?;

        Ok(service)
    }

    /// Validate service configuration
    fn validate_service(&self, service: &Service) -> Result<()> {
        if service.id.is_empty() {
            return Err(IsolateError::Config("Service ID cannot be empty".into()));
        }

        if service.name.is_empty() {
            return Err(IsolateError::Config(format!(
                "Service '{}' has empty name",
                service.id
            )));
        }

        if service.tests.is_empty() {
            return Err(IsolateError::Config(format!(
                "Service '{}' must have at least one test",
                service.id
            )));
        }

        Ok(())
    }

    /// Load a specific strategy by ID
    pub async fn load_strategy_by_id(&self, id: &str) -> Result<Strategy> {
        let strategies = self.load_strategies().await?;

        strategies
            .get(id)
            .cloned()
            .ok_or_else(|| IsolateError::StrategyNotFound(id.to_string()))
    }

    /// Load a specific service by ID
    pub async fn load_service_by_id(&self, id: &str) -> Result<Service> {
        let services = self.load_services().await?;

        services
            .get(id)
            .cloned()
            .ok_or_else(|| IsolateError::Config(format!("Service not found: {}", id)))
    }

    /// Reload all configurations
    pub async fn reload(&self) -> Result<(HashMap<String, Strategy>, HashMap<String, Service>)> {
        info!("Reloading all configurations");

        // Invalidate cache before reloading
        self.invalidate_cache().await;

        let strategies = self.load_strategies().await?;
        let services = self.load_services().await?;

        Ok((strategies, services))
    }

    /// Invalidate all caches (call after hot reload or config changes)
    pub async fn invalidate_cache(&self) {
        *self.strategies_cache.write().await = None;
        *self.services_cache.write().await = None;
        info!("Config cache invalidated");
    }
}

/// Check if path is a YAML file
fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext == "yaml" || ext == "yml")
        .unwrap_or(false)
}

/// Load strategies from a specific directory (convenience function)
pub async fn load_strategies_from_dir(dir: &Path) -> Result<HashMap<String, Strategy>> {
    let manager = ConfigManager::new(dir.to_path_buf(), PathBuf::new());
    manager.load_strategies().await
}

/// Load services from a specific directory (convenience function)
pub async fn load_services_from_dir(dir: &Path) -> Result<HashMap<String, Service>> {
    let manager = ConfigManager::new(PathBuf::new(), dir.to_path_buf());
    manager.load_services().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{
        LaunchTemplate, ModeCapabilities, Service, Strategy, StrategyEngine, StrategyFamily,
        StrategyRequirements, TestDefinition,
    };
    use std::collections::HashMap;

    // ========================================================================
    // Helper functions for creating test data
    // ========================================================================

    fn create_valid_strategy() -> Strategy {
        Strategy {
            id: "test-strategy".to_string(),
            name: "Test Strategy".to_string(),
            description: "A test strategy".to_string(),
            family: StrategyFamily::DnsBypass,
            engine: StrategyEngine::Zapret,
            mode_capabilities: ModeCapabilities {
                supports_socks: true,
                supports_global: false,
            },
            socks_template: Some(LaunchTemplate {
                binary: "winws.exe".to_string(),
                args: vec!["--arg1".to_string()],
                env: HashMap::new(),
                log_file: None,
                requires_admin: false,
            }),
            global_template: None,
            requirements: StrategyRequirements::default(),
            weight_hint: 100,
            services: vec!["youtube".to_string()],
        }
    }

    fn create_valid_service() -> Service {
        Service {
            id: "test-service".to_string(),
            name: "Test Service".to_string(),
            enabled_by_default: true,
            critical: false,
            tests: vec![TestDefinition::HttpsGet {
                url: "https://example.com".to_string(),
                timeout_ms: 5000,
                expected_status: vec![200],
                min_body_size: None,
            }],
            test_url: Some("https://example.com".to_string()),
        }
    }

    // ========================================================================
    // ConfigManager::new tests
    // ========================================================================

    #[test]
    fn test_config_manager_new() {
        let strategies_dir = PathBuf::from("/path/to/strategies");
        let services_dir = PathBuf::from("/path/to/services");

        let manager = ConfigManager::new(strategies_dir.clone(), services_dir.clone());

        assert_eq!(manager.strategies_dir, strategies_dir);
        assert_eq!(manager.services_dir, services_dir);
    }

    #[test]
    fn test_config_manager_new_with_empty_paths() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());

        assert_eq!(manager.strategies_dir, PathBuf::new());
        assert_eq!(manager.services_dir, PathBuf::new());
    }

    // ========================================================================
    // ConfigManager::with_default_paths tests
    // ========================================================================

    #[test]
    fn test_config_manager_with_default_paths() {
        let base_dir = Path::new("/app/data");
        let manager = ConfigManager::with_default_paths(base_dir);

        assert_eq!(
            manager.strategies_dir,
            PathBuf::from("/app/data/configs/strategies")
        );
        assert_eq!(
            manager.services_dir,
            PathBuf::from("/app/data/configs/services")
        );
    }

    #[test]
    fn test_config_manager_with_default_paths_relative() {
        let base_dir = Path::new(".");
        let manager = ConfigManager::with_default_paths(base_dir);

        assert_eq!(
            manager.strategies_dir,
            PathBuf::from("./configs/strategies")
        );
        assert_eq!(manager.services_dir, PathBuf::from("./configs/services"));
    }

    // ========================================================================
    // is_yaml_file tests
    // ========================================================================

    #[test]
    fn test_is_yaml_file_with_yaml_extension() {
        assert!(is_yaml_file(Path::new("config.yaml")));
        assert!(is_yaml_file(Path::new("/path/to/strategy.yaml")));
        assert!(is_yaml_file(Path::new("C:\\configs\\service.yaml")));
    }

    #[test]
    fn test_is_yaml_file_with_yml_extension() {
        assert!(is_yaml_file(Path::new("config.yml")));
        assert!(is_yaml_file(Path::new("/path/to/strategy.yml")));
        assert!(is_yaml_file(Path::new("C:\\configs\\service.yml")));
    }

    #[test]
    fn test_is_yaml_file_with_non_yaml_extension() {
        assert!(!is_yaml_file(Path::new("config.json")));
        assert!(!is_yaml_file(Path::new("config.toml")));
        assert!(!is_yaml_file(Path::new("config.txt")));
        assert!(!is_yaml_file(Path::new("config.xml")));
        assert!(!is_yaml_file(Path::new("README.md")));
    }

    #[test]
    fn test_is_yaml_file_without_extension() {
        assert!(!is_yaml_file(Path::new("config")));
        assert!(!is_yaml_file(Path::new("/path/to/file")));
    }

    #[test]
    fn test_is_yaml_file_with_hidden_file() {
        assert!(!is_yaml_file(Path::new(".gitignore")));
        assert!(is_yaml_file(Path::new(".config.yaml")));
    }

    #[test]
    fn test_is_yaml_file_case_sensitivity() {
        // Rust's OsStr comparison is case-sensitive on most platforms
        // but we should test the actual behavior
        assert!(!is_yaml_file(Path::new("config.YAML")));
        assert!(!is_yaml_file(Path::new("config.YML")));
        assert!(!is_yaml_file(Path::new("config.Yaml")));
    }

    // ========================================================================
    // validate_strategy tests
    // ========================================================================

    #[test]
    fn test_validate_strategy_valid() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let strategy = create_valid_strategy();

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_empty_id() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.id = String::new();

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("ID cannot be empty"));
    }

    #[test]
    fn test_validate_strategy_empty_name() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.name = String::new();

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("empty name"));
    }

    #[test]
    fn test_validate_strategy_no_templates() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.socks_template = None;
        strategy.global_template = None;

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("at least one launch template"));
    }

    #[test]
    fn test_validate_strategy_with_global_template_only() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.socks_template = None;
        strategy.global_template = Some(LaunchTemplate {
            binary: "winws.exe".to_string(),
            args: vec!["--global".to_string()],
            env: HashMap::new(),
            log_file: None,
            requires_admin: true,
        });

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_with_both_templates() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.global_template = Some(LaunchTemplate {
            binary: "winws.exe".to_string(),
            args: vec!["--global".to_string()],
            env: HashMap::new(),
            log_file: None,
            requires_admin: true,
        });

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_empty_binary_in_socks_template() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        if let Some(ref mut template) = strategy.socks_template {
            template.binary = String::new();
        }

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("empty binary path"));
    }

    #[test]
    fn test_validate_strategy_empty_binary_in_global_template() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.socks_template = None;
        strategy.global_template = Some(LaunchTemplate {
            binary: String::new(),
            args: vec![],
            env: HashMap::new(),
            log_file: None,
            requires_admin: false,
        });

        let result = manager.validate_strategy(&strategy);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("empty binary path"));
    }

    // ========================================================================
    // validate_service tests
    // ========================================================================

    #[test]
    fn test_validate_service_valid() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let service = create_valid_service();

        let result = manager.validate_service(&service);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_service_empty_id() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut service = create_valid_service();
        service.id = String::new();

        let result = manager.validate_service(&service);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("ID cannot be empty"));
    }

    #[test]
    fn test_validate_service_empty_name() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut service = create_valid_service();
        service.name = String::new();

        let result = manager.validate_service(&service);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("empty name"));
    }

    #[test]
    fn test_validate_service_empty_tests() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut service = create_valid_service();
        service.tests = vec![];

        let result = manager.validate_service(&service);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("at least one test"));
    }

    #[test]
    fn test_validate_service_with_multiple_tests() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut service = create_valid_service();
        service.tests = vec![
            TestDefinition::HttpsGet {
                url: "https://example.com".to_string(),
                timeout_ms: 5000,
                expected_status: vec![200],
                min_body_size: None,
            },
            TestDefinition::TcpConnect {
                host: "example.com".to_string(),
                port: 443,
                timeout_ms: 3000,
            },
            TestDefinition::Dns {
                domain: "example.com".to_string(),
                timeout_ms: 2000,
            },
        ];

        let result = manager.validate_service(&service);
        assert!(result.is_ok());
    }

    // ========================================================================
    // Async tests for load_strategies and load_services
    // ========================================================================

    #[tokio::test]
    async fn test_load_strategies_nonexistent_directory() {
        let manager = ConfigManager::new(
            PathBuf::from("/nonexistent/path/to/strategies"),
            PathBuf::new(),
        );

        let result = manager.load_strategies().await;
        assert!(result.is_ok());

        let strategies = result.unwrap();
        assert!(strategies.is_empty());
    }

    #[tokio::test]
    async fn test_load_services_nonexistent_directory() {
        let manager = ConfigManager::new(
            PathBuf::new(),
            PathBuf::from("/nonexistent/path/to/services"),
        );

        let result = manager.load_services().await;
        assert!(result.is_ok());

        let services = result.unwrap();
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_load_strategy_by_id_not_found() {
        let manager = ConfigManager::new(
            PathBuf::from("/nonexistent/path"),
            PathBuf::new(),
        );

        let result = manager.load_strategy_by_id("nonexistent").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::StrategyNotFound(_)));
    }

    #[tokio::test]
    async fn test_load_service_by_id_not_found() {
        let manager = ConfigManager::new(
            PathBuf::new(),
            PathBuf::from("/nonexistent/path"),
        );

        let result = manager.load_service_by_id("nonexistent").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Config(_)));
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_reload_with_nonexistent_directories() {
        let manager = ConfigManager::new(
            PathBuf::from("/nonexistent/strategies"),
            PathBuf::from("/nonexistent/services"),
        );

        let result = manager.reload().await;
        assert!(result.is_ok());

        let (strategies, services) = result.unwrap();
        assert!(strategies.is_empty());
        assert!(services.is_empty());
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_validate_strategy_whitespace_id() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut strategy = create_valid_strategy();
        strategy.id = "   ".to_string(); // whitespace only

        // Current implementation doesn't trim, so this passes
        // This test documents current behavior
        let result = manager.validate_strategy(&strategy);
        assert!(result.is_ok()); // whitespace-only ID is currently allowed
    }

    #[test]
    fn test_validate_service_whitespace_id() {
        let manager = ConfigManager::new(PathBuf::new(), PathBuf::new());
        let mut service = create_valid_service();
        service.id = "   ".to_string(); // whitespace only

        // Current implementation doesn't trim, so this passes
        // This test documents current behavior
        let result = manager.validate_service(&service);
        assert!(result.is_ok()); // whitespace-only ID is currently allowed
    }

    #[test]
    fn test_config_manager_debug() {
        let manager = ConfigManager::new(
            PathBuf::from("/strategies"),
            PathBuf::from("/services"),
        );

        let debug_str = format!("{:?}", manager);
        assert!(debug_str.contains("ConfigManager"));
        assert!(debug_str.contains("strategies_dir"));
        assert!(debug_str.contains("services_dir"));
    }

    #[tokio::test]
    async fn test_invalidate_cache() {
        let manager = ConfigManager::new(
            PathBuf::from("/nonexistent/strategies"),
            PathBuf::from("/nonexistent/services"),
        );

        // Load to populate cache
        let _ = manager.load_strategies().await;
        let _ = manager.load_services().await;

        // Invalidate
        manager.invalidate_cache().await;

        // Verify cache is empty
        let strategies_cache = manager.strategies_cache.read().await;
        assert!(strategies_cache.is_none());
        drop(strategies_cache);

        let services_cache = manager.services_cache.read().await;
        assert!(services_cache.is_none());
    }
}

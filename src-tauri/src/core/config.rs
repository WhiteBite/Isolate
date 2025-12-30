//! Configuration loader for Isolate
//!
//! Loads and validates YAML configs for strategies and services.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::fs;
use tracing::{debug, error, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{Service, Strategy};

/// Configuration manager for loading strategies and services
#[derive(Debug, Clone)]
pub struct ConfigManager {
    strategies_dir: PathBuf,
    services_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager with specified directories
    pub fn new(strategies_dir: PathBuf, services_dir: PathBuf) -> Self {
        Self {
            strategies_dir,
            services_dir,
        }
    }

    /// Create ConfigManager with default paths relative to app data
    pub fn with_default_paths(base_dir: &Path) -> Self {
        Self {
            strategies_dir: base_dir.join("configs").join("strategies"),
            services_dir: base_dir.join("configs").join("services"),
        }
    }

    /// Load all strategies from the strategies directory
    pub async fn load_strategies(&self) -> Result<HashMap<String, Strategy>> {
        info!(dir = ?self.strategies_dir, "Loading strategies");

        let mut strategies = HashMap::new();

        if !self.strategies_dir.exists() {
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

    /// Load all services from the services directory
    pub async fn load_services(&self) -> Result<HashMap<String, Service>> {
        info!(dir = ?self.services_dir, "Loading services");

        let mut services = HashMap::new();

        if !self.services_dir.exists() {
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

        let strategies = self.load_strategies().await?;
        let services = self.load_services().await?;

        Ok((strategies, services))
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

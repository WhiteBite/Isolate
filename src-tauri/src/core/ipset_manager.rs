//! IP Set Manager module
//!
//! Manages ipset modes and provides unified interface for ipset operations.
//! Supports three modes:
//! - Any: Allow all traffic (ipset not used)
//! - None: Block all traffic matching ipset rules
//! - Loaded: Use loaded ipset for filtering

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::ipset_updater::{
    get_ipset_info, update_ipset, update_ipset_from_sources, validate_ipset,
    IpsetUpdateResult, IpsetValidationResult,
};
use crate::core::paths::get_hostlists_dir;

// ============================================================================
// Constants
// ============================================================================

/// Config file for ipset manager settings
const IPSET_MANAGER_CONFIG: &str = ".ipset_manager.json";

/// Default ipset filename
const DEFAULT_IPSET_FILE: &str = "ipset-all.txt";

// ============================================================================
// Types
// ============================================================================

/// Ipset operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IpsetMode {
    /// Allow all traffic - ipset filtering disabled
    #[default]
    Any,
    /// Block all traffic that would match ipset rules
    None,
    /// Use loaded ipset for filtering
    Loaded,
}

impl std::fmt::Display for IpsetMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpsetMode::Any => write!(f, "any"),
            IpsetMode::None => write!(f, "none"),
            IpsetMode::Loaded => write!(f, "loaded"),
        }
    }
}

impl std::str::FromStr for IpsetMode {
    type Err = IsolateError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "any" => Ok(IpsetMode::Any),
            "none" => Ok(IpsetMode::None),
            "loaded" => Ok(IpsetMode::Loaded),
            _ => Err(IsolateError::Validation(format!(
                "Invalid ipset mode: {}. Expected: any, none, or loaded",
                s
            ))),
        }
    }
}

/// Ipset manager configuration (persisted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsetManagerConfig {
    /// Current mode
    pub mode: IpsetMode,
    /// Auto-update enabled
    pub auto_update: bool,
    /// Custom ipset file path (if not using default)
    pub custom_path: Option<String>,
    /// Last mode change timestamp
    pub last_mode_change: Option<String>,
}

impl Default for IpsetManagerConfig {
    fn default() -> Self {
        Self {
            mode: IpsetMode::Any,
            auto_update: false,
            custom_path: None,
            last_mode_change: None,
        }
    }
}

/// Statistics about the current ipset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsetStats {
    /// Current mode
    pub mode: IpsetMode,
    /// Total IP count
    pub ip_count: usize,
    /// IPv4 count
    pub ipv4_count: usize,
    /// IPv6 count
    pub ipv6_count: usize,
    /// CIDR count
    pub cidr_count: usize,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Last update timestamp (ISO 8601)
    pub last_updated: Option<String>,
    /// Source URL used for last update
    pub source_url: Option<String>,
    /// Whether ipset file exists
    pub file_exists: bool,
    /// Auto-update enabled
    pub auto_update: bool,
}

// ============================================================================
// IpsetManager
// ============================================================================

/// IP Set Manager
///
/// Provides unified interface for managing ipset modes and operations.
pub struct IpsetManager {
    /// Configuration
    config: Arc<RwLock<IpsetManagerConfig>>,
    /// Hostlists directory
    hostlists_dir: PathBuf,
}

impl IpsetManager {
    /// Create a new IpsetManager
    pub async fn new() -> Result<Self> {
        let hostlists_dir = get_hostlists_dir();

        // Ensure directory exists
        if !hostlists_dir.exists() {
            fs::create_dir_all(&hostlists_dir).await?;
        }

        // Load config
        let config = Self::load_config(&hostlists_dir).await.unwrap_or_default();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            hostlists_dir,
        })
    }

    /// Load configuration from disk
    async fn load_config(dir: &PathBuf) -> Result<IpsetManagerConfig> {
        let config_path = dir.join(IPSET_MANAGER_CONFIG);

        if !config_path.exists() {
            return Ok(IpsetManagerConfig::default());
        }

        let content = fs::read_to_string(&config_path).await?;
        let config: IpsetManagerConfig = serde_json::from_str(&content)
            .map_err(|e| IsolateError::Config(format!("Failed to parse ipset manager config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to disk
    async fn save_config(&self) -> Result<()> {
        let config_path = self.hostlists_dir.join(IPSET_MANAGER_CONFIG);
        let config = self.config.read().clone();
        
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| IsolateError::Config(format!("Failed to serialize ipset manager config: {}", e)))?;

        fs::write(&config_path, content).await?;
        Ok(())
    }

    /// Get current ipset file path
    fn get_ipset_path(&self) -> PathBuf {
        let config = self.config.read();
        
        if let Some(custom_path) = &config.custom_path {
            PathBuf::from(custom_path)
        } else {
            self.hostlists_dir.join(DEFAULT_IPSET_FILE)
        }
    }

    /// Load ipset from a file
    ///
    /// Validates the file content and updates the manager state.
    pub async fn load_ipset(&self, path: &str) -> Result<IpsetValidationResult> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(IsolateError::Config(format!(
                "Ipset file not found: {}",
                path
            )));
        }

        let content = fs::read_to_string(&file_path).await?;
        let validation = validate_ipset(&content);

        if !validation.is_valid {
            return Err(IsolateError::Validation(format!(
                "Invalid ipset file: {} valid entries, {} invalid entries",
                validation.valid_entries,
                validation.invalid_entries.len()
            )));
        }

        // Copy to default location if not already there
        let default_path = self.hostlists_dir.join(DEFAULT_IPSET_FILE);
        if file_path != default_path {
            fs::copy(&file_path, &default_path).await?;
            info!(
                source = %path,
                dest = %default_path.display(),
                "Copied ipset to default location"
            );
        }

        // Update config
        {
            let mut config = self.config.write();
            config.custom_path = Some(path.to_string());
            config.last_mode_change = Some(chrono::Utc::now().to_rfc3339());
        }
        self.save_config().await?;

        info!(
            path = %path,
            ip_count = validation.valid_entries,
            "Loaded ipset from file"
        );

        Ok(validation)
    }

    /// Update ipset from a URL
    pub async fn update_from_url(&self, url: &str) -> Result<IpsetUpdateResult> {
        info!(url = %url, "Updating ipset from URL");
        
        let result = update_ipset(url).await?;

        // Update last change timestamp
        {
            let mut config = self.config.write();
            config.last_mode_change = Some(chrono::Utc::now().to_rfc3339());
        }
        self.save_config().await?;

        Ok(result)
    }

    /// Update ipset from configured sources
    pub async fn update_from_sources(&self) -> Result<IpsetUpdateResult> {
        info!("Updating ipset from configured sources");
        
        let result = update_ipset_from_sources().await?;

        // Update last change timestamp
        {
            let mut config = self.config.write();
            config.last_mode_change = Some(chrono::Utc::now().to_rfc3339());
        }
        self.save_config().await?;

        Ok(result)
    }

    /// Get current mode
    pub fn get_mode(&self) -> IpsetMode {
        self.config.read().mode
    }

    /// Set ipset mode
    pub async fn set_mode(&self, mode: IpsetMode) -> Result<()> {
        // If switching to Loaded mode, verify ipset file exists
        if mode == IpsetMode::Loaded {
            let ipset_path = self.get_ipset_path();
            if !ipset_path.exists() {
                return Err(IsolateError::Config(
                    "Cannot switch to Loaded mode: ipset file not found. Please update ipset first.".to_string()
                ));
            }
        }

        {
            let mut config = self.config.write();
            let old_mode = config.mode;
            config.mode = mode;
            config.last_mode_change = Some(chrono::Utc::now().to_rfc3339());
            
            info!(
                old_mode = %old_mode,
                new_mode = %mode,
                "Ipset mode changed"
            );
        }

        self.save_config().await?;
        Ok(())
    }

    /// Get ipset statistics
    pub async fn get_stats(&self) -> Result<IpsetStats> {
        let config = self.config.read().clone();
        let ipset_path = self.get_ipset_path();
        let file_exists = ipset_path.exists();

        // Get detailed info from ipset_updater
        let info = get_ipset_info().await.ok();

        let (ip_count, ipv4_count, ipv6_count, cidr_count, file_size, last_updated, source_url) = 
            if let Some(info) = info {
                (
                    info.ip_count.unwrap_or(0),
                    info.ipv4_count.unwrap_or(0),
                    info.ipv6_count.unwrap_or(0),
                    info.cidr_count.unwrap_or(0),
                    info.size,
                    info.last_updated,
                    info.source_url,
                )
            } else if file_exists {
                // Fallback: read file directly
                match fs::read_to_string(&ipset_path).await {
                    Ok(content) => {
                        let validation = validate_ipset(&content);
                        let metadata = fs::metadata(&ipset_path).await.ok();
                        (
                            validation.valid_entries,
                            validation.ipv4_count,
                            validation.ipv6_count,
                            validation.cidr_count,
                            metadata.map(|m| m.len()),
                            None,
                            None,
                        )
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to read ipset file");
                        (0, 0, 0, 0, None, None, None)
                    }
                }
            } else {
                (0, 0, 0, 0, None, None, None)
            };

        Ok(IpsetStats {
            mode: config.mode,
            ip_count,
            ipv4_count,
            ipv6_count,
            cidr_count,
            file_size,
            last_updated,
            source_url,
            file_exists,
            auto_update: config.auto_update,
        })
    }

    /// Set auto-update enabled/disabled
    pub async fn set_auto_update(&self, enabled: bool) -> Result<()> {
        {
            let mut config = self.config.write();
            config.auto_update = enabled;
        }
        self.save_config().await?;

        info!(enabled = enabled, "Ipset auto-update setting changed");
        Ok(())
    }

    /// Check if ipset should be applied based on current mode
    pub fn should_apply_ipset(&self) -> bool {
        matches!(self.get_mode(), IpsetMode::Loaded)
    }

    /// Get ipset file path if mode is Loaded and file exists
    pub fn get_active_ipset_path(&self) -> Option<PathBuf> {
        if self.get_mode() == IpsetMode::Loaded {
            let path = self.get_ipset_path();
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

// ============================================================================
// Global Instance
// ============================================================================

use once_cell::sync::OnceCell;

static IPSET_MANAGER: OnceCell<Arc<tokio::sync::RwLock<IpsetManager>>> = OnceCell::new();

/// Get or initialize the global IpsetManager instance
pub async fn get_ipset_manager() -> Result<Arc<tokio::sync::RwLock<IpsetManager>>> {
    if let Some(manager) = IPSET_MANAGER.get() {
        return Ok(manager.clone());
    }

    let manager = IpsetManager::new().await?;
    let manager = Arc::new(tokio::sync::RwLock::new(manager));

    // Try to set, but if another thread beat us, use their instance
    match IPSET_MANAGER.set(manager.clone()) {
        Ok(()) => Ok(manager),
        Err(_) => Ok(IPSET_MANAGER.get().unwrap().clone()),
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Get current ipset mode
pub async fn get_current_mode() -> Result<IpsetMode> {
    let manager = get_ipset_manager().await?;
    let manager = manager.read().await;
    Ok(manager.get_mode())
}

/// Set ipset mode
pub async fn set_current_mode(mode: IpsetMode) -> Result<()> {
    let manager = get_ipset_manager().await?;
    let manager = manager.read().await;
    manager.set_mode(mode).await
}

/// Get ipset statistics
pub async fn get_ipset_stats() -> Result<IpsetStats> {
    let manager = get_ipset_manager().await?;
    let manager = manager.read().await;
    manager.get_stats().await
}

/// Load ipset from file
pub async fn load_ipset_from_file(path: &str) -> Result<IpsetValidationResult> {
    let manager = get_ipset_manager().await?;
    let manager = manager.read().await;
    manager.load_ipset(path).await
}

/// Update ipset from URL
pub async fn update_ipset_from_url(url: &str) -> Result<IpsetUpdateResult> {
    let manager = get_ipset_manager().await?;
    let manager = manager.read().await;
    manager.update_from_url(url).await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipset_mode_display() {
        assert_eq!(IpsetMode::Any.to_string(), "any");
        assert_eq!(IpsetMode::None.to_string(), "none");
        assert_eq!(IpsetMode::Loaded.to_string(), "loaded");
    }

    #[test]
    fn test_ipset_mode_from_str() {
        assert_eq!("any".parse::<IpsetMode>().unwrap(), IpsetMode::Any);
        assert_eq!("none".parse::<IpsetMode>().unwrap(), IpsetMode::None);
        assert_eq!("loaded".parse::<IpsetMode>().unwrap(), IpsetMode::Loaded);
        assert_eq!("ANY".parse::<IpsetMode>().unwrap(), IpsetMode::Any);
        assert_eq!("LOADED".parse::<IpsetMode>().unwrap(), IpsetMode::Loaded);
    }

    #[test]
    fn test_ipset_mode_from_str_invalid() {
        assert!("invalid".parse::<IpsetMode>().is_err());
        assert!("".parse::<IpsetMode>().is_err());
    }

    #[test]
    fn test_ipset_manager_config_default() {
        let config = IpsetManagerConfig::default();
        assert_eq!(config.mode, IpsetMode::Any);
        assert!(!config.auto_update);
        assert!(config.custom_path.is_none());
        assert!(config.last_mode_change.is_none());
    }

    #[test]
    fn test_ipset_mode_serialization() {
        let mode = IpsetMode::Loaded;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"loaded\"");

        let deserialized: IpsetMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, IpsetMode::Loaded);
    }

    #[test]
    fn test_ipset_stats_serialization() {
        let stats = IpsetStats {
            mode: IpsetMode::Loaded,
            ip_count: 1000,
            ipv4_count: 800,
            ipv6_count: 200,
            cidr_count: 150,
            file_size: Some(50000),
            last_updated: Some("2024-01-01T00:00:00Z".to_string()),
            source_url: Some("https://example.com/ipset.txt".to_string()),
            file_exists: true,
            auto_update: true,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"mode\":\"loaded\""));
        assert!(json.contains("\"ip_count\":1000"));
    }
}

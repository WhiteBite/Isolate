//! Unified Strategy Loader for Isolate
//!
//! Provides a single entry point for loading strategies from YAML files.
//! Supports two formats:
//! - **Legacy format** (Strategy from models.rs): High-level with LaunchTemplate
//! - **Zapret format** (ZapretStrategy from strategy_loader.rs): Low-level with profiles
//!
//! The loader automatically detects the format and provides unified access.
//!
//! NOTE: Some methods are prepared for future UI integration.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::core::unified_strategy_loader::UnifiedStrategyLoader;
//!
//! let loader = UnifiedStrategyLoader::new("configs/strategies");
//!
//! // Load all strategies (both formats)
//! let all = loader.load_all().await?;
//!
//! // Load only high-level strategies (for UI)
//! let ui_strategies = loader.load_ui_strategies().await?;
//!
//! // Load only zapret strategies (for winws args generation)
//! let zapret_strategies = loader.load_zapret_strategies().await?;
//! ```

// Public API for unified strategy loading
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::core::models::Strategy;
use crate::core::strategy_loader::{ZapretStrategy, StrategyCategory, StrategyFile, StrategyLoader};

// ============================================================================
// Unified Strategy Types
// ============================================================================

/// Unified strategy that can hold either format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnifiedStrategy {
    /// High-level strategy with LaunchTemplate (for UI and direct execution)
    HighLevel(Strategy),
    /// Low-level zapret strategy with profiles (for winws args generation)
    Zapret(ZapretStrategy),
}

impl UnifiedStrategy {
    /// Get the strategy ID
    pub fn id(&self) -> &str {
        match self {
            UnifiedStrategy::HighLevel(s) => &s.id,
            UnifiedStrategy::Zapret(s) => &s.id,
        }
    }

    /// Get the strategy name
    pub fn name(&self) -> &str {
        match self {
            UnifiedStrategy::HighLevel(s) => &s.name,
            UnifiedStrategy::Zapret(s) => &s.name,
        }
    }

    /// Get the strategy description
    pub fn description(&self) -> &str {
        match self {
            UnifiedStrategy::HighLevel(s) => &s.description,
            UnifiedStrategy::Zapret(s) => &s.description,
        }
    }

    /// Check if this is a high-level strategy
    pub fn is_high_level(&self) -> bool {
        matches!(self, UnifiedStrategy::HighLevel(_))
    }

    /// Check if this is a zapret strategy
    pub fn is_zapret(&self) -> bool {
        matches!(self, UnifiedStrategy::Zapret(_))
    }

    /// Try to get as high-level strategy
    pub fn as_high_level(&self) -> Option<&Strategy> {
        match self {
            UnifiedStrategy::HighLevel(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as zapret strategy
    pub fn as_zapret(&self) -> Option<&ZapretStrategy> {
        match self {
            UnifiedStrategy::Zapret(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to owned high-level strategy if possible
    pub fn into_high_level(self) -> Option<Strategy> {
        match self {
            UnifiedStrategy::HighLevel(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to owned zapret strategy if possible
    pub fn into_zapret(self) -> Option<ZapretStrategy> {
        match self {
            UnifiedStrategy::Zapret(s) => Some(s),
            _ => None,
        }
    }
}

// ============================================================================
// Format Detection
// ============================================================================

/// Detected file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyFormat {
    /// High-level format with LaunchTemplate (single strategy per file)
    HighLevel,
    /// Zapret format with profiles (multiple strategies per file)
    Zapret,
    /// Unknown format
    Unknown,
}

/// Detect the format of a YAML strategy file
fn detect_format(content: &str) -> StrategyFormat {
    // Check for zapret format markers (has "version" and "strategies" array)
    if content.contains("version:") && content.contains("strategies:") {
        return StrategyFormat::Zapret;
    }

    // Check for high-level format markers (has templates)
    if content.contains("global_template:") || content.contains("socks_template:") {
        return StrategyFormat::HighLevel;
    }

    // Check for profiles (zapret format)
    if content.contains("profiles:") && content.contains("desync:") {
        return StrategyFormat::Zapret;
    }

    // Check for mode_capabilities (high-level format)
    if content.contains("mode_capabilities:") {
        return StrategyFormat::HighLevel;
    }

    StrategyFormat::Unknown
}

// ============================================================================
// Unified Strategy Loader
// ============================================================================

/// Unified loader for all strategy formats
pub struct UnifiedStrategyLoader {
    /// Directory containing strategy files
    strategies_dir: PathBuf,
    /// Internal zapret loader
    zapret_loader: StrategyLoader,
}

impl UnifiedStrategyLoader {
    /// Create a new unified strategy loader
    ///
    /// # Arguments
    /// * `strategies_dir` - Path to directory containing strategy YAML files
    pub fn new(strategies_dir: impl AsRef<Path>) -> Self {
        let dir = strategies_dir.as_ref().to_path_buf();
        Self {
            strategies_dir: dir.clone(),
            zapret_loader: StrategyLoader::new(dir),
        }
    }

    /// Load all strategies from the directory (both formats)
    ///
    /// # Returns
    /// * `Ok(Vec<UnifiedStrategy>)` - All loaded strategies
    /// * `Err` - Failed to read directory or parse files
    pub async fn load_all(&self) -> Result<Vec<UnifiedStrategy>> {
        let mut all_strategies = Vec::new();

        if !tokio::fs::try_exists(&self.strategies_dir).await.unwrap_or(false) {
            warn!(
                "Strategies directory does not exist: {}",
                self.strategies_dir.display()
            );
            return Ok(all_strategies);
        }

        let mut entries = tokio::fs::read_dir(&self.strategies_dir)
            .await
            .with_context(|| {
                format!(
                    "Failed to read strategies directory: {}",
                    self.strategies_dir.display()
                )
            })?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process .yaml and .yml files
            let is_file = tokio::fs::metadata(&path)
                .await
                .map(|m| m.is_file())
                .unwrap_or(false);

            let is_yaml = path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml");

            if is_file && is_yaml {
                match self.load_file(&path).await {
                    Ok(strategies) => {
                        info!(
                            "Loaded {} strategies from {}",
                            strategies.len(),
                            path.display()
                        );
                        all_strategies.extend(strategies);
                    }
                    Err(e) => {
                        warn!("Failed to load strategy file {}: {}", path.display(), e);
                    }
                }
            }
        }

        info!("Loaded {} total strategies (unified)", all_strategies.len());
        Ok(all_strategies)
    }

    /// Load strategies from a single file
    ///
    /// Automatically detects the format and parses accordingly.
    pub async fn load_file(&self, path: impl AsRef<Path>) -> Result<Vec<UnifiedStrategy>> {
        let path = path.as_ref();
        debug!("Loading strategy file: {}", path.display());

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read strategy file: {}", path.display()))?;

        let format = detect_format(&content);
        debug!("Detected format {:?} for {}", format, path.display());

        match format {
            StrategyFormat::HighLevel => {
                let strategy: Strategy = serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse high-level strategy: {}", path.display()))?;
                Ok(vec![UnifiedStrategy::HighLevel(strategy)])
            }
            StrategyFormat::Zapret => {
                let strategy_file: StrategyFile = serde_yaml::from_str(&content)
                    .with_context(|| format!("Failed to parse zapret strategy file: {}", path.display()))?;
                Ok(strategy_file
                    .strategies
                    .into_iter()
                    .map(UnifiedStrategy::Zapret)
                    .collect())
            }
            StrategyFormat::Unknown => {
                // Try high-level first, then zapret
                if let Ok(strategy) = serde_yaml::from_str::<Strategy>(&content) {
                    return Ok(vec![UnifiedStrategy::HighLevel(strategy)]);
                }
                if let Ok(strategy_file) = serde_yaml::from_str::<StrategyFile>(&content) {
                    return Ok(strategy_file
                        .strategies
                        .into_iter()
                        .map(UnifiedStrategy::Zapret)
                        .collect());
                }
                Err(anyhow::anyhow!(
                    "Could not parse strategy file as any known format: {}",
                    path.display()
                ))
            }
        }
    }

    /// Load only high-level strategies (for UI)
    ///
    /// Returns strategies that have LaunchTemplate and can be directly executed.
    pub async fn load_ui_strategies(&self) -> Result<Vec<Strategy>> {
        let all = self.load_all().await?;
        Ok(all
            .into_iter()
            .filter_map(|s| s.into_high_level())
            .collect())
    }

    /// Load only zapret strategies (for winws args generation)
    ///
    /// Returns strategies with profiles that can be converted to winws arguments.
    pub async fn load_zapret_strategies(&self) -> Result<Vec<ZapretStrategy>> {
        // Use the dedicated zapret loader for efficiency
        self.zapret_loader.load_all().await
    }

    /// Get the underlying zapret loader for advanced operations
    pub fn zapret_loader(&self) -> &StrategyLoader {
        &self.zapret_loader
    }

    /// Find a strategy by ID (searches both formats)
    pub async fn find_by_id(&self, id: &str) -> Result<Option<UnifiedStrategy>> {
        let all = self.load_all().await?;
        Ok(all.into_iter().find(|s| s.id() == id))
    }

    /// Get strategies grouped by format
    pub async fn load_grouped(&self) -> Result<GroupedStrategies> {
        let all = self.load_all().await?;
        
        let mut high_level = Vec::new();
        let mut zapret = Vec::new();

        for strategy in all {
            match strategy {
                UnifiedStrategy::HighLevel(s) => high_level.push(s),
                UnifiedStrategy::Zapret(s) => zapret.push(s),
            }
        }

        Ok(GroupedStrategies { high_level, zapret })
    }
}

/// Strategies grouped by format
#[derive(Debug, Clone)]
pub struct GroupedStrategies {
    /// High-level strategies with LaunchTemplate
    pub high_level: Vec<Strategy>,
    /// Zapret strategies with profiles
    pub zapret: Vec<ZapretStrategy>,
}

impl GroupedStrategies {
    /// Total number of strategies
    pub fn total(&self) -> usize {
        self.high_level.len() + self.zapret.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.high_level.is_empty() && self.zapret.is_empty()
    }
}

// ============================================================================
// Conversion: ZapretStrategy → Strategy
// ============================================================================

impl ZapretStrategy {
    /// Convert ZapretStrategy to high-level Strategy format
    ///
    /// This allows zapret strategies to be used with the same API as high-level strategies.
    /// The conversion generates a LaunchTemplate from the profiles.
    ///
    /// # Arguments
    /// * `hostlists_dir` - Directory containing hostlist files
    /// * `blobs_dir` - Directory containing binary blob files
    ///
    /// # Returns
    /// * `Strategy` - High-level strategy with generated templates
    pub fn to_strategy(&self, hostlists_dir: &Path, blobs_dir: &Path) -> Strategy {
        use crate::core::models::{
            LaunchTemplate, ModeCapabilities, Strategy, StrategyEngine, StrategyFamily,
            StrategyRequirements,
        };
        use std::collections::HashMap;

        // Generate winws args from profiles
        let loader = StrategyLoader::new(PathBuf::new());
        let args = loader
            .to_winws_args(self, hostlists_dir, blobs_dir)
            .unwrap_or_default();

        // Map category to services
        let services = match self.category {
            StrategyCategory::YouTube => vec!["youtube".to_string()],
            StrategyCategory::Discord => vec!["discord".to_string()],
            StrategyCategory::Telegram => vec!["telegram".to_string()],
            StrategyCategory::Games => vec!["games".to_string()],
            StrategyCategory::Warp => vec!["warp".to_string()],
            StrategyCategory::General | StrategyCategory::Custom => vec![],
        };

        // Map family string to StrategyFamily enum
        let family = match self.family.to_lowercase().as_str() {
            "zapret" | "winws" => StrategyFamily::SniFrag,
            "vless" => StrategyFamily::Vless,
            "dns" | "dns_bypass" => StrategyFamily::DnsBypass,
            "tls" | "tls_frag" => StrategyFamily::TlsFrag,
            _ => StrategyFamily::Hybrid,
        };

        // Create global template (zapret strategies are typically global)
        let global_template = Some(LaunchTemplate {
            binary: "winws.exe".to_string(),
            args,
            env: HashMap::new(),
            log_file: None,
            requires_admin: true,
        });

        Strategy {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            family,
            engine: StrategyEngine::Zapret,
            mode_capabilities: ModeCapabilities {
                supports_socks: false,
                supports_global: true,
            },
            socks_template: None,
            global_template,
            requirements: StrategyRequirements {
                min_rights: "admin".to_string(),
                os: vec!["windows".to_string()],
                binaries: vec!["winws.exe".to_string()],
            },
            weight_hint: 0,
            services,
        }
    }
}

impl UnifiedStrategy {
    /// Convert to high-level Strategy format
    ///
    /// For HighLevel strategies, returns a clone.
    /// For Zapret strategies, converts using to_strategy().
    ///
    /// # Arguments
    /// * `hostlists_dir` - Directory containing hostlist files
    /// * `blobs_dir` - Directory containing binary blob files
    pub fn to_high_level_strategy(&self, hostlists_dir: &Path, blobs_dir: &Path) -> Strategy {
        match self {
            UnifiedStrategy::HighLevel(s) => s.clone(),
            UnifiedStrategy::Zapret(s) => s.to_strategy(hostlists_dir, blobs_dir),
        }
    }
}

impl UnifiedStrategyLoader {
    /// Load all strategies and convert to high-level format
    ///
    /// This provides a unified API where all strategies (both high-level and zapret)
    /// are returned as Strategy objects that can be used with StrategyEngine.
    ///
    /// # Arguments
    /// * `hostlists_dir` - Directory containing hostlist files
    /// * `blobs_dir` - Directory containing binary blob files
    ///
    /// # Returns
    /// * `Ok(Vec<Strategy>)` - All strategies in high-level format
    pub async fn load_all_as_strategies(
        &self,
        hostlists_dir: &Path,
        blobs_dir: &Path,
    ) -> Result<Vec<Strategy>> {
        let all = self.load_all().await?;
        Ok(all
            .into_iter()
            .map(|s| s.to_high_level_strategy(hostlists_dir, blobs_dir))
            .collect())
    }

    /// Find a strategy by ID and convert to high-level format
    ///
    /// # Arguments
    /// * `id` - Strategy ID to find
    /// * `hostlists_dir` - Directory containing hostlist files
    /// * `blobs_dir` - Directory containing binary blob files
    pub async fn find_strategy_by_id(
        &self,
        id: &str,
        hostlists_dir: &Path,
        blobs_dir: &Path,
    ) -> Result<Option<Strategy>> {
        let unified = self.find_by_id(id).await?;
        Ok(unified.map(|s| s.to_high_level_strategy(hostlists_dir, blobs_dir)))
    }
}

// ============================================================================
// DTO for Frontend
// ============================================================================

/// Unified strategy DTO for frontend
///
/// This provides a consistent format for the frontend regardless of
/// whether the strategy is high-level or zapret format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub family: String,
    pub engine: String,
    pub category: Option<String>,
    pub supports_socks: bool,
    pub supports_global: bool,
    pub author: Option<String>,
    pub label: Option<String>,
    /// Source format: "high_level" or "zapret"
    pub source_format: String,
}

impl From<&UnifiedStrategy> for StrategyDto {
    fn from(strategy: &UnifiedStrategy) -> Self {
        match strategy {
            UnifiedStrategy::HighLevel(s) => StrategyDto {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                family: format!("{:?}", s.family).to_lowercase(),
                engine: format!("{:?}", s.engine).to_lowercase(),
                category: None,
                supports_socks: s.mode_capabilities.supports_socks,
                supports_global: s.mode_capabilities.supports_global,
                author: None,
                label: None,
                source_format: "high_level".to_string(),
            },
            UnifiedStrategy::Zapret(s) => StrategyDto {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                family: s.family.clone(),
                engine: "zapret".to_string(),
                category: Some(format!("{}", s.category)),
                supports_socks: false,
                supports_global: true,
                author: s.author.clone(),
                label: s.label.clone(),
                source_format: "zapret".to_string(),
            },
        }
    }
}

impl UnifiedStrategyLoader {
    /// Load all strategies as DTOs for frontend
    ///
    /// Returns a unified format suitable for UI display.
    pub async fn load_all_as_dto(&self) -> Result<Vec<StrategyDto>> {
        let all = self.load_all().await?;
        Ok(all.iter().map(StrategyDto::from).collect())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_zapret() {
        let content = r#"
version: "1.0"
strategies:
  - id: test
    name: Test
    profiles:
      - filter: "tcp=443"
        desync: fake
"#;
        assert_eq!(detect_format(content), StrategyFormat::Zapret);
    }

    #[test]
    fn test_detect_format_high_level() {
        let content = r#"
id: test
name: Test
mode_capabilities:
  supports_socks: true
  supports_global: true
global_template:
  binary: winws.exe
  args: []
"#;
        assert_eq!(detect_format(content), StrategyFormat::HighLevel);
    }

    #[test]
    fn test_detect_format_unknown() {
        let content = r#"
some_random: value
another: field
"#;
        assert_eq!(detect_format(content), StrategyFormat::Unknown);
    }

    #[test]
    fn test_unified_strategy_id() {
        use crate::core::models::{
            LaunchTemplate, ModeCapabilities, Strategy, StrategyEngine, StrategyFamily,
            StrategyRequirements,
        };
        use crate::core::strategy_loader::{ZapretStrategy, StrategyCategory, StrategyPorts};
        use std::collections::HashMap;

        let high_level = UnifiedStrategy::HighLevel(Strategy {
            id: "high-level-id".to_string(),
            name: "High Level".to_string(),
            description: "Test".to_string(),
            family: StrategyFamily::SniFrag,
            engine: StrategyEngine::Zapret,
            mode_capabilities: ModeCapabilities::default(),
            socks_template: None,
            global_template: Some(LaunchTemplate {
                binary: "test".to_string(),
                args: vec![],
                env: HashMap::new(),
                log_file: None,
                requires_admin: false,
            }),
            requirements: StrategyRequirements::default(),
            weight_hint: 0,
            services: vec![],
        });

        let zapret = UnifiedStrategy::Zapret(ZapretStrategy {
            id: "zapret-id".to_string(),
            name: "Zapret".to_string(),
            description: "Test".to_string(),
            category: StrategyCategory::General,
            family: "zapret".to_string(),
            author: None,
            label: None,
            ports: StrategyPorts::default(),
            profiles: vec![],
        });

        assert_eq!(high_level.id(), "high-level-id");
        assert_eq!(zapret.id(), "zapret-id");
        assert!(high_level.is_high_level());
        assert!(zapret.is_zapret());
    }
}

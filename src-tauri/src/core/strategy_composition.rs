//! Strategy Composition Module for Isolate
//!
//! Enables combining different strategies for different services.
//! For example: YouTube via Zapret, Discord via VLESS.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::core::strategy_composition::{CompositionManager, CompositionRule};
//!
//! let manager = CompositionManager::new(strategies_dir);
//! manager.add_rule(CompositionRule {
//!     service_pattern: "youtube".to_string(),
//!     strategy_id: "zapret-youtube".to_string(),
//!     priority: 100,
//!     enabled: true,
//! });
//! manager.add_rule(CompositionRule {
//!     service_pattern: "discord".to_string(),
//!     strategy_id: "vless-discord".to_string(),
//!     priority: 90,
//!     enabled: true,
//! });
//!
//! // Get strategy for a specific service
//! let strategy = manager.get_strategy_for_service("youtube.com")?;
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::errors::IsolateError;
use crate::core::models::strategy::Strategy;

// ============================================================================
// Pattern Validation
// ============================================================================

/// Validates a service pattern to prevent overly broad matching.
/// 
/// # Security
/// Overly broad patterns like `*` or `**` could match unintended services,
/// potentially routing traffic through wrong strategies.
/// 
/// # Rules
/// - Patterns cannot be just wildcards (`*`, `**`, `*.*`, `?`)
/// - Patterns must contain at least one specific (non-wildcard) character
/// - Empty patterns are not allowed
/// 
/// # Examples
/// ```rust,ignore
/// validate_pattern("*.youtube.com")?;  // OK - specific domain
/// validate_pattern("discord")?;         // OK - exact match
/// validate_pattern("*")?;               // Error - too broad
/// validate_pattern("**")?;              // Error - too broad
/// ```
pub fn validate_pattern(pattern: &str) -> std::result::Result<(), IsolateError> {
    let pattern = pattern.trim();
    
    // Empty pattern is not allowed
    if pattern.is_empty() {
        return Err(IsolateError::Validation(
            "Pattern cannot be empty".to_string()
        ));
    }
    
    // Explicitly forbidden overly broad patterns
    const FORBIDDEN_PATTERNS: &[&str] = &["*", "**", "*.*", "?", "??", "???", "*?", "?*"];
    if FORBIDDEN_PATTERNS.contains(&pattern) {
        return Err(IsolateError::Validation(format!(
            "Pattern '{}' is too broad. Use more specific patterns like '*.youtube.com' or 'discord'",
            pattern
        )));
    }
    
    // Pattern must contain at least one specific (non-wildcard) character
    let has_specific_char = pattern.chars().any(|c| c != '*' && c != '?' && c != '.');
    if !has_specific_char {
        return Err(IsolateError::Validation(format!(
            "Pattern '{}' must contain at least one specific character (not just wildcards and dots)",
            pattern
        )));
    }
    
    // Pattern should not be just dots and wildcards
    let non_wildcard_count = pattern.chars().filter(|c| *c != '*' && *c != '?' && *c != '.').count();
    if non_wildcard_count < 2 {
        return Err(IsolateError::Validation(format!(
            "Pattern '{}' is too broad. Include at least 2 specific characters",
            pattern
        )));
    }
    
    Ok(())
}

// ============================================================================
// Data Structures
// ============================================================================

/// Rule for mapping a service pattern to a strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompositionRule {
    /// Service pattern (e.g., "youtube", "discord", "*.google.com")
    /// Supports glob patterns with * wildcard
    pub service_pattern: String,
    
    /// Strategy ID to use for matching services
    pub strategy_id: String,
    
    /// Priority (higher = more important, checked first)
    /// Default: 50
    #[serde(default = "default_priority")]
    pub priority: u32,
    
    /// Whether this rule is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Optional description for the rule
    #[serde(default)]
    pub description: Option<String>,
}

fn default_priority() -> u32 {
    50
}

fn default_enabled() -> bool {
    true
}

impl CompositionRule {
    /// Create a new composition rule
    pub fn new(service_pattern: impl Into<String>, strategy_id: impl Into<String>) -> Self {
        Self {
            service_pattern: service_pattern.into(),
            strategy_id: strategy_id.into(),
            priority: default_priority(),
            enabled: true,
            description: None,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if service matches this rule's pattern
    pub fn matches(&self, service_id: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let pattern = self.service_pattern.to_lowercase();
        let service = service_id.to_lowercase();
        
        // Exact match
        if pattern == service {
            return true;
        }
        
        // Glob pattern matching with *
        if pattern.contains('*') {
            return Self::glob_match(&pattern, &service);
        }
        
        // Substring match (service contains pattern)
        service.contains(&pattern)
    }

    /// Simple glob pattern matching
    fn glob_match(pattern: &str, text: &str) -> bool {
        let parts: Vec<&str> = pattern.split('*').collect();
        
        if parts.is_empty() {
            return true;
        }
        
        let mut pos = 0;
        
        // Check first part (must be at start if not empty)
        if !parts[0].is_empty() {
            if !text.starts_with(parts[0]) {
                return false;
            }
            pos = parts[0].len();
        }
        
        // Check middle parts
        for part in &parts[1..parts.len().saturating_sub(1)] {
            if part.is_empty() {
                continue;
            }
            if let Some(found) = text[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }
        
        // Check last part (must be at end if not empty)
        if parts.len() > 1 {
            let last = parts[parts.len() - 1];
            if !last.is_empty() && !text.ends_with(last) {
                return false;
            }
        }
        
        true
    }
}

/// Composite strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompositeStrategy {
    /// Unique identifier for this composition
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Description
    #[serde(default)]
    pub description: String,
    
    /// Mapping of service_id to strategy_id
    #[serde(default)]
    pub service_mappings: HashMap<String, String>,
    
    /// Composition rules (evaluated in priority order)
    #[serde(default)]
    pub rules: Vec<CompositionRule>,
    
    /// Default strategy if no rule matches
    #[serde(default)]
    pub default_strategy: Option<String>,
    
    /// Whether this composition is active
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl CompositeStrategy {
    /// Create a new composite strategy
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            service_mappings: HashMap::new(),
            rules: Vec::new(),
            default_strategy: None,
            enabled: true,
        }
    }

    /// Add a direct service mapping
    pub fn add_mapping(&mut self, service_id: impl Into<String>, strategy_id: impl Into<String>) {
        self.service_mappings.insert(service_id.into(), strategy_id.into());
    }

    /// Add a composition rule
    /// 
    /// Note: Pattern validation should be done before calling this method
    /// using `validate_pattern()` function.
    pub fn add_rule(&mut self, rule: CompositionRule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add a composition rule with pattern validation
    /// 
    /// Returns an error if the pattern is too broad (e.g., "*", "**")
    pub fn add_rule_validated(&mut self, rule: CompositionRule) -> std::result::Result<(), IsolateError> {
        validate_pattern(&rule.service_pattern)?;
        self.add_rule(rule);
        Ok(())
    }

    /// Get strategy ID for a service
    pub fn get_strategy_for_service(&self, service_id: &str) -> Option<&str> {
        // First check direct mappings
        if let Some(strategy_id) = self.service_mappings.get(service_id) {
            return Some(strategy_id);
        }
        
        // Then check rules (already sorted by priority)
        for rule in &self.rules {
            if rule.matches(service_id) {
                return Some(&rule.strategy_id);
            }
        }
        
        // Fall back to default
        self.default_strategy.as_deref()
    }

    /// Get all unique strategy IDs used in this composition
    pub fn get_used_strategies(&self) -> Vec<&str> {
        let mut strategies: Vec<&str> = Vec::new();
        
        // From mappings
        for strategy_id in self.service_mappings.values() {
            if !strategies.contains(&strategy_id.as_str()) {
                strategies.push(strategy_id);
            }
        }
        
        // From rules
        for rule in &self.rules {
            if rule.enabled && !strategies.contains(&rule.strategy_id.as_str()) {
                strategies.push(&rule.strategy_id);
            }
        }
        
        // Default
        if let Some(ref default) = self.default_strategy {
            if !strategies.contains(&default.as_str()) {
                strategies.push(default);
            }
        }
        
        strategies
    }
}

/// YAML configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionConfig {
    /// Config version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Active composition ID
    pub active_composition: Option<String>,
    
    /// All defined compositions
    #[serde(default)]
    pub compositions: Vec<CompositeStrategy>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Default for CompositionConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            active_composition: None,
            compositions: Vec::new(),
        }
    }
}

// ============================================================================
// Composition Manager
// ============================================================================

/// Manager for strategy compositions
pub struct CompositionManager {
    /// Configuration directory
    config_dir: PathBuf,
    
    /// Current configuration
    config: Arc<RwLock<CompositionConfig>>,
    
    /// Available strategies (for validation)
    available_strategies: Arc<RwLock<HashMap<String, Strategy>>>,
}

impl CompositionManager {
    /// Create a new composition manager
    pub fn new(config_dir: impl AsRef<Path>) -> Self {
        Self {
            config_dir: config_dir.as_ref().to_path_buf(),
            config: Arc::new(RwLock::new(CompositionConfig::default())),
            available_strategies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get config file path
    fn config_path(&self) -> PathBuf {
        self.config_dir.join("compositions.yaml")
    }

    /// Load configuration from YAML file
    pub async fn load(&self) -> Result<()> {
        let path = self.config_path();
        
        if !path.exists() {
            info!("Composition config not found, using defaults");
            return Ok(());
        }
        
        let content = tokio::fs::read_to_string(&path)
            .await
            .context("Failed to read composition config")?;
        
        let config: CompositionConfig = serde_yaml::from_str(&content)
            .context("Failed to parse composition config")?;
        
        let mut guard = self.config.write().await;
        *guard = config;
        
        info!("Loaded {} compositions", guard.compositions.len());
        Ok(())
    }

    /// Save configuration to YAML file
    pub async fn save(&self) -> Result<()> {
        let path = self.config_path();
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let config = self.config.read().await;
        let content = serde_yaml::to_string(&*config)
            .context("Failed to serialize composition config")?;
        
        tokio::fs::write(&path, content)
            .await
            .context("Failed to write composition config")?;
        
        info!("Saved composition config to {:?}", path);
        Ok(())
    }

    /// Set available strategies for validation
    pub async fn set_available_strategies(&self, strategies: Vec<Strategy>) {
        let mut guard = self.available_strategies.write().await;
        guard.clear();
        for strategy in strategies {
            guard.insert(strategy.id.clone(), strategy);
        }
    }

    /// Get all compositions
    pub async fn get_compositions(&self) -> Vec<CompositeStrategy> {
        self.config.read().await.compositions.clone()
    }

    /// Get active composition
    pub async fn get_active_composition(&self) -> Option<CompositeStrategy> {
        let config = self.config.read().await;
        
        if let Some(ref active_id) = config.active_composition {
            config.compositions.iter()
                .find(|c| &c.id == active_id)
                .cloned()
        } else {
            None
        }
    }

    /// Set active composition by ID
    pub async fn set_active_composition(&self, composition_id: Option<String>) -> Result<()> {
        let mut config = self.config.write().await;
        
        if let Some(ref id) = composition_id {
            // Validate composition exists
            if !config.compositions.iter().any(|c| &c.id == id) {
                anyhow::bail!("Composition '{}' not found", id);
            }
        }
        
        config.active_composition = composition_id;
        drop(config);
        
        self.save().await
    }

    /// Add or update a composition
    pub async fn upsert_composition(&self, composition: CompositeStrategy) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Validate strategies exist
        let available = self.available_strategies.read().await;
        for strategy_id in composition.get_used_strategies() {
            if !available.contains_key(strategy_id) {
                warn!("Strategy '{}' not found in available strategies", strategy_id);
            }
        }
        drop(available);
        
        // Update or insert
        if let Some(existing) = config.compositions.iter_mut().find(|c| c.id == composition.id) {
            *existing = composition;
            info!("Updated composition");
        } else {
            info!("Added new composition: {}", composition.id);
            config.compositions.push(composition);
        }
        
        drop(config);
        self.save().await
    }

    /// Remove a composition
    pub async fn remove_composition(&self, composition_id: &str) -> Result<()> {
        let mut config = self.config.write().await;
        
        let initial_len = config.compositions.len();
        config.compositions.retain(|c| c.id != composition_id);
        
        if config.compositions.len() == initial_len {
            anyhow::bail!("Composition '{}' not found", composition_id);
        }
        
        // Clear active if it was removed
        if config.active_composition.as_deref() == Some(composition_id) {
            config.active_composition = None;
        }
        
        drop(config);
        self.save().await
    }

    /// Get all composition rules from active composition
    pub async fn get_rules(&self) -> Vec<CompositionRule> {
        if let Some(composition) = self.get_active_composition().await {
            composition.rules
        } else {
            Vec::new()
        }
    }

    /// Set rules for active composition
    /// 
    /// Validates all patterns before applying. Returns error if any pattern is too broad.
    pub async fn set_rules(&self, rules: Vec<CompositionRule>) -> Result<()> {
        // Validate all patterns first
        for rule in &rules {
            validate_pattern(&rule.service_pattern)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        
        let mut config = self.config.write().await;
        
        let active_id = config.active_composition.clone()
            .ok_or_else(|| anyhow::anyhow!("No active composition"))?;
        
        if let Some(composition) = config.compositions.iter_mut().find(|c| c.id == active_id) {
            composition.rules = rules;
            // Sort by priority
            composition.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        } else {
            anyhow::bail!("Active composition '{}' not found", active_id);
        }
        
        drop(config);
        self.save().await
    }

    /// Get strategy for a specific service using active composition
    pub async fn get_strategy_for_service(&self, service_id: &str) -> Option<String> {
        let composition = self.get_active_composition().await?;
        composition.get_strategy_for_service(service_id).map(String::from)
    }

    /// Apply composition - returns mapping of service_id to strategy_id
    pub async fn apply_composition(&self, service_ids: &[String]) -> Result<HashMap<String, String>> {
        let composition = self.get_active_composition().await
            .ok_or_else(|| anyhow::anyhow!("No active composition"))?;
        
        let mut result = HashMap::new();
        
        for service_id in service_ids {
            if let Some(strategy_id) = composition.get_strategy_for_service(service_id) {
                result.insert(service_id.clone(), strategy_id.to_string());
            }
        }
        
        debug!("Applied composition to {} services", result.len());
        Ok(result)
    }

    /// Validate a composition
    /// 
    /// Checks:
    /// - All referenced strategies exist
    /// - No duplicate patterns with same priority
    /// - All patterns are not too broad
    pub async fn validate_composition(&self, composition: &CompositeStrategy) -> Vec<String> {
        let mut errors = Vec::new();
        let available = self.available_strategies.read().await;
        
        // Check all referenced strategies exist
        for strategy_id in composition.get_used_strategies() {
            if !available.contains_key(strategy_id) {
                errors.push(format!("Strategy '{}' not found", strategy_id));
            }
        }
        
        // Validate all patterns
        for (idx, rule) in composition.rules.iter().enumerate() {
            if let Err(e) = validate_pattern(&rule.service_pattern) {
                errors.push(format!("Rule {}: {}", idx, e));
            }
        }
        
        // Check for duplicate patterns with same priority
        let mut seen: HashMap<(String, u32), usize> = HashMap::new();
        for (idx, rule) in composition.rules.iter().enumerate() {
            let key = (rule.service_pattern.clone(), rule.priority);
            if let Some(prev_idx) = seen.get(&key) {
                errors.push(format!(
                    "Duplicate pattern '{}' with priority {} at rules {} and {}",
                    rule.service_pattern, rule.priority, prev_idx, idx
                ));
            } else {
                seen.insert(key, idx);
            }
        }
        
        errors
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Pattern Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_pattern_valid_patterns() {
        // Valid patterns should pass
        assert!(validate_pattern("youtube").is_ok());
        assert!(validate_pattern("*.youtube.com").is_ok());
        assert!(validate_pattern("discord.gg").is_ok());
        assert!(validate_pattern("mail.google.com").is_ok());
        assert!(validate_pattern("youtube*").is_ok());
        assert!(validate_pattern("*google*").is_ok());
    }

    #[test]
    fn test_validate_pattern_rejects_star() {
        let result = validate_pattern("*");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, IsolateError::Validation(_)));
        assert!(err.to_string().contains("too broad"));
    }

    #[test]
    fn test_validate_pattern_rejects_double_star() {
        let result = validate_pattern("**");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too broad"));
    }

    #[test]
    fn test_validate_pattern_rejects_star_dot_star() {
        let result = validate_pattern("*.*");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too broad"));
    }

    #[test]
    fn test_validate_pattern_rejects_question_marks() {
        assert!(validate_pattern("?").is_err());
        assert!(validate_pattern("??").is_err());
        assert!(validate_pattern("???").is_err());
    }

    #[test]
    fn test_validate_pattern_rejects_empty() {
        let result = validate_pattern("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_pattern_rejects_only_wildcards_and_dots() {
        assert!(validate_pattern("*.").is_err());
        assert!(validate_pattern(".*").is_err());
        assert!(validate_pattern("*.*.*").is_err());
    }

    #[test]
    fn test_validate_pattern_requires_minimum_specific_chars() {
        // Single char is not enough
        assert!(validate_pattern("*.a").is_err());
        // Two chars is minimum
        assert!(validate_pattern("*.ab").is_ok());
        assert!(validate_pattern("ab*").is_ok());
    }

    // ============================================================================
    // CompositionRule Tests
    // ============================================================================

    #[test]
    fn test_composition_rule_exact_match() {
        let rule = CompositionRule::new("youtube", "zapret-yt");
        
        assert!(rule.matches("youtube"));
        assert!(rule.matches("YouTube")); // case insensitive
        assert!(!rule.matches("discord"));
    }

    #[test]
    fn test_composition_rule_substring_match() {
        let rule = CompositionRule::new("google", "zapret-google");
        
        assert!(rule.matches("google.com"));
        assert!(rule.matches("mail.google.com"));
        assert!(rule.matches("GOOGLE"));
        assert!(!rule.matches("youtube.com"));
    }

    #[test]
    fn test_composition_rule_glob_match() {
        let rule = CompositionRule::new("*.google.com", "zapret-google");
        
        assert!(rule.matches("mail.google.com"));
        assert!(rule.matches("drive.google.com"));
        assert!(!rule.matches("google.com")); // no subdomain
        assert!(!rule.matches("google.com.evil.com"));
    }

    #[test]
    fn test_composition_rule_glob_prefix() {
        let rule = CompositionRule::new("youtube*", "zapret-yt");
        
        assert!(rule.matches("youtube.com"));
        assert!(rule.matches("youtubekids.com"));
        assert!(!rule.matches("notyoutube.com"));
    }

    #[test]
    fn test_composition_rule_disabled() {
        let rule = CompositionRule::new("youtube", "zapret-yt")
            .with_enabled(false);
        
        assert!(!rule.matches("youtube"));
    }

    #[test]
    fn test_composite_strategy_direct_mapping() {
        let mut comp = CompositeStrategy::new("test", "Test Composition");
        comp.add_mapping("youtube", "zapret-yt");
        comp.add_mapping("discord", "vless-discord");
        
        assert_eq!(comp.get_strategy_for_service("youtube"), Some("zapret-yt"));
        assert_eq!(comp.get_strategy_for_service("discord"), Some("vless-discord"));
        assert_eq!(comp.get_strategy_for_service("telegram"), None);
    }

    #[test]
    fn test_composite_strategy_rules() {
        let mut comp = CompositeStrategy::new("test", "Test Composition");
        comp.add_rule(CompositionRule::new("youtube", "zapret-yt").with_priority(100));
        comp.add_rule(CompositionRule::new("discord", "vless-discord").with_priority(90));
        comp.default_strategy = Some("default-strategy".to_string());
        
        assert_eq!(comp.get_strategy_for_service("youtube.com"), Some("zapret-yt"));
        assert_eq!(comp.get_strategy_for_service("discord.gg"), Some("vless-discord"));
        assert_eq!(comp.get_strategy_for_service("unknown.com"), Some("default-strategy"));
    }

    #[test]
    fn test_composite_strategy_priority_order() {
        let mut comp = CompositeStrategy::new("test", "Test Composition");
        
        // Add rules in wrong order
        comp.add_rule(CompositionRule::new("*", "catch-all").with_priority(10));
        comp.add_rule(CompositionRule::new("youtube", "zapret-yt").with_priority(100));
        
        // Higher priority should match first
        assert_eq!(comp.get_strategy_for_service("youtube.com"), Some("zapret-yt"));
        assert_eq!(comp.get_strategy_for_service("other.com"), Some("catch-all"));
    }

    #[test]
    fn test_composite_strategy_mapping_over_rules() {
        let mut comp = CompositeStrategy::new("test", "Test Composition");
        comp.add_mapping("youtube", "direct-mapping");
        comp.add_rule(CompositionRule::new("youtube", "rule-mapping").with_priority(100));
        
        // Direct mapping takes precedence
        assert_eq!(comp.get_strategy_for_service("youtube"), Some("direct-mapping"));
    }

    #[test]
    fn test_composite_strategy_get_used_strategies() {
        let mut comp = CompositeStrategy::new("test", "Test Composition");
        comp.add_mapping("youtube", "zapret-yt");
        comp.add_rule(CompositionRule::new("discord", "vless-discord"));
        comp.add_rule(CompositionRule::new("telegram", "zapret-yt")); // duplicate
        comp.default_strategy = Some("default".to_string());
        
        let used = comp.get_used_strategies();
        assert!(used.contains(&"zapret-yt"));
        assert!(used.contains(&"vless-discord"));
        assert!(used.contains(&"default"));
        // Should not have duplicates
        assert_eq!(used.iter().filter(|&&s| s == "zapret-yt").count(), 1);
    }

    #[test]
    fn test_composition_rule_builder() {
        let rule = CompositionRule::new("youtube", "zapret-yt")
            .with_priority(150)
            .with_enabled(true)
            .with_description("YouTube via Zapret");
        
        assert_eq!(rule.service_pattern, "youtube");
        assert_eq!(rule.strategy_id, "zapret-yt");
        assert_eq!(rule.priority, 150);
        assert!(rule.enabled);
        assert_eq!(rule.description, Some("YouTube via Zapret".to_string()));
    }

    #[test]
    fn test_add_rule_validated_accepts_valid_pattern() {
        let mut comp = CompositeStrategy::new("test", "Test");
        let rule = CompositionRule::new("*.youtube.com", "zapret-yt");
        
        let result = comp.add_rule_validated(rule);
        assert!(result.is_ok());
        assert_eq!(comp.rules.len(), 1);
    }

    #[test]
    fn test_add_rule_validated_rejects_broad_pattern() {
        let mut comp = CompositeStrategy::new("test", "Test");
        let rule = CompositionRule::new("*", "catch-all");
        
        let result = comp.add_rule_validated(rule);
        assert!(result.is_err());
        assert_eq!(comp.rules.len(), 0); // Rule should not be added
    }

    #[test]
    fn test_composition_config_serialization() {
        let mut comp = CompositeStrategy::new("my-comp", "My Composition");
        comp.add_rule(CompositionRule::new("youtube", "zapret-yt"));
        
        let config = CompositionConfig {
            version: "1.0".to_string(),
            active_composition: Some("my-comp".to_string()),
            compositions: vec![comp],
        };
        
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: CompositionConfig = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.active_composition, Some("my-comp".to_string()));
        assert_eq!(parsed.compositions.len(), 1);
        assert_eq!(parsed.compositions[0].rules.len(), 1);
    }

    #[tokio::test]
    async fn test_composition_manager_basic() {
        let temp_dir = std::env::temp_dir().join("isolate_test_composition");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let manager = CompositionManager::new(&temp_dir);
        
        // Create and add composition
        let mut comp = CompositeStrategy::new("test-comp", "Test Composition");
        comp.add_rule(CompositionRule::new("youtube", "zapret-yt"));
        comp.add_rule(CompositionRule::new("discord", "vless-discord"));
        
        manager.upsert_composition(comp).await.unwrap();
        manager.set_active_composition(Some("test-comp".to_string())).await.unwrap();
        
        // Test retrieval
        let strategy = manager.get_strategy_for_service("youtube.com").await;
        assert_eq!(strategy, Some("zapret-yt".to_string()));
        
        let strategy = manager.get_strategy_for_service("discord.gg").await;
        assert_eq!(strategy, Some("vless-discord".to_string()));
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_composition_manager_persistence() {
        let temp_dir = std::env::temp_dir().join("isolate_test_composition_persist");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        // Create and save
        {
            let manager = CompositionManager::new(&temp_dir);
            let mut comp = CompositeStrategy::new("persist-test", "Persistence Test");
            comp.add_rule(CompositionRule::new("test", "test-strategy"));
            manager.upsert_composition(comp).await.unwrap();
        }
        
        // Load in new instance
        {
            let manager = CompositionManager::new(&temp_dir);
            manager.load().await.unwrap();
            
            let compositions = manager.get_compositions().await;
            assert_eq!(compositions.len(), 1);
            assert_eq!(compositions[0].id, "persist-test");
        }
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_composition_manager_apply() {
        let temp_dir = std::env::temp_dir().join("isolate_test_composition_apply");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let manager = CompositionManager::new(&temp_dir);
        
        let mut comp = CompositeStrategy::new("apply-test", "Apply Test");
        comp.add_rule(CompositionRule::new("youtube", "zapret-yt"));
        comp.add_rule(CompositionRule::new("discord", "vless-discord"));
        comp.default_strategy = Some("default".to_string());
        
        manager.upsert_composition(comp).await.unwrap();
        manager.set_active_composition(Some("apply-test".to_string())).await.unwrap();
        
        let services = vec![
            "youtube.com".to_string(),
            "discord.gg".to_string(),
            "unknown.com".to_string(),
        ];
        
        let result = manager.apply_composition(&services).await.unwrap();
        
        assert_eq!(result.get("youtube.com"), Some(&"zapret-yt".to_string()));
        assert_eq!(result.get("discord.gg"), Some(&"vless-discord".to_string()));
        assert_eq!(result.get("unknown.com"), Some(&"default".to_string()));
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

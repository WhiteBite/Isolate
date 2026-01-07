//! Provider profiles for ISP-specific strategy recommendations
//!
//! This module handles loading and managing ISP provider profiles
//! that contain recommended strategies for different Russian ISPs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::core::paths;

/// Provider profile containing ISP information and recommended strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    /// Unique identifier (e.g., "rostelecom", "mts")
    pub id: String,
    
    /// Display name (e.g., "Ростелеком", "МТС")
    pub name: String,
    
    /// Description of the provider
    pub description: String,
    
    /// Type of DPI system used (e.g., "ТСПУ", "EcoDPI")
    pub dpi_type: String,
    
    /// Regions where this provider operates
    #[serde(default)]
    pub regions: Vec<String>,
    
    /// List of recommended strategy IDs in priority order
    pub recommended_strategies: Vec<String>,
    
    /// Additional notes and recommendations
    #[serde(default)]
    pub notes: String,
}

/// Summary of a provider for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dpi_type: String,
    pub strategy_count: usize,
}

/// Recommendations for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRecommendations {
    pub provider_id: String,
    pub provider_name: String,
    pub strategies: Vec<String>,
    pub notes: String,
}

/// Provider registry holding all loaded profiles
#[derive(Debug, Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderProfile>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    
    /// Load all provider profiles from the configs/providers directory
    pub async fn load_providers(&mut self) -> Result<usize, String> {
        let providers_dir = paths::get_configs_dir().join("providers");
        
        if !providers_dir.exists() {
            warn!("Providers directory not found: {:?}", providers_dir);
            return Ok(0);
        }
        
        self.providers.clear();
        let mut loaded_count = 0;
        
        // Read all YAML files in the providers directory
        let entries = std::fs::read_dir(&providers_dir)
            .map_err(|e| format!("Failed to read providers directory: {}", e))?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                match self.load_provider_file(&path).await {
                    Ok(profile) => {
                        info!("Loaded provider profile: {} ({})", profile.name, profile.id);
                        self.providers.insert(profile.id.clone(), profile);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to load provider {:?}: {}", path, e);
                    }
                }
            }
        }
        
        info!("Loaded {} provider profiles", loaded_count);
        Ok(loaded_count)
    }
    
    /// Load a single provider profile from a YAML file
    async fn load_provider_file(&self, path: &Path) -> Result<ProviderProfile, String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let profile: ProviderProfile = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        
        // Validate required fields
        if profile.id.is_empty() {
            return Err("Provider ID is required".to_string());
        }
        if profile.name.is_empty() {
            return Err("Provider name is required".to_string());
        }
        if profile.recommended_strategies.is_empty() {
            return Err("At least one recommended strategy is required".to_string());
        }
        
        Ok(profile)
    }
    
    /// Get all providers as summaries
    pub fn get_all_providers(&self) -> Vec<ProviderSummary> {
        let mut providers: Vec<_> = self.providers.values()
            .map(|p| ProviderSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                description: p.description.clone(),
                dpi_type: p.dpi_type.clone(),
                strategy_count: p.recommended_strategies.len(),
            })
            .collect();
        
        // Sort by name for consistent ordering
        providers.sort_by(|a, b| a.name.cmp(&b.name));
        providers
    }
    
    /// Get a specific provider by ID
    pub fn get_provider(&self, id: &str) -> Option<&ProviderProfile> {
        self.providers.get(id)
    }
    
    /// Get recommendations for a specific provider
    pub fn get_recommendations(&self, provider_id: &str) -> Option<ProviderRecommendations> {
        self.providers.get(provider_id).map(|p| ProviderRecommendations {
            provider_id: p.id.clone(),
            provider_name: p.name.clone(),
            strategies: p.recommended_strategies.clone(),
            notes: p.notes.clone(),
        })
    }
    
    /// Check if a strategy is recommended for a provider
    pub fn is_strategy_recommended(&self, provider_id: &str, strategy_id: &str) -> bool {
        self.providers
            .get(provider_id)
            .map_or(false, |p| p.recommended_strategies.contains(&strategy_id.to_string()))
    }
    
    /// Get the priority of a strategy for a provider (lower is better, None if not recommended)
    pub fn get_strategy_priority(&self, provider_id: &str, strategy_id: &str) -> Option<usize> {
        self.providers.get(provider_id).and_then(|p| {
            p.recommended_strategies
                .iter()
                .position(|s| s == strategy_id)
        })
    }
    
    /// Get all providers that recommend a specific strategy
    pub fn get_providers_for_strategy(&self, strategy_id: &str) -> Vec<String> {
        self.providers
            .iter()
            .filter(|(_, p)| p.recommended_strategies.contains(&strategy_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect()
    }
}

/// Global provider registry instance
static PROVIDER_REGISTRY: tokio::sync::OnceCell<tokio::sync::RwLock<ProviderRegistry>> = 
    tokio::sync::OnceCell::const_new();

/// Get or initialize the global provider registry
pub async fn get_provider_registry() -> &'static tokio::sync::RwLock<ProviderRegistry> {
    PROVIDER_REGISTRY
        .get_or_init(|| async {
            let mut registry = ProviderRegistry::new();
            if let Err(e) = registry.load_providers().await {
                error!("Failed to load provider profiles: {}", e);
            }
            tokio::sync::RwLock::new(registry)
        })
        .await
}

/// Load all provider profiles (convenience function)
pub async fn load_providers() -> Result<Vec<ProviderSummary>, String> {
    let registry = get_provider_registry().await;
    let mut registry = registry.write().await;
    registry.load_providers().await?;
    Ok(registry.get_all_providers())
}

/// Get all providers (convenience function)
pub async fn get_providers() -> Vec<ProviderSummary> {
    let registry = get_provider_registry().await;
    let registry = registry.read().await;
    registry.get_all_providers()
}

/// Get recommendations for a provider (convenience function)
pub async fn get_provider_recommendations(provider_id: &str) -> Option<ProviderRecommendations> {
    let registry = get_provider_registry().await;
    let registry = registry.read().await;
    registry.get_recommendations(provider_id)
}

/// Check if a strategy is recommended for the selected provider
pub async fn is_recommended_for_provider(provider_id: &str, strategy_id: &str) -> bool {
    let registry = get_provider_registry().await;
    let registry = registry.read().await;
    registry.is_strategy_recommended(provider_id, strategy_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_profile_deserialize() {
        let yaml = r#"
id: test_provider
name: Test Provider
description: A test provider
dpi_type: ТСПУ
regions:
  - Moscow
recommended_strategies:
  - strategy1
  - strategy2
notes: |
  Test notes
"#;
        
        let profile: ProviderProfile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(profile.id, "test_provider");
        assert_eq!(profile.name, "Test Provider");
        assert_eq!(profile.recommended_strategies.len(), 2);
    }
}

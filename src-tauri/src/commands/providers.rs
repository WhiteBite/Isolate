//! Provider-related Tauri commands
//!
//! Commands for managing ISP provider profiles and getting strategy recommendations.

use crate::core::providers::{
    self, ProviderRecommendations, ProviderSummary,
};
use crate::core::errors::IsolateError;
use tracing::info;

/// Get list of all available ISP providers
/// 
/// Returns a list of provider summaries with basic information.
#[tauri::command]
pub async fn get_providers() -> Result<Vec<ProviderSummary>, IsolateError> {
    info!("Getting all providers");
    let providers = providers::get_providers().await;
    info!("Found {} providers", providers.len());
    Ok(providers)
}

/// Get strategy recommendations for a specific provider
/// 
/// Returns recommended strategies in priority order along with notes.
#[tauri::command]
pub async fn get_provider_recommendations(
    provider_id: String,
) -> Result<Option<ProviderRecommendations>, IsolateError> {
    info!(provider_id = %provider_id, "Getting recommendations for provider");
    let recommendations = providers::get_provider_recommendations(&provider_id).await;
    
    if let Some(ref rec) = recommendations {
        info!(
            provider = %rec.provider_name,
            strategies = rec.strategies.len(),
            "Found recommendations"
        );
    } else {
        info!(provider_id = %provider_id, "Provider not found");
    }
    
    Ok(recommendations)
}

/// Check if a strategy is recommended for the currently selected provider
/// 
/// This is used to show "Recommended" badges on strategies.
#[tauri::command]
pub async fn is_strategy_recommended(
    provider_id: String,
    strategy_id: String,
) -> Result<bool, IsolateError> {
    let is_recommended = providers::is_recommended_for_provider(&provider_id, &strategy_id).await;
    Ok(is_recommended)
}

/// Reload provider profiles from disk
/// 
/// Useful after adding new provider YAML files.
#[tauri::command]
pub async fn reload_providers() -> Result<Vec<ProviderSummary>, IsolateError> {
    info!("Reloading provider profiles");
    let providers = providers::load_providers().await
        .map_err(|e| IsolateError::Config(e))?;
    info!("Reloaded {} providers", providers.len());
    Ok(providers)
}

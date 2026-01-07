//! Tauri commands for Strategy Composition
//!
//! Provides IPC interface for managing composite strategies that combine
//! different strategies for different services (e.g., YouTube via Zapret, Discord via VLESS).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use tauri::AppHandle;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::commands::state_guard::get_state_or_error;
use crate::core::errors::IsolateError;
use crate::core::strategy_composition::{
    CompositeStrategy, CompositionManager, CompositionRule,
};

// ============================================================================
// Thread-safe Singleton for CompositionManager
// ============================================================================

/// Global singleton for CompositionManager with thread-safe initialization
static COMPOSITION_MANAGER: OnceLock<Arc<CompositionManager>> = OnceLock::new();

/// Mutex to prevent race condition during async initialization
static INIT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_init_lock() -> &'static Mutex<()> {
    INIT_LOCK.get_or_init(|| Mutex::new(()))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get or create CompositionManager with thread-safe initialization
async fn get_composition_manager(app: &AppHandle) -> Result<Arc<CompositionManager>, IsolateError> {
    // Fast path: already initialized
    if let Some(manager) = COMPOSITION_MANAGER.get() {
        return Ok(manager.clone());
    }
    
    // Slow path: need to initialize (with lock to prevent race condition)
    let _guard = get_init_lock().lock().await;
    
    // Double-check after acquiring lock
    if let Some(manager) = COMPOSITION_MANAGER.get() {
        return Ok(manager.clone());
    }
    
    // Get AppState for loading strategies
    let state = get_state_or_error(app)?;
    
    // Create new manager with configs directory
    let configs_dir = crate::core::paths::get_configs_dir();
    let manager = Arc::new(CompositionManager::new(configs_dir));
    
    // Load existing configuration
    if let Err(e) = manager.load().await {
        tracing::warn!("Failed to load composition config: {}", e);
    }
    
    // Set available strategies for validation
    let strategies_map = state.config_manager.load_strategies().await
        .unwrap_or_default();
    let strategies: Vec<_> = strategies_map.into_values().collect();
    manager.set_available_strategies(strategies).await;
    
    // Store in OnceLock (ignore error if another thread beat us)
    let _ = COMPOSITION_MANAGER.set(manager.clone());
    
    Ok(manager)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get all composition rules from the active composition
/// 
/// Returns an empty array if no active composition is set.
#[tauri::command]
pub async fn get_composition_rules(app: AppHandle) -> Result<Vec<CompositionRule>, IsolateError> {
    info!("Getting composition rules");
    
    let manager = get_composition_manager(&app).await?;
    let rules = manager.get_rules().await;
    
    debug!("Retrieved {} composition rules", rules.len());
    Ok(rules)
}

/// Set composition rules for the active composition
/// 
/// Creates a default composition if none exists.
/// Rules are automatically sorted by priority (descending).
#[tauri::command]
pub async fn set_composition_rules(
    app: AppHandle,
    rules: Vec<CompositionRule>,
) -> Result<(), IsolateError> {
    info!("Setting {} composition rules", rules.len());
    
    let manager = get_composition_manager(&app).await?;
    
    // Ensure we have an active composition
    if manager.get_active_composition().await.is_none() {
        // Create default composition
        let default_comp = CompositeStrategy::new("default", "Default Composition");
        manager.upsert_composition(default_comp).await
            .map_err(|e| IsolateError::Config(format!("Failed to create default composition: {}", e)))?;
        manager.set_active_composition(Some("default".to_string())).await
            .map_err(|e| IsolateError::Config(format!("Failed to set active composition: {}", e)))?;
    }
    
    manager.set_rules(rules).await
        .map_err(|e| IsolateError::Config(format!("Failed to set composition rules: {}", e)))?;
    
    info!("Composition rules updated successfully");
    Ok(())
}

/// Apply composite strategy to a list of services
/// 
/// Returns a mapping of service_id to strategy_id based on the active composition rules.
#[tauri::command]
pub async fn apply_composite_strategy(
    app: AppHandle,
    service_ids: Vec<String>,
) -> Result<HashMap<String, String>, IsolateError> {
    info!("Applying composite strategy to {} services", service_ids.len());
    
    let manager = get_composition_manager(&app).await?;
    
    let result = manager.apply_composition(&service_ids).await
        .map_err(|e| IsolateError::Config(format!("Failed to apply composition: {}", e)))?;
    
    debug!("Applied composition: {:?}", result);
    Ok(result)
}

/// Get all defined compositions
#[tauri::command]
pub async fn get_compositions(app: AppHandle) -> Result<Vec<CompositeStrategy>, IsolateError> {
    info!("Getting all compositions");
    
    let manager = get_composition_manager(&app).await?;
    let compositions = manager.get_compositions().await;
    
    debug!("Retrieved {} compositions", compositions.len());
    Ok(compositions)
}

/// Get the active composition
#[tauri::command]
pub async fn get_active_composition(app: AppHandle) -> Result<Option<CompositeStrategy>, IsolateError> {
    info!("Getting active composition");
    
    let manager = get_composition_manager(&app).await?;
    Ok(manager.get_active_composition().await)
}

/// Set the active composition by ID
#[tauri::command]
pub async fn set_active_composition(
    app: AppHandle,
    composition_id: Option<String>,
) -> Result<(), IsolateError> {
    info!("Setting active composition: {:?}", composition_id);
    
    let manager = get_composition_manager(&app).await?;
    
    manager.set_active_composition(composition_id).await
        .map_err(|e| IsolateError::Config(format!("Failed to set active composition: {}", e)))?;
    
    Ok(())
}

/// Create or update a composition
#[tauri::command]
pub async fn upsert_composition(
    app: AppHandle,
    composition: CompositeStrategy,
) -> Result<(), IsolateError> {
    info!("Upserting composition: {}", composition.id);
    
    let manager = get_composition_manager(&app).await?;
    
    manager.upsert_composition(composition).await
        .map_err(|e| IsolateError::Config(format!("Failed to upsert composition: {}", e)))?;
    
    Ok(())
}

/// Remove a composition by ID
#[tauri::command]
pub async fn remove_composition(
    app: AppHandle,
    composition_id: String,
) -> Result<(), IsolateError> {
    info!("Removing composition: {}", composition_id);
    
    let manager = get_composition_manager(&app).await?;
    
    manager.remove_composition(&composition_id).await
        .map_err(|e| IsolateError::Config(format!("Failed to remove composition: {}", e)))?;
    
    Ok(())
}

/// Get strategy for a specific service using active composition
#[tauri::command]
pub async fn get_strategy_for_service(
    app: AppHandle,
    service_id: String,
) -> Result<Option<String>, IsolateError> {
    debug!("Getting strategy for service: {}", service_id);
    
    let manager = get_composition_manager(&app).await?;
    Ok(manager.get_strategy_for_service(&service_id).await)
}

/// Validate a composition and return any errors
#[tauri::command]
pub async fn validate_composition(
    app: AppHandle,
    composition: CompositeStrategy,
) -> Result<Vec<String>, IsolateError> {
    info!("Validating composition: {}", composition.id);
    
    let manager = get_composition_manager(&app).await?;
    let errors = manager.validate_composition(&composition).await;
    
    if errors.is_empty() {
        debug!("Composition is valid");
    } else {
        debug!("Composition has {} validation errors", errors.len());
    }
    
    Ok(errors)
}

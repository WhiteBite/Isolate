//! Service-related Tauri commands

use std::sync::Arc;
use tauri::State;
use tracing::info;

use crate::commands::validation::{validate_not_empty, validate_url};
use crate::core::errors::IsolateError;
use crate::core::models::Service;
use crate::state::AppState;

/// Get list of services to test
#[tauri::command]
pub async fn get_services(state: State<'_, Arc<AppState>>) -> Result<Vec<Service>, IsolateError> {
    info!("Loading services");
    
    let services_map = state
        .config_manager
        .load_services()
        .await?;
    
    Ok(services_map.into_values().collect())
}

// ============================================================================
// Service Registry Commands (New Services System)
// ============================================================================

use crate::services::{Service as RegistryService, ServiceStatus as NewServiceStatus};

/// Get all registered services from the service registry
#[tauri::command]
pub async fn get_registry_services(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<RegistryService>, IsolateError> {
    info!("Getting all registered services");
    
    let services = state.service_registry.get_all().await;
    Ok(services)
}

/// Get service status by ID (with caching)
#[tauri::command]
pub async fn get_service_status(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<NewServiceStatus, IsolateError> {
    info!(service_id = %service_id, "Getting service status");
    
    state
        .service_checker
        .check_service(&service_id)
        .await
        .map_err(|e| IsolateError::Network(e.to_string()))
}

/// Check a specific service (fresh check, no cache)
#[tauri::command]
pub async fn check_single_service(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<NewServiceStatus, IsolateError> {
    info!(service_id = %service_id, "Checking service (fresh)");
    
    let result = state
        .service_checker
        .check_service_fresh(&service_id)
        .await
        .map_err(|e| IsolateError::Network(e.to_string()))?;
    
    // Record health history
    let error_msg = result.error.as_deref();
    if let Err(e) = state
        .storage
        .record_service_check(&service_id, result.accessible, result.avg_latency_ms, error_msg)
        .await
    {
        tracing::warn!(error = %e, "Failed to record service health history");
    }
    
    Ok(result)
}

/// Check all registered services
#[tauri::command]
pub async fn check_all_registry_services(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<Vec<NewServiceStatus>, IsolateError> {
    // Rate limiting: 2 requests per minute (heavy operation)
    crate::commands::rate_limiter::check_rate_limit_with_config(
        "check_all_registry_services",
        crate::commands::rate_limiter::limits::CHECK_ALL_SERVICES,
    )?;
    
    info!("Checking all registered services");
    
    let results = state.service_checker.check_all_services().await;
    
    // Record health history for all checked services
    for result in &results {
        let error_msg = result.error.as_deref();
        if let Err(e) = state
            .storage
            .record_service_check(&result.service_id, result.accessible, result.avg_latency_ms, error_msg)
            .await
        {
            tracing::warn!(
                service_id = %result.service_id,
                error = %e,
                "Failed to record service health history"
            );
        }
    }
    
    // Cleanup old history records (older than 7 days)
    if let Err(e) = state.storage.cleanup_old_health_history().await {
        tracing::warn!(error = %e, "Failed to cleanup old health history");
    }
    
    // Check for auto-failover if enabled
    if state.auto_failover.is_enabled().await {
        check_and_trigger_failover(&state, &results, &app).await;
    }
    
    Ok(results)
}

/// Evaluates service check results and triggers failover if needed
async fn check_and_trigger_failover(
    state: &Arc<AppState>,
    results: &[NewServiceStatus],
    app: &tauri::AppHandle,
) {
    use tauri::Emitter;
    
    // Get current strategy
    let current_strategy = state.strategy_engine.get_global_strategy().await;
    let Some(strategy_id) = current_strategy else {
        return; // No active strategy, nothing to failover
    };
    
    // Evaluate degradation: consider degraded if less than 50% services are accessible
    let total = results.len();
    if total == 0 {
        return;
    }
    
    let accessible = results.iter().filter(|s| s.accessible).count();
    let success_rate = accessible as f64 / total as f64;
    let is_degraded = success_rate < 0.5;
    
    if is_degraded {
        // Record failure
        let reason = format!(
            "Service degradation: {}/{} services accessible ({:.0}%)",
            accessible, total, success_rate * 100.0
        );
        
        let should_failover = state.auto_failover.record_failure(&strategy_id, &reason).await;
        
        if should_failover && state.auto_failover.should_failover(&strategy_id).await {
            // Get backup strategy
            if let Some(backup_strategy) = state.auto_failover.get_next_backup_strategy(&strategy_id).await {
                tracing::info!(
                    current = %strategy_id,
                    backup = %backup_strategy,
                    "Auto-failover triggered"
                );
                
                // Emit event for frontend to apply the backup strategy
                let _ = app.emit("failover:apply_strategy", &backup_strategy);
                let _ = app.emit("failover:triggered", serde_json::json!({
                    "previousStrategy": strategy_id,
                    "newStrategy": backup_strategy,
                    "reason": reason,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));
            }
        }
    } else {
        // Services are healthy, record success
        state.auto_failover.record_success(&strategy_id).await;
    }
}

/// Get services by category
#[tauri::command]
pub async fn get_services_by_category(
    state: State<'_, Arc<AppState>>,
    category: String,
) -> Result<Vec<RegistryService>, IsolateError> {
    info!(category = %category, "Getting services by category");
    
    let category = match category.to_lowercase().as_str() {
        "social" => crate::services::registry::ServiceCategory::Social,
        "video" => crate::services::registry::ServiceCategory::Video,
        "gaming" => crate::services::registry::ServiceCategory::Gaming,
        "messaging" => crate::services::registry::ServiceCategory::Messaging,
        "streaming" => crate::services::registry::ServiceCategory::Streaming,
        _ => crate::services::registry::ServiceCategory::Other,
    };
    
    let services = state.service_registry.get_by_category(category).await;
    Ok(services)
}

/// Clear service checker cache
#[tauri::command]
pub async fn clear_service_cache(
    state: State<'_, Arc<AppState>>,
) -> Result<(), IsolateError> {
    info!("Clearing service checker cache");
    state.service_checker.clear_cache().await;
    Ok(())
}

/// Register a custom service
#[tauri::command]
pub async fn register_custom_service(
    state: State<'_, Arc<AppState>>,
    id: String,
    name: String,
    category: String,
    endpoints: Vec<String>,
) -> Result<(), IsolateError> {
    // Validate service ID
    validate_service_id(&id)?;
    
    // Validate service name
    validate_not_empty(&name, "Service name")?;
    if name.len() > 100 {
        return Err(IsolateError::Validation(
            "Service name exceeds maximum length of 100 characters".to_string()
        ));
    }
    
    // Validate category
    validate_not_empty(&category, "Category")?;
    
    // Validate endpoints
    if endpoints.is_empty() {
        return Err(IsolateError::Validation(
            "At least one endpoint is required".to_string()
        ));
    }
    if endpoints.len() > 10 {
        return Err(IsolateError::Validation(
            "Maximum 10 endpoints allowed per service".to_string()
        ));
    }
    for (i, endpoint) in endpoints.iter().enumerate() {
        validate_url(endpoint)
            .map_err(|_| IsolateError::Validation(
                format!("Invalid URL format for endpoint {}: {}", i + 1, endpoint)
            ))?;
    }
    
    info!(id = %id, name = %name, "Registering custom service");
    
    use crate::services::registry::{Service, ServiceCategory, ServiceEndpoint, HttpMethod};
    
    let category = match category.to_lowercase().as_str() {
        "social" => ServiceCategory::Social,
        "video" => ServiceCategory::Video,
        "gaming" => ServiceCategory::Gaming,
        "messaging" => ServiceCategory::Messaging,
        "streaming" => ServiceCategory::Streaming,
        _ => ServiceCategory::Other,
    };
    
    let service_endpoints: Vec<ServiceEndpoint> = endpoints
        .into_iter()
        .enumerate()
        .map(|(i, url)| ServiceEndpoint {
            url,
            name: format!("Endpoint {}", i + 1),
            method: HttpMethod::GET,
            expected_status: Vec::new(),
            timeout_ms: 5000,
        })
        .collect();
    
    let service = Service {
        id: id.clone(),
        name,
        icon: None,
        category,
        endpoints: service_endpoints,
        description: None,
        plugin_id: Some("user-custom".to_string()),
    };
    
    state
        .service_registry
        .register(service)
        .await
        .map_err(|e| IsolateError::Config(e.to_string()))
}

/// Validate service ID format
/// 
/// Rules:
/// - Non-empty
/// - Max 64 characters
/// - Alphanumeric, underscores, hyphens only
/// - Cannot start with hyphen or underscore
fn validate_service_id(id: &str) -> Result<(), IsolateError> {
    validate_not_empty(id, "Service ID")?;
    
    let id = id.trim();
    
    if id.len() > 64 {
        return Err(IsolateError::Validation(
            "Service ID exceeds maximum length of 64 characters".to_string()
        ));
    }
    
    if id.starts_with('-') || id.starts_with('_') {
        return Err(IsolateError::Validation(
            "Service ID cannot start with hyphen or underscore".to_string()
        ));
    }
    
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(IsolateError::Validation(
            "Service ID can only contain alphanumeric characters, underscores, and hyphens".to_string()
        ));
    }
    
    Ok(())
}

/// Unregister a custom service
#[tauri::command]
pub async fn unregister_custom_service(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<(), IsolateError> {
    info!(service_id = %service_id, "Unregistering custom service");
    
    state
        .service_registry
        .unregister(&service_id)
        .await
        .map_err(|e| IsolateError::Config(e.to_string()))
}

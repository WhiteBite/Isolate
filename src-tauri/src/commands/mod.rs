//! Tauri commands — IPC interface between frontend and backend

pub mod diagnostics;
pub mod hostlists;
pub mod logs;
pub mod proxies;
pub mod quic;
pub mod routing;
pub mod settings;
pub mod system;
pub mod tray;
pub mod updates;
pub mod vless;

pub use diagnostics::*;
pub use hostlists::*;
pub use logs::*;
pub use proxies::*;
pub use quic::*;
pub use routing::*;
pub use settings::*;
pub use system::*;
pub use tray::*;
pub use updates::*;
pub use vless::*;

use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tracing::{error, info, warn};

use crate::core::models::{AppStatus, Service, Strategy};
use crate::state::AppState;

/// Check if backend is ready (AppState initialized)
/// This command doesn't require State, so it works even before AppState is ready
#[tauri::command]
pub fn is_backend_ready(app: AppHandle) -> bool {
    app.try_state::<Arc<AppState>>().is_some()
}

/// Get current application status
#[tauri::command]
pub async fn get_status(state: State<'_, Arc<AppState>>) -> Result<AppStatus, String> {
    info!("Getting application status");
    
    let strategy_engine = &state.strategy_engine;
    let current_strategy = strategy_engine.get_global_strategy().await;
    
    Ok(AppStatus {
        is_active: current_strategy.is_some(),
        current_strategy: current_strategy.clone(),
        current_strategy_name: current_strategy,
        services_status: std::collections::HashMap::new(),
    })
}

/// Get list of available strategies
#[tauri::command]
pub async fn get_strategies(state: State<'_, Arc<AppState>>) -> Result<Vec<Strategy>, String> {
    info!("Loading strategies");
    
    let strategies_map = state
        .config_manager
        .load_strategies()
        .await
        .map_err(|e| format!("Failed to load strategies: {}", e))?;
    
    Ok(strategies_map.into_values().collect())
}

/// Get list of services to test
#[tauri::command]
pub async fn get_services(state: State<'_, Arc<AppState>>) -> Result<Vec<Service>, String> {
    info!("Loading services");
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    Ok(services_map.into_values().collect())
}

/// Run optimization process
#[tauri::command]
pub async fn run_optimization(
    window: Window,
    state: State<'_, Arc<AppState>>,
    mode: String,
) -> Result<String, String> {
    info!("Starting optimization in {} mode", mode);
    
    // Emit initial progress
    let _ = window.emit(
        "optimization:progress",
        crate::core::orchestrator::OptimizationProgress::default(),
    );
    
    // Get strategies and services
    let strategies_map = state
        .config_manager
        .load_strategies()
        .await
        .map_err(|e| format!("Failed to load strategies: {}", e))?;
    
    let strategies: Vec<Strategy> = strategies_map.into_values().collect();
    
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    let service_ids: Vec<String> = services_map.keys().cloned().collect();
    
    // Get environment info
    let env_info = state.get_env_info().await;
    
    // Clone state for async task
    let orchestrator = state.orchestrator.clone();
    let window_clone = window.clone();
    
    // Subscribe to progress events
    let mut progress_rx = orchestrator.subscribe_progress();
    
    // Spawn progress listener
    tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            if let Err(e) = window_clone.emit("optimization:progress", &progress) {
                error!("Failed to emit progress: {}", e);
            }
        }
    });
    
    // Run optimization
    match orchestrator.optimize(&env_info, &strategies, &service_ids).await {
        Ok(result) => {
            info!(
                strategy_id = %result.strategy_id,
                score = result.score,
                "Optimization completed successfully"
            );
            
            let _ = window.emit("optimization:complete", serde_json::json!({
                "strategy_id": result.strategy_id,
                "strategy_name": result.strategy_name,
                "score": result.score,
                "from_cache": result.from_cache,
            }));
            
            Ok(result.strategy_id)
        }
        Err(e) => {
            error!("Optimization failed: {}", e);
            let _ = window.emit("optimization:failed", e.to_string());
            Err(format!("Optimization failed: {}", e))
        }
    }
}

/// Cancel ongoing optimization
#[tauri::command]
pub async fn cancel_optimization(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    info!("Cancelling optimization");
    state.orchestrator.cancel().await;
    Ok(())
}

/// Apply specific strategy
#[tauri::command]
pub async fn apply_strategy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<(), String> {
    info!("Applying strategy: {}", strategy_id);
    
    // Load strategy by ID
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await
        .map_err(|e| format!("Strategy not found: {}", e))?;
    
    let strategy_name = strategy.name.clone();
    
    state
        .strategy_engine
        .start_global(&strategy)
        .await
        .map_err(|e| format!("Failed to apply strategy: {}", e))?;
    
    // Emit event for frontend to update status
    let _ = app.emit("strategy:applied", serde_json::json!({
        "strategy_id": strategy_id,
        "strategy_name": strategy_name,
    }));
    
    Ok(())
}

/// Stop current strategy
#[tauri::command]
pub async fn stop_strategy(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Stopping current strategy");
    
    state
        .strategy_engine
        .stop_global()
        .await
        .map_err(|e| format!("Failed to stop strategy: {}", e))?;
    
    // Emit event for frontend to update status
    let _ = app.emit("strategy:stopped", ());
    
    Ok(())
}

// ============================================================================
// Testing Commands
// ============================================================================

use std::sync::atomic::{AtomicBool, Ordering};

/// Test progress event payload
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestProgress {
    pub current_item: String,
    pub current_type: String, // "proxy" or "strategy"
    pub tested_count: usize,
    pub total_count: usize,
    pub percent: u8,
}

/// Test result for a single item
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestItemResult {
    pub id: String,
    pub name: String,
    pub item_type: String, // "proxy" or "strategy"
    pub success_rate: f32,
    pub latency_ms: u32,
    pub score: f32,
    pub services_tested: Vec<String>,
    pub services_passed: Vec<String>,
}

// Global cancellation flag
static TESTS_CANCELLED: AtomicBool = AtomicBool::new(false);

/// Run tests on proxies and/or strategies
#[tauri::command]
pub async fn run_tests(
    window: Window,
    state: State<'_, Arc<AppState>>,
    proxy_ids: Vec<String>,
    strategy_ids: Vec<String>,
    service_ids: Vec<String>,
    mode: String,
) -> Result<(), String> {
    info!(
        proxy_count = proxy_ids.len(),
        strategy_count = strategy_ids.len(),
        service_count = service_ids.len(),
        mode = %mode,
        "Starting tests"
    );
    
    TESTS_CANCELLED.store(false, Ordering::SeqCst);
    
    let total_count = proxy_ids.len() + strategy_ids.len();
    let mut tested_count = 0;
    let mut results: Vec<TestItemResult> = Vec::new();
    
    // Load services for testing
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    let test_services: Vec<_> = if service_ids.is_empty() {
        services_map.values().cloned().collect()
    } else {
        services_map
            .values()
            .filter(|s| service_ids.contains(&s.id))
            .cloned()
            .collect()
    };
    
    let timeout_secs = if mode == "turbo" { 3 } else { 5 };
    
    // Test proxies
    for proxy_id in &proxy_ids {
        if TESTS_CANCELLED.load(Ordering::SeqCst) {
            info!("Tests cancelled by user");
            break;
        }
        
        // Get proxy
        let proxy = match state.storage.get_proxy(proxy_id).await {
            Ok(Some(p)) => p,
            _ => continue,
        };
        
        // Emit progress
        tested_count += 1;
        let progress = TestProgress {
            current_item: proxy.name.clone(),
            current_type: "proxy".to_string(),
            tested_count,
            total_count,
            percent: ((tested_count * 100) / total_count.max(1)) as u8,
        };
        let _ = window.emit("test:progress", &progress);
        
        // Test proxy against services
        let mut services_passed = Vec::new();
        let mut total_latency = 0u32;
        let mut test_count = 0u32;
        
        for service in &test_services {
            if TESTS_CANCELLED.load(Ordering::SeqCst) {
                break;
            }
            
            // Test connectivity through proxy
            if let Ok(latency) = test_proxy_for_service(&state, &proxy, service, timeout_secs).await {
                services_passed.push(service.id.clone());
                total_latency += latency;
                test_count += 1;
            }
        }
        
        let success_rate = if test_services.is_empty() {
            0.0
        } else {
            (services_passed.len() as f32 / test_services.len() as f32) * 100.0
        };
        
        let avg_latency = if test_count > 0 {
            total_latency / test_count
        } else {
            9999
        };
        
        // Score: higher is better (success_rate * 10 - latency_penalty)
        let score = success_rate * 10.0 - (avg_latency as f32 / 100.0);
        
        let result = TestItemResult {
            id: proxy.id.clone(),
            name: proxy.name.clone(),
            item_type: "proxy".to_string(),
            success_rate,
            latency_ms: avg_latency,
            score,
            services_tested: test_services.iter().map(|s| s.id.clone()).collect(),
            services_passed,
        };
        
        let _ = window.emit("test:result", &result);
        results.push(result);
    }
    
    // Test strategies (ВАЖНО: последовательно, не параллельно!)
    for strategy_id in &strategy_ids {
        if TESTS_CANCELLED.load(Ordering::SeqCst) {
            info!("Tests cancelled by user");
            break;
        }
        
        // Load strategy
        let strategy = match state.config_manager.load_strategy_by_id(strategy_id).await {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        // Emit progress
        tested_count += 1;
        let progress = TestProgress {
            current_item: strategy.name.clone(),
            current_type: "strategy".to_string(),
            tested_count,
            total_count,
            percent: ((tested_count * 100) / total_count.max(1)) as u8,
        };
        let _ = window.emit("test:progress", &progress);
        
        // Start strategy
        if let Err(e) = state.strategy_engine.start_global(&strategy).await {
            warn!(strategy_id = %strategy_id, error = %e, "Failed to start strategy for testing");
            continue;
        }
        
        // Wait for strategy to initialize
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        // Test services
        let mut services_passed = Vec::new();
        let mut total_latency = 0u32;
        let mut test_count = 0u32;
        
        for service in &test_services {
            if TESTS_CANCELLED.load(Ordering::SeqCst) {
                break;
            }
            
            if let Ok(latency) = test_service_direct(service, timeout_secs).await {
                services_passed.push(service.id.clone());
                total_latency += latency;
                test_count += 1;
            }
        }
        
        // Stop strategy
        let _ = state.strategy_engine.stop_global().await;
        
        // Wait before next strategy (prevent BSOD with WinDivert)
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        let success_rate = if test_services.is_empty() {
            0.0
        } else {
            (services_passed.len() as f32 / test_services.len() as f32) * 100.0
        };
        
        let avg_latency = if test_count > 0 {
            total_latency / test_count
        } else {
            9999
        };
        
        let score = success_rate * 10.0 - (avg_latency as f32 / 100.0);
        
        let result = TestItemResult {
            id: strategy.id.clone(),
            name: strategy.name.clone(),
            item_type: "strategy".to_string(),
            success_rate,
            latency_ms: avg_latency,
            score,
            services_tested: test_services.iter().map(|s| s.id.clone()).collect(),
            services_passed,
        };
        
        let _ = window.emit("test:result", &result);
        results.push(result);
    }
    
    // Emit completion
    let _ = window.emit("test:complete", &results);
    
    info!(results_count = results.len(), "Tests completed");
    Ok(())
}

/// Cancel running tests
#[tauri::command]
pub async fn cancel_tests() -> Result<(), String> {
    info!("Cancelling tests");
    TESTS_CANCELLED.store(true, Ordering::SeqCst);
    Ok(())
}

// Helper: test proxy for a specific service
async fn test_proxy_for_service(
    _state: &State<'_, Arc<AppState>>,
    proxy: &crate::core::models::ProxyConfig,
    service: &crate::core::models::Service,
    timeout_secs: u64,
) -> Result<u32, String> {
    let start = std::time::Instant::now();
    
    // Get test URL from service
    let test_url = service.get_test_url()
        .ok_or_else(|| "Service has no test URL".to_string())?;
    
    // For SOCKS5/HTTP proxies, test directly
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Socks5 => {
            let proxy_url = format!("socks5://{}:{}", proxy.server, proxy.port);
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| e.to_string())?;
            
            client
                .get(&test_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            let scheme = if proxy.protocol == crate::core::models::ProxyProtocol::Https { "https" } else { "http" };
            let proxy_url = format!("{}://{}:{}", scheme, proxy.server, proxy.port);
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| e.to_string())?;
            
            client
                .get(&test_url)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        _ => {
            // For other protocols, need sing-box running
            let manager = crate::core::singbox_manager::get_manager();
            
            if let Some(socks_port) = manager.get_socks_port(&proxy.id).await {
                let proxy_url = format!("socks5://127.0.0.1:{}", socks_port);
                let client = reqwest::Client::builder()
                    .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                    .timeout(std::time::Duration::from_secs(timeout_secs))
                    .danger_accept_invalid_certs(true)
                    .build()
                    .map_err(|e| e.to_string())?;
                
                client
                    .get(&test_url)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                
                Ok(start.elapsed().as_millis() as u32)
            } else {
                Err("Proxy not running".to_string())
            }
        }
    }
}

// Helper: test service directly (for strategy testing)
async fn test_service_direct(
    service: &crate::core::models::Service,
    timeout_secs: u64,
) -> Result<u32, String> {
    let start = std::time::Instant::now();
    
    // Get test URL from service
    let test_url = service.get_test_url()
        .ok_or_else(|| "Service has no test URL".to_string())?;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;
    
    client
        .get(&test_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(start.elapsed().as_millis() as u32)
}

// ============================================================================
// TUN Mode Commands
// ============================================================================

use crate::core::tun_manager::{TunConfig, TunInstance};

/// Start TUN mode
///
/// Routes all system traffic through the specified SOCKS proxy.
/// Requires administrator privileges.
#[tauri::command]
pub async fn start_tun(socks_port: u16) -> Result<TunInstance, String> {
    info!(socks_port, "Starting TUN mode");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    
    manager
        .start(socks_port)
        .await
        .map_err(|e| format!("Failed to start TUN: {}", e))
}

/// Stop TUN mode
#[tauri::command]
pub async fn stop_tun() -> Result<(), String> {
    info!("Stopping TUN mode");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    
    manager
        .stop()
        .await
        .map_err(|e| format!("Failed to stop TUN: {}", e))
}

/// Check if TUN is running
#[tauri::command]
pub async fn is_tun_running() -> Result<bool, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.is_running())
}

/// Get TUN status
#[tauri::command]
pub async fn get_tun_status() -> Result<TunInstance, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.get_instance())
}

/// Get TUN configuration
#[tauri::command]
pub async fn get_tun_config() -> Result<TunConfig, String> {
    let manager = crate::core::tun_manager::get_tun_manager();
    Ok(manager.get_config())
}

/// Update TUN configuration
///
/// Note: Changes take effect on next TUN start
#[tauri::command]
pub async fn set_tun_config(config: TunConfig) -> Result<(), String> {
    info!(interface = %config.interface_name, mtu = config.mtu, "Updating TUN config");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    manager.set_config(config);
    
    Ok(())
}

/// Check if TUN mode is available
///
/// Returns true if sing-box exists and running with admin privileges
#[tauri::command]
pub async fn is_tun_available() -> Result<bool, String> {
    Ok(crate::core::tun_manager::is_tun_available())
}

/// Restart TUN with optional new SOCKS port
#[tauri::command]
pub async fn restart_tun(socks_port: Option<u16>) -> Result<TunInstance, String> {
    info!(socks_port = ?socks_port, "Restarting TUN mode");
    
    let manager = crate::core::tun_manager::get_tun_manager();
    
    manager
        .restart(socks_port)
        .await
        .map_err(|e| format!("Failed to restart TUN: {}", e))
}

// ============================================================================
// System Proxy Commands
// ============================================================================

/// Set system proxy
///
/// Configures Windows system proxy settings.
#[tauri::command]
pub async fn set_system_proxy(host: String, port: u16, scheme: String) -> Result<(), String> {
    info!(host = %host, port, scheme = %scheme, "Setting system proxy");
    
    crate::core::system_proxy::set_system_proxy(&host, port, &scheme)
        .await
        .map_err(|e| format!("Failed to set system proxy: {}", e))
}

/// Clear system proxy settings
#[tauri::command]
pub async fn clear_system_proxy() -> Result<(), String> {
    info!("Clearing system proxy");
    
    crate::core::system_proxy::clear_system_proxy()
        .await
        .map_err(|e| format!("Failed to clear system proxy: {}", e))
}

/// Check if system proxy is currently set
#[tauri::command]
pub async fn is_system_proxy_set() -> Result<bool, String> {
    crate::core::system_proxy::is_system_proxy_set()
        .await
        .map_err(|e| format!("Failed to check system proxy: {}", e))
}

// ============================================================================
// Monitor Commands
// ============================================================================

// Types HealthCheckResult, DegradationEvent, RecoveryEvent are used internally
// by the monitor module for emitting events to frontend

/// Start strategy health monitoring
///
/// Begins periodic health checks of the active strategy.
/// Emits events: monitor:health_check, strategy:degraded, strategy:recovered
#[tauri::command]
pub async fn start_monitor(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Starting strategy monitor");
    
    state.monitor
        .start(app)
        .await
        .map_err(|e| format!("Failed to start monitor: {}", e))
}

/// Stop strategy health monitoring
#[tauri::command]
pub async fn stop_monitor(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Stopping strategy monitor");
    state.monitor.stop();
    Ok(())
}

/// Check if monitor is running
#[tauri::command]
pub async fn is_monitor_running(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    Ok(state.monitor.is_running())
}

/// Check if strategy is degraded
#[tauri::command]
pub async fn is_strategy_degraded(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    Ok(state.monitor.is_degraded())
}

/// Perform manual health check
#[tauri::command]
pub async fn check_strategy_health(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    state.monitor
        .check_strategy_health()
        .await
        .map_err(|e| format!("Health check failed: {}", e))
}

/// Set monitor test URLs
#[tauri::command]
pub async fn set_monitor_urls(
    state: State<'_, Arc<AppState>>,
    urls: Vec<String>,
) -> Result<(), String> {
    info!(count = urls.len(), "Setting monitor test URLs");
    state.monitor.set_test_urls(urls).await;
    Ok(())
}

/// Enable/disable auto-restart on degradation
#[tauri::command]
pub async fn set_monitor_auto_restart(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    info!(enabled, "Setting monitor auto-restart");
    state.monitor.set_auto_restart(enabled);
    Ok(())
}

// ============================================================================
// Telemetry Commands
// ============================================================================

/// Enable or disable telemetry (opt-in)
#[tauri::command]
pub async fn set_telemetry_enabled(
    state: State<'_, Arc<AppState>>,
    enabled: bool,
) -> Result<(), String> {
    info!(enabled, "Setting telemetry enabled");
    state.telemetry.set_enabled(enabled);
    Ok(())
}

/// Check if telemetry is enabled
#[tauri::command]
pub async fn is_telemetry_enabled(
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    Ok(state.telemetry.is_enabled())
}

/// Get number of pending telemetry events
#[tauri::command]
pub async fn get_telemetry_pending_count(
    state: State<'_, Arc<AppState>>,
) -> Result<usize, String> {
    Ok(state.telemetry.pending_events().await)
}

/// Manually flush telemetry events
#[tauri::command]
pub async fn flush_telemetry(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Manually flushing telemetry");
    state.telemetry
        .flush()
        .await
        .map_err(|e| format!("Failed to flush telemetry: {}", e))
}

/// Clear pending telemetry events without sending
#[tauri::command]
pub async fn clear_telemetry(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    info!("Clearing telemetry events");
    state.telemetry.clear().await;
    Ok(())
}

/// Report optimization result to telemetry
#[tauri::command]
pub async fn report_optimization_telemetry(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
    score: f32,
    success: bool,
) -> Result<(), String> {
    state.telemetry
        .report_optimization(&strategy_id, score, success)
        .await;
    Ok(())
}

/// Report strategy usage to telemetry
#[tauri::command]
pub async fn report_strategy_usage_telemetry(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
    duration_secs: u64,
) -> Result<(), String> {
    state.telemetry
        .report_strategy_usage(&strategy_id, duration_secs)
        .await;
    Ok(())
}


// ============================================================================
// Config Updater Commands
// ============================================================================

use crate::core::config_updater::{ConfigUpdate, UpdateResult};

/// Check for config updates from remote repository
#[tauri::command]
pub async fn check_config_updates() -> Result<Vec<ConfigUpdate>, String> {
    info!("Checking for config updates");
    
    crate::core::config_updater::check_config_updates()
        .await
        .map_err(|e| format!("Failed to check config updates: {}", e))
}

/// Download and apply config updates
#[tauri::command]
pub async fn download_config_updates() -> Result<UpdateResult, String> {
    info!("Downloading config updates");
    
    crate::core::config_updater::download_config_updates()
        .await
        .map_err(|e| format!("Failed to download config updates: {}", e))
}

// ============================================================================
// Single Strategy Testing Command
// ============================================================================

/// Test result for a single strategy
#[derive(Debug, Clone, serde::Serialize)]
pub struct StrategyTestResult {
    pub strategy_id: String,
    pub score: f32,
    pub success_rate: f32,
    pub avg_latency_ms: u32,
    pub services_passed: Vec<String>,
    pub services_failed: Vec<String>,
}

/// Test a single strategy against all enabled services
///
/// Starts the strategy, runs tests, calculates score, then stops.
/// ВАЖНО: Zapret стратегии тестируются последовательно!
#[tauri::command]
pub async fn test_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<StrategyTestResult, String> {
    info!(strategy_id = %strategy_id, "Testing single strategy");
    
    // Load strategy
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await
        .map_err(|e| format!("Strategy not found: {}", e))?;
    
    // Load services
    let services_map = state
        .config_manager
        .load_services()
        .await
        .map_err(|e| format!("Failed to load services: {}", e))?;
    
    let services: Vec<_> = services_map.values().cloned().collect();
    
    // Start strategy
    state
        .strategy_engine
        .start_global(&strategy)
        .await
        .map_err(|e| format!("Failed to start strategy: {}", e))?;
    
    // Wait for strategy to initialize
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // Test services
    let mut services_passed = Vec::new();
    let mut services_failed = Vec::new();
    let mut total_latency = 0u32;
    let mut test_count = 0u32;
    
    for service in &services {
        match test_service_direct(service, 5).await {
            Ok(latency) => {
                services_passed.push(service.id.clone());
                total_latency += latency;
                test_count += 1;
            }
            Err(_) => {
                services_failed.push(service.id.clone());
            }
        }
    }
    
    // Stop strategy
    let _ = state.strategy_engine.stop_global().await;
    
    // Calculate results
    let success_rate = if services.is_empty() {
        0.0
    } else {
        (services_passed.len() as f32 / services.len() as f32) * 100.0
    };
    
    let avg_latency_ms = if test_count > 0 {
        total_latency / test_count
    } else {
        9999
    };
    
    // Score formula: success_rate * 10 - latency_penalty
    let score = success_rate * 10.0 - (avg_latency_ms as f32 / 100.0);
    
    info!(
        strategy_id = %strategy_id,
        score,
        success_rate,
        avg_latency_ms,
        "Strategy test completed"
    );
    
    Ok(StrategyTestResult {
        strategy_id,
        score,
        success_rate,
        avg_latency_ms,
        services_passed,
        services_failed,
    })
}

// ============================================================================
// Orchestra Commands
// ============================================================================

use crate::core::orchestra::{LockedStrategy, OrchestraConfig};
use std::collections::HashMap;

/// Orchestra state for frontend
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrchestraState {
    pub is_running: bool,
    pub domains: HashMap<String, LockedStrategy>,
    pub strategies_count: usize,
}

/// Start Orchestra auto-learning
///
/// Begins automatic strategy testing for specified domains.
/// Emits events: orchestra:progress, orchestra:locked, orchestra:complete
#[tauri::command]
pub async fn start_orchestra(
    window: Window,
    _state: State<'_, Arc<AppState>>,
    domains: Vec<String>,
    config: Option<OrchestraConfig>,
) -> Result<(), String> {
    #![allow(unused_variables)]
    info!(domains = ?domains, "Starting Orchestra");
    
    // Load strategies using StrategyLoader
    let strategies_dir = crate::core::paths::get_configs_dir().join("strategies");
    let loader = crate::core::strategy_loader::StrategyLoader::new(&strategies_dir);
    let strategies = loader.load_all()
        .map_err(|e| format!("Failed to load strategies: {}", e))?;
    
    if strategies.is_empty() {
        return Err("No strategies available".to_string());
    }
    
    let config = config.unwrap_or_default();
    let orchestra = crate::core::orchestra::Orchestra::new(strategies, config);
    
    // Clone domains for the spawned task
    let domains_owned = domains.clone();
    
    // Clone window for progress events
    let window_clone = window.clone();
    
    // Spawn orchestra task
    tokio::spawn(async move {
        let domain_refs: Vec<&str> = domains_owned.iter().map(|s| s.as_str()).collect();
        
        if let Err(e) = orchestra.start(&domain_refs).await {
            error!("Orchestra failed: {}", e);
            let _ = window_clone.emit("orchestra:error", e.to_string());
        }
        
        // Emit completion with locked strategies
        let locked = orchestra.get_locked_strategies().await;
        let _ = window_clone.emit("orchestra:complete", &locked);
    });
    
    Ok(())
}

/// Stop Orchestra
#[tauri::command]
pub async fn stop_orchestra() -> Result<(), String> {
    info!("Stopping Orchestra");
    // Orchestra stops via its internal running flag
    // In a real implementation, we'd store the Orchestra instance in AppState
    Ok(())
}

/// Get Orchestra results (locked strategies)
#[tauri::command]
pub async fn get_orchestra_results() -> Result<HashMap<String, LockedStrategy>, String> {
    info!("Getting Orchestra results");
    // In a real implementation, we'd get this from stored Orchestra instance
    Ok(HashMap::new())
}

/// Apply Orchestra results
///
/// Creates a combined strategy from locked domain strategies
#[tauri::command]
pub async fn apply_orchestra_results(
    state: State<'_, Arc<AppState>>,
    locked_strategies: HashMap<String, String>, // domain -> strategy_id
) -> Result<(), String> {
    info!(count = locked_strategies.len(), "Applying Orchestra results");
    
    if locked_strategies.is_empty() {
        return Err("No strategies to apply".to_string());
    }
    
    // Get the first strategy as base (in real impl, would combine them)
    let first_strategy_id = locked_strategies.values().next()
        .ok_or("No strategies found")?;
    
    // Load and apply the strategy
    let strategy = state
        .config_manager
        .load_strategy_by_id(first_strategy_id)
        .await
        .map_err(|e| format!("Strategy not found: {}", e))?;
    
    state
        .strategy_engine
        .start_global(&strategy)
        .await
        .map_err(|e| format!("Failed to apply strategy: {}", e))?;
    
    info!(strategy_id = %first_strategy_id, "Orchestra results applied");
    Ok(())
}

/// Save Orchestra learned strategies to storage
#[tauri::command]
pub async fn save_orchestra_results(
    state: State<'_, Arc<AppState>>,
    results: HashMap<String, LockedStrategy>,
) -> Result<(), String> {
    info!(count = results.len(), "Saving Orchestra results");
    
    state
        .storage
        .set_setting("orchestra_results", &results)
        .await
        .map_err(|e| format!("Failed to save Orchestra results: {}", e))?;
    
    Ok(())
}

/// Load saved Orchestra results
#[tauri::command]
pub async fn load_orchestra_results(
    state: State<'_, Arc<AppState>>,
) -> Result<HashMap<String, LockedStrategy>, String> {
    info!("Loading saved Orchestra results");
    
    let results: Option<HashMap<String, LockedStrategy>> = state
        .storage
        .get_setting("orchestra_results")
        .await
        .map_err(|e| format!("Failed to load Orchestra results: {}", e))?;
    
    Ok(results.unwrap_or_default())
}

// ============================================================================
// Engine Mode Commands
// ============================================================================

/// Get current engine mode (mock/real/dpi_test)
#[tauri::command]
pub async fn get_engine_mode(
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let mode = state.strategy_engine.get_mode().await;
    Ok(format!("{:?}", mode))
}

/// Set engine mode (mock/real/dpi_test)
///
/// - mock: Simulates strategy execution without running real processes
/// - real: Runs actual winws/sing-box processes
/// - dpi_test: Special mode for DPI testing
#[tauri::command]
pub async fn set_engine_mode(
    state: State<'_, Arc<AppState>>,
    mode: String,
) -> Result<(), String> {
    use crate::core::strategy_engine::EngineMode;
    
    let engine_mode = match mode.to_lowercase().as_str() {
        "mock" => EngineMode::Mock,
        "real" => EngineMode::Real,
        "dpi_test" | "dpitest" => EngineMode::DpiTest,
        _ => return Err(format!("Unknown mode: {}. Valid modes: mock, real, dpi_test", mode)),
    };
    
    info!(mode = %mode, "Setting engine mode");
    state.strategy_engine.set_mode(engine_mode).await;
    
    Ok(())
}

// ============================================================================
// AutoRun Commands
// ============================================================================

// ============================================================================
// DPI Simulator Testing Commands
// ============================================================================

use crate::core::strategy_tester::{StrategyTester, StrategyTestResult as DpiTestResult};

/// Test a strategy using the DPI simulator
///
/// This command:
/// 1. Gets the strategy by ID from state
/// 2. Creates a StrategyTester
/// 3. Checks DPI simulator availability
/// 4. Runs the test through test_strategy()
/// 5. Returns the result
///
/// Requires DPI simulator VM to be running and accessible.
#[allow(dead_code)]
#[tauri::command]
pub async fn test_strategy_with_dpi(
    strategy_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<DpiTestResult, String> {
    info!(strategy_id = %strategy_id, "Testing strategy with DPI simulator");

    // 1. Get strategy by ID from state
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await
        .map_err(|e| format!("Strategy not found: {}", e))?;

    // 2. Create StrategyTester
    let tester = StrategyTester::new();

    // 3. Check DPI simulator availability
    let available = tester
        .check_availability()
        .await
        .map_err(|e| format!("Failed to check DPI simulator: {}", e))?;

    if !available {
        return Err("DPI simulator is not available. Make sure the VM is running.".to_string());
    }

    info!(strategy_id = %strategy_id, "DPI simulator available, starting test");

    // 4. Run test through test_strategy()
    let result = tester
        .test_strategy(&strategy)
        .await
        .map_err(|e| format!("Strategy test failed: {}", e))?;

    info!(
        strategy_id = %strategy_id,
        success = result.success,
        blocked_before = result.blocked_before,
        blocked_after = result.blocked_after,
        "DPI strategy test completed"
    );

    // 5. Return result
    Ok(result)
}

// ============================================================================
// AutoRun Commands
// ============================================================================

/// Get current autorun status
///
/// Returns true if the app is configured to start with Windows.
#[tauri::command]
pub async fn get_autorun_status() -> Result<bool, String> {
    info!("Checking autorun status");
    
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|e| format!("Failed to open registry key: {}", e))?;
        
        let result: Result<String, _> = run_key.get_value("Isolate");
        Ok(result.is_ok())
    }
    
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

/// Set autorun status
///
/// Enables or disables automatic startup with Windows.
/// When enabled, the app will start in silent mode (minimized to tray).
#[tauri::command]
pub async fn set_autorun(enabled: bool) -> Result<(), String> {
    info!(enabled, "Setting autorun status");
    
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;
        
        if enabled {
            // Get current executable path
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get executable path: {}", e))?;
            
            // Add --silent flag for autorun
            let value = format!("\"{}\" --silent", exe_path.display());
            
            run_key
                .set_value("Isolate", &value)
                .map_err(|e| format!("Failed to set registry value: {}", e))?;
            
            info!("Autorun enabled");
        } else {
            // Remove the registry value (ignore error if it doesn't exist)
            let _ = run_key.delete_value("Isolate");
            info!("Autorun disabled");
        }
        
        Ok(())
    }
    
    #[cfg(not(windows))]
    {
        Err("Autorun is only supported on Windows".to_string())
    }
}

// ============================================================================
// Plugin Commands (JS Plugins)
// ============================================================================

use crate::plugins::js_loader;
use crate::plugins::manifest::{LoadedPluginInfo, PluginManifest, ServiceDefinition};
use crate::plugins::checker::{check_service_endpoints, ServiceStatus as PluginServiceStatus};

/// Get plugins directory path
#[tauri::command]
pub async fn get_plugins_dir(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    Ok(state.plugins_dir.display().to_string())
}

/// Scan for plugins and return their paths
#[tauri::command]
pub async fn scan_plugin_directories(state: State<'_, Arc<AppState>>) -> Result<Vec<String>, String> {
    info!("Scanning for plugins");
    
    js_loader::scan_plugins(&state.plugins_dir)
        .map(|paths| paths.iter().map(|p| p.display().to_string()).collect())
        .map_err(|e| e.to_string())
}

/// Load plugin manifest (plugin.json)
#[tauri::command]
pub async fn load_plugin_manifest(path: String) -> Result<PluginManifest, String> {
    info!(path = %path, "Loading plugin manifest");
    let plugin_dir = std::path::PathBuf::from(&path);
    
    js_loader::load_manifest(&plugin_dir)
        .map_err(|e| e.to_string())
}

/// Get all plugins with their info
#[tauri::command]
pub async fn get_all_plugins_cmd(state: State<'_, Arc<AppState>>) -> Result<Vec<LoadedPluginInfo>, String> {
    info!("Getting all plugins");
    Ok(js_loader::get_all_plugins(&state.plugins_dir))
}

/// Get all services from service-checker plugins
#[tauri::command]
pub async fn get_plugin_services(state: State<'_, Arc<AppState>>) -> Result<Vec<ServiceDefinition>, String> {
    info!("Getting plugin services");
    Ok(crate::plugins::get_all_services(&state.plugins_dir))
}

/// Check a specific service from plugins by ID
#[tauri::command]
pub async fn check_plugin_service(service_id: String, state: State<'_, Arc<AppState>>) -> Result<PluginServiceStatus, String> {
    info!(service_id = %service_id, "Checking plugin service");
    let services = crate::plugins::get_all_services(&state.plugins_dir);
    
    let service = services.iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| format!("Service not found: {}", service_id))?;
    
    let mut status = check_service_endpoints(&service.endpoints).await;
    status.service_id = service.id.clone();
    Ok(status)
}

/// Check all services from plugins
#[tauri::command]
pub async fn check_all_plugin_services(state: State<'_, Arc<AppState>>) -> Result<Vec<PluginServiceStatus>, String> {
    info!("Checking all plugin services");
    let services = crate::plugins::get_all_services(&state.plugins_dir);
    
    let mut results = Vec::new();
    for service in services {
        let mut status = check_service_endpoints(&service.endpoints).await;
        status.service_id = service.id.clone();
        results.push(status);
    }
    
    Ok(results)
}

// ============================================================================
// Service Registry Commands (New Services System)
// ============================================================================

use crate::services::{Service as RegistryService, ServiceStatus as NewServiceStatus};

/// Get all registered services from the service registry
#[tauri::command]
pub async fn get_registry_services(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<RegistryService>, String> {
    info!("Getting all registered services");
    
    let services = state.service_registry.get_all().await;
    Ok(services)
}

/// Get service status by ID (with caching)
#[tauri::command]
pub async fn get_service_status(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<NewServiceStatus, String> {
    info!(service_id = %service_id, "Getting service status");
    
    state
        .service_checker
        .check_service(&service_id)
        .await
        .map_err(|e| e.to_string())
}

/// Check a specific service (fresh check, no cache)
#[tauri::command]
pub async fn check_single_service(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<NewServiceStatus, String> {
    info!(service_id = %service_id, "Checking service (fresh)");
    
    state
        .service_checker
        .check_service_fresh(&service_id)
        .await
        .map_err(|e| e.to_string())
}

/// Check all registered services
#[tauri::command]
pub async fn check_all_registry_services(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<NewServiceStatus>, String> {
    info!("Checking all registered services");
    
    let results = state.service_checker.check_all_services().await;
    Ok(results)
}

/// Get services by category
#[tauri::command]
pub async fn get_services_by_category(
    state: State<'_, Arc<AppState>>,
    category: String,
) -> Result<Vec<RegistryService>, String> {
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
) -> Result<(), String> {
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
) -> Result<(), String> {
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
        .map_err(|e| format!("Failed to register service: {}", e))
}

/// Unregister a custom service
#[tauri::command]
pub async fn unregister_custom_service(
    state: State<'_, Arc<AppState>>,
    service_id: String,
) -> Result<(), String> {
    info!(service_id = %service_id, "Unregistering custom service");
    
    state
        .service_registry
        .unregister(&service_id)
        .await
        .map_err(|e| format!("Failed to unregister service: {}", e))
}

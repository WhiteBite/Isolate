//! Testing-related Tauri commands

use std::sync::Arc;
use tauri::{State, Window};
use tracing::{info, warn};

use crate::core::errors::IsolateError;
use crate::core::models::Service;
use crate::state::AppState;
use super::rate_limiter;

// ============================================================================
// Testing Commands
// ============================================================================

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

/// Run tests on proxies and/or strategies
#[tauri::command]
pub async fn run_tests(
    window: Window,
    state: State<'_, Arc<AppState>>,
    proxy_ids: Vec<String>,
    strategy_ids: Vec<String>,
    service_ids: Vec<String>,
    mode: String,
) -> Result<(), IsolateError> {
    use tauri::Emitter;
    use tokio_util::sync::CancellationToken;
    
    // Rate limit: max once per 5 seconds
    rate_limiter::check_rate_limit("run_tests", 5)?;
    
    info!(
        proxy_count = proxy_ids.len(),
        strategy_count = strategy_ids.len(),
        service_count = service_ids.len(),
        mode = %mode,
        "Starting tests"
    );
    
    // Create new cancellation token for this test session
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = state.tests_cancel_token.write().await;
        *token_guard = cancel_token.clone();
    }
    
    let total_count = proxy_ids.len() + strategy_ids.len();
    let mut tested_count = 0;
    let mut results: Vec<TestItemResult> = Vec::new();
    
    // Load services for testing
    let services_map = state
        .config_manager
        .load_services()
        .await?;
    
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
        if cancel_token.is_cancelled() {
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
            if cancel_token.is_cancelled() {
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
        if cancel_token.is_cancelled() {
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
            if cancel_token.is_cancelled() {
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
pub async fn cancel_tests(state: State<'_, Arc<AppState>>) -> Result<(), IsolateError> {
    info!("Cancelling tests");
    let token_guard = state.tests_cancel_token.read().await;
    token_guard.cancel();
    Ok(())
}

// Helper: test proxy for a specific service
async fn test_proxy_for_service(
    _state: &State<'_, Arc<AppState>>,
    proxy: &crate::core::models::ProxyConfig,
    service: &Service,
    timeout_secs: u64,
) -> Result<u32, IsolateError> {
    let start = std::time::Instant::now();
    
    // Get test URL from service
    let test_url = service.get_test_url()
        .ok_or_else(|| IsolateError::TestFailed("Service has no test URL".to_string()))?;
    
    // For SOCKS5/HTTP proxies, test directly
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Socks5 => {
            let proxy_url = format!("socks5://{}:{}", proxy.server, proxy.port);
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| IsolateError::Network(e.to_string()))?)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| IsolateError::Network(e.to_string()))?;
            
            client
                .get(&test_url)
                .send()
                .await
                .map_err(|e| IsolateError::Network(e.to_string()))?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            let scheme = if proxy.protocol == crate::core::models::ProxyProtocol::Https { "https" } else { "http" };
            let proxy_url = format!("{}://{}:{}", scheme, proxy.server, proxy.port);
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| IsolateError::Network(e.to_string()))?)
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| IsolateError::Network(e.to_string()))?;
            
            client
                .get(&test_url)
                .send()
                .await
                .map_err(|e| IsolateError::Network(e.to_string()))?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        _ => {
            // For other protocols, need sing-box running
            let manager = crate::core::singbox_manager::get_manager();
            
            if let Some(socks_port) = manager.get_socks_port(&proxy.id).await {
                let proxy_url = format!("socks5://127.0.0.1:{}", socks_port);
                let client = reqwest::Client::builder()
                    .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| IsolateError::Network(e.to_string()))?)
                    .timeout(std::time::Duration::from_secs(timeout_secs))
                    .danger_accept_invalid_certs(true)
                    .build()
                    .map_err(|e| IsolateError::Network(e.to_string()))?;
                
                client
                    .get(&test_url)
                    .send()
                    .await
                    .map_err(|e| IsolateError::Network(e.to_string()))?;
                
                Ok(start.elapsed().as_millis() as u32)
            } else {
                Err(IsolateError::Process("Proxy not running".to_string()))
            }
        }
    }
}

// Helper: test service directly (for strategy testing)
async fn test_service_direct(
    service: &Service,
    timeout_secs: u64,
) -> Result<u32, IsolateError> {
    let start = std::time::Instant::now();
    
    // Get test URL from service
    let test_url = service.get_test_url()
        .ok_or_else(|| IsolateError::TestFailed("Service has no test URL".to_string()))?;
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| IsolateError::Network(e.to_string()))?;
    
    client
        .get(&test_url)
        .send()
        .await
        .map_err(|e| IsolateError::Network(e.to_string()))?;
    
    Ok(start.elapsed().as_millis() as u32)
}

/// Test a single strategy against all enabled services
///
/// Starts the strategy, runs tests, calculates score, then stops.
/// ВАЖНО: Zapret стратегии тестируются последовательно!
#[tauri::command]
pub async fn test_strategy(
    state: State<'_, Arc<AppState>>,
    strategy_id: String,
) -> Result<StrategyTestResult, IsolateError> {
    info!(strategy_id = %strategy_id, "Testing single strategy");
    
    // Load strategy
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await?;
    
    // Load services
    let services_map = state
        .config_manager
        .load_services()
        .await?;
    
    let services: Vec<_> = services_map.values().cloned().collect();
    
    // Start strategy
    state
        .strategy_engine
        .start_global(&strategy)
        .await?;
    
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
) -> Result<DpiTestResult, IsolateError> {
    info!(strategy_id = %strategy_id, "Testing strategy with DPI simulator");

    // 1. Get strategy by ID from state
    let strategy = state
        .config_manager
        .load_strategy_by_id(&strategy_id)
        .await?;

    // 2. Create StrategyTester
    let tester = StrategyTester::new();

    // 3. Check DPI simulator availability
    let available = tester
        .check_availability()
        .await?;

    if !available {
        return Err(IsolateError::Network(
            "DPI simulator is not available. Make sure the VM is running.".to_string()
        ));
    }

    info!(strategy_id = %strategy_id, "DPI simulator available, starting test");

    // 4. Run test through test_strategy()
    let result = tester
        .test_strategy(&strategy)
        .await?;

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

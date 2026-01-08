//! Proxy management commands
//!
//! Commands for managing proxy configurations (SOCKS5, HTTP, Shadowsocks, Trojan, etc.)
//! Note: VLESS-specific commands are in the `vless` module.

#![allow(dead_code)] // Public proxy commands API

use std::sync::Arc;
use tauri::State;
use tracing::{info, warn};

use crate::commands::validation::{validate_not_empty, validate_public_url};
use crate::core::errors::{IsolateError, TypedResultExt};
use crate::core::models::ProxyConfig;
use crate::state::AppState;

// ============================================================================
// Proxy CRUD Commands
// ============================================================================

/// Get all saved proxies
#[tauri::command]
pub async fn get_proxies(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ProxyConfig>, IsolateError> {
    info!("Loading all proxies");
    
    state
        .storage
        .get_all_proxies()
        .await
        .storage_context("Failed to get proxies")
}

/// Add a new proxy
#[tauri::command]
pub async fn add_proxy(
    state: State<'_, Arc<AppState>>,
    proxy: ProxyConfig,
) -> Result<ProxyConfig, IsolateError> {
    validate_not_empty(&proxy.name, "Proxy name")?;
    validate_not_empty(&proxy.server, "Server address")?;
    
    info!(id = %proxy.id, name = %proxy.name, protocol = ?proxy.protocol, "Adding new proxy");
    
    // Generate ID if empty
    let mut proxy = proxy;
    if proxy.id.is_empty() {
        proxy.id = format!(
            "{}_{}", 
            format!("{:?}", proxy.protocol).to_lowercase(),
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")
        );
    }
    
    state
        .storage
        .save_proxy(&proxy)
        .await
        .storage_context("Failed to add proxy")?;
    
    info!(id = %proxy.id, "Proxy added successfully");
    Ok(proxy)
}

/// Update existing proxy
#[tauri::command]
pub async fn update_proxy(
    state: State<'_, Arc<AppState>>,
    proxy: ProxyConfig,
) -> Result<(), IsolateError> {
    info!(id = %proxy.id, name = %proxy.name, "Updating proxy");
    
    state
        .storage
        .update_proxy(&proxy)
        .await
        .storage_context("Failed to update proxy")
}

/// Delete proxy by ID
#[tauri::command]
pub async fn delete_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), IsolateError> {
    // Validate proxy ID
    validate_not_empty(&id, "Proxy ID")?;
    if id.len() > 64 {
        return Err(IsolateError::Validation(
            "Proxy ID exceeds maximum length of 64 characters".to_string()
        ));
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(IsolateError::Validation(
            "Proxy ID can only contain alphanumeric characters, underscores, and hyphens".to_string()
        ));
    }
    
    info!(id = %id, "Deleting proxy");
    
    // Stop proxy if running
    let manager = crate::core::singbox_manager::get_manager();
    if manager.is_running(&id).await {
        let _ = manager.stop(&id).await;
    }
    
    state
        .storage
        .delete_proxy(&id)
        .await
        .storage_context("Failed to delete proxy")
}

// ============================================================================
// Proxy Control Commands
// ============================================================================

/// Helper function to validate proxy ID format
fn validate_proxy_id(id: &str) -> Result<(), IsolateError> {
    validate_not_empty(id, "Proxy ID")?;
    
    if id.len() > 64 {
        return Err(IsolateError::Validation(
            "Proxy ID exceeds maximum length of 64 characters".to_string()
        ));
    }
    
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(IsolateError::Validation(
            "Proxy ID can only contain alphanumeric characters, underscores, and hyphens".to_string()
        ));
    }
    
    Ok(())
}

/// Apply proxy (start sing-box with this proxy)
#[tauri::command]
pub async fn apply_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), IsolateError> {
    // Validate proxy ID
    validate_proxy_id(&id)?;
    
    info!(id = %id, "Applying proxy");
    
    // Get proxy config
    let proxy = state
        .storage
        .get_proxy(&id)
        .await
        .storage_context("Failed to get proxy")?
        .ok_or_else(|| IsolateError::Other(format!("Proxy '{}' not found", id)))?;
    
    // Check if protocol is supported for sing-box
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Vless |
        crate::core::models::ProxyProtocol::Vmess |
        crate::core::models::ProxyProtocol::Shadowsocks |
        crate::core::models::ProxyProtocol::Trojan |
        crate::core::models::ProxyProtocol::Hysteria |
        crate::core::models::ProxyProtocol::Hysteria2 |
        crate::core::models::ProxyProtocol::Tuic => {
            // Convert to VlessConfig for sing-box (generic proxy config)
            let vless_config = crate::core::vless_engine::VlessConfig::new(
                proxy.server.clone(),
                proxy.port,
                proxy.uuid.clone().unwrap_or_default(),
            )
            .with_name(&proxy.name)
            .with_id(&proxy.id)
            .with_sni(proxy.sni.clone().unwrap_or_else(|| proxy.server.clone()));
            
            let manager = crate::core::singbox_manager::get_manager();
            let port = manager.allocate_port(1080).await;
            
            manager
                .start(&vless_config, port)
                .await
                .process_context("Failed to start proxy")?;
            
            // Mark as active in storage
            state
                .storage
                .set_proxy_active(&id, true)
                .await
                .storage_context("Failed to set proxy active")?;
            
            info!(id = %id, socks_port = port, "Proxy applied successfully");
            Ok(())
        }
        crate::core::models::ProxyProtocol::Socks5 |
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            // These protocols don't need sing-box, just mark as active
            state
                .storage
                .set_proxy_active(&id, true)
                .await
                .storage_context("Failed to set proxy active")?;
            
            info!(id = %id, "Direct proxy marked as active");
            Ok(())
        }
        _ => Err(IsolateError::Other(format!("Protocol {:?} is not supported for apply", proxy.protocol))),
    }
}

/// Result of proxy test
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProxyTestResult {
    pub success: bool,
    pub latency: Option<u32>,
    pub error: Option<String>,
}

/// Test proxy connectivity
#[tauri::command]
pub async fn test_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<ProxyTestResult, IsolateError> {
    // Rate limiting: 10 requests per minute
    crate::commands::rate_limiter::check_rate_limit_with_config(
        "test_proxy",
        crate::commands::rate_limiter::limits::TEST_PROXY,
    )?;
    
    // Validate proxy ID
    validate_proxy_id(&id)?;
    
    info!(id = %id, "Testing proxy connectivity");
    
    // Get proxy config
    let proxy = state
        .storage
        .get_proxy(&id)
        .await
        .storage_context("Failed to get proxy")?
        .ok_or_else(|| IsolateError::Other(format!("Proxy '{}' not found", id)))?;
    
    let manager = crate::core::singbox_manager::get_manager();
    
    // Check if already running
    if let Some(socks_port) = manager.get_socks_port(&id).await {
        // Test existing connection
        match crate::core::vless_engine::test_proxy_connectivity(socks_port, "https://www.google.com").await {
            Ok(latency) => return Ok(ProxyTestResult { success: true, latency: Some(latency), error: None }),
            Err(e) => return Ok(ProxyTestResult { success: false, latency: None, error: Some(e.to_string()) }),
        }
    }
    
    // Start temporary proxy for testing
    match proxy.protocol {
        crate::core::models::ProxyProtocol::Vless |
        crate::core::models::ProxyProtocol::Vmess |
        crate::core::models::ProxyProtocol::Shadowsocks |
        crate::core::models::ProxyProtocol::Trojan |
        crate::core::models::ProxyProtocol::Hysteria |
        crate::core::models::ProxyProtocol::Hysteria2 |
        crate::core::models::ProxyProtocol::Tuic => {
            let vless_config = crate::core::vless_engine::VlessConfig::new(
                proxy.server.clone(),
                proxy.port,
                proxy.uuid.clone().unwrap_or_default(),
            )
            .with_name(&proxy.name)
            .with_id(format!("test_{}", proxy.id))
            .with_sni(proxy.sni.clone().unwrap_or_else(|| proxy.server.clone()));
            
            let port = manager.allocate_port(10800).await;
            
            // Start proxy
            if let Err(e) = manager.start(&vless_config, port).await {
                return Ok(ProxyTestResult { success: false, latency: None, error: Some(format!("Failed to start: {}", e)) });
            }
            
            // Wait for proxy to initialize
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            
            // Test connectivity
            let result = crate::core::vless_engine::test_proxy_connectivity(port, "https://www.google.com").await;
            
            // Stop test proxy
            let _ = manager.stop(&format!("test_{}", proxy.id)).await;
            
            match result {
                Ok(latency) => Ok(ProxyTestResult { success: true, latency: Some(latency), error: None }),
                Err(e) => Ok(ProxyTestResult { success: false, latency: None, error: Some(e.to_string()) }),
            }
        }
        crate::core::models::ProxyProtocol::Socks5 => {
            // Test SOCKS5 directly
            let start = std::time::Instant::now();
            let addr = format!("{}:{}", proxy.server, proxy.port);
            
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tokio::net::TcpStream::connect(&addr)
            ).await {
                Ok(Ok(_)) => Ok(ProxyTestResult { success: true, latency: Some(start.elapsed().as_millis() as u32), error: None }),
                Ok(Err(e)) => Ok(ProxyTestResult { success: false, latency: None, error: Some(format!("Connection failed: {}", e)) }),
                Err(_) => Ok(ProxyTestResult { success: false, latency: None, error: Some("Connection timeout".to_string()) }),
            }
        }
        crate::core::models::ProxyProtocol::Http |
        crate::core::models::ProxyProtocol::Https => {
            // Test HTTP proxy
            let start = std::time::Instant::now();
            let proxy_url = format!(
                "{}://{}:{}",
                if proxy.protocol == crate::core::models::ProxyProtocol::Https { "https" } else { "http" },
                proxy.server,
                proxy.port
            );
            
            let client = match reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).network_context("Invalid proxy URL")?)
                .timeout(std::time::Duration::from_secs(5))
                .build() {
                    Ok(c) => c,
                    Err(e) => return Ok(ProxyTestResult { success: false, latency: None, error: Some(format!("Failed to create client: {}", e)) }),
                };
            
            match client.get("https://www.google.com").send().await {
                Ok(_) => Ok(ProxyTestResult { success: true, latency: Some(start.elapsed().as_millis() as u32), error: None }),
                Err(e) => Ok(ProxyTestResult { success: false, latency: None, error: Some(format!("Request failed: {}", e)) }),
            }
        }
        _ => Ok(ProxyTestResult { success: false, latency: None, error: Some(format!("Protocol {:?} is not supported for testing", proxy.protocol)) }),
    }
}

// ============================================================================
// Proxy Import Commands
// ============================================================================

/// Import proxy from URL (vless://, vmess://, ss://, etc.)
#[tauri::command]
pub async fn import_proxy_url(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<ProxyConfig, IsolateError> {
    validate_not_empty(&url, "Proxy URL")?;
    
    info!("Importing proxy from URL");
    
    let proxy = crate::core::proxy_parser::parse_proxy_url(&url)
        .config_context("Failed to parse proxy URL")?;
    
    // Save to storage
    state
        .storage
        .save_proxy(&proxy)
        .await
        .storage_context("Failed to save proxy")?;
    
    info!(id = %proxy.id, name = %proxy.name, protocol = ?proxy.protocol, "Proxy imported from URL");
    Ok(proxy)
}

/// Import subscription (multiple proxies from URL)
#[tauri::command]
pub async fn import_subscription(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<Vec<ProxyConfig>, IsolateError> {
    // SSRF Protection: Validate that URL points to a public address
    // This prevents attacks targeting internal services (localhost, 192.168.x.x, etc.)
    validate_public_url(&url)?;
    
    // Rate limiting: 5 requests per minute (additional SSRF protection)
    crate::commands::rate_limiter::check_rate_limit_with_config(
        "import_subscription",
        crate::commands::rate_limiter::limits::IMPORT_SUBSCRIPTION,
    )?;
    
    info!(url = %url, "Importing subscription");
    
    // Fetch subscription content
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .network_context("Failed to create HTTP client")?;
    
    let response = client
        .get(&url)
        .send()
        .await
        .network_context("Failed to fetch subscription")?;
    
    if !response.status().is_success() {
        return Err(IsolateError::Network(format!("Subscription request failed with status: {}", response.status())));
    }
    
    let content = response
        .text()
        .await
        .network_context("Failed to read subscription content")?;
    
    // Parse subscription
    let proxies = crate::core::proxy_parser::parse_subscription(&content)
        .config_context("Failed to parse subscription")?;
    
    // Save all proxies
    let mut saved_proxies = Vec::new();
    for proxy in proxies {
        match state.storage.save_proxy(&proxy).await {
            Ok(_) => {
                info!(id = %proxy.id, name = %proxy.name, "Proxy from subscription saved");
                saved_proxies.push(proxy);
            }
            Err(e) => {
                warn!(id = %proxy.id, error = %e, "Failed to save proxy from subscription");
            }
        }
    }
    
    info!(count = saved_proxies.len(), "Subscription import completed");
    Ok(saved_proxies)
}

/// Parse proxy URL without saving (for preview)
#[tauri::command]
pub async fn parse_proxy_url(url: String) -> Result<ProxyConfig, IsolateError> {
    info!("Parsing proxy URL for preview");
    
    crate::core::proxy_parser::parse_proxy_url(&url)
        .config_context("Failed to parse proxy URL")
}

/// Export proxy config to URL format
#[tauri::command]
pub async fn export_proxy_url(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<String, IsolateError> {
    // Validate proxy ID
    validate_proxy_id(&id)?;
    
    info!(id = %id, "Exporting proxy to URL");
    
    let proxy = state
        .storage
        .get_proxy(&id)
        .await
        .storage_context("Failed to get proxy")?
        .ok_or_else(|| IsolateError::Other(format!("Proxy '{}' not found", id)))?;
    
    crate::core::proxy_parser::export_proxy_url(&proxy)
        .config_context("Failed to export proxy URL")
}

/// Deactivate proxy (stop sing-box and mark as inactive)
#[tauri::command]
pub async fn deactivate_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), IsolateError> {
    // Validate proxy ID
    validate_proxy_id(&id)?;
    
    info!(id = %id, "Deactivating proxy");
    
    // Stop sing-box if running
    let manager = crate::core::singbox_manager::get_manager();
    if manager.is_running(&id).await {
        manager
            .stop(&id)
            .await
            .process_context("Failed to stop proxy")?;
    }
    
    // Mark as inactive in storage
    state
        .storage
        .set_proxy_active(&id, false)
        .await
        .storage_context("Failed to set proxy inactive")?;
    
    info!(id = %id, "Proxy deactivated successfully");
    Ok(())
}


// ============================================================================
// Subscription Fetch Command (for frontend)
// ============================================================================

/// Fetch subscription content from URL (CORS-free)
#[tauri::command]
pub async fn fetch_subscription_content(url: String) -> Result<String, IsolateError> {
    // SSRF Protection
    validate_public_url(&url)?;
    
    // Rate limiting
    crate::commands::rate_limiter::check_rate_limit_with_config(
        "fetch_subscription_content",
        crate::commands::rate_limiter::limits::IMPORT_SUBSCRIPTION,
    )?;
    
    info!(url = %url, "Fetching subscription content");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Isolate/1.0")
        .build()
        .network_context("Failed to create HTTP client")?;
    
    let response = client
        .get(&url)
        .send()
        .await
        .network_context("Failed to fetch subscription")?;
    
    if !response.status().is_success() {
        return Err(IsolateError::Network(format!(
            "HTTP {}: {}", 
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }
    
    response
        .text()
        .await
        .network_context("Failed to read response body")
}

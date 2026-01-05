//! Proxy management commands
//!
//! Commands for managing proxy configurations (SOCKS5, HTTP, Shadowsocks, Trojan, etc.)
//! Note: VLESS-specific commands are in the `vless` module.

use std::sync::Arc;
use tauri::State;
use tracing::{info, warn};

use crate::core::models::ProxyConfig;
use crate::state::AppState;

// ============================================================================
// Proxy CRUD Commands
// ============================================================================

/// Get all saved proxies
#[tauri::command]
pub async fn get_proxies(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<ProxyConfig>, String> {
    info!("Loading all proxies");
    
    state
        .storage
        .get_all_proxies()
        .await
        .map_err(|e| format!("Failed to get proxies: {}", e))
}

/// Add a new proxy
#[tauri::command]
pub async fn add_proxy(
    state: State<'_, Arc<AppState>>,
    proxy: ProxyConfig,
) -> Result<ProxyConfig, String> {
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
        .map_err(|e| format!("Failed to add proxy: {}", e))?;
    
    info!(id = %proxy.id, "Proxy added successfully");
    Ok(proxy)
}

/// Update existing proxy
#[tauri::command]
pub async fn update_proxy(
    state: State<'_, Arc<AppState>>,
    proxy: ProxyConfig,
) -> Result<(), String> {
    info!(id = %proxy.id, name = %proxy.name, "Updating proxy");
    
    state
        .storage
        .update_proxy(&proxy)
        .await
        .map_err(|e| format!("Failed to update proxy: {}", e))
}

/// Delete proxy by ID
#[tauri::command]
pub async fn delete_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
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
        .map_err(|e| format!("Failed to delete proxy: {}", e))
}

// ============================================================================
// Proxy Control Commands
// ============================================================================

/// Apply proxy (start sing-box with this proxy)
#[tauri::command]
pub async fn apply_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    info!(id = %id, "Applying proxy");
    
    // Get proxy config
    let proxy = state
        .storage
        .get_proxy(&id)
        .await
        .map_err(|e| format!("Failed to get proxy: {}", e))?
        .ok_or_else(|| format!("Proxy '{}' not found", id))?;
    
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
                .map_err(|e| format!("Failed to start proxy: {}", e))?;
            
            // Mark as active in storage
            state
                .storage
                .set_proxy_active(&id, true)
                .await
                .map_err(|e| format!("Failed to set proxy active: {}", e))?;
            
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
                .map_err(|e| format!("Failed to set proxy active: {}", e))?;
            
            info!(id = %id, "Direct proxy marked as active");
            Ok(())
        }
        _ => Err(format!("Protocol {:?} is not supported for apply", proxy.protocol)),
    }
}

/// Test proxy connectivity
#[tauri::command]
pub async fn test_proxy(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<u32, String> {
    info!(id = %id, "Testing proxy connectivity");
    
    // Get proxy config
    let proxy = state
        .storage
        .get_proxy(&id)
        .await
        .map_err(|e| format!("Failed to get proxy: {}", e))?
        .ok_or_else(|| format!("Proxy '{}' not found", id))?;
    
    let manager = crate::core::singbox_manager::get_manager();
    
    // Check if already running
    if let Some(socks_port) = manager.get_socks_port(&id).await {
        // Test existing connection
        return crate::core::vless_engine::test_proxy_connectivity(socks_port, "https://www.google.com")
            .await
            .map_err(|e| format!("Connectivity test failed: {}", e));
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
            manager
                .start(&vless_config, port)
                .await
                .map_err(|e| format!("Failed to start proxy for testing: {}", e))?;
            
            // Wait for proxy to initialize
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            
            // Test connectivity
            let result = crate::core::vless_engine::test_proxy_connectivity(port, "https://www.google.com")
                .await;
            
            // Stop test proxy
            let _ = manager.stop(&format!("test_{}", proxy.id)).await;
            
            result.map_err(|e| format!("Connectivity test failed: {}", e))
        }
        crate::core::models::ProxyProtocol::Socks5 => {
            // Test SOCKS5 directly
            let start = std::time::Instant::now();
            let addr = format!("{}:{}", proxy.server, proxy.port);
            
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                tokio::net::TcpStream::connect(&addr)
            ).await {
                Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u32),
                Ok(Err(e)) => Err(format!("Connection failed: {}", e)),
                Err(_) => Err("Connection timeout".to_string()),
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
            
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy URL: {}", e))?)
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| format!("Failed to create client: {}", e))?;
            
            client
                .get("https://www.google.com")
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;
            
            Ok(start.elapsed().as_millis() as u32)
        }
        _ => Err(format!("Protocol {:?} is not supported for testing", proxy.protocol)),
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
) -> Result<ProxyConfig, String> {
    info!("Importing proxy from URL");
    
    let proxy = crate::core::proxy_parser::parse_proxy_url(&url)
        .map_err(|e| format!("Failed to parse proxy URL: {}", e))?;
    
    // Save to storage
    state
        .storage
        .save_proxy(&proxy)
        .await
        .map_err(|e| format!("Failed to save proxy: {}", e))?;
    
    info!(id = %proxy.id, name = %proxy.name, protocol = ?proxy.protocol, "Proxy imported from URL");
    Ok(proxy)
}

/// Import subscription (multiple proxies from URL)
#[tauri::command]
pub async fn import_subscription(
    state: State<'_, Arc<AppState>>,
    url: String,
) -> Result<Vec<ProxyConfig>, String> {
    info!(url = %url, "Importing subscription");
    
    // Fetch subscription content
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch subscription: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Subscription request failed with status: {}", response.status()));
    }
    
    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read subscription content: {}", e))?;
    
    // Parse subscription
    let proxies = crate::core::proxy_parser::parse_subscription(&content)
        .map_err(|e| format!("Failed to parse subscription: {}", e))?;
    
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
pub async fn parse_proxy_url(url: String) -> Result<ProxyConfig, String> {
    info!("Parsing proxy URL for preview");
    
    crate::core::proxy_parser::parse_proxy_url(&url)
        .map_err(|e| format!("Failed to parse proxy URL: {}", e))
}

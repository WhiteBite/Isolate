//! Proxy Chain management commands
//!
//! Commands for managing proxy chains - ordered sequences of proxies
//! for traffic routing through multiple hops.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tracing::{info, warn};

use crate::commands::get_state_or_error;
use crate::core::errors::IsolateError;
use crate::core::models::ProxyProtocol;
use crate::state::AppState;

// ============================================================================
// Data Structures
// ============================================================================

/// Parsed proxy information from URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedProxy {
    /// Protocol type (socks5, http, vless, etc.)
    pub protocol: String,
    /// Server hostname or IP
    pub host: String,
    /// Server port
    pub port: u16,
    /// Username for authentication (optional)
    pub username: Option<String>,
    /// Password for authentication (optional)
    pub password: Option<String>,
    /// Country code (optional, from URL fragment or metadata)
    pub country: Option<String>,
    /// Display name (optional)
    pub name: Option<String>,
}

/// Proxy chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyChain {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Ordered list of proxy IDs in the chain
    pub proxies: Vec<String>,
    /// Whether this chain is currently active
    pub is_active: bool,
    /// Creation timestamp (Unix milliseconds)
    pub created_at: i64,
}

/// Result of batch proxy import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of successfully imported proxies
    pub success_count: u32,
    /// Number of failed imports
    pub failed_count: u32,
    /// Error messages for failed imports
    pub errors: Vec<String>,
    /// Successfully imported proxy IDs
    pub imported_ids: Vec<String>,
}

// ============================================================================
// Proxy Parsing Commands
// ============================================================================

/// Parse a single proxy URL and return parsed information
/// 
/// Supports formats:
/// - socks5://user:pass@host:port
/// - http://user:pass@host:port
/// - vless://uuid@host:port?params#name
/// - vmess://base64
/// - ss://base64@host:port#name
/// - trojan://password@host:port?params#name
#[tauri::command]
pub async fn parse_proxy_url_info(url: String) -> Result<ParsedProxy, IsolateError> {
    info!("Parsing proxy URL for info");
    
    let url = url.trim();
    
    // Use existing proxy_parser for full parsing
    let config = crate::core::proxy_parser::parse_proxy_url(url)
        .map_err(|e| IsolateError::Config(format!("Failed to parse proxy URL: {}", e)))?;
    
    // Convert to ParsedProxy
    let protocol = match config.protocol {
        ProxyProtocol::Socks5 => "socks5",
        ProxyProtocol::Http => "http",
        ProxyProtocol::Https => "https",
        ProxyProtocol::Vless => "vless",
        ProxyProtocol::Vmess => "vmess",
        ProxyProtocol::Shadowsocks => "shadowsocks",
        ProxyProtocol::Trojan => "trojan",
        ProxyProtocol::Tuic => "tuic",
        ProxyProtocol::Hysteria => "hysteria",
        ProxyProtocol::Hysteria2 => "hysteria2",
        ProxyProtocol::Wireguard => "wireguard",
        ProxyProtocol::Ssh => "ssh",
    };
    
    // Try to extract country from name or custom fields
    let country = config.custom_fields.get("country").cloned()
        .or_else(|| extract_country_from_name(&config.name));
    
    Ok(ParsedProxy {
        protocol: protocol.to_string(),
        host: config.server,
        port: config.port,
        username: config.username,
        password: config.password,
        country,
        name: Some(config.name),
    })
}

/// Batch import proxies from multiple URLs
/// 
/// Returns import statistics and list of successfully imported proxy IDs
#[tauri::command]
pub async fn batch_import_proxies(
    state: State<'_, Arc<AppState>>,
    urls: Vec<String>,
) -> Result<ImportResult, IsolateError> {
    info!(count = urls.len(), "Batch importing proxies");
    
    let mut success_count = 0u32;
    let mut failed_count = 0u32;
    let mut errors = Vec::new();
    let mut imported_ids = Vec::new();
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() || url.starts_with('#') || url.starts_with("//") {
            continue;
        }
        
        match crate::core::proxy_parser::parse_proxy_url(url) {
            Ok(config) => {
                let proxy_id = config.id.clone();
                match state.storage.save_proxy(&config).await {
                    Ok(_) => {
                        info!(id = %proxy_id, name = %config.name, "Proxy imported successfully");
                        success_count += 1;
                        imported_ids.push(proxy_id);
                    }
                    Err(e) => {
                        warn!(url = %truncate_url(url), error = %e, "Failed to save proxy");
                        failed_count += 1;
                        errors.push(format!("Failed to save '{}': {}", truncate_url(url), e));
                    }
                }
            }
            Err(e) => {
                warn!(url = %truncate_url(url), error = %e, "Failed to parse proxy URL");
                failed_count += 1;
                errors.push(format!("Failed to parse '{}': {}", truncate_url(url), e));
            }
        }
    }
    
    info!(
        success = success_count,
        failed = failed_count,
        "Batch import completed"
    );
    
    Ok(ImportResult {
        success_count,
        failed_count,
        errors,
        imported_ids,
    })
}

// ============================================================================
// Proxy Chain Commands
// ============================================================================

/// Get all proxy chains
/// 
/// Returns list of all configured proxy chains (currently mock data)
#[tauri::command]
pub async fn get_proxy_chains(
    app: tauri::AppHandle,
) -> Result<Vec<ProxyChain>, IsolateError> {
    let _state = get_state_or_error(&app)?;
    
    info!("Getting all proxy chains");
    
    // TODO: Implement real storage for proxy chains
    // For now, return mock data
    Ok(vec![
        ProxyChain {
            id: "chain_default".to_string(),
            name: "Default Chain".to_string(),
            proxies: vec![],
            is_active: false,
            created_at: chrono::Utc::now().timestamp_millis(),
        },
    ])
}

/// Save or update a proxy chain
/// 
/// Creates new chain if ID doesn't exist, updates if it does
#[tauri::command]
pub async fn save_proxy_chain(
    app: tauri::AppHandle,
    chain: ProxyChain,
) -> Result<ProxyChain, IsolateError> {
    let _state = get_state_or_error(&app)?;
    
    info!(id = %chain.id, name = %chain.name, proxies = chain.proxies.len(), "Saving proxy chain");
    
    // Validate chain
    if chain.name.trim().is_empty() {
        return Err(IsolateError::Validation("Chain name cannot be empty".to_string()));
    }
    
    // Generate ID if empty
    let mut chain = chain;
    if chain.id.is_empty() {
        chain.id = format!(
            "chain_{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")
        );
    }
    
    // Set creation time if not set
    if chain.created_at == 0 {
        chain.created_at = chrono::Utc::now().timestamp_millis();
    }
    
    // TODO: Implement real storage for proxy chains
    // For now, just return the chain as-is
    
    info!(id = %chain.id, "Proxy chain saved");
    Ok(chain)
}

/// Apply a proxy chain (activate it)
/// 
/// Starts the proxy chain routing through all proxies in order
#[tauri::command]
pub async fn apply_proxy_chain(
    app: tauri::AppHandle,
    chain_id: String,
) -> Result<(), IsolateError> {
    let _state = get_state_or_error(&app)?;
    
    info!(chain_id = %chain_id, "Applying proxy chain");
    
    // Validate chain ID
    if chain_id.trim().is_empty() {
        return Err(IsolateError::Validation("Chain ID cannot be empty".to_string()));
    }
    
    // TODO: Implement real proxy chain activation
    // This would involve:
    // 1. Loading the chain configuration
    // 2. Starting sing-box with chained outbound configuration
    // 3. Updating system proxy settings if needed
    
    info!(chain_id = %chain_id, "Proxy chain applied (mock)");
    Ok(())
}

/// Delete a proxy chain
#[tauri::command]
pub async fn delete_proxy_chain(
    app: tauri::AppHandle,
    chain_id: String,
) -> Result<(), IsolateError> {
    let _state = get_state_or_error(&app)?;
    
    info!(chain_id = %chain_id, "Deleting proxy chain");
    
    // Validate chain ID
    if chain_id.trim().is_empty() {
        return Err(IsolateError::Validation("Chain ID cannot be empty".to_string()));
    }
    
    // TODO: Implement real deletion
    // 1. Stop chain if active
    // 2. Remove from storage
    
    info!(chain_id = %chain_id, "Proxy chain deleted (mock)");
    Ok(())
}

/// Deactivate a proxy chain
#[tauri::command]
pub async fn deactivate_proxy_chain(
    app: tauri::AppHandle,
    chain_id: String,
) -> Result<(), IsolateError> {
    let _state = get_state_or_error(&app)?;
    
    info!(chain_id = %chain_id, "Deactivating proxy chain");
    
    // Validate chain ID
    if chain_id.trim().is_empty() {
        return Err(IsolateError::Validation("Chain ID cannot be empty".to_string()));
    }
    
    // TODO: Implement real deactivation
    // 1. Stop sing-box chain configuration
    // 2. Restore system proxy settings
    
    info!(chain_id = %chain_id, "Proxy chain deactivated (mock)");
    Ok(())
}

/// Validate a proxy chain configuration
/// 
/// Checks that all proxy IDs exist and chain is valid
#[tauri::command]
pub async fn validate_proxy_chain(
    state: State<'_, Arc<AppState>>,
    chain: ProxyChain,
) -> Result<Vec<String>, IsolateError> {
    info!(id = %chain.id, "Validating proxy chain");
    
    let mut warnings = Vec::new();
    
    // Check chain name
    if chain.name.trim().is_empty() {
        warnings.push("Chain name is empty".to_string());
    }
    
    // Check proxies exist
    for proxy_id in &chain.proxies {
        match state.storage.get_proxy(proxy_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                warnings.push(format!("Proxy '{}' not found", proxy_id));
            }
            Err(e) => {
                warnings.push(format!("Error checking proxy '{}': {}", proxy_id, e));
            }
        }
    }
    
    // Check chain length
    if chain.proxies.is_empty() {
        warnings.push("Chain has no proxies".to_string());
    } else if chain.proxies.len() > 10 {
        warnings.push("Chain has more than 10 proxies, this may cause performance issues".to_string());
    }
    
    Ok(warnings)
}

/// Get chain statistics
#[tauri::command]
pub async fn get_proxy_chain_stats(
    app: tauri::AppHandle,
) -> Result<HashMap<String, serde_json::Value>, IsolateError> {
    let _state = get_state_or_error(&app)?;
    
    info!("Getting proxy chain statistics");
    
    // TODO: Implement real statistics
    let mut stats = HashMap::new();
    stats.insert("total_chains".to_string(), serde_json::json!(1));
    stats.insert("active_chains".to_string(), serde_json::json!(0));
    stats.insert("total_proxies_in_chains".to_string(), serde_json::json!(0));
    
    Ok(stats)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Truncates URL for logging (hides sensitive parts)
fn truncate_url(url: &str) -> String {
    if url.len() > 50 {
        format!("{}...", &url[..50])
    } else {
        url.to_string()
    }
}

/// Try to extract country code from proxy name
/// Common patterns: "US - Server", "🇺🇸 USA", "[US]", etc.
fn extract_country_from_name(name: &str) -> Option<String> {
    // Common country code patterns
    let patterns = [
        // ISO codes at start: "US - Server"
        (r"^([A-Z]{2})\s*[-–—]", 1),
        // ISO codes in brackets: "[US] Server"
        (r"\[([A-Z]{2})\]", 1),
        // ISO codes at end: "Server - US"
        (r"[-–—]\s*([A-Z]{2})$", 1),
    ];
    
    for (pattern, group) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(name) {
                if let Some(m) = caps.get(group) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }
    
    // Check for flag emojis
    let flag_map = [
        ("🇺🇸", "US"), ("🇬🇧", "GB"), ("🇩🇪", "DE"), ("🇫🇷", "FR"),
        ("🇯🇵", "JP"), ("🇰🇷", "KR"), ("🇸🇬", "SG"), ("🇭🇰", "HK"),
        ("🇹🇼", "TW"), ("🇳🇱", "NL"), ("🇨🇦", "CA"), ("🇦🇺", "AU"),
        ("🇷🇺", "RU"), ("🇮🇳", "IN"), ("🇧🇷", "BR"), ("🇮🇹", "IT"),
    ];
    
    for (flag, code) in flag_map {
        if name.contains(flag) {
            return Some(code.to_string());
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_country_from_name() {
        assert_eq!(extract_country_from_name("US - Fast Server"), Some("US".to_string()));
        assert_eq!(extract_country_from_name("[DE] Germany Server"), Some("DE".to_string()));
        assert_eq!(extract_country_from_name("Server - JP"), Some("JP".to_string()));
        assert_eq!(extract_country_from_name("🇺🇸 USA Server"), Some("US".to_string()));
        assert_eq!(extract_country_from_name("Random Server"), None);
    }

    #[test]
    fn test_truncate_url() {
        let short = "socks5://host:1080";
        assert_eq!(truncate_url(short), short);
        
        let long = "vless://very-long-uuid-here@very-long-server-hostname.example.com:443?security=tls&sni=example.com#VeryLongServerName";
        assert!(truncate_url(long).len() <= 53); // 50 + "..."
        assert!(truncate_url(long).ends_with("..."));
    }

    #[test]
    fn test_proxy_chain_default() {
        let chain = ProxyChain {
            id: "test".to_string(),
            name: "Test Chain".to_string(),
            proxies: vec!["proxy1".to_string(), "proxy2".to_string()],
            is_active: false,
            created_at: 0,
        };
        
        assert_eq!(chain.proxies.len(), 2);
        assert!(!chain.is_active);
    }

    #[test]
    fn test_import_result() {
        let result = ImportResult {
            success_count: 5,
            failed_count: 2,
            errors: vec!["Error 1".to_string()],
            imported_ids: vec!["id1".to_string(), "id2".to_string()],
        };
        
        assert_eq!(result.success_count, 5);
        assert_eq!(result.failed_count, 2);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.imported_ids.len(), 2);
    }
}

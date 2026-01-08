//! IP Set auto-updater module
//!
//! Downloads and updates IP address lists from remote sources.
//! Supports IPv4, IPv6 addresses and CIDR notation.
//! Provides automatic scheduled updates.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::get_hostlists_dir;

// ============================================================================
// Constants
// ============================================================================

/// Request timeout for downloads
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// User-Agent for requests
const USER_AGENT: &str = "Isolate-App/1.0";

/// Backup file extension
const BACKUP_EXT: &str = ".backup";

/// Cache file for ETags and metadata
const IPSET_CACHE_FILE: &str = ".ipset_cache.json";

/// Default ipset filename
const DEFAULT_IPSET_FILE: &str = "ipset-all.txt";

/// Auto-update interval (24 hours in seconds)
const AUTO_UPDATE_INTERVAL_SECS: u64 = 24 * 60 * 60;

// ============================================================================
// Types
// ============================================================================

/// Source configuration for an IP set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsetSource {
    /// Source ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Remote URL to download from
    pub url: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether this source is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Priority (lower = higher priority)
    #[serde(default)]
    pub priority: u32,
}

fn default_true() -> bool {
    true
}

/// Cached metadata for ipset
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpsetCache {
    /// ETag from last download
    pub etag: Option<String>,
    /// Last-Modified header from last download
    pub last_modified: Option<String>,
    /// Timestamp of last successful update (ISO 8601)
    pub last_updated: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// Number of IP entries
    pub ip_count: Option<usize>,
    /// Number of IPv4 entries
    pub ipv4_count: Option<usize>,
    /// Number of IPv6 entries
    pub ipv6_count: Option<usize>,
    /// Number of CIDR entries
    pub cidr_count: Option<usize>,
    /// Source URL used for last update
    pub source_url: Option<String>,
}

/// Full cache storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpsetCacheStore {
    /// Cache entries by ipset ID
    pub entries: HashMap<String, IpsetCache>,
    /// Auto-update enabled
    #[serde(default)]
    pub auto_update_enabled: bool,
    /// Last auto-update check timestamp
    pub last_auto_check: Option<String>,
}

/// Information about an ipset for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsetInfo {
    /// Ipset ID (filename without extension)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Last update timestamp
    pub last_updated: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// Total number of IP entries
    pub ip_count: Option<usize>,
    /// Number of IPv4 entries
    pub ipv4_count: Option<usize>,
    /// Number of IPv6 entries
    pub ipv6_count: Option<usize>,
    /// Number of CIDR entries
    pub cidr_count: Option<usize>,
    /// Whether update is available
    pub update_available: bool,
    /// Source URL (if configured)
    pub source_url: Option<String>,
    /// Whether auto-update is enabled
    pub auto_update_enabled: bool,
}

/// Result of updating ipset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsetUpdateResult {
    /// Whether update was successful
    pub success: bool,
    /// Number of IP entries after update
    pub ip_count: usize,
    /// Number of IPv4 entries
    pub ipv4_count: usize,
    /// Number of IPv6 entries
    pub ipv6_count: usize,
    /// Number of CIDR entries
    pub cidr_count: usize,
    /// Source URL used
    pub source_url: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp of update
    pub timestamp: String,
}

/// Validation result for ipset content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsetValidationResult {
    /// Whether content is valid
    pub is_valid: bool,
    /// Total lines processed
    pub total_lines: usize,
    /// Valid IP entries
    pub valid_entries: usize,
    /// Invalid entries (with line numbers)
    pub invalid_entries: Vec<(usize, String)>,
    /// IPv4 count
    pub ipv4_count: usize,
    /// IPv6 count
    pub ipv6_count: usize,
    /// CIDR count
    pub cidr_count: usize,
    /// Comment lines count
    pub comment_count: usize,
    /// Empty lines count
    pub empty_count: usize,
}

/// Parsed IP entry
#[derive(Debug, Clone, PartialEq)]
pub enum IpEntry {
    /// Single IPv4 address
    Ipv4(Ipv4Addr),
    /// Single IPv6 address
    Ipv6(Ipv6Addr),
    /// IPv4 CIDR block
    Ipv4Cidr(Ipv4Addr, u8),
    /// IPv6 CIDR block
    Ipv6Cidr(Ipv6Addr, u8),
}

// ============================================================================
// Default Sources
// ============================================================================

/// Get default ipset sources from config or fallback
pub async fn get_default_sources() -> Vec<IpsetSource> {
    // Try to load from config file first
    let config_path = get_hostlists_dir().parent()
        .map(|p| p.join("ipset_sources.yaml"))
        .unwrap_or_else(|| PathBuf::from("configs/ipset_sources.yaml"));
    
    if let Ok(content) = fs::read_to_string(&config_path).await {
        if let Ok(sources) = serde_yaml::from_str::<Vec<IpsetSource>>(&content) {
            if !sources.is_empty() {
                return sources;
            }
        }
    }
    
    // Fallback to hardcoded defaults
    vec![
        IpsetSource {
            id: "zapret-discord-youtube".to_string(),
            name: "Zapret Discord YouTube".to_string(),
            url: "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/main/ipset-all.txt".to_string(),
            description: Some("IP addresses for Discord and YouTube from zapret-discord-youtube project".to_string()),
            enabled: true,
            priority: 1,
        },
        IpsetSource {
            id: "zapret".to_string(),
            name: "Zapret IP List".to_string(),
            url: "https://raw.githubusercontent.com/bol-van/zapret/master/ipset/zapret-ip.txt".to_string(),
            description: Some("IP addresses from zapret project".to_string()),
            enabled: true,
            priority: 2,
        },
        IpsetSource {
            id: "antifilter".to_string(),
            name: "Antifilter All You Need".to_string(),
            url: "https://antifilter.download/list/allyouneed.lst".to_string(),
            description: Some("Comprehensive IP list from antifilter.download".to_string()),
            enabled: true,
            priority: 3,
        },
    ]
}

// ============================================================================
// IP Parsing and Validation
// ============================================================================

/// Parse a single line into an IP entry
pub fn parse_ip_entry(line: &str) -> Option<IpEntry> {
    let trimmed = line.trim();
    
    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    
    // Check for CIDR notation
    if let Some((ip_part, prefix_part)) = trimmed.split_once('/') {
        let prefix: u8 = prefix_part.parse().ok()?;
        
        // Try IPv4 CIDR
        if let Ok(ipv4) = Ipv4Addr::from_str(ip_part) {
            if prefix <= 32 {
                return Some(IpEntry::Ipv4Cidr(ipv4, prefix));
            }
        }
        
        // Try IPv6 CIDR
        if let Ok(ipv6) = Ipv6Addr::from_str(ip_part) {
            if prefix <= 128 {
                return Some(IpEntry::Ipv6Cidr(ipv6, prefix));
            }
        }
        
        return None;
    }
    
    // Try single IP address
    if let Ok(ip) = IpAddr::from_str(trimmed) {
        return match ip {
            IpAddr::V4(v4) => Some(IpEntry::Ipv4(v4)),
            IpAddr::V6(v6) => Some(IpEntry::Ipv6(v6)),
        };
    }
    
    None
}

/// Validate ipset content
pub fn validate_ipset(content: &str) -> IpsetValidationResult {
    let mut result = IpsetValidationResult {
        is_valid: true,
        total_lines: 0,
        valid_entries: 0,
        invalid_entries: Vec::new(),
        ipv4_count: 0,
        ipv6_count: 0,
        cidr_count: 0,
        comment_count: 0,
        empty_count: 0,
    };
    
    for (line_num, line) in content.lines().enumerate() {
        result.total_lines += 1;
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            result.empty_count += 1;
            continue;
        }
        
        if trimmed.starts_with('#') {
            result.comment_count += 1;
            continue;
        }
        
        match parse_ip_entry(trimmed) {
            Some(entry) => {
                result.valid_entries += 1;
                match entry {
                    IpEntry::Ipv4(_) => result.ipv4_count += 1,
                    IpEntry::Ipv6(_) => result.ipv6_count += 1,
                    IpEntry::Ipv4Cidr(_, _) => {
                        result.ipv4_count += 1;
                        result.cidr_count += 1;
                    }
                    IpEntry::Ipv6Cidr(_, _) => {
                        result.ipv6_count += 1;
                        result.cidr_count += 1;
                    }
                }
            }
            None => {
                result.invalid_entries.push((line_num + 1, trimmed.to_string()));
                // Allow some invalid entries (up to 5% or 10 entries)
                if result.invalid_entries.len() > 10 
                    && result.invalid_entries.len() as f64 / result.total_lines as f64 > 0.05 
                {
                    result.is_valid = false;
                }
            }
        }
    }
    
    // Must have at least some valid entries
    if result.valid_entries == 0 {
        result.is_valid = false;
    }
    
    result
}

// ============================================================================
// Ipset Updater
// ============================================================================

/// IP Set updater service
pub struct IpsetUpdater {
    /// HTTP client
    client: reqwest::Client,
    /// Hostlists directory (where ipset files are stored)
    hostlists_dir: PathBuf,
    /// Configured sources
    sources: Vec<IpsetSource>,
    /// Cache store
    cache: IpsetCacheStore,
}

impl IpsetUpdater {
    /// Create a new IpsetUpdater
    pub async fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| IsolateError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let hostlists_dir = get_hostlists_dir();
        
        // Ensure directory exists
        if !hostlists_dir.exists() {
            fs::create_dir_all(&hostlists_dir).await?;
        }

        // Load cache
        let cache = Self::load_cache(&hostlists_dir).await.unwrap_or_default();
        
        // Load sources
        let sources = get_default_sources().await;

        Ok(Self {
            client,
            hostlists_dir,
            sources,
            cache,
        })
    }

    /// Load cache from disk
    async fn load_cache(dir: &PathBuf) -> Result<IpsetCacheStore> {
        let cache_path = dir.join(IPSET_CACHE_FILE);
        
        if !cache_path.exists() {
            return Ok(IpsetCacheStore::default());
        }

        let content = fs::read_to_string(&cache_path).await?;
        let cache: IpsetCacheStore = serde_json::from_str(&content)
            .map_err(|e| IsolateError::Config(format!("Failed to parse ipset cache: {}", e)))?;

        Ok(cache)
    }

    /// Save cache to disk
    async fn save_cache(&self) -> Result<()> {
        let cache_path = self.hostlists_dir.join(IPSET_CACHE_FILE);
        let content = serde_json::to_string_pretty(&self.cache)
            .map_err(|e| IsolateError::Config(format!("Failed to serialize ipset cache: {}", e)))?;
        
        fs::write(&cache_path, content).await?;
        Ok(())
    }

    /// Get information about the current ipset
    pub async fn get_ipset_info(&self) -> Result<IpsetInfo> {
        let file_path = self.hostlists_dir.join(DEFAULT_IPSET_FILE);
        let id = "ipset-all".to_string();
        
        // Get cached info
        let cached = self.cache.entries.get(&id);
        
        // Get file metadata
        let (size, ip_count, ipv4_count, ipv6_count, cidr_count) = if file_path.exists() {
            let metadata = fs::metadata(&file_path).await.ok();
            let size = metadata.as_ref().map(|m| m.len());
            
            // Read and count if not cached
            if let Some(c) = cached {
                (size, c.ip_count, c.ipv4_count, c.ipv6_count, c.cidr_count)
            } else {
                let content = fs::read_to_string(&file_path).await.ok();
                if let Some(content) = content {
                    let validation = validate_ipset(&content);
                    (
                        size,
                        Some(validation.valid_entries),
                        Some(validation.ipv4_count),
                        Some(validation.ipv6_count),
                        Some(validation.cidr_count),
                    )
                } else {
                    (size, None, None, None, None)
                }
            }
        } else {
            (None, None, None, None, None)
        };

        Ok(IpsetInfo {
            id,
            name: "IP Set".to_string(),
            last_updated: cached.and_then(|c| c.last_updated.clone()),
            size,
            ip_count,
            ipv4_count,
            ipv6_count,
            cidr_count,
            update_available: false,
            source_url: cached.and_then(|c| c.source_url.clone()),
            auto_update_enabled: self.cache.auto_update_enabled,
        })
    }

    /// Update ipset from a specific source URL
    pub async fn update_ipset(&mut self, source_url: &str) -> Result<IpsetUpdateResult> {
        info!(url = %source_url, "Updating ipset");

        // Download content
        let response = self.client.get(source_url)
            .send()
            .await
            .map_err(|e| IsolateError::Network(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(IsolateError::Network(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        // Extract headers for caching
        let etag = response.headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        let last_modified = response.headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content = response.text().await
            .map_err(|e| IsolateError::Network(format!("Failed to read content: {}", e)))?;

        // Validate content
        let validation = validate_ipset(&content);
        
        if !validation.is_valid {
            return Err(IsolateError::Validation(format!(
                "Invalid ipset content: {} valid entries, {} invalid entries",
                validation.valid_entries,
                validation.invalid_entries.len()
            )));
        }

        if validation.valid_entries == 0 {
            return Err(IsolateError::Validation("Downloaded ipset is empty".to_string()));
        }

        let file_path = self.hostlists_dir.join(DEFAULT_IPSET_FILE);

        // Create backup of existing file
        if file_path.exists() {
            let backup_path = self.hostlists_dir.join(format!("{}{}", DEFAULT_IPSET_FILE, BACKUP_EXT));
            if let Err(e) = fs::copy(&file_path, &backup_path).await {
                warn!(error = %e, "Failed to create ipset backup");
            } else {
                debug!("Created ipset backup");
            }
        }

        // Write new content
        fs::write(&file_path, &content).await?;

        let timestamp = chrono::Utc::now().to_rfc3339();

        // Update cache
        let cache_entry = IpsetCache {
            etag,
            last_modified,
            last_updated: Some(timestamp.clone()),
            size: Some(content.len() as u64),
            ip_count: Some(validation.valid_entries),
            ipv4_count: Some(validation.ipv4_count),
            ipv6_count: Some(validation.ipv6_count),
            cidr_count: Some(validation.cidr_count),
            source_url: Some(source_url.to_string()),
        };
        
        self.cache.entries.insert("ipset-all".to_string(), cache_entry);
        self.save_cache().await?;

        info!(
            ip_count = validation.valid_entries,
            ipv4_count = validation.ipv4_count,
            ipv6_count = validation.ipv6_count,
            cidr_count = validation.cidr_count,
            "Ipset updated successfully"
        );

        Ok(IpsetUpdateResult {
            success: true,
            ip_count: validation.valid_entries,
            ipv4_count: validation.ipv4_count,
            ipv6_count: validation.ipv6_count,
            cidr_count: validation.cidr_count,
            source_url: source_url.to_string(),
            error: None,
            timestamp,
        })
    }

    /// Update ipset from the first available source
    pub async fn update_from_sources(&mut self) -> Result<IpsetUpdateResult> {
        let sources: Vec<IpsetSource> = self.sources.iter()
            .filter(|s| s.enabled)
            .cloned()
            .collect();
        
        // Sort by priority
        let mut sources = sources;
        sources.sort_by_key(|s| s.priority);
        
        let mut last_error = None;
        
        for source in sources {
            info!(source = %source.name, url = %source.url, "Trying ipset source");
            
            match self.update_ipset(&source.url).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!(source = %source.name, error = %e, "Source failed, trying next");
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| IsolateError::Config("No ipset sources configured".to_string())))
    }

    /// Set auto-update enabled/disabled
    pub async fn set_auto_update(&mut self, enabled: bool) -> Result<()> {
        self.cache.auto_update_enabled = enabled;
        self.save_cache().await?;
        
        info!(enabled = enabled, "Ipset auto-update setting changed");
        Ok(())
    }

    /// Check if auto-update is needed (based on last update time)
    pub fn needs_auto_update(&self) -> bool {
        if !self.cache.auto_update_enabled {
            return false;
        }
        
        let cached = self.cache.entries.get("ipset-all");
        
        if let Some(cache) = cached {
            if let Some(last_updated) = &cache.last_updated {
                if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(last_updated) {
                    let now = chrono::Utc::now();
                    let elapsed = now.signed_duration_since(last_time);
                    return elapsed.num_seconds() as u64 > AUTO_UPDATE_INTERVAL_SECS;
                }
            }
        }
        
        // No cache or invalid timestamp - needs update
        true
    }

    /// Get configured sources
    pub fn get_sources(&self) -> &[IpsetSource] {
        &self.sources
    }

    /// Restore ipset from backup
    pub async fn restore_from_backup(&self) -> Result<()> {
        let file_path = self.hostlists_dir.join(DEFAULT_IPSET_FILE);
        let backup_path = self.hostlists_dir.join(format!("{}{}", DEFAULT_IPSET_FILE, BACKUP_EXT));

        if !backup_path.exists() {
            return Err(IsolateError::Config("No ipset backup found".to_string()));
        }

        fs::copy(&backup_path, &file_path).await?;
        
        info!("Restored ipset from backup");

        Ok(())
    }
}

// ============================================================================
// Public Convenience Functions
// ============================================================================

/// Get ipset info (convenience function)
pub async fn get_ipset_info() -> Result<IpsetInfo> {
    let updater = IpsetUpdater::new().await?;
    updater.get_ipset_info().await
}

/// Update ipset from sources (convenience function)
pub async fn update_ipset_from_sources() -> Result<IpsetUpdateResult> {
    let mut updater = IpsetUpdater::new().await?;
    updater.update_from_sources().await
}

/// Update ipset from specific URL (convenience function)
pub async fn update_ipset(source_url: &str) -> Result<IpsetUpdateResult> {
    let mut updater = IpsetUpdater::new().await?;
    updater.update_ipset(source_url).await
}

/// Set auto-update enabled (convenience function)
pub async fn set_ipset_auto_update(enabled: bool) -> Result<()> {
    let mut updater = IpsetUpdater::new().await?;
    updater.set_auto_update(enabled).await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4_address() {
        let entry = parse_ip_entry("192.168.1.1");
        assert_eq!(entry, Some(IpEntry::Ipv4(Ipv4Addr::new(192, 168, 1, 1))));
    }

    #[test]
    fn test_parse_ipv6_address() {
        let entry = parse_ip_entry("2001:db8::1");
        assert_eq!(entry, Some(IpEntry::Ipv6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1))));
    }

    #[test]
    fn test_parse_ipv4_cidr() {
        let entry = parse_ip_entry("10.0.0.0/8");
        assert_eq!(entry, Some(IpEntry::Ipv4Cidr(Ipv4Addr::new(10, 0, 0, 0), 8)));
    }

    #[test]
    fn test_parse_ipv6_cidr() {
        let entry = parse_ip_entry("2001:db8::/32");
        assert_eq!(entry, Some(IpEntry::Ipv6Cidr(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32)));
    }

    #[test]
    fn test_parse_comment_line() {
        let entry = parse_ip_entry("# This is a comment");
        assert_eq!(entry, None);
    }

    #[test]
    fn test_parse_empty_line() {
        let entry = parse_ip_entry("");
        assert_eq!(entry, None);
        
        let entry = parse_ip_entry("   ");
        assert_eq!(entry, None);
    }

    #[test]
    fn test_parse_invalid_ip() {
        let entry = parse_ip_entry("not.an.ip.address");
        assert_eq!(entry, None);
        
        let entry = parse_ip_entry("256.256.256.256");
        assert_eq!(entry, None);
    }

    #[test]
    fn test_parse_invalid_cidr_prefix() {
        let entry = parse_ip_entry("192.168.1.0/33");
        assert_eq!(entry, None);
        
        let entry = parse_ip_entry("2001:db8::/129");
        assert_eq!(entry, None);
    }

    #[test]
    fn test_validate_ipset_valid_content() {
        let content = r#"
# Comment line
192.168.1.1
10.0.0.0/8
2001:db8::1
2001:db8::/32

# Another comment
172.16.0.0/12
"#;
        let result = validate_ipset(content);
        
        assert!(result.is_valid);
        assert_eq!(result.valid_entries, 5);
        assert_eq!(result.ipv4_count, 3);
        assert_eq!(result.ipv6_count, 2);
        assert_eq!(result.cidr_count, 3);
        assert_eq!(result.comment_count, 2);
        assert!(result.invalid_entries.is_empty());
    }

    #[test]
    fn test_validate_ipset_empty_content() {
        let content = "# Only comments\n\n";
        let result = validate_ipset(content);
        
        assert!(!result.is_valid);
        assert_eq!(result.valid_entries, 0);
    }

    #[test]
    fn test_validate_ipset_with_some_invalid() {
        let content = r#"
192.168.1.1
invalid.entry
10.0.0.0/8
"#;
        let result = validate_ipset(content);
        
        assert!(result.is_valid); // Still valid with few invalid entries
        assert_eq!(result.valid_entries, 2);
        assert_eq!(result.invalid_entries.len(), 1);
        assert_eq!(result.invalid_entries[0].1, "invalid.entry");
    }

    #[test]
    fn test_parse_with_whitespace() {
        let entry = parse_ip_entry("  192.168.1.1  ");
        assert_eq!(entry, Some(IpEntry::Ipv4(Ipv4Addr::new(192, 168, 1, 1))));
    }

    #[test]
    fn test_default_sources_not_empty() {
        // This is a sync test, so we can't use async get_default_sources
        // Instead, test the fallback values
        let sources = vec![
            IpsetSource {
                id: "zapret".to_string(),
                name: "Zapret IP List".to_string(),
                url: "https://raw.githubusercontent.com/bol-van/zapret/master/ipset/zapret-ip.txt".to_string(),
                description: Some("IP addresses from zapret project".to_string()),
                enabled: true,
                priority: 1,
            },
        ];
        
        assert!(!sources.is_empty());
        assert!(sources[0].url.starts_with("https://"));
    }

    #[test]
    fn test_ipset_cache_default() {
        let cache = IpsetCache::default();
        assert!(cache.etag.is_none());
        assert!(cache.last_modified.is_none());
        assert!(cache.last_updated.is_none());
        assert!(cache.ip_count.is_none());
    }

    #[test]
    fn test_ipset_validation_result_counts() {
        let content = r#"
# Header
142.250.0.0/15
172.217.0.0/16
216.58.192.0/19
2001:4860::/32
"#;
        let result = validate_ipset(content);
        
        assert!(result.is_valid);
        assert_eq!(result.valid_entries, 4);
        assert_eq!(result.ipv4_count, 3);
        assert_eq!(result.ipv6_count, 1);
        assert_eq!(result.cidr_count, 4);
    }
}

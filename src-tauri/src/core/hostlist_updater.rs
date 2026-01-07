//! Hostlist auto-updater module
//!
//! Downloads and updates hostlists from remote sources (GitHub).
//! Supports ETag/Last-Modified caching for efficient updates.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info, warn, error};

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
const CACHE_FILE: &str = ".hostlist_cache.json";

// ============================================================================
// Types
// ============================================================================

/// Source configuration for a hostlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostlistSource {
    /// Hostlist ID (filename without extension)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Remote URL to download from
    pub url: String,
    /// Optional description
    pub description: Option<String>,
}

/// Cached metadata for a hostlist
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostlistCache {
    /// ETag from last download
    pub etag: Option<String>,
    /// Last-Modified header from last download
    pub last_modified: Option<String>,
    /// Timestamp of last successful update (ISO 8601)
    pub last_updated: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// Number of domains
    pub domain_count: Option<usize>,
}

/// Full cache storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostlistCacheStore {
    /// Cache entries by hostlist ID
    pub entries: HashMap<String, HostlistCache>,
}

/// Information about a hostlist for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostlistInfo {
    /// Hostlist ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Last update timestamp
    pub last_updated: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// Number of domains
    pub domain_count: Option<usize>,
    /// Whether update is available
    pub update_available: bool,
    /// Source URL (if configured)
    pub source_url: Option<String>,
}

/// Result of checking for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    /// Hostlist ID
    pub id: String,
    /// Whether update is available
    pub has_update: bool,
    /// Current domain count
    pub current_count: Option<usize>,
    /// Error message if check failed
    pub error: Option<String>,
}

/// Result of updating hostlists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// Number of hostlists updated
    pub updated_count: usize,
    /// Number of hostlists that failed
    pub failed_count: usize,
    /// List of updated hostlist IDs
    pub updated: Vec<String>,
    /// List of failed hostlist IDs with errors
    pub failed: Vec<(String, String)>,
}

// ============================================================================
// Default Sources
// ============================================================================

/// Get default hostlist sources
pub fn get_default_sources() -> Vec<HostlistSource> {
    vec![
        HostlistSource {
            id: "youtube".to_string(),
            name: "YouTube".to_string(),
            url: "https://raw.githubusercontent.com/nickspaargaren/no-google/master/categories/youtubeparsed".to_string(),
            description: Some("YouTube domains for DPI bypass".to_string()),
        },
        HostlistSource {
            id: "google".to_string(),
            name: "Google".to_string(),
            url: "https://raw.githubusercontent.com/nickspaargaren/no-google/master/categories/googleparsed".to_string(),
            description: Some("Google services domains".to_string()),
        },
        // Discord - using zapret-discord-youtube list
        HostlistSource {
            id: "discord".to_string(),
            name: "Discord".to_string(),
            url: "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/main/lists/list-discord.txt".to_string(),
            description: Some("Discord domains".to_string()),
        },
        // General list from zapret-discord-youtube
        HostlistSource {
            id: "general".to_string(),
            name: "General".to_string(),
            url: "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/main/lists/list-general.txt".to_string(),
            description: Some("General blocked domains".to_string()),
        },
    ]
}

// ============================================================================
// Hostlist Updater
// ============================================================================

/// Hostlist updater service
pub struct HostlistUpdater {
    /// HTTP client
    client: reqwest::Client,
    /// Hostlists directory
    hostlists_dir: PathBuf,
    /// Configured sources
    sources: Vec<HostlistSource>,
    /// Cache store
    cache: HostlistCacheStore,
}

impl HostlistUpdater {
    /// Create a new HostlistUpdater
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

        Ok(Self {
            client,
            hostlists_dir,
            sources: get_default_sources(),
            cache,
        })
    }

    /// Load cache from disk
    async fn load_cache(dir: &PathBuf) -> Result<HostlistCacheStore> {
        let cache_path = dir.join(CACHE_FILE);
        
        if !cache_path.exists() {
            return Ok(HostlistCacheStore::default());
        }

        let content = fs::read_to_string(&cache_path).await?;
        let cache: HostlistCacheStore = serde_json::from_str(&content)
            .map_err(|e| IsolateError::Config(format!("Failed to parse cache: {}", e)))?;

        Ok(cache)
    }

    /// Save cache to disk
    async fn save_cache(&self) -> Result<()> {
        let cache_path = self.hostlists_dir.join(CACHE_FILE);
        let content = serde_json::to_string_pretty(&self.cache)
            .map_err(|e| IsolateError::Config(format!("Failed to serialize cache: {}", e)))?;
        
        fs::write(&cache_path, content).await?;
        Ok(())
    }

    /// Get information about all hostlists
    pub async fn get_hostlist_info(&self) -> Result<Vec<HostlistInfo>> {
        let mut infos = Vec::new();

        // Read all .txt files in hostlists directory
        let mut entries = fs::read_dir(&self.hostlists_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().is_some_and(|ext| ext == "txt") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let id = stem.to_string();
                    
                    // Get cached info
                    let cached = self.cache.entries.get(&id);
                    
                    // Find source URL if configured
                    let source = self.sources.iter().find(|s| s.id == id);
                    
                    // Get file metadata
                    let metadata = fs::metadata(&path).await.ok();
                    let size = metadata.as_ref().map(|m| m.len());
                    
                    // Count domains if not cached
                    let domain_count = if let Some(c) = cached.and_then(|c| c.domain_count) {
                        Some(c)
                    } else {
                        // Read and count
                        fs::read_to_string(&path).await.ok().map(|content| {
                            content.lines()
                                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                                .count()
                        })
                    };

                    infos.push(HostlistInfo {
                        id: id.clone(),
                        name: source.map(|s| s.name.clone()).unwrap_or_else(|| capitalize_first(&id)),
                        last_updated: cached.and_then(|c| c.last_updated.clone()),
                        size,
                        domain_count,
                        update_available: false, // Will be set by check_for_updates
                        source_url: source.map(|s| s.url.clone()),
                    });
                }
            }
        }

        // Sort by name
        infos.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(infos)
    }

    /// Check for updates for all configured sources
    pub async fn check_for_updates(&self) -> Vec<UpdateCheckResult> {
        let mut results = Vec::new();

        for source in &self.sources {
            let result = self.check_single_update(source).await;
            results.push(result);
        }

        results
    }

    /// Check for update for a single hostlist
    async fn check_single_update(&self, source: &HostlistSource) -> UpdateCheckResult {
        debug!(id = %source.id, url = %source.url, "Checking for hostlist update");

        let cached = self.cache.entries.get(&source.id);

        // Build request with conditional headers
        let mut request = self.client.head(&source.url);
        
        if let Some(cache) = cached {
            if let Some(etag) = &cache.etag {
                request = request.header("If-None-Match", etag);
            }
            if let Some(last_modified) = &cache.last_modified {
                request = request.header("If-Modified-Since", last_modified);
            }
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                
                if status == reqwest::StatusCode::NOT_MODIFIED {
                    // No update available
                    UpdateCheckResult {
                        id: source.id.clone(),
                        has_update: false,
                        current_count: cached.and_then(|c| c.domain_count),
                        error: None,
                    }
                } else if status.is_success() {
                    // Update available
                    UpdateCheckResult {
                        id: source.id.clone(),
                        has_update: true,
                        current_count: cached.and_then(|c| c.domain_count),
                        error: None,
                    }
                } else {
                    UpdateCheckResult {
                        id: source.id.clone(),
                        has_update: false,
                        current_count: cached.and_then(|c| c.domain_count),
                        error: Some(format!("HTTP {}", status)),
                    }
                }
            }
            Err(e) => {
                warn!(id = %source.id, error = %e, "Failed to check for update");
                UpdateCheckResult {
                    id: source.id.clone(),
                    has_update: false,
                    current_count: cached.and_then(|c| c.domain_count),
                    error: Some(e.to_string()),
                }
            }
        }
    }

    /// Update a single hostlist
    pub async fn update_hostlist(&mut self, id: &str) -> Result<()> {
        let source = self.sources.iter()
            .find(|s| s.id == id)
            .ok_or_else(|| IsolateError::Config(format!("Unknown hostlist source: {}", id)))?
            .clone();

        info!(id = %id, url = %source.url, "Updating hostlist");

        // Download content
        let response = self.client.get(&source.url)
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

        // Parse and validate domains
        let domains: Vec<String> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_lowercase())
            .collect();

        if domains.is_empty() {
            return Err(IsolateError::Config("Downloaded hostlist is empty".to_string()));
        }

        let file_path = self.hostlists_dir.join(format!("{}.txt", id));

        // Create backup of existing file
        if file_path.exists() {
            let backup_path = self.hostlists_dir.join(format!("{}.txt{}", id, BACKUP_EXT));
            if let Err(e) = fs::copy(&file_path, &backup_path).await {
                warn!(id = %id, error = %e, "Failed to create backup");
            } else {
                debug!(id = %id, "Created backup");
            }
        }

        // Write new content
        let new_content = domains.join("\n");
        fs::write(&file_path, &new_content).await?;

        // Update cache
        let cache_entry = HostlistCache {
            etag,
            last_modified,
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            size: Some(new_content.len() as u64),
            domain_count: Some(domains.len()),
        };
        
        self.cache.entries.insert(id.to_string(), cache_entry);
        self.save_cache().await?;

        info!(id = %id, domain_count = domains.len(), "Hostlist updated successfully");

        Ok(())
    }

    /// Update all hostlists with configured sources
    pub async fn update_all_hostlists(&mut self) -> UpdateResult {
        info!("Updating all hostlists");

        let mut result = UpdateResult {
            updated_count: 0,
            failed_count: 0,
            updated: Vec::new(),
            failed: Vec::new(),
        };

        let source_ids: Vec<String> = self.sources.iter().map(|s| s.id.clone()).collect();

        for id in source_ids {
            match self.update_hostlist(&id).await {
                Ok(_) => {
                    result.updated_count += 1;
                    result.updated.push(id);
                }
                Err(e) => {
                    error!(id = %id, error = %e, "Failed to update hostlist");
                    result.failed_count += 1;
                    result.failed.push((id, e.to_string()));
                }
            }
        }

        info!(
            updated = result.updated_count,
            failed = result.failed_count,
            "Hostlist update completed"
        );

        result
    }

    /// Restore hostlist from backup
    pub async fn restore_from_backup(&self, id: &str) -> Result<()> {
        let file_path = self.hostlists_dir.join(format!("{}.txt", id));
        let backup_path = self.hostlists_dir.join(format!("{}.txt{}", id, BACKUP_EXT));

        if !backup_path.exists() {
            return Err(IsolateError::Config(format!("No backup found for hostlist: {}", id)));
        }

        fs::copy(&backup_path, &file_path).await?;
        
        info!(id = %id, "Restored hostlist from backup");

        Ok(())
    }

    /// Get configured sources
    pub fn get_sources(&self) -> &[HostlistSource] {
        &self.sources
    }

    /// Add a custom source
    pub fn add_source(&mut self, source: HostlistSource) {
        // Remove existing source with same ID
        self.sources.retain(|s| s.id != source.id);
        self.sources.push(source);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

// ============================================================================
// Public Convenience Functions
// ============================================================================

/// Check for hostlist updates (convenience function)
pub async fn check_hostlist_updates() -> Result<Vec<UpdateCheckResult>> {
    let updater = HostlistUpdater::new().await?;
    Ok(updater.check_for_updates().await)
}

/// Update all hostlists (convenience function)
pub async fn update_all_hostlists() -> Result<UpdateResult> {
    let mut updater = HostlistUpdater::new().await?;
    Ok(updater.update_all_hostlists().await)
}

/// Get hostlist info (convenience function)
pub async fn get_hostlist_info() -> Result<Vec<HostlistInfo>> {
    let updater = HostlistUpdater::new().await?;
    updater.get_hostlist_info().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("discord"), "Discord");
        assert_eq!(capitalize_first("youtube"), "Youtube");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("a"), "A");
        assert_eq!(capitalize_first("ABC"), "ABC");
    }

    #[test]
    fn test_default_sources() {
        let sources = get_default_sources();
        assert!(!sources.is_empty());
        
        // Check that all sources have valid URLs
        for source in &sources {
            assert!(!source.id.is_empty());
            assert!(!source.name.is_empty());
            assert!(source.url.starts_with("https://"));
        }
    }

    #[test]
    fn test_hostlist_cache_default() {
        let cache = HostlistCache::default();
        assert!(cache.etag.is_none());
        assert!(cache.last_modified.is_none());
        assert!(cache.last_updated.is_none());
    }
}

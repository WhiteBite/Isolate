//! Hostlist Registry for Plugin System
//!
//! Manages hostlists contributed by plugins. Provides registration,
//! lookup, and domain retrieval functionality.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::manifest::{HostlistDefinition, HostlistFormat};

/// Error types for hostlist registry operations
#[derive(Debug, thiserror::Error)]
pub enum HostlistRegistryError {
    #[error("Hostlist not found: {0}")]
    NotFound(String),
    
    #[error("Hostlist already registered: {0}")]
    AlreadyExists(String),
    
    #[error("Failed to load hostlist file: {0}")]
    FileError(String),
    
    #[error("Failed to fetch remote hostlist: {0}")]
    NetworkError(String),
    
    #[error("Invalid hostlist format: {0}")]
    FormatError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HostlistRegistryError>;

/// Registered hostlist with resolved domains
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegisteredHostlist {
    /// Original definition from plugin
    pub definition: HostlistDefinition,
    /// Plugin ID that registered this hostlist
    pub plugin_id: String,
    /// Resolved domains (loaded from file/inline/remote)
    pub domains: Vec<String>,
    /// Path to the plugin directory (for resolving relative file paths)
    pub plugin_path: PathBuf,
    /// Last update timestamp
    pub updated_at: Option<String>,
    /// Whether the hostlist is enabled
    pub enabled: bool,
}

impl RegisteredHostlist {
    /// Get unique domains as a HashSet
    pub fn unique_domains(&self) -> HashSet<&str> {
        self.domains.iter().map(|s| s.as_str()).collect()
    }
    
    /// Get domain count
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }
    
    /// Check if a domain matches this hostlist
    pub fn matches(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        
        match self.definition.format {
            HostlistFormat::Plain => {
                self.domains.iter().any(|d| d == &domain_lower)
            }
            HostlistFormat::Wildcard => {
                self.domains.iter().any(|pattern| {
                    wildcard_match(pattern, &domain_lower)
                })
            }
            HostlistFormat::Regex => {
                // For regex format, try simple contains match as fallback
                // Full regex support requires adding regex crate
                self.domains.iter().any(|pattern| {
                    domain_lower.contains(pattern) || pattern.contains(&domain_lower)
                })
            }
        }
    }
}

/// Hostlist Registry - manages all registered hostlists
pub struct HostlistRegistry {
    /// Registered hostlists by ID
    hostlists: RwLock<HashMap<String, RegisteredHostlist>>,
}

impl HostlistRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            hostlists: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a hostlist from a plugin
    ///
    /// # Arguments
    /// * `plugin_id` - ID of the plugin registering the hostlist
    /// * `plugin_path` - Path to the plugin directory
    /// * `definition` - Hostlist definition from plugin manifest
    ///
    /// # Returns
    /// * `Ok(())` - Successfully registered
    /// * `Err(HostlistRegistryError)` - If registration fails
    pub async fn register(
        &self,
        plugin_id: &str,
        plugin_path: PathBuf,
        definition: HostlistDefinition,
    ) -> Result<()> {
        let hostlist_id = definition.id.clone();
        
        // Check if already registered
        {
            let hostlists = self.hostlists.read().await;
            if hostlists.contains_key(&hostlist_id) {
                return Err(HostlistRegistryError::AlreadyExists(hostlist_id));
            }
        }
        
        // Load domains
        let domains = self.load_domains(&definition, &plugin_path).await?;
        
        let registered = RegisteredHostlist {
            definition,
            plugin_id: plugin_id.to_string(),
            domains,
            plugin_path,
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
            enabled: true,
        };
        
        let mut hostlists = self.hostlists.write().await;
        hostlists.insert(hostlist_id.clone(), registered);
        
        info!(
            hostlist_id = %hostlist_id,
            plugin_id = %plugin_id,
            "Registered hostlist"
        );
        
        Ok(())
    }
    
    /// Unregister a hostlist
    ///
    /// # Arguments
    /// * `hostlist_id` - ID of the hostlist to unregister
    ///
    /// # Returns
    /// * `Ok(())` - Successfully unregistered
    /// * `Err(HostlistRegistryError::NotFound)` - If hostlist not found
    pub async fn unregister(&self, hostlist_id: &str) -> Result<()> {
        let mut hostlists = self.hostlists.write().await;
        
        if hostlists.remove(hostlist_id).is_some() {
            info!(hostlist_id = %hostlist_id, "Unregistered hostlist");
            Ok(())
        } else {
            Err(HostlistRegistryError::NotFound(hostlist_id.to_string()))
        }
    }
    
    /// Unregister all hostlists from a specific plugin
    pub async fn unregister_plugin(&self, plugin_id: &str) -> usize {
        let mut hostlists = self.hostlists.write().await;
        let before = hostlists.len();
        
        hostlists.retain(|_, h| h.plugin_id != plugin_id);
        
        let removed = before - hostlists.len();
        if removed > 0 {
            info!(
                plugin_id = %plugin_id,
                count = removed,
                "Unregistered hostlists from plugin"
            );
        }
        
        removed
    }
    
    /// Get a hostlist by ID
    pub async fn get(&self, hostlist_id: &str) -> Option<RegisteredHostlist> {
        let hostlists = self.hostlists.read().await;
        hostlists.get(hostlist_id).cloned()
    }
    
    /// List all registered hostlists
    pub async fn list(&self) -> Vec<RegisteredHostlist> {
        let hostlists = self.hostlists.read().await;
        hostlists.values().cloned().collect()
    }
    
    /// List hostlists by plugin ID
    pub async fn list_by_plugin(&self, plugin_id: &str) -> Vec<RegisteredHostlist> {
        let hostlists = self.hostlists.read().await;
        hostlists
            .values()
            .filter(|h| h.plugin_id == plugin_id)
            .cloned()
            .collect()
    }
    
    /// List hostlists by category
    pub async fn list_by_category(&self, category: &str) -> Vec<RegisteredHostlist> {
        let hostlists = self.hostlists.read().await;
        hostlists
            .values()
            .filter(|h| h.definition.category.as_deref() == Some(category))
            .cloned()
            .collect()
    }
    
    /// Get domains from a specific hostlist
    pub async fn get_domains(&self, hostlist_id: &str) -> Result<Vec<String>> {
        let hostlists = self.hostlists.read().await;
        
        hostlists
            .get(hostlist_id)
            .map(|h| h.domains.clone())
            .ok_or_else(|| HostlistRegistryError::NotFound(hostlist_id.to_string()))
    }
    
    /// Merge multiple hostlists into a single domain list
    ///
    /// # Arguments
    /// * `hostlist_ids` - IDs of hostlists to merge
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Merged unique domains (sorted)
    /// * `Err(HostlistRegistryError)` - If any hostlist not found
    pub async fn merge_hostlists(&self, hostlist_ids: &[&str]) -> Result<Vec<String>> {
        let hostlists = self.hostlists.read().await;
        let mut all_domains: HashSet<String> = HashSet::new();
        
        for id in hostlist_ids {
            let hostlist = hostlists
                .get(*id)
                .ok_or_else(|| HostlistRegistryError::NotFound(id.to_string()))?;
            
            all_domains.extend(hostlist.domains.iter().cloned());
        }
        
        let mut domains: Vec<String> = all_domains.into_iter().collect();
        domains.sort();
        
        debug!(
            hostlist_count = hostlist_ids.len(),
            domain_count = domains.len(),
            "Merged hostlists"
        );
        
        Ok(domains)
    }
    
    /// Merge all enabled hostlists
    pub async fn merge_all(&self) -> Vec<String> {
        let hostlists = self.hostlists.read().await;
        let mut all_domains: HashSet<String> = HashSet::new();
        
        for hostlist in hostlists.values() {
            if hostlist.enabled {
                all_domains.extend(hostlist.domains.iter().cloned());
            }
        }
        
        let mut domains: Vec<String> = all_domains.into_iter().collect();
        domains.sort();
        domains
    }
    
    /// Check if a domain matches any registered hostlist
    pub async fn domain_matches_any(&self, domain: &str) -> bool {
        let hostlists = self.hostlists.read().await;
        
        for hostlist in hostlists.values() {
            if hostlist.enabled && hostlist.matches(domain) {
                return true;
            }
        }
        
        false
    }
    
    /// Find hostlists that match a domain
    pub async fn find_matching_hostlists(&self, domain: &str) -> Vec<String> {
        let hostlists = self.hostlists.read().await;
        
        hostlists
            .iter()
            .filter(|(_, h)| h.enabled && h.matches(domain))
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Reload domains for a specific hostlist
    pub async fn reload(&self, hostlist_id: &str) -> Result<()> {
        let mut hostlists = self.hostlists.write().await;
        
        let hostlist = hostlists
            .get_mut(hostlist_id)
            .ok_or_else(|| HostlistRegistryError::NotFound(hostlist_id.to_string()))?;
        
        let domains = self.load_domains_internal(&hostlist.definition, &hostlist.plugin_path).await?;
        hostlist.domains = domains;
        hostlist.updated_at = Some(chrono::Utc::now().to_rfc3339());
        
        info!(hostlist_id = %hostlist_id, "Reloaded hostlist");
        
        Ok(())
    }
    
    /// Enable or disable a hostlist
    pub async fn set_enabled(&self, hostlist_id: &str, enabled: bool) -> Result<()> {
        let mut hostlists = self.hostlists.write().await;
        
        let hostlist = hostlists
            .get_mut(hostlist_id)
            .ok_or_else(|| HostlistRegistryError::NotFound(hostlist_id.to_string()))?;
        
        hostlist.enabled = enabled;
        
        debug!(hostlist_id = %hostlist_id, enabled, "Set hostlist enabled state");
        
        Ok(())
    }
    
    /// Get registry statistics
    pub async fn stats(&self) -> RegistryStats {
        let hostlists = self.hostlists.read().await;
        
        let total_hostlists = hostlists.len();
        let enabled_hostlists = hostlists.values().filter(|h| h.enabled).count();
        let total_domains: usize = hostlists.values().map(|h| h.domains.len()).sum();
        
        let unique_domains: HashSet<&str> = hostlists
            .values()
            .filter(|h| h.enabled)
            .flat_map(|h| h.domains.iter().map(|d| d.as_str()))
            .collect();
        
        RegistryStats {
            total_hostlists,
            enabled_hostlists,
            total_domains,
            unique_domains: unique_domains.len(),
        }
    }
    
    /// Get count of registered hostlists
    pub async fn count(&self) -> usize {
        let hostlists = self.hostlists.read().await;
        hostlists.len()
    }
    
    // ========================================================================
    // Private helpers
    // ========================================================================
    
    async fn load_domains(
        &self,
        definition: &HostlistDefinition,
        plugin_path: &PathBuf,
    ) -> Result<Vec<String>> {
        self.load_domains_internal(definition, plugin_path).await
    }
    
    async fn load_domains_internal(
        &self,
        definition: &HostlistDefinition,
        plugin_path: &PathBuf,
    ) -> Result<Vec<String>> {
        let mut domains = Vec::new();
        
        // 1. Add inline domains
        domains.extend(definition.domains.iter().cloned());
        
        // 2. Load from file if specified
        if let Some(ref file) = definition.file {
            let file_path = plugin_path.join(file);
            
            if tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
                let content = tokio::fs::read_to_string(&file_path)
                    .await
                    .map_err(|e| HostlistRegistryError::FileError(format!(
                        "Failed to read {}: {}", file_path.display(), e
                    )))?;
                
                let file_domains = parse_domain_list(&content);
                domains.extend(file_domains);
                
                debug!(
                    file = %file_path.display(),
                    count = domains.len(),
                    "Loaded domains from file"
                );
            } else {
                warn!(
                    file = %file_path.display(),
                    "Hostlist file not found"
                );
            }
        }
        
        // 3. Fetch from remote URL if specified and no local data
        if domains.is_empty() {
            if let Some(ref url) = definition.update_url {
                domains = self.fetch_remote_domains(url).await?;
            }
        }
        
        // Normalize domains
        let normalized: Vec<String> = domains
            .into_iter()
            .map(|d| d.trim().to_lowercase())
            .filter(|d| !d.is_empty() && !d.starts_with('#'))
            .collect();
        
        Ok(normalized)
    }
    
    async fn fetch_remote_domains(&self, url: &str) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| HostlistRegistryError::NetworkError(e.to_string()))?;
        
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| HostlistRegistryError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(HostlistRegistryError::NetworkError(format!(
                "HTTP {}", response.status()
            )));
        }
        
        let content = response
            .text()
            .await
            .map_err(|e| HostlistRegistryError::NetworkError(e.to_string()))?;
        
        let domains = parse_domain_list(&content);
        
        info!(url = %url, count = domains.len(), "Fetched remote hostlist");
        
        Ok(domains)
    }
}

impl Default for HostlistRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryStats {
    pub total_hostlists: usize,
    pub enabled_hostlists: usize,
    pub total_domains: usize,
    pub unique_domains: usize,
}

/// Create a shared hostlist registry
pub fn create_hostlist_registry() -> Arc<HostlistRegistry> {
    Arc::new(HostlistRegistry::new())
}

// ============================================================================
// Helper functions
// ============================================================================

/// Parse domain list from text content
fn parse_domain_list(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}

/// Simple wildcard matching (supports * at start)
fn wildcard_match(pattern: &str, domain: &str) -> bool {
    if pattern.starts_with("*.") {
        let suffix = &pattern[1..]; // ".example.com"
        domain.ends_with(suffix) || domain == &pattern[2..]
    } else if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        domain.ends_with(suffix)
    } else {
        pattern == domain
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_definition(id: &str) -> HostlistDefinition {
        HostlistDefinition {
            id: id.to_string(),
            name: format!("Test {}", id),
            description: None,
            format: HostlistFormat::Plain,
            domains: vec!["example.com".to_string(), "test.com".to_string()],
            file: None,
            update_url: None,
            update_interval: None,
            category: Some("test".to_string()),
            tags: vec![],
        }
    }

    #[tokio::test]
    async fn test_register_hostlist() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-hostlist");
        
        let result = registry
            .register("test-plugin", PathBuf::from("."), definition)
            .await;
        
        assert!(result.is_ok());
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_fails() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-hostlist");
        
        registry
            .register("plugin-1", PathBuf::from("."), definition.clone())
            .await
            .unwrap();
        
        let result = registry
            .register("plugin-2", PathBuf::from("."), definition)
            .await;
        
        assert!(matches!(result, Err(HostlistRegistryError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_unregister_hostlist() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-hostlist");
        
        registry
            .register("test-plugin", PathBuf::from("."), definition)
            .await
            .unwrap();
        
        assert_eq!(registry.count().await, 1);
        
        registry.unregister("test-hostlist").await.unwrap();
        
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_not_found() {
        let registry = HostlistRegistry::new();
        
        let result = registry.unregister("nonexistent").await;
        
        assert!(matches!(result, Err(HostlistRegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_hostlist() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-hostlist");
        
        registry
            .register("test-plugin", PathBuf::from("."), definition)
            .await
            .unwrap();
        
        let hostlist = registry.get("test-hostlist").await;
        
        assert!(hostlist.is_some());
        let hostlist = hostlist.unwrap();
        assert_eq!(hostlist.definition.id, "test-hostlist");
        assert_eq!(hostlist.plugin_id, "test-plugin");
    }

    #[tokio::test]
    async fn test_get_domains() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-hostlist");
        
        registry
            .register("test-plugin", PathBuf::from("."), definition)
            .await
            .unwrap();
        
        let domains = registry.get_domains("test-hostlist").await.unwrap();
        
        assert_eq!(domains.len(), 2);
        assert!(domains.contains(&"example.com".to_string()));
        assert!(domains.contains(&"test.com".to_string()));
    }

    #[tokio::test]
    async fn test_merge_hostlists() {
        let registry = HostlistRegistry::new();
        
        let mut def1 = create_test_definition("hostlist-1");
        def1.domains = vec!["a.com".to_string(), "b.com".to_string()];
        
        let mut def2 = create_test_definition("hostlist-2");
        def2.domains = vec!["b.com".to_string(), "c.com".to_string()];
        
        registry
            .register("plugin", PathBuf::from("."), def1)
            .await
            .unwrap();
        registry
            .register("plugin", PathBuf::from("."), def2)
            .await
            .unwrap();
        
        let merged = registry
            .merge_hostlists(&["hostlist-1", "hostlist-2"])
            .await
            .unwrap();
        
        assert_eq!(merged.len(), 3); // a.com, b.com, c.com (deduplicated)
    }

    #[tokio::test]
    async fn test_unregister_plugin() {
        let registry = HostlistRegistry::new();
        
        let def1 = create_test_definition("hostlist-1");
        let def2 = create_test_definition("hostlist-2");
        let def3 = create_test_definition("hostlist-3");
        
        registry
            .register("plugin-a", PathBuf::from("."), def1)
            .await
            .unwrap();
        registry
            .register("plugin-a", PathBuf::from("."), def2)
            .await
            .unwrap();
        registry
            .register("plugin-b", PathBuf::from("."), def3)
            .await
            .unwrap();
        
        assert_eq!(registry.count().await, 3);
        
        let removed = registry.unregister_plugin("plugin-a").await;
        
        assert_eq!(removed, 2);
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_list_by_category() {
        let registry = HostlistRegistry::new();
        
        let mut def1 = create_test_definition("hostlist-1");
        def1.category = Some("social".to_string());
        
        let mut def2 = create_test_definition("hostlist-2");
        def2.category = Some("video".to_string());
        
        let mut def3 = create_test_definition("hostlist-3");
        def3.category = Some("social".to_string());
        
        registry
            .register("plugin", PathBuf::from("."), def1)
            .await
            .unwrap();
        registry
            .register("plugin", PathBuf::from("."), def2)
            .await
            .unwrap();
        registry
            .register("plugin", PathBuf::from("."), def3)
            .await
            .unwrap();
        
        let social = registry.list_by_category("social").await;
        assert_eq!(social.len(), 2);
        
        let video = registry.list_by_category("video").await;
        assert_eq!(video.len(), 1);
    }

    #[tokio::test]
    async fn test_wildcard_matching() {
        assert!(wildcard_match("*.example.com", "sub.example.com"));
        assert!(wildcard_match("*.example.com", "example.com"));
        assert!(!wildcard_match("*.example.com", "other.com"));
        assert!(wildcard_match("example.com", "example.com"));
        assert!(!wildcard_match("example.com", "sub.example.com"));
    }

    #[tokio::test]
    async fn test_domain_matches_any() {
        let registry = HostlistRegistry::new();
        
        let mut definition = create_test_definition("test-hostlist");
        definition.domains = vec!["example.com".to_string()];
        
        registry
            .register("plugin", PathBuf::from("."), definition)
            .await
            .unwrap();
        
        assert!(registry.domain_matches_any("example.com").await);
        assert!(!registry.domain_matches_any("other.com").await);
    }

    #[tokio::test]
    async fn test_set_enabled() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-hostlist");
        
        registry
            .register("plugin", PathBuf::from("."), definition)
            .await
            .unwrap();
        
        // Initially enabled
        let hostlist = registry.get("test-hostlist").await.unwrap();
        assert!(hostlist.enabled);
        
        // Disable
        registry.set_enabled("test-hostlist", false).await.unwrap();
        let hostlist = registry.get("test-hostlist").await.unwrap();
        assert!(!hostlist.enabled);
        
        // Disabled hostlist should not match
        assert!(!registry.domain_matches_any("example.com").await);
    }

    #[tokio::test]
    async fn test_stats() {
        let registry = HostlistRegistry::new();
        
        let mut def1 = create_test_definition("hostlist-1");
        def1.domains = vec!["a.com".to_string(), "b.com".to_string()];
        
        let mut def2 = create_test_definition("hostlist-2");
        def2.domains = vec!["b.com".to_string(), "c.com".to_string()];
        
        registry
            .register("plugin", PathBuf::from("."), def1)
            .await
            .unwrap();
        registry
            .register("plugin", PathBuf::from("."), def2)
            .await
            .unwrap();
        
        let stats = registry.stats().await;
        
        assert_eq!(stats.total_hostlists, 2);
        assert_eq!(stats.enabled_hostlists, 2);
        assert_eq!(stats.total_domains, 4); // 2 + 2
        assert_eq!(stats.unique_domains, 3); // a, b, c
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("domains.txt");
        
        tokio::fs::write(&file_path, "domain1.com\ndomain2.com\n# comment\ndomain3.com")
            .await
            .unwrap();
        
        let registry = HostlistRegistry::new();
        
        let definition = HostlistDefinition {
            id: "file-hostlist".to_string(),
            name: "File Hostlist".to_string(),
            description: None,
            format: HostlistFormat::Plain,
            domains: vec![],
            file: Some("domains.txt".to_string()),
            update_url: None,
            update_interval: None,
            category: None,
            tags: vec![],
        };
        
        registry
            .register("plugin", temp_dir.path().to_path_buf(), definition)
            .await
            .unwrap();
        
        let domains = registry.get_domains("file-hostlist").await.unwrap();
        
        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"domain1.com".to_string()));
        assert!(domains.contains(&"domain2.com".to_string()));
        assert!(domains.contains(&"domain3.com".to_string()));
    }

    #[tokio::test]
    async fn test_registered_hostlist_matches_wildcard() {
        let hostlist = RegisteredHostlist {
            definition: HostlistDefinition {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: None,
                format: HostlistFormat::Wildcard,
                domains: vec![],
                file: None,
                update_url: None,
                update_interval: None,
                category: None,
                tags: vec![],
            },
            plugin_id: "plugin".to_string(),
            domains: vec!["*.discord.com".to_string(), "discord.gg".to_string()],
            plugin_path: PathBuf::from("."),
            updated_at: None,
            enabled: true,
        };
        
        assert!(hostlist.matches("cdn.discord.com"));
        assert!(hostlist.matches("discord.com"));
        assert!(hostlist.matches("discord.gg"));
        assert!(!hostlist.matches("other.com"));
    }
}

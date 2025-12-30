//! Hostlists management for domain filtering
//!
//! Manages lists of domains for different services (Discord, YouTube, etc.)
//! Hostlists are stored as plain text files with one domain per line.

use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::get_hostlists_dir;

/// Hostlist for a specific service
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Hostlist {
    /// Unique identifier (filename without extension)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// List of domains
    pub domains: Vec<String>,
    /// Last update timestamp (ISO 8601)
    pub updated_at: Option<String>,
}

impl Hostlist {
    /// Create a new hostlist
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            domains: Vec::new(),
            updated_at: None,
        }
    }

    /// Get the file path for this hostlist
    pub fn file_path(&self) -> PathBuf {
        get_hostlists_dir().join(format!("{}.txt", self.id))
    }

    /// Get unique domains as a HashSet
    pub fn unique_domains(&self) -> HashSet<&str> {
        self.domains.iter().map(|s| s.as_str()).collect()
    }

    /// Get domain count
    pub fn domain_count(&self) -> usize {
        self.domains.len()
    }
}

/// Get human-readable name from hostlist ID
fn id_to_name(id: &str) -> String {
    match id {
        "discord" => "Discord".to_string(),
        "youtube" => "YouTube".to_string(),
        "telegram" => "Telegram".to_string(),
        "cloudflare" => "Cloudflare".to_string(),
        "general" => "General".to_string(),
        _ => id
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().next().unwrap_or(c)
                } else {
                    c
                }
            })
            .collect(),
    }
}

/// Load hostlist from file
///
/// # Arguments
/// * `id` - Hostlist identifier (filename without .txt extension)
///
/// # Returns
/// * `Ok(Hostlist)` - Loaded hostlist with domains
/// * `Err(IsolateError)` - If file not found or read error
pub async fn load_hostlist(id: &str) -> Result<Hostlist> {
    let path = get_hostlists_dir().join(format!("{}.txt", id));

    if !path.exists() {
        return Err(IsolateError::Config(format!(
            "Hostlist '{}' not found at {:?}",
            id, path
        )));
    }

    let content = fs::read_to_string(&path).await?;

    let domains: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_lowercase())
        .collect();

    // Get file modification time
    let metadata = fs::metadata(&path).await?;
    let updated_at = metadata
        .modified()
        .ok()
        .and_then(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            Some(datetime.to_rfc3339())
        });

    debug!(id, domain_count = domains.len(), "Loaded hostlist");

    Ok(Hostlist {
        id: id.to_string(),
        name: id_to_name(id),
        domains,
        updated_at,
    })
}

/// Save hostlist to file
///
/// # Arguments
/// * `hostlist` - Hostlist to save
///
/// # Returns
/// * `Ok(())` - Successfully saved
/// * `Err(IsolateError)` - If write error
pub async fn save_hostlist(hostlist: &Hostlist) -> Result<()> {
    let path = hostlist.file_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write domains, one per line
    let content = hostlist.domains.join("\n");
    fs::write(&path, content).await?;

    info!(id = %hostlist.id, domain_count = hostlist.domains.len(), "Saved hostlist");

    Ok(())
}

/// Get all available hostlists
///
/// Scans the hostlists directory for .txt files and loads them.
///
/// # Returns
/// * `Ok(Vec<Hostlist>)` - List of all hostlists
/// * `Err(IsolateError)` - If directory read error
pub async fn get_all_hostlists() -> Result<Vec<Hostlist>> {
    let dir = get_hostlists_dir();

    // Ensure directory exists
    if !dir.exists() {
        fs::create_dir_all(&dir).await?;
        return Ok(Vec::new());
    }

    let mut hostlists = Vec::new();
    let mut entries = fs::read_dir(&dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "txt") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                match load_hostlist(stem).await {
                    Ok(hostlist) => hostlists.push(hostlist),
                    Err(e) => {
                        warn!(path = ?path, error = %e, "Failed to load hostlist");
                    }
                }
            }
        }
    }

    // Sort by name
    hostlists.sort_by(|a, b| a.name.cmp(&b.name));

    debug!(count = hostlists.len(), "Loaded all hostlists");

    Ok(hostlists)
}

/// Add custom domain to hostlist
///
/// # Arguments
/// * `hostlist_id` - ID of the hostlist to modify
/// * `domain` - Domain to add (will be normalized to lowercase)
///
/// # Returns
/// * `Ok(())` - Domain added successfully
/// * `Err(IsolateError)` - If hostlist not found or write error
pub async fn add_domain(hostlist_id: &str, domain: &str) -> Result<()> {
    let mut hostlist = load_hostlist(hostlist_id).await?;

    let normalized = domain.trim().to_lowercase();

    if normalized.is_empty() {
        return Err(IsolateError::Config("Domain cannot be empty".to_string()));
    }

    // Check if domain already exists
    if hostlist.domains.contains(&normalized) {
        debug!(hostlist_id, domain = %normalized, "Domain already exists");
        return Ok(());
    }

    hostlist.domains.push(normalized.clone());
    save_hostlist(&hostlist).await?;

    info!(hostlist_id, domain = %normalized, "Added domain to hostlist");

    Ok(())
}

/// Remove domain from hostlist
///
/// # Arguments
/// * `hostlist_id` - ID of the hostlist to modify
/// * `domain` - Domain to remove
///
/// # Returns
/// * `Ok(())` - Domain removed successfully (or wasn't present)
/// * `Err(IsolateError)` - If hostlist not found or write error
pub async fn remove_domain(hostlist_id: &str, domain: &str) -> Result<()> {
    let mut hostlist = load_hostlist(hostlist_id).await?;

    let normalized = domain.trim().to_lowercase();
    let original_len = hostlist.domains.len();

    hostlist.domains.retain(|d| d != &normalized);

    if hostlist.domains.len() < original_len {
        save_hostlist(&hostlist).await?;
        info!(hostlist_id, domain = %normalized, "Removed domain from hostlist");
    } else {
        debug!(hostlist_id, domain = %normalized, "Domain not found in hostlist");
    }

    Ok(())
}

/// Update hostlist from remote URL
///
/// Downloads domain list from URL and replaces current hostlist content.
///
/// # Arguments
/// * `id` - Hostlist ID to update
/// * `url` - URL to download domains from (plain text, one domain per line)
///
/// # Returns
/// * `Ok(())` - Successfully updated
/// * `Err(IsolateError)` - If download or write error
pub async fn update_hostlist(id: &str, url: &str) -> Result<()> {
    info!(id, url, "Updating hostlist from remote URL");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(IsolateError::Network(format!(
            "Failed to download hostlist: HTTP {}",
            response.status()
        )));
    }

    let content = response.text().await?;

    let domains: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_lowercase())
        .collect();

    if domains.is_empty() {
        return Err(IsolateError::Config(
            "Downloaded hostlist is empty".to_string(),
        ));
    }

    let hostlist = Hostlist {
        id: id.to_string(),
        name: id_to_name(id),
        domains,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    save_hostlist(&hostlist).await?;

    info!(
        id,
        domain_count = hostlist.domains.len(),
        "Updated hostlist from remote"
    );

    Ok(())
}

/// Create a new empty hostlist
///
/// # Arguments
/// * `id` - Unique identifier for the hostlist
/// * `name` - Human-readable name
///
/// # Returns
/// * `Ok(Hostlist)` - Created hostlist
/// * `Err(IsolateError)` - If hostlist already exists or write error
pub async fn create_hostlist(id: &str, name: &str) -> Result<Hostlist> {
    let path = get_hostlists_dir().join(format!("{}.txt", id));

    if path.exists() {
        return Err(IsolateError::Config(format!(
            "Hostlist '{}' already exists",
            id
        )));
    }

    let hostlist = Hostlist {
        id: id.to_string(),
        name: name.to_string(),
        domains: Vec::new(),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    save_hostlist(&hostlist).await?;

    info!(id, name, "Created new hostlist");

    Ok(hostlist)
}

/// Delete a hostlist
///
/// # Arguments
/// * `id` - ID of the hostlist to delete
///
/// # Returns
/// * `Ok(())` - Successfully deleted
/// * `Err(IsolateError)` - If hostlist not found or delete error
pub async fn delete_hostlist(id: &str) -> Result<()> {
    let path = get_hostlists_dir().join(format!("{}.txt", id));

    if !path.exists() {
        return Err(IsolateError::Config(format!(
            "Hostlist '{}' not found",
            id
        )));
    }

    fs::remove_file(&path).await?;

    info!(id, "Deleted hostlist");

    Ok(())
}

/// Merge multiple hostlists into one combined list
///
/// # Arguments
/// * `ids` - List of hostlist IDs to merge
///
/// # Returns
/// * `Ok(Vec<String>)` - Combined unique domains
/// * `Err(IsolateError)` - If any hostlist not found
pub async fn merge_hostlists(ids: &[&str]) -> Result<Vec<String>> {
    let mut all_domains: HashSet<String> = HashSet::new();

    for id in ids {
        let hostlist = load_hostlist(id).await?;
        all_domains.extend(hostlist.domains);
    }

    let mut domains: Vec<String> = all_domains.into_iter().collect();
    domains.sort();

    debug!(
        hostlist_count = ids.len(),
        domain_count = domains.len(),
        "Merged hostlists"
    );

    Ok(domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_to_name() {
        assert_eq!(id_to_name("discord"), "Discord");
        assert_eq!(id_to_name("youtube"), "YouTube");
        assert_eq!(id_to_name("telegram"), "Telegram");
        assert_eq!(id_to_name("custom"), "Custom");
    }

    #[test]
    fn test_hostlist_new() {
        let hostlist = Hostlist::new("test", "Test List");
        assert_eq!(hostlist.id, "test");
        assert_eq!(hostlist.name, "Test List");
        assert!(hostlist.domains.is_empty());
    }

    #[test]
    fn test_unique_domains() {
        let mut hostlist = Hostlist::new("test", "Test");
        hostlist.domains = vec![
            "example.com".to_string(),
            "test.com".to_string(),
            "example.com".to_string(),
        ];

        let unique = hostlist.unique_domains();
        assert_eq!(unique.len(), 2);
        assert!(unique.contains("example.com"));
        assert!(unique.contains("test.com"));
    }
}

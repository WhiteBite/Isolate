//! Dynamic Hostlist Generator
//!
//! Creates temporary hostlist files from DPI bypass routing rules.
//! These hostlists are used by winws to filter which domains
//! should have DPI bypass applied.
//!
//! NOTE: Some functions are prepared for future hostlist management.
//!
//! ## Usage
//!
//! ```ignore
//! let domains = vec!["youtube.com".to_string(), "discord.com".to_string()];
//! let hostlist_path = create_dynamic_hostlist(&domains).await?;
//! 
//! // Use hostlist_path with winws: --hostlist=<hostlist_path>
//! ```

// Public API for dynamic hostlist generation
#![allow(dead_code)]

use crate::core::errors::{IsolateError, Result};
use crate::core::paths::get_app_data_dir;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};

/// Name of the dynamic hostlist file
const DYNAMIC_HOSTLIST_FILENAME: &str = "dpi-bypass-domains.txt";

/// Get path to the dynamic hostlist file
pub fn get_dynamic_hostlist_path() -> PathBuf {
    get_app_data_dir().join("hostlists").join(DYNAMIC_HOSTLIST_FILENAME)
}

/// Create or update dynamic hostlist from DPI bypass domains
///
/// # Arguments
/// * `domains` - List of domains that need DPI bypass
///
/// # Returns
/// Path to the created hostlist file
///
/// # Example
/// ```ignore
/// let domains = vec!["youtube.com".to_string(), "googlevideo.com".to_string()];
/// let path = create_dynamic_hostlist(&domains).await?;
/// // path = "C:\Users\...\AppData\Local\isolate\hostlists\dpi-bypass-domains.txt"
/// ```
pub async fn create_dynamic_hostlist(domains: &[String]) -> Result<PathBuf> {
    let hostlist_path = get_dynamic_hostlist_path();

    // Ensure parent directory exists
    if let Some(parent) = hostlist_path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            IsolateError::Io(format!(
                "Failed to create hostlist directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Normalize and deduplicate domains
    let mut normalized_domains: Vec<String> = domains
        .iter()
        .map(|d| normalize_domain_for_hostlist(d))
        .collect();
    
    normalized_domains.sort();
    normalized_domains.dedup();

    // Generate hostlist content
    let content = generate_hostlist_content(&normalized_domains);

    // Write to file
    fs::write(&hostlist_path, &content).await.map_err(|e| {
        IsolateError::Io(format!(
            "Failed to write hostlist to {}: {}",
            hostlist_path.display(),
            e
        ))
    })?;

    info!(
        path = %hostlist_path.display(),
        domains_count = normalized_domains.len(),
        "Created dynamic hostlist"
    );

    Ok(hostlist_path)
}

/// Check if dynamic hostlist exists and is not empty
pub async fn has_dynamic_hostlist() -> bool {
    let path = get_dynamic_hostlist_path();
    
    if !path.exists() {
        return false;
    }

    match fs::metadata(&path).await {
        Ok(meta) => meta.len() > 0,
        Err(_) => false,
    }
}

/// Get domains from existing dynamic hostlist
pub async fn get_dynamic_hostlist_domains() -> Result<Vec<String>> {
    let path = get_dynamic_hostlist_path();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).await.map_err(|e| {
        IsolateError::Io(format!(
            "Failed to read hostlist from {}: {}",
            path.display(),
            e
        ))
    })?;

    let domains: Vec<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| line.trim().to_string())
        .collect();

    debug!(
        path = %path.display(),
        domains_count = domains.len(),
        "Read dynamic hostlist"
    );

    Ok(domains)
}

/// Delete dynamic hostlist file
pub async fn delete_dynamic_hostlist() -> Result<()> {
    let path = get_dynamic_hostlist_path();

    if path.exists() {
        fs::remove_file(&path).await.map_err(|e| {
            IsolateError::Io(format!(
                "Failed to delete hostlist {}: {}",
                path.display(),
                e
            ))
        })?;

        info!(path = %path.display(), "Deleted dynamic hostlist");
    }

    Ok(())
}

/// Normalize domain for hostlist format
///
/// Zapret hostlist format:
/// - One domain per line
/// - No wildcards (zapret handles subdomains automatically)
/// - Lowercase
fn normalize_domain_for_hostlist(domain: &str) -> String {
    let domain = domain.trim().to_lowercase();

    // Remove protocol if present
    let domain = domain
        .strip_prefix("https://")
        .or_else(|| domain.strip_prefix("http://"))
        .unwrap_or(&domain);

    // Remove path if present
    let domain = domain.split('/').next().unwrap_or(domain);

    // Remove port if present
    let domain = domain.split(':').next().unwrap_or(domain);

    // Remove leading dot (wildcard notation)
    let domain = domain.strip_prefix('.').unwrap_or(domain);

    // Remove leading wildcard
    let domain = domain.strip_prefix("*.").unwrap_or(domain);

    domain.to_string()
}

/// Generate hostlist file content
fn generate_hostlist_content(domains: &[String]) -> String {
    let mut content = String::new();

    // Header comment
    content.push_str("# Isolate Dynamic DPI Bypass Hostlist\n");
    content.push_str("# Auto-generated from routing rules\n");
    content.push_str("# Do not edit manually - changes will be overwritten\n");
    content.push_str("#\n");
    content.push_str(&format!("# Generated: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    content.push_str(&format!("# Domains: {}\n", domains.len()));
    content.push_str("\n");

    // Domains
    for domain in domains {
        content.push_str(domain);
        content.push('\n');
    }

    content
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_domain() {
        assert_eq!(normalize_domain_for_hostlist("YouTube.com"), "youtube.com");
        assert_eq!(normalize_domain_for_hostlist("https://youtube.com/watch"), "youtube.com");
        assert_eq!(normalize_domain_for_hostlist("youtube.com:443"), "youtube.com");
        assert_eq!(normalize_domain_for_hostlist("  youtube.com  "), "youtube.com");
        assert_eq!(normalize_domain_for_hostlist(".youtube.com"), "youtube.com");
        assert_eq!(normalize_domain_for_hostlist("*.youtube.com"), "youtube.com");
    }

    #[test]
    fn test_generate_hostlist_content() {
        let domains = vec![
            "youtube.com".to_string(),
            "googlevideo.com".to_string(),
        ];

        let content = generate_hostlist_content(&domains);

        assert!(content.contains("# Isolate Dynamic DPI Bypass Hostlist"));
        assert!(content.contains("youtube.com"));
        assert!(content.contains("googlevideo.com"));
        assert!(content.contains("Domains: 2"));
    }

    #[tokio::test]
    async fn test_create_and_read_hostlist() {
        let domains = vec![
            "youtube.com".to_string(),
            "discord.com".to_string(),
        ];

        // Create hostlist
        let path = create_dynamic_hostlist(&domains).await.unwrap();
        assert!(path.exists());

        // Read back
        let read_domains = get_dynamic_hostlist_domains().await.unwrap();
        assert!(read_domains.contains(&"youtube.com".to_string()));
        assert!(read_domains.contains(&"discord.com".to_string()));

        // Cleanup
        delete_dynamic_hostlist().await.unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_deduplication() {
        let domains = vec![
            "youtube.com".to_string(),
            "YouTube.com".to_string(),
            "https://youtube.com".to_string(),
        ];

        let mut normalized: Vec<String> = domains
            .iter()
            .map(|d| normalize_domain_for_hostlist(d))
            .collect();
        
        normalized.sort();
        normalized.dedup();

        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0], "youtube.com");
    }
}

//! Update checker via GitHub Releases API
//!
//! Checks for new versions by querying GitHub Releases API.
//! Does NOT use Tauri updater plugin - just notifies user about new version
//! with a link to download from GitHub.

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

/// GitHub Release response structure
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    /// Release tag (e.g., "v1.2.3")
    pub tag_name: String,
    /// URL to the release page
    pub html_url: String,
    /// Release notes (markdown)
    pub body: Option<String>,
    /// Publication date
    pub published_at: Option<String>,
    /// Whether this is a prerelease
    pub prerelease: bool,
    /// Whether this is a draft
    pub draft: bool,
}

/// Update information for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubUpdateInfo {
    /// New version string (without 'v' prefix)
    pub version: String,
    /// URL to download page
    pub download_url: String,
    /// Release notes (markdown)
    pub release_notes: Option<String>,
    /// Publication date
    pub published_at: Option<String>,
}

/// GitHub repository info
const GITHUB_OWNER: &str = "WhiteBite";
const GITHUB_REPO: &str = "Isolate";

/// Current app version from Cargo.toml
fn get_current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Parse version string to comparable tuple
/// Handles formats: "1.2.3", "v1.2.3", "1.2.3-beta"
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let version = version.trim_start_matches('v');
    // Remove any suffix like -beta, -rc1, etc.
    let version = version.split('-').next()?;
    
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    
    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2].parse().ok()?;
    
    Some((major, minor, patch))
}

/// Compare two versions
/// Returns true if new_version > current_version
fn is_newer_version(current: &str, new: &str) -> bool {
    let current_parsed = parse_version(current);
    let new_parsed = parse_version(new);
    
    match (current_parsed, new_parsed) {
        (Some(current), Some(new)) => new > current,
        _ => {
            warn!(
                current = current,
                new = new,
                "Failed to parse versions, falling back to string comparison"
            );
            // Fallback to string comparison
            new.trim_start_matches('v') > current.trim_start_matches('v')
        }
    }
}

/// Check for updates via GitHub Releases API
/// 
/// Returns Some(GitHubUpdateInfo) if a newer version is available,
/// None if current version is up to date.
pub async fn check_for_updates() -> Result<Option<GitHubUpdateInfo>, String> {
    let current_version = get_current_version();
    info!(version = current_version, "Checking for updates");
    
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );
    
    debug!(url = %url, "Fetching latest release");
    
    let client = reqwest::Client::builder()
        .user_agent(format!("Isolate/{}", current_version))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;
    
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        info!("No releases found on GitHub");
        return Ok(None);
    }
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(status = %status, body = %body, "GitHub API error");
        return Err(format!("GitHub API returned status {}", status));
    }
    
    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;
    
    // Skip drafts and prereleases
    if release.draft || release.prerelease {
        info!(
            tag = %release.tag_name,
            draft = release.draft,
            prerelease = release.prerelease,
            "Skipping non-stable release"
        );
        return Ok(None);
    }
    
    let new_version = release.tag_name.trim_start_matches('v');
    
    if is_newer_version(current_version, &release.tag_name) {
        info!(
            current = current_version,
            new = new_version,
            "New version available"
        );
        
        Ok(Some(GitHubUpdateInfo {
            version: new_version.to_string(),
            download_url: release.html_url,
            release_notes: release.body,
            published_at: release.published_at,
        }))
    } else {
        info!(
            current = current_version,
            latest = new_version,
            "Already on latest version"
        );
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("1.2.3-beta"), Some((1, 2, 3)));
        assert_eq!(parse_version("v2.0.0-rc1"), Some((2, 0, 0)));
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version("1.2"), None);
    }

    #[test]
    fn test_is_newer_version() {
        // Basic comparisons
        assert!(is_newer_version("1.0.0", "1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("0.1.0", "0.2.0"));
        
        // With v prefix
        assert!(is_newer_version("1.0.0", "v1.0.1"));
        assert!(is_newer_version("v1.0.0", "1.0.1"));
        assert!(is_newer_version("v1.0.0", "v1.0.1"));
        
        // Not newer
        assert!(!is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("2.0.0", "1.9.9"));
    }

    #[test]
    fn test_get_current_version() {
        let version = get_current_version();
        assert!(!version.is_empty());
        // Should be parseable
        assert!(parse_version(version).is_some());
    }
}

//! Configuration auto-updater module
//!
//! Downloads and updates strategy configurations from remote repository.

#![allow(dead_code)] // Public config updater API

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths;

// ============================================================================
// Constants
// ============================================================================

/// Default configs repository URL
const DEFAULT_CONFIGS_URL: &str = "https://api.github.com/repos/aspect-build/isolate-configs/contents";

/// User-Agent for GitHub API requests
const USER_AGENT: &str = "Isolate-App";

/// Request timeout
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

// ============================================================================
// Types
// ============================================================================

/// GitHub API file entry
#[derive(Debug, Clone, Deserialize)]
struct GitHubFile {
    name: String,
    path: String,
    sha: String,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    download_url: Option<String>,
    #[serde(rename = "type")]
    file_type: String,
}

/// Config update info
#[derive(Debug, Clone, Serialize)]
pub struct ConfigUpdate {
    /// File name
    pub name: String,
    /// Remote path
    pub path: String,
    /// SHA hash for comparison
    pub sha: String,
    /// Whether this is a new file or update
    pub is_new: bool,
}

/// Update result
#[derive(Debug, Clone, Serialize)]
pub struct UpdateResult {
    /// Number of configs updated
    pub updated_count: usize,
    /// Number of new configs added
    pub new_count: usize,
    /// List of updated files
    pub files: Vec<String>,
}

// ============================================================================
// Config Updater
// ============================================================================

/// Configuration updater service
pub struct ConfigUpdater {
    /// Base URL for configs repository
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Local configs directory
    configs_dir: PathBuf,
}

impl ConfigUpdater {
    /// Creates a new ConfigUpdater with default settings
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| IsolateError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let configs_dir = paths::get_configs_dir();

        Ok(Self {
            base_url: DEFAULT_CONFIGS_URL.to_string(),
            client,
            configs_dir,
        })
    }

    /// Creates a ConfigUpdater with custom repository URL
    pub fn with_url(url: &str) -> Result<Self> {
        let mut updater = Self::new()?;
        updater.base_url = url.to_string();
        Ok(updater)
    }

    /// Check for available config updates
    ///
    /// Returns list of configs that have updates available
    pub async fn check_updates(&self) -> Result<Vec<ConfigUpdate>> {
        info!("Checking for config updates");

        let mut updates = Vec::new();

        // Check strategies
        let strategies = self.check_directory_updates("strategies").await?;
        updates.extend(strategies);

        // Check services
        let services = self.check_directory_updates("services").await?;
        updates.extend(services);

        // Check hostlists
        let hostlists = self.check_directory_updates("hostlists").await?;
        updates.extend(hostlists);

        info!(count = updates.len(), "Found config updates");
        Ok(updates)
    }

    /// Check updates for a specific directory
    async fn check_directory_updates(&self, dir: &str) -> Result<Vec<ConfigUpdate>> {
        let url = format!("{}/{}", self.base_url, dir);
        debug!(url = %url, "Fetching directory listing");

        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to fetch {}: {}", dir, e)))?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                debug!(dir = %dir, "Directory not found in remote");
                return Ok(Vec::new());
            }
            return Err(IsolateError::Network(format!(
                "GitHub API error: {}",
                response.status()
            )));
        }

        let files: Vec<GitHubFile> = response
            .json()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to parse response: {}", e)))?;

        let mut updates = Vec::new();
        let local_dir = self.configs_dir.join(dir);

        for file in files {
            if file.file_type != "file" {
                continue;
            }

            // Check if file exists locally
            let local_path = local_dir.join(&file.name);
            let is_new = !local_path.exists();

            // For existing files, compare SHA (simplified - just check if file changed)
            let needs_update = if is_new {
                true
            } else {
                // Read local file and compute simple hash comparison
                // In production, you'd store SHA in a manifest file
                self.file_needs_update(&local_path, &file.sha).await
            };

            if needs_update {
                updates.push(ConfigUpdate {
                    name: file.name,
                    path: file.path,
                    sha: file.sha,
                    is_new,
                });
            }
        }

        Ok(updates)
    }

    /// Check if local file needs update
    async fn file_needs_update(&self, local_path: &PathBuf, _remote_sha: &str) -> bool {
        // Simple implementation: check file modification time
        // In production, store and compare SHA hashes
        match tokio::fs::metadata(local_path).await {
            Ok(metadata) => {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.elapsed() {
                        // Consider files older than 7 days as potentially outdated
                        return duration.as_secs() > 7 * 24 * 60 * 60;
                    }
                }
                false
            }
            Err(_) => true,
        }
    }

    /// Download and apply config updates
    pub async fn download_updates(&self, updates: &[ConfigUpdate]) -> Result<UpdateResult> {
        info!(count = updates.len(), "Downloading config updates");

        let mut result = UpdateResult {
            updated_count: 0,
            new_count: 0,
            files: Vec::new(),
        };

        for update in updates {
            match self.download_file(update).await {
                Ok(_) => {
                    if update.is_new {
                        result.new_count += 1;
                    } else {
                        result.updated_count += 1;
                    }
                    result.files.push(update.name.clone());
                    info!(file = %update.name, "Config updated");
                }
                Err(e) => {
                    warn!(file = %update.name, error = %e, "Failed to update config");
                }
            }
        }

        Ok(result)
    }

    /// Download a single config file
    async fn download_file(&self, update: &ConfigUpdate) -> Result<()> {
        // Construct raw content URL
        let download_url = format!(
            "https://raw.githubusercontent.com/aspect-build/isolate-configs/main/{}",
            update.path
        );

        debug!(url = %download_url, "Downloading config file");

        let response = self.client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| IsolateError::Network(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(IsolateError::Network(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| IsolateError::Network(format!("Failed to read content: {}", e)))?;

        // Determine local path
        let local_path = self.configs_dir.join(&update.path);

        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write file
        tokio::fs::write(&local_path, &content).await?;

        debug!(path = %local_path.display(), "Config file saved");
        Ok(())
    }

    /// Check and download all updates in one call
    pub async fn update_all(&self) -> Result<UpdateResult> {
        let updates = self.check_updates().await?;
        
        if updates.is_empty() {
            info!("No config updates available");
            return Ok(UpdateResult {
                updated_count: 0,
                new_count: 0,
                files: Vec::new(),
            });
        }

        self.download_updates(&updates).await
    }
}

impl Default for ConfigUpdater {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigUpdater")
    }
}

// ============================================================================
// Public Functions
// ============================================================================

/// Check for config updates (convenience function)
pub async fn check_config_updates() -> Result<Vec<ConfigUpdate>> {
    let updater = ConfigUpdater::new()?;
    updater.check_updates().await
}

/// Download all available config updates (convenience function)
pub async fn download_config_updates() -> Result<UpdateResult> {
    let updater = ConfigUpdater::new()?;
    updater.update_all().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_updater_creation() {
        // This will fail without network, but tests the structure
        let result = ConfigUpdater::new();
        assert!(result.is_ok());
    }
}

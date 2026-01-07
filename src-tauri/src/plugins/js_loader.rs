//! Plugin Loader
//!
//! Loads plugin.json manifests from plugins directory.
//! Uses async I/O for better performance.

use crate::plugins::manifest::{LoadedPluginInfo, PluginContributes, PluginManifest, PluginType, ServiceDefinition};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginLoaderError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Scan plugins directory asynchronously
pub async fn scan_plugins_async(plugins_dir: &Path) -> Result<Vec<PathBuf>, PluginLoaderError> {
    let mut found = Vec::new();

    if !plugins_dir.exists() {
        tokio::fs::create_dir_all(plugins_dir).await?;
        return Ok(found);
    }

    let mut entries = tokio::fs::read_dir(plugins_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        // Check if it's a directory (need to use metadata for async)
        if let Ok(metadata) = tokio::fs::metadata(&path).await {
            if metadata.is_dir() {
                let manifest_path = path.join("plugin.json");
                
                // Check if manifest exists
                if tokio::fs::try_exists(&manifest_path).await.unwrap_or(false) {
                    found.push(path);
                }
            }
        }
    }

    Ok(found)
}

/// Scan plugins directory (sync wrapper for backward compatibility)
pub fn scan_plugins(plugins_dir: &Path) -> Result<Vec<PathBuf>, PluginLoaderError> {
    let mut found = Vec::new();

    if !plugins_dir.exists() {
        std::fs::create_dir_all(plugins_dir)?;
        return Ok(found);
    }

    for entry in std::fs::read_dir(plugins_dir)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let manifest_path = path.join("plugin.json");
            if manifest_path.exists() {
                found.push(path);
            }
        }
    }

    Ok(found)
}

/// Load plugin manifest asynchronously
pub async fn load_manifest_async(plugin_dir: &Path) -> Result<PluginManifest, PluginLoaderError> {
    let manifest_path = plugin_dir.join("plugin.json");
    let content = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
        PluginLoaderError::InvalidManifest(format!("{}: {}", manifest_path.display(), e))
    })?;
    Ok(manifest)
}

/// Load plugin manifest (sync wrapper for backward compatibility)
pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, PluginLoaderError> {
    let manifest_path = plugin_dir.join("plugin.json");
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
        PluginLoaderError::InvalidManifest(format!("{}: {}", manifest_path.display(), e))
    })?;
    Ok(manifest)
}

/// Get all plugins with info asynchronously
pub async fn get_all_plugins_async(plugins_dir: &Path) -> Vec<LoadedPluginInfo> {
    let dirs = match scan_plugins_async(plugins_dir).await {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut plugins = Vec::new();

    for dir in dirs {
        let info = match load_manifest_async(&dir).await {
            Ok(manifest) => LoadedPluginInfo {
                manifest,
                enabled: true,
                path: dir.display().to_string(),
                error: None,
            },
            Err(e) => LoadedPluginInfo {
                manifest: PluginManifest {
                    id: dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    name: "Invalid Plugin".to_string(),
                    version: "0.0.0".to_string(),
                    author: "Unknown".to_string(),
                    description: None,
                    plugin_type: PluginType::ServiceChecker,
                    service: None,
                    strategy: None,
                    hostlist: None,
                    contributes: PluginContributes::default(),
                    permissions: Default::default(),
                },
                enabled: false,
                path: dir.display().to_string(),
                error: Some(e.to_string()),
            },
        };
        plugins.push(info);
    }

    plugins
}

/// Get all plugins with info (sync wrapper for backward compatibility)
pub fn get_all_plugins(plugins_dir: &Path) -> Vec<LoadedPluginInfo> {
    let dirs = match scan_plugins(plugins_dir) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    dirs.iter()
        .map(|dir| match load_manifest(dir) {
            Ok(manifest) => LoadedPluginInfo {
                manifest,
                enabled: true,
                path: dir.display().to_string(),
                error: None,
            },
            Err(e) => LoadedPluginInfo {
                manifest: PluginManifest {
                    id: dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    name: "Invalid Plugin".to_string(),
                    version: "0.0.0".to_string(),
                    author: "Unknown".to_string(),
                    description: None,
                    plugin_type: PluginType::ServiceChecker,
                    service: None,
                    strategy: None,
                    hostlist: None,
                    contributes: PluginContributes::default(),
                    permissions: Default::default(),
                },
                enabled: false,
                path: dir.display().to_string(),
                error: Some(e.to_string()),
            },
        })
        .collect()
}

/// Get all services from service-checker plugins asynchronously
pub async fn get_all_services_async(plugins_dir: &Path) -> Vec<ServiceDefinition> {
    get_all_plugins_async(plugins_dir)
        .await
        .into_iter()
        .filter(|p| p.enabled && p.error.is_none())
        .filter(|p| p.manifest.plugin_type == PluginType::ServiceChecker)
        .filter_map(|p| p.manifest.service)
        .collect()
}

/// Get all services from service-checker plugins (sync wrapper)
pub fn get_all_services(plugins_dir: &Path) -> Vec<ServiceDefinition> {
    get_all_plugins(plugins_dir)
        .into_iter()
        .filter(|p| p.enabled && p.error.is_none())
        .filter(|p| p.manifest.plugin_type == PluginType::ServiceChecker)
        .filter_map(|p| p.manifest.service)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scan_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = scan_plugins_async(temp_dir.path()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_scan_creates_missing_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        
        assert!(!plugins_dir.exists());
        let result = scan_plugins_async(&plugins_dir).await;
        assert!(result.is_ok());
        assert!(plugins_dir.exists());
    }

    #[tokio::test]
    async fn test_load_valid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        let manifest_content = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "type": "service-checker"
        }"#;

        tokio::fs::write(plugin_dir.join("plugin.json"), manifest_content)
            .await
            .unwrap();

        let result = load_manifest_async(&plugin_dir).await;
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.name, "Test Plugin");
    }

    #[tokio::test]
    async fn test_load_invalid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("invalid-plugin");
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        tokio::fs::write(plugin_dir.join("plugin.json"), "invalid json")
            .await
            .unwrap();

        let result = load_manifest_async(&plugin_dir).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_plugins_with_valid_and_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        // Valid plugin
        let valid_dir = plugins_dir.join("valid-plugin");
        tokio::fs::create_dir_all(&valid_dir).await.unwrap();
        tokio::fs::write(
            valid_dir.join("plugin.json"),
            r#"{"id": "valid", "name": "Valid", "version": "1.0.0", "author": "Test", "type": "service-checker"}"#,
        )
        .await
        .unwrap();

        // Invalid plugin
        let invalid_dir = plugins_dir.join("invalid-plugin");
        tokio::fs::create_dir_all(&invalid_dir).await.unwrap();
        tokio::fs::write(invalid_dir.join("plugin.json"), "not json")
            .await
            .unwrap();

        let plugins = get_all_plugins_async(plugins_dir).await;
        assert_eq!(plugins.len(), 2);

        let valid = plugins.iter().find(|p| p.manifest.id == "valid");
        assert!(valid.is_some());
        assert!(valid.unwrap().enabled);
        assert!(valid.unwrap().error.is_none());

        let invalid = plugins.iter().find(|p| !p.enabled);
        assert!(invalid.is_some());
        assert!(invalid.unwrap().error.is_some());
    }
}

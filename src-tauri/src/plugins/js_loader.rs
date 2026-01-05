//! Plugin Loader
//! 
//! Loads plugin.json manifests from plugins directory.

use crate::plugins::manifest::{LoadedPluginInfo, PluginManifest, PluginType, ServiceDefinition};
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

/// Scan plugins directory
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

/// Load plugin manifest
pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, PluginLoaderError> {
    let manifest_path = plugin_dir.join("plugin.json");
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: PluginManifest = serde_json::from_str(&content)
        .map_err(|e| PluginLoaderError::InvalidManifest(format!("{}: {}", manifest_path.display(), e)))?;
    Ok(manifest)
}

/// Get all plugins with info
pub fn get_all_plugins(plugins_dir: &Path) -> Vec<LoadedPluginInfo> {
    let dirs = match scan_plugins(plugins_dir) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    
    dirs.iter()
        .map(|dir| {
            match load_manifest(dir) {
                Ok(manifest) => LoadedPluginInfo {
                    manifest,
                    enabled: true,
                    path: dir.display().to_string(),
                    error: None,
                },
                Err(e) => LoadedPluginInfo {
                    manifest: PluginManifest {
                        id: dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                        name: "Invalid Plugin".to_string(),
                        version: "0.0.0".to_string(),
                        author: "Unknown".to_string(),
                        description: None,
                        plugin_type: PluginType::ServiceChecker,
                        service: None,
                        strategy: None,
                        hostlist: None,
                        permissions: Default::default(),
                    },
                    enabled: false,
                    path: dir.display().to_string(),
                    error: Some(e.to_string()),
                }
            }
        })
        .collect()
}

/// Get all services from service-checker plugins
pub fn get_all_services(plugins_dir: &Path) -> Vec<ServiceDefinition> {
    get_all_plugins(plugins_dir)
        .into_iter()
        .filter(|p| p.enabled && p.error.is_none())
        .filter(|p| p.manifest.plugin_type == PluginType::ServiceChecker)
        .filter_map(|p| p.manifest.service)
        .collect()
}

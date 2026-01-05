//! Application-based routing module
//!
//! Manages per-application routing rules for sing-box configuration
//! Includes Windows application discovery via registry and filesystem

use crate::core::models::AppRoute;
use crate::core::storage::Storage;
use crate::core::errors::{IsolateError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};

// ============================================================================
// Types
// ============================================================================

/// Information about an installed application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

// ============================================================================
// AppRouter
// ============================================================================

/// Application-based routing manager
pub struct AppRouter {
    storage: Arc<Storage>,
}

impl AppRouter {
    /// Create a new app router
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Add an application route
    pub async fn add_route(&self, app_name: &str, app_path: &str, proxy_id: &str) -> Result<()> {
        // Validate app_name
        if app_name.trim().is_empty() {
            return Err(IsolateError::Validation("app_name cannot be empty".into()));
        }

        // Validate and normalize path
        let app_path = Self::validate_path(app_path)?;

        // Validate proxy_id
        if proxy_id.trim().is_empty() {
            return Err(IsolateError::Validation("proxy_id cannot be empty".into()));
        }

        let route = AppRoute {
            app_name: app_name.trim().to_string(),
            app_path: app_path.clone(),
            proxy_id: proxy_id.to_string(),
        };

        self.storage.save_app_route(&route).await?;
        info!(app_name = %app_name, app_path = %app_path, proxy_id = %proxy_id, "App route added");

        Ok(())
    }

    /// Remove an application route
    pub async fn remove_route(&self, app_path: &str) -> Result<()> {
        let app_path = Self::normalize_path(app_path);
        self.storage.delete_app_route(&app_path).await?;
        info!(app_path = %app_path, "App route removed");
        Ok(())
    }

    /// Get all application routes
    pub async fn get_routes(&self) -> Result<Vec<AppRoute>> {
        self.storage.get_app_routes().await
    }

    /// Get list of installed applications (Windows)
    pub async fn get_installed_apps(&self) -> Result<Vec<InstalledApp>> {
        let mut apps = HashSet::new();

        // Scan registry for installed apps
        #[cfg(windows)]
        {
            self.scan_registry_apps(&mut apps).await;
        }

        // Scan common directories
        self.scan_directory_apps(&mut apps).await;

        let mut result: Vec<InstalledApp> = apps.into_iter().collect();
        result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        info!(count = result.len(), "Found installed applications");
        Ok(result)
    }

    /// Generate sing-box routing rules from app routes
    pub fn generate_rules(&self, routes: &[AppRoute]) -> Vec<serde_json::Value> {
        let mut rules = Vec::new();

        // Group apps by proxy_id for efficient rules
        let mut proxy_apps: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for route in routes {
            // Extract process name from path
            let process_name = Self::extract_process_name(&route.app_path);
            if let Some(name) = process_name {
                proxy_apps
                    .entry(route.proxy_id.clone())
                    .or_default()
                    .push(name);
            }
        }

        // Generate rule for each proxy
        for (proxy_id, process_names) in proxy_apps {
            let rule = serde_json::json!({
                "process_name": process_names,
                "outbound": proxy_id
            });
            rules.push(rule);
            debug!(proxy_id = %proxy_id, apps_count = process_names.len(), "Generated app rule");
        }

        rules
    }

    /// Find route for a specific application
    pub async fn find_route(&self, app_path: &str) -> Result<Option<AppRoute>> {
        let routes = self.get_routes().await?;
        let normalized = Self::normalize_path(app_path);

        Ok(routes.into_iter().find(|r| {
            Self::normalize_path(&r.app_path) == normalized
        }))
    }

    // ========================================================================
    // Private Methods - Validation
    // ========================================================================

    /// Validate application path
    fn validate_path(path: &str) -> Result<String> {
        let path = Self::normalize_path(path);

        if path.is_empty() {
            return Err(IsolateError::Validation("Path cannot be empty".into()));
        }

        // Check if path looks like an executable
        let lower = path.to_lowercase();
        if !lower.ends_with(".exe") && !lower.ends_with(".com") {
            return Err(IsolateError::Validation(
                "Path must point to an executable (.exe or .com)".into()
            ));
        }

        Ok(path)
    }

    /// Normalize path (consistent separators, trim)
    fn normalize_path(path: &str) -> String {
        path.trim()
            .replace('/', "\\")
            .to_string()
    }

    /// Extract process name from full path
    fn extract_process_name(path: &str) -> Option<String> {
        Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    // ========================================================================
    // Private Methods - App Discovery
    // ========================================================================

    /// Scan Windows registry for installed applications
    #[cfg(windows)]
    async fn scan_registry_apps(&self, apps: &mut HashSet<InstalledApp>) {
        use winreg::enums::*;
        use winreg::RegKey;

        let registry_paths = [
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
            (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        ];

        for (hkey, path) in registry_paths {
            if let Ok(key) = RegKey::predef(hkey).open_subkey(path) {
                for subkey_name in key.enum_keys().filter_map(|k| k.ok()) {
                    if let Ok(subkey) = key.open_subkey(&subkey_name) {
                        if let Some(app) = Self::parse_registry_app(&subkey) {
                            apps.insert(app);
                        }
                    }
                }
            }
        }

        debug!("Registry scan completed");
    }

    /// Parse application info from registry key
    #[cfg(windows)]
    fn parse_registry_app(key: &winreg::RegKey) -> Option<InstalledApp> {
        // Get display name
        let name: String = key.get_value("DisplayName").ok()?;
        
        // Skip system components and updates
        if name.contains("Update") || name.contains("Hotfix") || name.starts_with("KB") {
            return None;
        }

        // Try to get executable path
        let path: Option<String> = key.get_value("DisplayIcon")
            .ok()
            .or_else(|| key.get_value("InstallLocation").ok())
            .and_then(|p: String| {
                // Clean up path (remove icon index like ",0")
                let clean = p.split(',').next().unwrap_or(&p).trim().to_string();
                if clean.to_lowercase().ends_with(".exe") {
                    Some(clean)
                } else {
                    None
                }
            });

        let path = path?;

        // Get icon path (same as exe or separate)
        let icon: Option<String> = key.get_value("DisplayIcon").ok();

        Some(InstalledApp {
            name,
            path,
            icon,
        })
    }

    /// Scan common directories for applications
    async fn scan_directory_apps(&self, apps: &mut HashSet<InstalledApp>) {
        let directories = Self::get_scan_directories();

        for dir in directories {
            if dir.exists() {
                self.scan_directory(&dir, apps, 2).await;
            }
        }

        debug!("Directory scan completed");
    }

    /// Get list of directories to scan for applications
    fn get_scan_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // Program Files
        if let Ok(pf) = std::env::var("ProgramFiles") {
            dirs.push(PathBuf::from(pf));
        }

        // Program Files (x86)
        if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
            dirs.push(PathBuf::from(pf86));
        }

        // Local AppData
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            dirs.push(PathBuf::from(&local));
            dirs.push(PathBuf::from(local).join("Programs"));
        }

        // User profile apps
        if let Ok(profile) = std::env::var("USERPROFILE") {
            dirs.push(PathBuf::from(&profile).join("AppData\\Local\\Programs"));
        }

        dirs
    }

    /// Recursively scan directory for executables
    async fn scan_directory(&self, dir: &Path, apps: &mut HashSet<InstalledApp>, depth: u32) {
        if depth == 0 {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_dir() {
                // Skip common non-app directories
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if !Self::should_skip_directory(name) {
                    Box::pin(self.scan_directory(&path, apps, depth - 1)).await;
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "exe" {
                        if let Some(app) = Self::create_app_from_path(&path) {
                            apps.insert(app);
                        }
                    }
                }
            }
        }
    }

    /// Check if directory should be skipped during scan
    fn should_skip_directory(name: &str) -> bool {
        let skip_dirs = [
            "cache", "temp", "tmp", "logs", "log", "data",
            "resources", "locales", "node_modules", ".git",
            "uninstall", "update", "updater", "crashpad",
        ];

        let lower = name.to_lowercase();
        skip_dirs.iter().any(|&d| lower.contains(d))
    }

    /// Create InstalledApp from executable path
    fn create_app_from_path(path: &Path) -> Option<InstalledApp> {
        let file_name = path.file_stem()?.to_str()?;
        let full_path = path.to_str()?;

        // Skip common non-user executables
        let lower = file_name.to_lowercase();
        if lower.contains("uninstall") || lower.contains("update") || lower.contains("crash") {
            return None;
        }

        Some(InstalledApp {
            name: Self::format_app_name(file_name),
            path: full_path.to_string(),
            icon: Some(full_path.to_string()),
        })
    }

    /// Format executable name to readable app name
    fn format_app_name(name: &str) -> String {
        // Convert camelCase and snake_case to spaces
        let mut result = String::new();
        let mut prev_lower = false;

        for c in name.chars() {
            if c == '_' || c == '-' {
                result.push(' ');
                prev_lower = false;
            } else if c.is_uppercase() && prev_lower {
                result.push(' ');
                result.push(c);
                prev_lower = false;
            } else {
                result.push(c);
                prev_lower = c.is_lowercase();
            }
        }

        // Capitalize first letter of each word
        result
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ============================================================================
// Hash implementation for InstalledApp (for HashSet)
// ============================================================================

impl std::hash::Hash for InstalledApp {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.to_lowercase().hash(state);
    }
}

impl PartialEq for InstalledApp {
    fn eq(&self, other: &Self) -> bool {
        self.path.to_lowercase() == other.path.to_lowercase()
    }
}

impl Eq for InstalledApp {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            AppRouter::normalize_path("C:/Program Files/App/app.exe"),
            "C:\\Program Files\\App\\app.exe"
        );
        assert_eq!(
            AppRouter::normalize_path("  C:\\App\\app.exe  "),
            "C:\\App\\app.exe"
        );
    }

    #[test]
    fn test_validate_path() {
        assert!(AppRouter::validate_path("C:\\App\\app.exe").is_ok());
        assert!(AppRouter::validate_path("C:\\App\\app.com").is_ok());
        assert!(AppRouter::validate_path("C:\\App\\app.txt").is_err());
        assert!(AppRouter::validate_path("").is_err());
    }

    #[test]
    fn test_extract_process_name() {
        assert_eq!(
            AppRouter::extract_process_name("C:\\Program Files\\App\\chrome.exe"),
            Some("chrome.exe".to_string())
        );
        assert_eq!(
            AppRouter::extract_process_name("discord.exe"),
            Some("discord.exe".to_string())
        );
    }

    #[test]
    fn test_format_app_name() {
        assert_eq!(AppRouter::format_app_name("myApp"), "My App");
        assert_eq!(AppRouter::format_app_name("my_app"), "My App");
        assert_eq!(AppRouter::format_app_name("my-app"), "My App");
        assert_eq!(AppRouter::format_app_name("chrome"), "Chrome");
    }

    #[test]
    fn test_generate_rules() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = Arc::new(rt.block_on(Storage::new()).unwrap());
        let router = AppRouter::new(storage);

        let routes = vec![
            AppRoute {
                app_name: "Chrome".to_string(),
                app_path: "C:\\Program Files\\Google\\Chrome\\chrome.exe".to_string(),
                proxy_id: "proxy1".to_string(),
            },
            AppRoute {
                app_name: "Firefox".to_string(),
                app_path: "C:\\Program Files\\Mozilla Firefox\\firefox.exe".to_string(),
                proxy_id: "proxy1".to_string(),
            },
            AppRoute {
                app_name: "Discord".to_string(),
                app_path: "C:\\Users\\User\\AppData\\Local\\Discord\\discord.exe".to_string(),
                proxy_id: "proxy2".to_string(),
            },
        ];

        let rules = router.generate_rules(&routes);
        assert_eq!(rules.len(), 2); // Two proxies
    }
}

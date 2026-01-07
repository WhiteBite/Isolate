//! Script Executor for Level 3 Plugins
//!
//! Manages execution of Lua scripts with proper sandboxing,
//! timeout handling, and permission enforcement.

use crate::plugins::lua_runtime::{CheckResult, LuaRuntime, PluginStorage, ScriptPermissions};
use crate::plugins::manifest::{PluginManifest, PluginType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Script not found: {0}")]
    NotFound(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Invalid plugin type: expected script-plugin")]
    InvalidPluginType,

    #[error("Script execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Script timeout after {0:?}")]
    Timeout(Duration),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lua error: {0}")]
    LuaError(String),
}

impl From<mlua::Error> for ScriptError {
    fn from(e: mlua::Error) -> Self {
        ScriptError::LuaError(e.to_string())
    }
}

/// Script execution context
#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub plugin_id: String,
    pub script_name: String,
    pub script_path: PathBuf,
    pub permissions: ScriptPermissions,
}

/// Script Executor - manages Lua script execution
pub struct ScriptExecutor {
    /// Plugin directory
    plugins_dir: PathBuf,
    /// Per-plugin storage instances
    storages: RwLock<HashMap<String, PluginStorage>>,
    /// Default timeout
    default_timeout: Duration,
    /// Default memory limit
    default_memory_limit: usize,
}

impl ScriptExecutor {
    /// Create a new script executor
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            storages: RwLock::new(HashMap::new()),
            default_timeout: Duration::from_secs(10),
            default_memory_limit: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Set default timeout
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Set default memory limit
    pub fn with_default_memory_limit(mut self, limit: usize) -> Self {
        self.default_memory_limit = limit;
        self
    }

    /// Get or create storage for a plugin
    async fn get_storage(&self, plugin_id: &str) -> PluginStorage {
        let mut storages = self.storages.write().await;
        storages
            .entry(plugin_id.to_string())
            .or_insert_with(PluginStorage::new)
            .clone()
    }

    /// Load plugin manifest
    async fn load_manifest(&self, plugin_id: &str) -> Result<PluginManifest, ScriptError> {
        let plugin_dir = self.plugins_dir.join(plugin_id);
        let manifest_path = plugin_dir.join("plugin.json");

        if !tokio::fs::try_exists(&manifest_path).await.unwrap_or(false) {
            return Err(ScriptError::PluginNotFound(plugin_id.to_string()));
        }

        let content = tokio::fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| ScriptError::ExecutionFailed(format!("Invalid manifest: {}", e)))?;

        Ok(manifest)
    }

    /// Build permissions from manifest
    fn build_permissions(&self, manifest: &PluginManifest) -> ScriptPermissions {
        ScriptPermissions {
            http_whitelist: manifest.permissions.http.clone(),
            timeout: self.default_timeout,
            memory_limit: self.default_memory_limit,
        }
    }

    /// Execute a script's check() function
    pub async fn execute_check(
        &self,
        plugin_id: &str,
        script_name: &str,
    ) -> Result<CheckResult, ScriptError> {
        info!(plugin = %plugin_id, script = %script_name, "Executing script check");

        // Load manifest
        let manifest = self.load_manifest(plugin_id).await?;

        // Verify plugin type
        if manifest.plugin_type != PluginType::ServiceChecker {
            // For now, allow any plugin type to have scripts
            // In the future, we might want a dedicated ScriptPlugin type
            debug!(
                plugin = %plugin_id,
                plugin_type = ?manifest.plugin_type,
                "Plugin type is not script-plugin, but allowing execution"
            );
        }

        // Build script path
        let script_path = self
            .plugins_dir
            .join(plugin_id)
            .join("scripts")
            .join(script_name);

        if !tokio::fs::try_exists(&script_path).await.unwrap_or(false) {
            return Err(ScriptError::NotFound(script_path.display().to_string()));
        }

        // Read script content
        let script_content = tokio::fs::read_to_string(&script_path).await?;

        // Build permissions
        let permissions = self.build_permissions(&manifest);

        // Get storage
        let storage = self.get_storage(plugin_id).await;

        // Execute in blocking thread (Lua is not Send)
        let timeout_duration = permissions.timeout;
        let plugin_id_owned = plugin_id.to_string();
        let _script_name_owned = script_name.to_string();
        
        let result = timeout(timeout_duration, tokio::task::spawn_blocking(move || {
            // Create runtime in blocking context
            let runtime = LuaRuntime::new(plugin_id_owned.clone(), permissions, storage)?;
            runtime.register_host_api()?;
            
            // Execute synchronously
            runtime.execute_check_sync(&script_content)
        }))
        .await;

        match result {
            Ok(Ok(Ok(check_result))) => {
                info!(
                    plugin = %plugin_id,
                    script = %script_name,
                    success = check_result.success,
                    latency = ?check_result.latency,
                    "Script check completed"
                );
                Ok(check_result)
            }
            Ok(Ok(Err(e))) => {
                error!(
                    plugin = %plugin_id,
                    script = %script_name,
                    error = %e,
                    "Script execution failed"
                );
                Err(ScriptError::from(e))
            }
            Ok(Err(e)) => {
                error!(
                    plugin = %plugin_id,
                    script = %script_name,
                    error = %e,
                    "Task join error"
                );
                Err(ScriptError::ExecutionFailed(e.to_string()))
            }
            Err(_) => {
                warn!(
                    plugin = %plugin_id,
                    script = %script_name,
                    timeout = ?timeout_duration,
                    "Script execution timed out"
                );
                Err(ScriptError::Timeout(timeout_duration))
            }
        }
    }

    /// Execute arbitrary Lua code (for testing/debugging)
    pub async fn execute_raw(
        &self,
        plugin_id: &str,
        script: &str,
    ) -> Result<serde_json::Value, ScriptError> {
        // Load manifest for permissions
        let manifest = self.load_manifest(plugin_id).await?;
        let permissions = self.build_permissions(&manifest);
        let storage = self.get_storage(plugin_id).await;

        // Execute in blocking thread (Lua is not Send)
        let timeout_duration = permissions.timeout;
        let plugin_id_owned = plugin_id.to_string();
        let script_owned = script.to_string();
        
        let result = timeout(timeout_duration, tokio::task::spawn_blocking(move || {
            // Create runtime in blocking context
            let runtime = LuaRuntime::new(plugin_id_owned, permissions, storage)?;
            runtime.register_host_api()?;

            // Execute synchronously
            let value = runtime.execute_sync(&script_owned)?;
            
            // Convert Lua value to JSON
            Ok::<_, mlua::Error>(lua_value_to_json(&value))
        }))
        .await;

        match result {
            Ok(Ok(Ok(json))) => Ok(json),
            Ok(Ok(Err(e))) => Err(ScriptError::from(e)),
            Ok(Err(e)) => Err(ScriptError::ExecutionFailed(e.to_string())),
            Err(_) => Err(ScriptError::Timeout(timeout_duration)),
        }
    }

    /// List available scripts for a plugin
    pub async fn list_scripts(&self, plugin_id: &str) -> Result<Vec<String>, ScriptError> {
        let scripts_dir = self.plugins_dir.join(plugin_id).join("scripts");

        if !tokio::fs::try_exists(&scripts_dir).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut scripts = Vec::new();
        let mut entries = tokio::fs::read_dir(&scripts_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "lua").unwrap_or(false) {
                if let Some(name) = path.file_name() {
                    scripts.push(name.to_string_lossy().to_string());
                }
            }
        }

        Ok(scripts)
    }

    /// Clear storage for a plugin
    pub async fn clear_storage(&self, plugin_id: &str) {
        let storage = self.get_storage(plugin_id).await;
        storage.clear().await;
    }

    /// Get storage value for a plugin
    pub async fn get_storage_value(
        &self,
        plugin_id: &str,
        key: &str,
    ) -> Option<serde_json::Value> {
        let storage = self.get_storage(plugin_id).await;
        storage.get(key).await
    }

    /// Set storage value for a plugin
    pub async fn set_storage_value(
        &self,
        plugin_id: &str,
        key: String,
        value: serde_json::Value,
    ) {
        let storage = self.get_storage(plugin_id).await;
        storage.set(key, value).await;
    }
}

/// Convert mlua::Value to serde_json::Value (simplified version)
fn lua_value_to_json(value: &mlua::Value) -> serde_json::Value {
    match value {
        mlua::Value::Nil => serde_json::Value::Null,
        mlua::Value::Boolean(b) => serde_json::Value::Bool(*b),
        mlua::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        mlua::Value::Number(n) => {
            serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        mlua::Value::String(s) => {
            serde_json::Value::String(s.to_str().unwrap_or_default().to_string())
        }
        mlua::Value::Table(table) => {
            // Try to determine if it's an array or object
            let mut is_array = true;
            let mut max_index = 0i64;
            
            for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
                if let Ok((k, _)) = pair {
                    match k {
                        mlua::Value::Integer(i) if i > 0 => {
                            max_index = max_index.max(i);
                        }
                        _ => {
                            is_array = false;
                            break;
                        }
                    }
                }
            }

            if is_array && max_index > 0 {
                let arr: Vec<serde_json::Value> = table
                    .clone()
                    .sequence_values::<mlua::Value>()
                    .filter_map(|r| r.ok())
                    .map(|v| lua_value_to_json(&v))
                    .collect();
                serde_json::Value::Array(arr)
            } else {
                let mut map = serde_json::Map::new();
                for pair in table.clone().pairs::<String, mlua::Value>() {
                    if let Ok((k, v)) = pair {
                        map.insert(k, lua_value_to_json(&v));
                    }
                }
                serde_json::Value::Object(map)
            }
        }
        _ => serde_json::Value::Null,
    }
}

/// Shared script executor instance
pub type SharedScriptExecutor = Arc<ScriptExecutor>;

/// Create a shared script executor
pub fn create_script_executor(plugins_dir: impl Into<PathBuf>) -> SharedScriptExecutor {
    Arc::new(ScriptExecutor::new(plugins_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    async fn create_test_plugin(plugins_dir: &Path, plugin_id: &str) {
        let plugin_dir = plugins_dir.join(plugin_id);
        let scripts_dir = plugin_dir.join("scripts");
        let _ = tokio::fs::create_dir_all(&scripts_dir).await;

        // Create manifest
        let manifest = format!(
            r#"{{
                "id": "{}",
                "name": "Test Plugin",
                "version": "1.0.0",
                "author": "Test",
                "type": "service-checker",
                "permissions": {{
                    "http": ["example.com", "*.test.com"]
                }}
            }}"#,
            plugin_id
        );
        let _ = tokio::fs::write(plugin_dir.join("plugin.json"), manifest).await;

        // Create test script
        let script = r#"
            function check()
                log_info("Running check")
                return {
                    success = true,
                    latency = 42,
                    details = { message = "Test passed" }
                }
            end
        "#;
        let _ = tokio::fs::write(scripts_dir.join("check.lua"), script).await;
    }

    #[tokio::test]
    async fn test_script_executor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ScriptExecutor::new(temp_dir.path());
        assert_eq!(executor.default_timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_list_scripts() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin").await;

        let executor = ScriptExecutor::new(temp_dir.path());
        let scripts = executor.list_scripts("test-plugin").await.unwrap();

        assert_eq!(scripts.len(), 1);
        assert!(scripts.contains(&"check.lua".to_string()));
    }

    #[tokio::test]
    async fn test_list_scripts_no_scripts_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("empty-plugin");
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        let manifest = r#"{
            "id": "empty-plugin",
            "name": "Empty Plugin",
            "version": "1.0.0",
            "author": "Test",
            "type": "service-checker"
        }"#;
        tokio::fs::write(plugin_dir.join("plugin.json"), manifest)
            .await
            .unwrap();

        let executor = ScriptExecutor::new(temp_dir.path());
        let scripts = executor.list_scripts("empty-plugin").await.unwrap();

        assert!(scripts.is_empty());
    }

    #[tokio::test]
    async fn test_execute_check() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin").await;

        let executor = ScriptExecutor::new(temp_dir.path());
        let result = executor
            .execute_check("test-plugin", "check.lua")
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.latency, Some(42));
        assert!(result.details.is_some());
    }

    #[tokio::test]
    async fn test_execute_check_not_found() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin").await;

        let executor = ScriptExecutor::new(temp_dir.path());
        let result = executor
            .execute_check("test-plugin", "nonexistent.lua")
            .await;

        assert!(matches!(result, Err(ScriptError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_execute_check_plugin_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let executor = ScriptExecutor::new(temp_dir.path());
        let result = executor
            .execute_check("nonexistent-plugin", "check.lua")
            .await;

        assert!(matches!(result, Err(ScriptError::PluginNotFound(_))));
    }

    #[tokio::test]
    async fn test_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin").await;

        let executor = ScriptExecutor::new(temp_dir.path());

        // Set value
        executor
            .set_storage_value(
                "test-plugin",
                "test_key".to_string(),
                serde_json::json!("test_value"),
            )
            .await;

        // Get value
        let value = executor
            .get_storage_value("test-plugin", "test_key")
            .await;
        assert_eq!(value, Some(serde_json::json!("test_value")));

        // Clear storage
        executor.clear_storage("test-plugin").await;
        let value = executor
            .get_storage_value("test-plugin", "test_key")
            .await;
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_execute_raw() {
        let temp_dir = TempDir::new().unwrap();
        create_test_plugin(temp_dir.path(), "test-plugin").await;

        let executor = ScriptExecutor::new(temp_dir.path());
        let result = executor
            .execute_raw("test-plugin", "return 1 + 2")
            .await
            .unwrap();

        assert_eq!(result, serde_json::json!(3));
    }

    #[tokio::test]
    async fn test_script_with_json() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("json-plugin");
        let scripts_dir = plugin_dir.join("scripts");
        tokio::fs::create_dir_all(&scripts_dir).await.unwrap();

        let manifest = r#"{
            "id": "json-plugin",
            "name": "JSON Plugin",
            "version": "1.0.0",
            "author": "Test",
            "type": "service-checker"
        }"#;
        tokio::fs::write(plugin_dir.join("plugin.json"), manifest)
            .await
            .unwrap();

        let script = r#"
            function check()
                local data = { name = "test", values = {1, 2, 3} }
                local json = json_encode(data)
                local decoded = json_decode(json)
                
                return {
                    success = decoded.name == "test",
                    details = decoded
                }
            end
        "#;
        tokio::fs::write(scripts_dir.join("check.lua"), script)
            .await
            .unwrap();

        let executor = ScriptExecutor::new(temp_dir.path());
        let result = executor
            .execute_check("json-plugin", "check.lua")
            .await
            .unwrap();

        assert!(result.success);
    }
}

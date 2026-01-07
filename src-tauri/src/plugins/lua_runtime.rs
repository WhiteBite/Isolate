//! Lua Runtime for Level 3 Script Plugins
//!
//! Provides a sandboxed Lua 5.4 runtime for executing plugin scripts.
//! Features:
//! - Timeout protection
//! - Memory limits
//! - HTTP whitelist
//! - Isolated storage per plugin

use crate::core::errors::IsolateError;
use mlua::{Function, Lua, Result as LuaResult, Table, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Result of a check() function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub success: bool,
    pub latency: Option<u64>,
    pub error: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl Default for CheckResult {
    fn default() -> Self {
        Self {
            success: false,
            latency: None,
            error: Some("No result".to_string()),
            details: None,
        }
    }
}

/// Script permissions configuration
#[derive(Debug, Clone, Default)]
pub struct ScriptPermissions {
    /// Allowed HTTP domains (supports wildcards like "*.discord.com")
    pub http_whitelist: Vec<String>,
    /// Maximum execution time
    pub timeout: Duration,
    /// Maximum memory usage in bytes
    pub memory_limit: usize,
}

impl ScriptPermissions {
    pub fn new() -> Self {
        Self {
            http_whitelist: Vec::new(),
            timeout: Duration::from_secs(10),
            memory_limit: 10 * 1024 * 1024, // 10MB
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    pub fn with_http_whitelist(mut self, domains: Vec<String>) -> Self {
        self.http_whitelist = domains;
        self
    }

    /// Check if a URL is allowed by the whitelist
    pub fn is_url_allowed(&self, url: &str) -> bool {
        if self.http_whitelist.is_empty() {
            return false;
        }

        // Parse the URL to get the host
        let host = match url::Url::parse(url) {
            Ok(parsed) => parsed.host_str().map(|s| s.to_string()),
            Err(_) => return false,
        };

        let host = match host {
            Some(h) => h,
            None => return false,
        };

        for pattern in &self.http_whitelist {
            if pattern.starts_with("*.") {
                // Wildcard pattern: *.discord.com matches voice.discord.com
                let suffix = &pattern[1..]; // .discord.com
                if host.ends_with(suffix) || host == &pattern[2..] {
                    return true;
                }
            } else if host == *pattern {
                return true;
            }
        }

        false
    }
}

/// Per-plugin storage
#[derive(Debug, Clone, Default)]
pub struct PluginStorage {
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl PluginStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }

    pub async fn set(&self, key: String, value: serde_json::Value) {
        let mut data = self.data.write().await;
        data.insert(key, value);
    }

    pub async fn delete(&self, key: &str) -> bool {
        let mut data = self.data.write().await;
        data.remove(key).is_some()
    }

    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }
}

/// HTTP client for Lua scripts
pub struct LuaHttpClient {
    client: reqwest::Client,
    permissions: ScriptPermissions,
}

impl LuaHttpClient {
    pub fn new(permissions: ScriptPermissions) -> Self {
        let client = reqwest::Client::builder()
            .timeout(permissions.timeout)
            .build()
            .unwrap_or_default();

        Self {
            client,
            permissions,
        }
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse, IsolateError> {
        if !self.permissions.is_url_allowed(url) {
            return Err(IsolateError::Network(format!("URL not in whitelist: {}", url)));
        }

        let start = std::time::Instant::now();
        
        match self.client.get(url).send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                
                Ok(HttpResponse {
                    status,
                    body,
                    latency_ms,
                })
            }
            Err(e) => Err(IsolateError::Network(e.to_string())),
        }
    }

    pub async fn post(&self, url: &str, body: &str) -> Result<HttpResponse, IsolateError> {
        if !self.permissions.is_url_allowed(url) {
            return Err(IsolateError::Network(format!("URL not in whitelist: {}", url)));
        }

        let start = std::time::Instant::now();
        
        match self.client.post(url).body(body.to_string()).send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let status = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                
                Ok(HttpResponse {
                    status,
                    body,
                    latency_ms,
                })
            }
            Err(e) => Err(IsolateError::Network(e.to_string())),
        }
    }

    pub async fn head(&self, url: &str) -> Result<HttpResponse, IsolateError> {
        if !self.permissions.is_url_allowed(url) {
            return Err(IsolateError::Network(format!("URL not in whitelist: {}", url)));
        }

        let start = std::time::Instant::now();
        
        match self.client.head(url).send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let status = response.status().as_u16();
                
                Ok(HttpResponse {
                    status,
                    body: String::new(),
                    latency_ms,
                })
            }
            Err(e) => Err(IsolateError::Network(e.to_string())),
        }
    }
}

/// HTTP response for Lua
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub latency_ms: u64,
}

/// Lua Runtime with sandbox
pub struct LuaRuntime {
    pub(crate) lua: Lua,
    plugin_id: String,
    permissions: ScriptPermissions,
    storage: PluginStorage,
}

impl LuaRuntime {
    /// Create a new Lua runtime for a plugin
    pub fn new(
        plugin_id: String,
        permissions: ScriptPermissions,
        storage: PluginStorage,
    ) -> LuaResult<Self> {
        let lua = Lua::new();

        // Set memory limit
        lua.set_memory_limit(permissions.memory_limit)?;

        Ok(Self {
            lua,
            plugin_id,
            permissions,
            storage,
        })
    }

    /// Register all Host API functions
    pub fn register_host_api(&self) -> LuaResult<()> {
        let globals = self.lua.globals();

        // Register plugin info
        globals.set("PLUGIN_ID", self.plugin_id.clone())?;

        // Register logging functions
        self.register_logging(&globals)?;

        // Register JSON functions
        self.register_json(&globals)?;

        // Note: HTTP and Storage functions need async context
        // They will be registered when executing scripts

        Ok(())
    }

    /// Register logging functions
    fn register_logging(&self, globals: &Table) -> LuaResult<()> {
        let plugin_id = self.plugin_id.clone();

        // log_info
        let pid = plugin_id.clone();
        let log_info = self.lua.create_function(move |_, msg: String| {
            info!(plugin = %pid, "{}", msg);
            Ok(())
        })?;
        globals.set("log_info", log_info)?;

        // log_warn
        let pid = plugin_id.clone();
        let log_warn = self.lua.create_function(move |_, msg: String| {
            warn!(plugin = %pid, "{}", msg);
            Ok(())
        })?;
        globals.set("log_warn", log_warn)?;

        // log_error
        let pid = plugin_id.clone();
        let log_error = self.lua.create_function(move |_, msg: String| {
            error!(plugin = %pid, "{}", msg);
            Ok(())
        })?;
        globals.set("log_error", log_error)?;

        // log_debug
        let pid = plugin_id.clone();
        let log_debug = self.lua.create_function(move |_, msg: String| {
            debug!(plugin = %pid, "{}", msg);
            Ok(())
        })?;
        globals.set("log_debug", log_debug)?;

        Ok(())
    }

    /// Register JSON functions
    fn register_json(&self, globals: &Table) -> LuaResult<()> {
        // json_encode
        let json_encode = self.lua.create_function(|lua, value: Value| {
            let json_value = lua_value_to_json(lua, value)?;
            serde_json::to_string(&json_value)
                .map_err(|e| mlua::Error::external(e))
        })?;
        globals.set("json_encode", json_encode)?;

        // json_decode
        let json_decode = self.lua.create_function(|lua, json_str: String| {
            let json_value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| mlua::Error::external(e))?;
            json_to_lua_value(lua, json_value)
        })?;
        globals.set("json_decode", json_decode)?;

        Ok(())
    }

    /// Execute a Lua script
    pub fn execute(&self, script: &str) -> LuaResult<Value<'_>> {
        self.lua.load(script).eval()
    }

    /// Execute a Lua script synchronously (for use in spawn_blocking)
    pub fn execute_sync(&self, script: &str) -> LuaResult<Value<'_>> {
        self.lua.load(script).eval()
    }

    /// Execute a Lua script and call check() function synchronously
    pub fn execute_check_sync(&self, script: &str) -> LuaResult<CheckResult> {
        // Register sync-capable functions
        self.register_sync_api()?;

        // Load and execute the script
        self.lua.load(script).exec()?;

        // Get the check function
        let globals = self.lua.globals();
        let check_fn: Function = globals.get("check")?;

        // Call check()
        let result: Value = check_fn.call(())?;

        // Convert result to CheckResult
        self.value_to_check_result(result)
    }

    /// Register sync-capable API (HTTP, Storage) for use in blocking context
    /// 
    /// Uses a shared Tokio runtime for efficiency instead of creating new runtime per call.
    fn register_sync_api(&self) -> LuaResult<()> {
        let globals = self.lua.globals();
        let http_client = Arc::new(LuaHttpClient::new(self.permissions.clone()));
        let storage = self.storage.clone();
        
        // Create a shared runtime for sync operations
        let shared_rt = Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| mlua::Error::external(format!("Failed to create runtime: {}", e)))?
        );

        // http_get - uses shared runtime with timeout
        let client = http_client.clone();
        let rt = shared_rt.clone();
        let http_get = self.lua.create_function(move |lua, url: String| {
            let client = client.clone();
            let rt = rt.clone();
            
            // Run in separate thread to avoid blocking if called from async context
            let result = std::thread::spawn(move || {
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(30),
                        client.get(&url)
                    ).await
                })
            }).join().map_err(|_| mlua::Error::external("Thread panic"))?;

            match result {
                Ok(Ok(response)) => {
                    let table = lua.create_table()?;
                    table.set("status", response.status)?;
                    table.set("body", response.body)?;
                    table.set("latency_ms", response.latency_ms)?;
                    Ok(Value::Table(table))
                }
                Ok(Err(e)) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", e.to_string())?;
                    Ok(Value::Table(table))
                }
                Err(_) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", "Request timeout")?;
                    Ok(Value::Table(table))
                }
            }
        })?;
        globals.set("http_get", http_get)?;

        // http_post - uses shared runtime with timeout
        let client = http_client.clone();
        let rt = shared_rt.clone();
        let http_post = self.lua.create_function(move |lua, (url, body): (String, String)| {
            let client = client.clone();
            let rt = rt.clone();
            
            let result = std::thread::spawn(move || {
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(30),
                        client.post(&url, &body)
                    ).await
                })
            }).join().map_err(|_| mlua::Error::external("Thread panic"))?;

            match result {
                Ok(Ok(response)) => {
                    let table = lua.create_table()?;
                    table.set("status", response.status)?;
                    table.set("body", response.body)?;
                    table.set("latency_ms", response.latency_ms)?;
                    Ok(Value::Table(table))
                }
                Ok(Err(e)) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", e.to_string())?;
                    Ok(Value::Table(table))
                }
                Err(_) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", "Request timeout")?;
                    Ok(Value::Table(table))
                }
            }
        })?;
        globals.set("http_post", http_post)?;

        // http_head - uses shared runtime with timeout
        let client = http_client.clone();
        let rt = shared_rt.clone();
        let http_head = self.lua.create_function(move |lua, url: String| {
            let client = client.clone();
            let rt = rt.clone();
            
            let result = std::thread::spawn(move || {
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(30),
                        client.head(&url)
                    ).await
                })
            }).join().map_err(|_| mlua::Error::external("Thread panic"))?;

            match result {
                Ok(Ok(response)) => {
                    let table = lua.create_table()?;
                    table.set("status", response.status)?;
                    table.set("latency_ms", response.latency_ms)?;
                    Ok(Value::Table(table))
                }
                Ok(Err(e)) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", e.to_string())?;
                    Ok(Value::Table(table))
                }
                Err(_) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", "Request timeout")?;
                    Ok(Value::Table(table))
                }
            }
        })?;
        globals.set("http_head", http_head)?;

        // storage_get - uses shared runtime with timeout
        let stor = storage.clone();
        let rt = shared_rt.clone();
        let storage_get = self.lua.create_function(move |lua, key: String| {
            let stor = stor.clone();
            let rt = rt.clone();
            
            let result = std::thread::spawn(move || {
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(5),
                        stor.get(&key)
                    ).await
                })
            }).join().map_err(|_| mlua::Error::external("Thread panic"))?;

            match result {
                Ok(Some(value)) => json_to_lua_value(lua, value),
                Ok(None) => Ok(Value::Nil),
                Err(_) => Err(mlua::Error::external("Storage operation timeout")),
            }
        })?;
        globals.set("storage_get", storage_get)?;

        // storage_set - uses shared runtime with timeout
        let stor = storage.clone();
        let rt = shared_rt.clone();
        let storage_set = self.lua.create_function(move |lua, (key, value): (String, Value)| {
            let stor = stor.clone();
            let json_value = lua_value_to_json(lua, value)?;
            let rt = rt.clone();
            
            let result = std::thread::spawn(move || {
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(5),
                        async { stor.set(key, json_value).await }
                    ).await
                })
            }).join().map_err(|_| mlua::Error::external("Thread panic"))?;

            match result {
                Ok(()) => Ok(()),
                Err(_) => Err(mlua::Error::external("Storage operation timeout")),
            }
        })?;
        globals.set("storage_set", storage_set)?;

        // storage_delete - uses shared runtime with timeout
        let stor = storage.clone();
        let rt = shared_rt.clone();
        let storage_delete = self.lua.create_function(move |_, key: String| {
            let stor = stor.clone();
            let rt = rt.clone();
            
            let result = std::thread::spawn(move || {
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_secs(5),
                        stor.delete(&key)
                    ).await
                })
            }).join().map_err(|_| mlua::Error::external("Thread panic"))?;

            match result {
                Ok(deleted) => Ok(deleted),
                Err(_) => Err(mlua::Error::external("Storage operation timeout")),
            }
        })?;
        globals.set("storage_delete", storage_delete)?;

        Ok(())
    }

    /// Execute a Lua script and call check() function
    pub async fn execute_check(&self, script: &str) -> LuaResult<CheckResult> {
        // Register async-capable functions
        self.register_async_api().await?;

        // Load and execute the script
        self.lua.load(script).exec()?;

        // Get the check function
        let globals = self.lua.globals();
        let check_fn: Function = globals.get("check")?;

        // Call check()
        let result: Value = check_fn.call(())?;

        // Convert result to CheckResult
        self.value_to_check_result(result)
    }

    /// Register async-capable API (HTTP, Storage)
    /// 
    /// Uses channels to communicate between Lua sync functions and async runtime,
    /// avoiding blocking the async runtime with block_on() calls.
    async fn register_async_api(&self) -> LuaResult<()> {
        let globals = self.lua.globals();
        let http_client = Arc::new(LuaHttpClient::new(self.permissions.clone()));
        let storage = self.storage.clone();

        // http_get - uses spawn_blocking to avoid blocking async runtime
        let client = http_client.clone();
        let http_get = self.lua.create_function(move |lua, url: String| {
            let client = client.clone();
            
            // Get handle to current runtime
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => return Err(mlua::Error::external("No async runtime available")),
            };
            
            // Use oneshot channel to get result without blocking runtime
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                let result = tokio::time::timeout(
                    Duration::from_secs(30),
                    client.get(&url)
                ).await;
                
                let _ = tx.send(match result {
                    Ok(r) => r,
                    Err(_) => Err(IsolateError::Network("Request timeout".to_string())),
                });
            });
            
            // Wait for result with timeout
            let result = rx.recv_timeout(Duration::from_secs(35))
                .map_err(|_| mlua::Error::external("HTTP request timeout"))?;

            match result {
                Ok(response) => {
                    let table = lua.create_table()?;
                    table.set("status", response.status)?;
                    table.set("body", response.body)?;
                    table.set("latency_ms", response.latency_ms)?;
                    Ok(Value::Table(table))
                }
                Err(e) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", e.to_string())?;
                    Ok(Value::Table(table))
                }
            }
        })?;
        globals.set("http_get", http_get)?;

        // http_post - uses spawn_blocking to avoid blocking async runtime
        let client = http_client.clone();
        let http_post = self.lua.create_function(move |lua, (url, body): (String, String)| {
            let client = client.clone();
            
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => return Err(mlua::Error::external("No async runtime available")),
            };
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                let result = tokio::time::timeout(
                    Duration::from_secs(30),
                    client.post(&url, &body)
                ).await;
                
                let _ = tx.send(match result {
                    Ok(r) => r,
                    Err(_) => Err(IsolateError::Network("Request timeout".to_string())),
                });
            });
            
            let result = rx.recv_timeout(Duration::from_secs(35))
                .map_err(|_| mlua::Error::external("HTTP request timeout"))?;

            match result {
                Ok(response) => {
                    let table = lua.create_table()?;
                    table.set("status", response.status)?;
                    table.set("body", response.body)?;
                    table.set("latency_ms", response.latency_ms)?;
                    Ok(Value::Table(table))
                }
                Err(e) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", e.to_string())?;
                    Ok(Value::Table(table))
                }
            }
        })?;
        globals.set("http_post", http_post)?;

        // http_head - uses spawn_blocking to avoid blocking async runtime
        let client = http_client.clone();
        let http_head = self.lua.create_function(move |lua, url: String| {
            let client = client.clone();
            
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => return Err(mlua::Error::external("No async runtime available")),
            };
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                let result = tokio::time::timeout(
                    Duration::from_secs(30),
                    client.head(&url)
                ).await;
                
                let _ = tx.send(match result {
                    Ok(r) => r,
                    Err(_) => Err(IsolateError::Network("Request timeout".to_string())),
                });
            });
            
            let result = rx.recv_timeout(Duration::from_secs(35))
                .map_err(|_| mlua::Error::external("HTTP request timeout"))?;

            match result {
                Ok(response) => {
                    let table = lua.create_table()?;
                    table.set("status", response.status)?;
                    table.set("latency_ms", response.latency_ms)?;
                    Ok(Value::Table(table))
                }
                Err(e) => {
                    let table = lua.create_table()?;
                    table.set("status", 0)?;
                    table.set("error", e.to_string())?;
                    Ok(Value::Table(table))
                }
            }
        })?;
        globals.set("http_head", http_head)?;

        // storage_get - uses channel for non-blocking async
        let stor = storage.clone();
        let storage_get = self.lua.create_function(move |lua, key: String| {
            let stor = stor.clone();
            
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => return Err(mlua::Error::external("No async runtime available")),
            };
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                let result = stor.get(&key).await;
                let _ = tx.send(result);
            });
            
            let result = rx.recv_timeout(Duration::from_secs(5))
                .map_err(|_| mlua::Error::external("Storage operation timeout"))?;

            match result {
                Some(value) => json_to_lua_value(lua, value),
                None => Ok(Value::Nil),
            }
        })?;
        globals.set("storage_get", storage_get)?;

        // storage_set - uses channel for non-blocking async
        let stor = storage.clone();
        let storage_set = self.lua.create_function(move |lua, (key, value): (String, Value)| {
            let stor = stor.clone();
            let json_value = lua_value_to_json(lua, value)?;
            
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => return Err(mlua::Error::external("No async runtime available")),
            };
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                stor.set(key, json_value).await;
                let _ = tx.send(());
            });
            
            rx.recv_timeout(Duration::from_secs(5))
                .map_err(|_| mlua::Error::external("Storage operation timeout"))?;

            Ok(())
        })?;
        globals.set("storage_set", storage_set)?;

        // storage_delete - uses channel for non-blocking async
        let stor = storage.clone();
        let storage_delete = self.lua.create_function(move |_, key: String| {
            let stor = stor.clone();
            
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(h) => h,
                Err(_) => return Err(mlua::Error::external("No async runtime available")),
            };
            
            let (tx, rx) = std::sync::mpsc::channel();
            
            handle.spawn(async move {
                let deleted = stor.delete(&key).await;
                let _ = tx.send(deleted);
            });
            
            let deleted = rx.recv_timeout(Duration::from_secs(5))
                .map_err(|_| mlua::Error::external("Storage operation timeout"))?;

            Ok(deleted)
        })?;
        globals.set("storage_delete", storage_delete)?;

        Ok(())
    }

    /// Convert Lua Value to CheckResult
    fn value_to_check_result(&self, value: Value) -> LuaResult<CheckResult> {
        match value {
            Value::Table(table) => {
                let success: bool = table.get("success").unwrap_or(false);
                let latency: Option<u64> = table.get("latency").ok();
                let error: Option<String> = table.get("error").ok();
                
                // Convert details table to JSON
                let details: Option<serde_json::Value> = table
                    .get::<_, Value>("details")
                    .ok()
                    .and_then(|v| {
                        if matches!(v, Value::Nil) {
                            None
                        } else {
                            lua_value_to_json(&self.lua, v).ok()
                        }
                    });

                Ok(CheckResult {
                    success,
                    latency,
                    error,
                    details,
                })
            }
            Value::Boolean(success) => Ok(CheckResult {
                success,
                latency: None,
                error: if success { None } else { Some("Check failed".to_string()) },
                details: None,
            }),
            _ => Ok(CheckResult::default()),
        }
    }
}

/// Convert Lua Value to serde_json::Value
fn lua_value_to_json(lua: &Lua, value: Value) -> LuaResult<serde_json::Value> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(i.into())),
        Value::Number(n) => {
            serde_json::Number::from_f64(n)
                .map(serde_json::Value::Number)
                .ok_or_else(|| mlua::Error::external("Invalid float"))
        }
        Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        Value::Table(table) => {
            // Check if it's an array (sequential integer keys starting from 1)
            let is_array = table.clone().pairs::<i64, Value>()
                .filter_map(|r| r.ok())
                .enumerate()
                .all(|(i, (k, _))| k == (i + 1) as i64);

            if is_array && table.len()? > 0 {
                let arr: Vec<serde_json::Value> = table
                    .sequence_values::<Value>()
                    .filter_map(|r| r.ok())
                    .filter_map(|v| lua_value_to_json(lua, v).ok())
                    .collect();
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for pair in table.pairs::<String, Value>() {
                    let (k, v) = pair?;
                    map.insert(k, lua_value_to_json(lua, v)?);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Ok(serde_json::Value::Null),
    }
}

/// Convert serde_json::Value to Lua Value
fn json_to_lua_value(lua: &Lua, value: serde_json::Value) -> LuaResult<Value<'_>> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(&s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map {
                table.set(k, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_permissions_url_whitelist() {
        let perms = ScriptPermissions::new()
            .with_http_whitelist(vec![
                "discord.com".to_string(),
                "*.discord.gg".to_string(),
            ]);

        assert!(perms.is_url_allowed("https://discord.com/api/v10"));
        assert!(perms.is_url_allowed("https://voice.discord.gg/"));
        assert!(perms.is_url_allowed("https://discord.gg/"));
        assert!(!perms.is_url_allowed("https://google.com/"));
        assert!(!perms.is_url_allowed("https://evil.com/discord.com"));
    }

    #[test]
    fn test_script_permissions_empty_whitelist() {
        let perms = ScriptPermissions::new();
        assert!(!perms.is_url_allowed("https://discord.com/"));
    }

    #[test]
    fn test_lua_runtime_creation() {
        let perms = ScriptPermissions::new();
        let storage = PluginStorage::new();
        let runtime = LuaRuntime::new("test-plugin".to_string(), perms, storage);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_lua_runtime_basic_execution() {
        let perms = ScriptPermissions::new();
        let storage = PluginStorage::new();
        let runtime = LuaRuntime::new("test-plugin".to_string(), perms, storage).unwrap();
        runtime.register_host_api().unwrap();

        // Execute and check result in same scope
        let n: i64 = runtime.lua.load("return 1 + 2").eval().unwrap();
        assert_eq!(n, 3);
    }

    #[test]
    fn test_lua_runtime_logging() {
        let perms = ScriptPermissions::new();
        let storage = PluginStorage::new();
        let runtime = LuaRuntime::new("test-plugin".to_string(), perms, storage).unwrap();
        runtime.register_host_api().unwrap();

        // Should not panic
        let result = runtime.execute(r#"
            log_info("Test info message")
            log_warn("Test warning")
            log_error("Test error")
            return true
        "#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lua_runtime_json() {
        let perms = ScriptPermissions::new();
        let storage = PluginStorage::new();
        let runtime = LuaRuntime::new("test-plugin".to_string(), perms, storage).unwrap();
        runtime.register_host_api().unwrap();

        let result = runtime.execute(r#"
            local data = { name = "test", value = 42 }
            local json = json_encode(data)
            local decoded = json_decode(json)
            return decoded.value
        "#);
        
        assert!(result.is_ok());
        let n: i64 = runtime.lua.load(r#"
            local data = { name = "test", value = 42 }
            local json = json_encode(data)
            local decoded = json_decode(json)
            return decoded.value
        "#).eval().unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_lua_runtime_plugin_id() {
        let perms = ScriptPermissions::new();
        let storage = PluginStorage::new();
        let runtime = LuaRuntime::new("my-plugin".to_string(), perms, storage).unwrap();
        runtime.register_host_api().unwrap();

        let plugin_id: String = runtime.lua.load("return PLUGIN_ID").eval().unwrap();
        assert_eq!(plugin_id, "my-plugin");
    }

    #[tokio::test]
    async fn test_plugin_storage() {
        let storage = PluginStorage::new();
        
        // Initially empty
        assert!(storage.get("key1").await.is_none());
        
        // Set and get
        storage.set("key1".to_string(), serde_json::json!("value1")).await;
        let value = storage.get("key1").await;
        assert_eq!(value, Some(serde_json::json!("value1")));
        
        // Delete
        assert!(storage.delete("key1").await);
        assert!(storage.get("key1").await.is_none());
        
        // Delete non-existent
        assert!(!storage.delete("key1").await);
    }

    #[test]
    fn test_check_result_default() {
        let result = CheckResult::default();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}

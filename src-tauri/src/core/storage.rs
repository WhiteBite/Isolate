//! Storage - SQLite хранилище для настроек и кэша стратегий
//!
//! Путь к БД: %APPDATA%/Isolate/data.db

use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::{AppRoute, DomainRoute, ProxyConfig, ProxyProtocol, Settings};

// ============================================================================
// Constants
// ============================================================================

/// Имя файла базы данных
const DB_FILENAME: &str = "data.db";
/// Имя директории приложения
const APP_DIR_NAME: &str = "Isolate";
/// Время жизни кэша стратегий (24 часа)
const CACHE_TTL_SECONDS: i64 = 24 * 60 * 60;

// ============================================================================
// Storage
// ============================================================================

/// SQLite хранилище
pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    /// Создаёт новое хранилище
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        Self::open(&db_path)
    }

    /// Открывает хранилище по указанному пути
    pub fn open(path: &PathBuf) -> Result<Self> {
        // Создаём директорию если не существует
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        let storage = Self {
            conn: Mutex::new(conn),
        };

        storage.init_schema()?;

        info!(path = %path.display(), "Storage initialized");
        Ok(storage)
    }

    /// Возвращает путь к базе данных
    pub fn get_db_path() -> Result<PathBuf> {
        let app_data = std::env::var("APPDATA").map_err(|_| {
            IsolateError::Storage("APPDATA environment variable not found".into())
        })?;

        let path = PathBuf::from(app_data)
            .join(APP_DIR_NAME)
            .join(DB_FILENAME);

        Ok(path)
    }

    // ========================================================================
    // Settings CRUD
    // ========================================================================

    /// Получает настройку по ключу
    pub fn get_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let result: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;

        match result {
            Some(json) => {
                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Устанавливает настройку
    pub fn set_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;

        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            params![key, json],
        )?;

        debug!(key, "Setting updated");
        Ok(())
    }

    /// Удаляет настройку
    pub fn delete_setting(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;

        debug!(key, "Setting deleted");
        Ok(())
    }

    /// Получает все настройки
    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut settings = Vec::new();
        for row in rows {
            settings.push(row?);
        }

        Ok(settings)
    }

    // ========================================================================
    // App Settings (typed)
    // ========================================================================

    /// Получает настройки приложения
    pub fn get_settings(&self) -> Result<Settings> {
        let settings: Option<Settings> = self.get_setting(settings_keys::APP_SETTINGS)?;
        Ok(settings.unwrap_or_default())
    }

    /// Сохраняет настройки приложения
    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.set_setting(settings_keys::APP_SETTINGS, settings)
    }

    /// Получает состояние включённости сервиса
    pub fn get_service_enabled(&self, service_id: &str) -> Result<bool> {
        let key = format!("{}{}", settings_keys::SERVICE_ENABLED_PREFIX, service_id);
        let enabled: Option<bool> = self.get_setting(&key)?;
        // По умолчанию сервисы включены
        Ok(enabled.unwrap_or(true))
    }

    /// Устанавливает состояние включённости сервиса
    pub fn set_service_enabled(&self, service_id: &str, enabled: bool) -> Result<()> {
        let key = format!("{}{}", settings_keys::SERVICE_ENABLED_PREFIX, service_id);
        self.set_setting(&key, &enabled)
    }

    // ========================================================================
    // Strategy Cache
    // ========================================================================

    /// Получает кэшированную стратегию для окружения
    pub fn get_cached_strategy(&self, env_key: &str) -> Result<Option<CachedStrategy>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let result = conn
            .query_row(
                r#"
                SELECT strategy_id, score, timestamp
                FROM strategy_cache
                WHERE env_key = ?1
                  AND timestamp > unixepoch() - ?2
                "#,
                params![env_key, CACHE_TTL_SECONDS],
                |row| {
                    Ok(CachedStrategy {
                        strategy_id: row.get(0)?,
                        score: row.get(1)?,
                        timestamp: row.get(2)?,
                    })
                },
            )
            .optional()?;

        if result.is_some() {
            debug!(env_key, "Cache hit for strategy");
        }

        Ok(result)
    }

    /// Сохраняет стратегию в кэш
    pub fn cache_strategy(&self, env_key: &str, strategy_id: &str, score: f64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO strategy_cache (env_key, strategy_id, score, timestamp)
            VALUES (?1, ?2, ?3, unixepoch())
            "#,
            params![env_key, strategy_id, score],
        )?;

        debug!(env_key, strategy_id, score, "Strategy cached");
        Ok(())
    }

    /// Инвалидирует кэш для окружения
    pub fn invalidate_cache(&self, env_key: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            "DELETE FROM strategy_cache WHERE env_key = ?1",
            params![env_key],
        )?;

        debug!(env_key, "Cache invalidated");
        Ok(())
    }

    /// Очищает весь кэш стратегий
    pub fn clear_cache(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute("DELETE FROM strategy_cache", [])?;

        info!("Strategy cache cleared");
        Ok(())
    }

    /// Очищает устаревшие записи кэша
    pub fn cleanup_expired_cache(&self) -> Result<u64> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let deleted = conn.execute(
            "DELETE FROM strategy_cache WHERE timestamp < unixepoch() - ?1",
            params![CACHE_TTL_SECONDS],
        )?;

        if deleted > 0 {
            debug!(deleted, "Expired cache entries cleaned up");
        }

        Ok(deleted as u64)
    }

    // ========================================================================
    // Proxy Management
    // ========================================================================

    /// Save a proxy configuration
    pub fn save_proxy(&self, proxy: &ProxyConfig) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let protocol = serde_json::to_string(&proxy.protocol)?;
        let custom_fields = serde_json::to_string(&proxy.custom_fields)?;

        conn.execute(
            r#"
            INSERT INTO proxies (id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'), datetime('now'))
            "#,
            params![
                proxy.id,
                proxy.name,
                protocol.trim_matches('"'),
                proxy.server,
                proxy.port,
                proxy.username,
                proxy.password,
                proxy.uuid,
                proxy.tls as i32,
                proxy.sni,
                proxy.transport,
                custom_fields,
                proxy.active as i32,
            ],
        )?;

        debug!(id = %proxy.id, name = %proxy.name, "Proxy saved");
        Ok(())
    }

    /// Get a proxy by ID
    pub fn get_proxy(&self, id: &str) -> Result<Option<ProxyConfig>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let result = conn
            .query_row(
                r#"
                SELECT id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active
                FROM proxies
                WHERE id = ?1
                "#,
                params![id],
                |row| {
                    let protocol_str: String = row.get(2)?;
                    let custom_fields_str: String = row.get(11)?;
                    let tls_int: i32 = row.get(8)?;
                    let active_int: i32 = row.get(12)?;

                    Ok(ProxyConfigRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: protocol_str,
                        server: row.get(3)?,
                        port: row.get(4)?,
                        username: row.get(5)?,
                        password: row.get(6)?,
                        uuid: row.get(7)?,
                        tls: tls_int != 0,
                        sni: row.get(9)?,
                        transport: row.get(10)?,
                        custom_fields: custom_fields_str,
                        active: active_int != 0,
                    })
                },
            )
            .optional()?;

        match result {
            Some(row) => Ok(Some(row.into_proxy_config()?)),
            None => Ok(None),
        }
    }

    /// Get all proxies
    pub fn get_all_proxies(&self) -> Result<Vec<ProxyConfig>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active
            FROM proxies
            ORDER BY created_at DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let protocol_str: String = row.get(2)?;
            let custom_fields_str: String = row.get(11)?;
            let tls_int: i32 = row.get(8)?;
            let active_int: i32 = row.get(12)?;

            Ok(ProxyConfigRow {
                id: row.get(0)?,
                name: row.get(1)?,
                protocol: protocol_str,
                server: row.get(3)?,
                port: row.get(4)?,
                username: row.get(5)?,
                password: row.get(6)?,
                uuid: row.get(7)?,
                tls: tls_int != 0,
                sni: row.get(9)?,
                transport: row.get(10)?,
                custom_fields: custom_fields_str,
                active: active_int != 0,
            })
        })?;

        let mut proxies = Vec::new();
        for row in rows {
            let row = row?;
            proxies.push(row.into_proxy_config()?);
        }

        Ok(proxies)
    }

    /// Delete a proxy by ID
    pub fn delete_proxy(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let deleted = conn.execute("DELETE FROM proxies WHERE id = ?1", params![id])?;

        if deleted == 0 {
            return Err(IsolateError::Storage(format!("Proxy '{}' not found", id)));
        }

        debug!(id = %id, "Proxy deleted");
        Ok(())
    }

    /// Update an existing proxy
    pub fn update_proxy(&self, proxy: &ProxyConfig) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let protocol = serde_json::to_string(&proxy.protocol)?;
        let custom_fields = serde_json::to_string(&proxy.custom_fields)?;

        let updated = conn.execute(
            r#"
            UPDATE proxies
            SET name = ?2, protocol = ?3, server = ?4, port = ?5, username = ?6, password = ?7,
                uuid = ?8, tls = ?9, sni = ?10, transport = ?11, custom_fields = ?12, active = ?13,
                updated_at = datetime('now')
            WHERE id = ?1
            "#,
            params![
                proxy.id,
                proxy.name,
                protocol.trim_matches('"'),
                proxy.server,
                proxy.port,
                proxy.username,
                proxy.password,
                proxy.uuid,
                proxy.tls as i32,
                proxy.sni,
                proxy.transport,
                custom_fields,
                proxy.active as i32,
            ],
        )?;

        if updated == 0 {
            return Err(IsolateError::Storage(format!("Proxy '{}' not found", proxy.id)));
        }

        debug!(id = %proxy.id, name = %proxy.name, "Proxy updated");
        Ok(())
    }

    /// Set proxy active state
    pub fn set_proxy_active(&self, id: &str, active: bool) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        // If activating, deactivate all other proxies first
        if active {
            conn.execute("UPDATE proxies SET active = 0", [])?;
        }

        let updated = conn.execute(
            "UPDATE proxies SET active = ?2, updated_at = datetime('now') WHERE id = ?1",
            params![id, active as i32],
        )?;

        if updated == 0 {
            return Err(IsolateError::Storage(format!("Proxy '{}' not found", id)));
        }

        debug!(id = %id, active, "Proxy active state updated");
        Ok(())
    }

    /// Get the currently active proxy
    pub fn get_active_proxy(&self) -> Result<Option<ProxyConfig>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let result = conn
            .query_row(
                r#"
                SELECT id, name, protocol, server, port, username, password, uuid, tls, sni, transport, custom_fields, active
                FROM proxies
                WHERE active = 1
                LIMIT 1
                "#,
                [],
                |row| {
                    let protocol_str: String = row.get(2)?;
                    let custom_fields_str: String = row.get(11)?;
                    let tls_int: i32 = row.get(8)?;
                    let active_int: i32 = row.get(12)?;

                    Ok(ProxyConfigRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: protocol_str,
                        server: row.get(3)?,
                        port: row.get(4)?,
                        username: row.get(5)?,
                        password: row.get(6)?,
                        uuid: row.get(7)?,
                        tls: tls_int != 0,
                        sni: row.get(9)?,
                        transport: row.get(10)?,
                        custom_fields: custom_fields_str,
                        active: active_int != 0,
                    })
                },
            )
            .optional()?;

        match result {
            Some(row) => Ok(Some(row.into_proxy_config()?)),
            None => Ok(None),
        }
    }

    // ========================================================================
    // Domain Routes
    // ========================================================================

    /// Save a domain route
    pub fn save_domain_route(&self, route: &DomainRoute) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO domain_routes (domain, proxy_id) VALUES (?1, ?2)",
            params![route.domain, route.proxy_id],
        )?;

        debug!(domain = %route.domain, proxy_id = %route.proxy_id, "Domain route saved");
        Ok(())
    }

    /// Delete a domain route
    pub fn delete_domain_route(&self, domain: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            "DELETE FROM domain_routes WHERE domain = ?1",
            params![domain],
        )?;

        debug!(domain = %domain, "Domain route deleted");
        Ok(())
    }

    /// Get all domain routes
    pub fn get_domain_routes(&self) -> Result<Vec<DomainRoute>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let mut stmt = conn.prepare("SELECT domain, proxy_id FROM domain_routes")?;
        let rows = stmt.query_map([], |row| {
            Ok(DomainRoute {
                domain: row.get(0)?,
                proxy_id: row.get(1)?,
            })
        })?;

        let mut routes = Vec::new();
        for row in rows {
            routes.push(row?);
        }

        Ok(routes)
    }

    // ========================================================================
    // App Routes
    // ========================================================================

    /// Save an app route
    pub fn save_app_route(&self, route: &AppRoute) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO app_routes (app_path, app_name, proxy_id) VALUES (?1, ?2, ?3)",
            params![route.app_path, route.app_name, route.proxy_id],
        )?;

        debug!(app_path = %route.app_path, app_name = %route.app_name, proxy_id = %route.proxy_id, "App route saved");
        Ok(())
    }

    /// Delete an app route
    pub fn delete_app_route(&self, app_path: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            "DELETE FROM app_routes WHERE app_path = ?1",
            params![app_path],
        )?;

        debug!(app_path = %app_path, "App route deleted");
        Ok(())
    }

    /// Get all app routes
    pub fn get_app_routes(&self) -> Result<Vec<AppRoute>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let mut stmt = conn.prepare("SELECT app_path, app_name, proxy_id FROM app_routes")?;
        let rows = stmt.query_map([], |row| {
            Ok(AppRoute {
                app_path: row.get(0)?,
                app_name: row.get(1)?,
                proxy_id: row.get(2)?,
            })
        })?;

        let mut routes = Vec::new();
        for row in rows {
            routes.push(row?);
        }

        Ok(routes)
    }

    // ========================================================================
    // Test History
    // ========================================================================

    /// Сохраняет результат теста стратегии
    pub fn save_test_result(
        &self,
        env_key: &str,
        strategy_id: &str,
        success: bool,
        score: f64,
        latency_ms: f64,
    ) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute(
            r#"
            INSERT INTO test_history (env_key, strategy_id, success, score, latency_ms, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, unixepoch())
            "#,
            params![env_key, strategy_id, success, score, latency_ms],
        )?;

        Ok(())
    }

    /// Получает историю тестов для стратегии
    pub fn get_test_history(
        &self,
        strategy_id: &str,
        limit: u32,
    ) -> Result<Vec<TestHistoryEntry>> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        let mut stmt = conn.prepare(
            r#"
            SELECT env_key, success, score, latency_ms, timestamp
            FROM test_history
            WHERE strategy_id = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![strategy_id, limit], |row| {
            Ok(TestHistoryEntry {
                env_key: row.get(0)?,
                success: row.get(1)?,
                score: row.get(2)?,
                latency_ms: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }

        Ok(history)
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Инициализирует схему базы данных
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            IsolateError::Storage(format!("Failed to lock connection: {}", e))
        })?;

        conn.execute_batch(
            r#"
            -- Настройки приложения
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Кэш стратегий по окружению
            CREATE TABLE IF NOT EXISTS strategy_cache (
                env_key TEXT PRIMARY KEY,
                strategy_id TEXT NOT NULL,
                score REAL NOT NULL,
                timestamp INTEGER NOT NULL
            );

            -- История тестов
            CREATE TABLE IF NOT EXISTS test_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                env_key TEXT NOT NULL,
                strategy_id TEXT NOT NULL,
                success INTEGER NOT NULL,
                score REAL NOT NULL,
                latency_ms REAL NOT NULL,
                timestamp INTEGER NOT NULL
            );

            -- Domain routing rules
            CREATE TABLE IF NOT EXISTS domain_routes (
                domain TEXT PRIMARY KEY,
                proxy_id TEXT NOT NULL
            );

            -- App routing rules
            CREATE TABLE IF NOT EXISTS app_routes (
                app_path TEXT PRIMARY KEY,
                app_name TEXT NOT NULL,
                proxy_id TEXT NOT NULL
            );

            -- Proxies table
            CREATE TABLE IF NOT EXISTS proxies (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                protocol TEXT NOT NULL,
                server TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT,
                password TEXT,
                uuid TEXT,
                tls INTEGER NOT NULL DEFAULT 0,
                sni TEXT,
                transport TEXT,
                custom_fields TEXT NOT NULL DEFAULT '{}',
                active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Индексы
            CREATE INDEX IF NOT EXISTS idx_test_history_strategy
                ON test_history(strategy_id, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_strategy_cache_timestamp
                ON strategy_cache(timestamp);
            CREATE INDEX IF NOT EXISTS idx_domain_routes_proxy
                ON domain_routes(proxy_id);
            CREATE INDEX IF NOT EXISTS idx_app_routes_proxy
                ON app_routes(proxy_id);
            CREATE INDEX IF NOT EXISTS idx_proxies_active
                ON proxies(active);
            "#,
        )?;

        debug!("Database schema initialized");
        Ok(())
    }
}

// ============================================================================
// Types
// ============================================================================

/// Кэшированная стратегия
#[derive(Debug, Clone)]
pub struct CachedStrategy {
    pub strategy_id: String,
    pub score: f64,
    pub timestamp: i64,
}

/// Запись истории тестов
#[derive(Debug, Clone)]
pub struct TestHistoryEntry {
    pub env_key: String,
    pub success: bool,
    pub score: f64,
    pub latency_ms: f64,
    pub timestamp: i64,
}

/// Helper struct for reading proxy from database
struct ProxyConfigRow {
    id: String,
    name: String,
    protocol: String,
    server: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    uuid: Option<String>,
    tls: bool,
    sni: Option<String>,
    transport: Option<String>,
    custom_fields: String,
    active: bool,
}

impl ProxyConfigRow {
    fn into_proxy_config(self) -> Result<ProxyConfig> {
        let protocol: ProxyProtocol = serde_json::from_str(&format!("\"{}\"", self.protocol))
            .unwrap_or(ProxyProtocol::Socks5);
        
        let custom_fields: std::collections::HashMap<String, String> = 
            serde_json::from_str(&self.custom_fields).unwrap_or_default();

        Ok(ProxyConfig {
            id: self.id,
            name: self.name,
            protocol,
            server: self.server,
            port: self.port,
            username: self.username,
            password: self.password,
            uuid: self.uuid,
            tls: self.tls,
            sni: self.sni,
            transport: self.transport,
            custom_fields,
            active: self.active,
        })
    }
}

// ============================================================================
// Convenience Settings Keys
// ============================================================================

/// Ключи настроек
pub mod settings_keys {
    /// Выбранные сервисы для тестирования
    pub const SELECTED_SERVICES: &str = "selected_services";
    /// Автозапуск при старте системы
    pub const AUTOSTART: &str = "autostart";
    /// Автоматическое применение стратегии
    pub const AUTO_APPLY: &str = "auto_apply";
    /// Последняя использованная стратегия
    pub const LAST_STRATEGY: &str = "last_strategy";
    /// Тема оформления
    pub const THEME: &str = "theme";
    /// Язык интерфейса
    pub const LANGUAGE: &str = "language";
    /// Показывать уведомления
    pub const NOTIFICATIONS: &str = "notifications";
    /// Минимизировать в трей
    pub const MINIMIZE_TO_TRAY: &str = "minimize_to_tray";
    /// Блокировать QUIC
    pub const BLOCK_QUIC: &str = "block_quic";
    /// Режим по умолчанию
    pub const DEFAULT_MODE: &str = "default_mode";
    /// Настройки приложения (объект)
    pub const APP_SETTINGS: &str = "app_settings";
    /// Префикс для состояния сервисов
    pub const SERVICE_ENABLED_PREFIX: &str = "service_enabled:";
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_storage() -> Storage {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        Storage::open(&path).unwrap()
    }

    #[test]
    fn test_settings_crud() {
        let storage = create_test_storage();

        // Set
        storage.set_setting("test_key", &"test_value").unwrap();

        // Get
        let value: Option<String> = storage.get_setting("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Delete
        storage.delete_setting("test_key").unwrap();
        let value: Option<String> = storage.get_setting("test_key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_strategy_cache() {
        let storage = create_test_storage();

        // Cache
        storage
            .cache_strategy("env1", "strategy1", 0.95)
            .unwrap();

        // Get
        let cached = storage.get_cached_strategy("env1").unwrap();
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.strategy_id, "strategy1");
        assert_eq!(cached.score, 0.95);

        // Invalidate
        storage.invalidate_cache("env1").unwrap();
        let cached = storage.get_cached_strategy("env1").unwrap();
        assert!(cached.is_none());
    }
}

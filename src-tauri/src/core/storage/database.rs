//! Core database operations

use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::Settings;
use super::types::CachedStrategy;
use super::migrations;

/// Имя файла базы данных
const DB_FILENAME: &str = "data.db";
/// Имя директории приложения
const APP_DIR_NAME: &str = "Isolate";
/// Время жизни кэша стратегий (24 часа)
const CACHE_TTL_SECONDS: i64 = 24 * 60 * 60;

/// SQLite хранилище
pub struct Storage {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

impl Storage {
    /// Создаёт новое хранилище
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        Self::open(&db_path).await
    }

    /// Открывает хранилище по указанному пути
    pub async fn open(path: &PathBuf) -> Result<Self> {
        // Создаём директорию если не существует
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let conn = Connection::open(path)?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema().await?;

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

    /// Инициализирует схему базы данных
    async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            migrations::init_schema(&conn)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!("Database schema initialized");
        Ok(())
    }

    // ========================================================================
    // Settings CRUD
    // ========================================================================

    /// Получает настройку по ключу
    pub async fn get_setting<T: DeserializeOwned + Send + 'static>(&self, key: &str) -> Result<Option<T>> {
        let conn = self.conn.clone();
        let key = key.to_string();

        let result: Option<String> = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        match result {
            Some(json) => {
                let value: T = serde_json::from_str(&json)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Устанавливает настройку
    pub async fn set_setting<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;
        let conn = self.conn.clone();
        let key_for_log = key.to_string();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
                params![key, json],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(key = key_for_log, "Setting updated");
        Ok(())
    }

    /// Удаляет настройку
    pub async fn delete_setting(&self, key: &str) -> Result<()> {
        let conn = self.conn.clone();
        let key_for_log = key.to_string();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(key = key_for_log, "Setting deleted");
        Ok(())
    }

    /// Получает все настройки
    pub async fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.clone();

        let settings = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

            let mut settings = Vec::new();
            for row in rows {
                settings.push(row?);
            }
            Ok::<_, rusqlite::Error>(settings)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(settings)
    }

    // ========================================================================
    // App Settings (typed)
    // ========================================================================

    /// Получает настройки приложения
    pub async fn get_settings(&self) -> Result<Settings> {
        let settings: Option<Settings> = self.get_setting(super::types::settings_keys::APP_SETTINGS).await?;
        Ok(settings.unwrap_or_default())
    }

    /// Сохраняет настройки приложения
    pub async fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.set_setting(super::types::settings_keys::APP_SETTINGS, settings).await
    }

    /// Получает состояние включённости сервиса
    pub async fn get_service_enabled(&self, service_id: &str) -> Result<bool> {
        let key = format!("{}{}", super::types::settings_keys::SERVICE_ENABLED_PREFIX, service_id);
        let enabled: Option<bool> = self.get_setting(&key).await?;
        // По умолчанию сервисы включены
        Ok(enabled.unwrap_or(true))
    }

    /// Устанавливает состояние включённости сервиса
    pub async fn set_service_enabled(&self, service_id: &str, enabled: bool) -> Result<()> {
        let key = format!("{}{}", super::types::settings_keys::SERVICE_ENABLED_PREFIX, service_id);
        self.set_setting(&key, &enabled).await
    }

    // ========================================================================
    // Strategy Cache
    // ========================================================================

    /// Получает кэшированную стратегию для окружения
    pub async fn get_cached_strategy(&self, env_key: &str) -> Result<Option<CachedStrategy>> {
        let conn = self.conn.clone();
        let env_key_for_log = env_key.to_string();
        let env_key = env_key.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.query_row(
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
            .optional()
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if result.is_some() {
            debug!(env_key = env_key_for_log, "Cache hit for strategy");
        }

        Ok(result)
    }

    /// Сохраняет стратегию в кэш
    pub async fn cache_strategy(&self, env_key: &str, strategy_id: &str, score: f64) -> Result<()> {
        let conn = self.conn.clone();
        let env_key_for_log = env_key.to_string();
        let strategy_id_for_log = strategy_id.to_string();
        let env_key = env_key.to_string();
        let strategy_id = strategy_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT OR REPLACE INTO strategy_cache (env_key, strategy_id, score, timestamp)
                VALUES (?1, ?2, ?3, unixepoch())
                "#,
                params![env_key, strategy_id, score],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(env_key = env_key_for_log, strategy_id = strategy_id_for_log, score, "Strategy cached");
        Ok(())
    }

    /// Инвалидирует кэш для окружения
    pub async fn invalidate_cache(&self, env_key: &str) -> Result<()> {
        let conn = self.conn.clone();
        let env_key_for_log = env_key.to_string();
        let env_key = env_key.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM strategy_cache WHERE env_key = ?1",
                params![env_key],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(env_key = env_key_for_log, "Cache invalidated");
        Ok(())
    }

    /// Очищает весь кэш стратегий
    pub async fn clear_cache(&self) -> Result<()> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM strategy_cache", [])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!("Strategy cache cleared");
        Ok(())
    }

    /// Очищает устаревшие записи кэша
    pub async fn cleanup_expired_cache(&self) -> Result<u64> {
        let conn = self.conn.clone();

        let deleted = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM strategy_cache WHERE timestamp < unixepoch() - ?1",
                params![CACHE_TTL_SECONDS],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if deleted > 0 {
            debug!(deleted, "Expired cache entries cleaned up");
        }

        Ok(deleted as u64)
    }
}

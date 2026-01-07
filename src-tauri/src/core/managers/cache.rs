//! Strategy Cache Manager
//!
//! Manages caching of optimal strategies per environment key.
//! Provides TTL-based expiration and persistence to SQLite.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::core::errors::{IsolateError, Result};
use crate::core::storage::Storage;

/// Default cache TTL (24 hours)
const DEFAULT_TTL_SECONDS: i64 = 24 * 60 * 60;

/// Cached strategy entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedStrategy {
    /// Strategy ID
    pub strategy_id: String,
    /// Score when cached
    pub score: f64,
    /// When the strategy was cached
    pub cached_at: DateTime<Utc>,
}

impl CachedStrategy {
    /// Creates a new cached strategy entry
    pub fn new(strategy_id: String, score: f64) -> Self {
        Self {
            strategy_id,
            score,
            cached_at: Utc::now(),
        }
    }

    /// Checks if the cache entry has expired
    pub fn is_expired(&self, ttl: Duration) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        age.num_seconds() > ttl.as_secs() as i64
    }

    /// Returns the age of the cache entry
    pub fn age(&self) -> Duration {
        let now = Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        Duration::from_secs(age.num_seconds().max(0) as u64)
    }
}

/// Manager for strategy cache (optimal strategy per environment)
pub struct StrategyCacheManager {
    storage: Arc<Storage>,
    /// env_key -> cached strategy
    cache: RwLock<HashMap<String, CachedStrategy>>,
    /// Time-to-live for cache entries
    ttl: Duration,
}

impl StrategyCacheManager {
    /// Creates a new cache manager with default TTL (24 hours)
    pub async fn new(storage: Arc<Storage>) -> Result<Self> {
        Self::with_ttl(storage, Duration::from_secs(DEFAULT_TTL_SECONDS as u64)).await
    }

    /// Creates a new cache manager with custom TTL
    pub async fn with_ttl(storage: Arc<Storage>, ttl: Duration) -> Result<Self> {
        let manager = Self {
            storage,
            cache: RwLock::new(HashMap::new()),
            ttl,
        };
        manager.load().await?;
        Ok(manager)
    }

    /// Loads cache from SQLite storage
    pub async fn load(&self) -> Result<()> {
        let conn = self.storage.conn.clone();
        let ttl_seconds = self.ttl.as_secs() as i64;

        let rows = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            
            let mut stmt = conn.prepare(
                r#"
                SELECT env_key, strategy_id, score, timestamp
                FROM strategy_cache
                WHERE timestamp > unixepoch() - ?1
                "#,
            )?;

            let rows = stmt.query_map(rusqlite::params![ttl_seconds], |row| {
                let env_key: String = row.get(0)?;
                let strategy_id: String = row.get(1)?;
                let score: f64 = row.get(2)?;
                let timestamp: i64 = row.get(3)?;
                Ok((env_key, strategy_id, score, timestamp))
            })?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok::<_, rusqlite::Error>(result)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        let mut cache = self.cache.write().await;
        cache.clear();

        for (env_key, strategy_id, score, timestamp) in rows {
            let cached_at = DateTime::from_timestamp(timestamp, 0)
                .unwrap_or_else(|| Utc::now());

            cache.insert(
                env_key,
                CachedStrategy {
                    strategy_id,
                    score,
                    cached_at,
                },
            );
        }

        info!(entries = cache.len(), "Strategy cache loaded");
        Ok(())
    }

    /// Gets a cached strategy for an environment key
    /// Returns None if not found or expired
    pub async fn get(&self, env_key: &str) -> Option<CachedStrategy> {
        let cache = self.cache.read().await;
        
        cache.get(env_key).and_then(|entry| {
            if entry.is_expired(self.ttl) {
                debug!(env_key, "Cache entry expired");
                None
            } else {
                debug!(env_key, strategy_id = %entry.strategy_id, "Cache hit");
                Some(entry.clone())
            }
        })
    }

    /// Sets a cached strategy for an environment key
    pub async fn set(&self, env_key: &str, strategy_id: &str, score: f64) -> Result<()> {
        let entry = CachedStrategy::new(strategy_id.to_string(), score);
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(env_key.to_string(), entry.clone());
        }

        // Persist to storage
        let conn = self.storage.conn.clone();
        let env_key_owned = env_key.to_string();
        let strategy_id_owned = strategy_id.to_string();
        let env_key_log = env_key.to_string();
        let strategy_id_log = strategy_id.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT OR REPLACE INTO strategy_cache (env_key, strategy_id, score, timestamp)
                VALUES (?1, ?2, ?3, unixepoch())
                "#,
                rusqlite::params![env_key_owned, strategy_id_owned, score],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(env_key = %env_key_log, strategy_id = %strategy_id_log, score, "Strategy cached");
        Ok(())
    }

    /// Invalidates cache for a specific environment key
    pub async fn invalidate(&self, env_key: &str) -> Result<()> {
        {
            let mut cache = self.cache.write().await;
            cache.remove(env_key);
        }

        let conn = self.storage.conn.clone();
        let env_key_owned = env_key.to_string();
        let env_key_log = env_key.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM strategy_cache WHERE env_key = ?1",
                rusqlite::params![env_key_owned],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!(env_key = %env_key_log, "Cache invalidated");
        Ok(())
    }

    /// Invalidates all cache entries
    pub async fn invalidate_all(&self) -> Result<()> {
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }

        let conn = self.storage.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM strategy_cache", [])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!("All cache invalidated");
        Ok(())
    }

    /// Removes expired cache entries
    /// Returns the number of entries removed
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let ttl = self.ttl;
        
        // Clean in-memory cache
        let expired_keys: Vec<String> = {
            let cache = self.cache.read().await;
            cache
                .iter()
                .filter(|(_, entry)| entry.is_expired(ttl))
                .map(|(key, _)| key.clone())
                .collect()
        };

        let _expired_count = expired_keys.len();
        
        if !expired_keys.is_empty() {
            let mut cache = self.cache.write().await;
            for key in &expired_keys {
                cache.remove(key);
            }
        }

        // Clean storage
        let conn = self.storage.conn.clone();
        let ttl_seconds = ttl.as_secs() as i64;

        let deleted = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM strategy_cache WHERE timestamp < unixepoch() - ?1",
                rusqlite::params![ttl_seconds],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        if deleted > 0 {
            debug!(deleted, "Expired cache entries cleaned up");
        }

        Ok(deleted as u64)
    }

    /// Gets the current TTL
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Gets the number of cached entries (including potentially expired)
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Checks if cache is empty
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }

    /// Gets all cached environment keys
    pub async fn get_all_keys(&self) -> Vec<String> {
        let cache = self.cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Gets all valid (non-expired) cache entries
    pub async fn get_all_valid(&self) -> HashMap<String, CachedStrategy> {
        let cache = self.cache.read().await;
        let ttl = self.ttl;
        
        cache
            .iter()
            .filter(|(_, entry)| !entry.is_expired(ttl))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Checks if a valid cache entry exists for an environment key
    pub async fn has_valid(&self, env_key: &str) -> bool {
        self.get(env_key).await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_storage() -> Arc<Storage> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        Arc::new(Storage::open(&path).await.unwrap())
    }

    #[tokio::test]
    async fn test_cached_strategy_expiration() {
        let entry = CachedStrategy {
            strategy_id: "test".to_string(),
            score: 0.95,
            cached_at: Utc::now() - chrono::Duration::hours(25),
        };

        // 24 hour TTL
        let ttl = Duration::from_secs(24 * 60 * 60);
        assert!(entry.is_expired(ttl));

        // 48 hour TTL
        let ttl = Duration::from_secs(48 * 60 * 60);
        assert!(!entry.is_expired(ttl));
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();

        let cached = manager.get("env-1").await;
        assert!(cached.is_some());
        
        let cached = cached.unwrap();
        assert_eq!(cached.strategy_id, "strategy-1");
        assert!((cached.score - 0.95).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        let cached = manager.get("nonexistent").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_invalidate() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        assert!(manager.get("env-1").await.is_some());

        manager.invalidate("env-1").await.unwrap();
        assert!(manager.get("env-1").await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_all() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        manager.set("env-2", "strategy-2", 0.90).await.unwrap();

        manager.invalidate_all().await.unwrap();

        assert!(manager.is_empty().await);
    }

    #[tokio::test]
    async fn test_custom_ttl() {
        let storage = create_test_storage().await;
        let ttl = Duration::from_secs(1); // 1 second TTL
        let manager = StrategyCacheManager::with_ttl(storage, ttl).await.unwrap();

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        assert!(manager.get("env-1").await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(manager.get("env-1").await.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let storage = create_test_storage().await;
        let ttl = Duration::from_secs(1);
        let manager = StrategyCacheManager::with_ttl(storage, ttl).await.unwrap();

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        manager.set("env-2", "strategy-2", 0.90).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        let deleted = manager.cleanup_expired().await.unwrap();
        assert_eq!(deleted, 2);
        assert!(manager.is_empty().await);
    }

    #[tokio::test]
    async fn test_get_all_valid() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        manager.set("env-2", "strategy-2", 0.90).await.unwrap();

        let valid = manager.get_all_valid().await;
        assert_eq!(valid.len(), 2);
        assert!(valid.contains_key("env-1"));
        assert!(valid.contains_key("env-2"));
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");

        // Create and populate
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            let manager = StrategyCacheManager::new(storage).await.unwrap();
            manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        }

        // Reload and verify
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            let manager = StrategyCacheManager::new(storage).await.unwrap();
            
            let cached = manager.get("env-1").await;
            assert!(cached.is_some());
            
            let cached = cached.unwrap();
            assert_eq!(cached.strategy_id, "strategy-1");
        }
    }

    #[tokio::test]
    async fn test_overwrite() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        manager.set("env-1", "strategy-1", 0.80).await.unwrap();
        manager.set("env-1", "strategy-2", 0.95).await.unwrap();

        let cached = manager.get("env-1").await.unwrap();
        assert_eq!(cached.strategy_id, "strategy-2");
        assert!((cached.score - 0.95).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_has_valid() {
        let storage = create_test_storage().await;
        let manager = StrategyCacheManager::new(storage).await.unwrap();

        assert!(!manager.has_valid("env-1").await);

        manager.set("env-1", "strategy-1", 0.95).await.unwrap();
        assert!(manager.has_valid("env-1").await);
    }
}

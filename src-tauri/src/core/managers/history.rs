//! Strategy History Manager
//!
//! Manages success/failure statistics for strategies per domain.
//! Used for intelligent strategy selection based on historical performance.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::core::errors::{IsolateError, Result};
use crate::core::storage::Storage;

/// Statistics for a strategy on a specific domain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStats {
    /// Number of successful applications
    pub successes: u32,
    /// Number of failed applications
    pub failures: u32,
    /// Last successful application timestamp
    pub last_success: Option<DateTime<Utc>>,
    /// Last failed application timestamp
    pub last_failure: Option<DateTime<Utc>>,
}

impl StrategyStats {
    /// Creates new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates success rate (0.0 to 1.0)
    /// Returns 0.0 if no attempts were made
    pub fn success_rate(&self) -> f64 {
        let total = self.successes + self.failures;
        if total == 0 {
            return 0.0;
        }
        self.successes as f64 / total as f64
    }

    /// Returns total number of attempts
    pub fn total_attempts(&self) -> u32 {
        self.successes + self.failures
    }

    /// Checks if strategy has any history
    pub fn has_history(&self) -> bool {
        self.total_attempts() > 0
    }
}

/// Manager for strategy history (success/failure tracking per domain)
pub struct StrategyHistoryManager {
    storage: Arc<Storage>,
    /// domain -> strategy_id -> stats
    history: RwLock<HashMap<String, HashMap<String, StrategyStats>>>,
}

impl StrategyHistoryManager {
    /// Creates a new history manager and loads data from storage
    pub async fn new(storage: Arc<Storage>) -> Result<Self> {
        let manager = Self {
            storage,
            history: RwLock::new(HashMap::new()),
        };
        manager.load().await?;
        Ok(manager)
    }

    /// Loads history from SQLite storage
    pub async fn load(&self) -> Result<()> {
        let conn = self.storage.conn.clone();

        let rows = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            
            // Check if table exists first
            let table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='strategy_history_v2'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if !table_exists {
                return Ok::<_, rusqlite::Error>(Vec::new());
            }

            let mut stmt = conn.prepare(
                r#"
                SELECT domain, strategy_id, successes, failures, last_success, last_failure
                FROM strategy_history_v2
                "#,
            )?;

            let rows = stmt.query_map([], |row| {
                let domain: String = row.get(0)?;
                let strategy_id: String = row.get(1)?;
                let successes: u32 = row.get::<_, i32>(2)? as u32;
                let failures: u32 = row.get::<_, i32>(3)? as u32;
                let last_success: Option<String> = row.get(4)?;
                let last_failure: Option<String> = row.get(5)?;

                Ok((domain, strategy_id, successes, failures, last_success, last_failure))
            })?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok(result)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        let mut history = self.history.write().await;
        history.clear();

        for (domain, strategy_id, successes, failures, last_success, last_failure) in rows {
            let stats = StrategyStats {
                successes,
                failures,
                last_success: last_success.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                last_failure: last_failure.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            };

            history
                .entry(domain)
                .or_insert_with(HashMap::new)
                .insert(strategy_id, stats);
        }

        info!(domains = history.len(), "Strategy history loaded");
        Ok(())
    }

    /// Saves all history to SQLite storage
    pub async fn save(&self) -> Result<()> {
        let history = self.history.read().await;
        let conn = self.storage.conn.clone();

        // Collect all entries
        let entries: Vec<_> = history
            .iter()
            .flat_map(|(domain, strategies)| {
                strategies.iter().map(move |(strategy_id, stats)| {
                    (
                        domain.clone(),
                        strategy_id.clone(),
                        stats.successes,
                        stats.failures,
                        stats.last_success.map(|dt| dt.to_rfc3339()),
                        stats.last_failure.map(|dt| dt.to_rfc3339()),
                    )
                })
            })
            .collect();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();

            // Clear existing data
            conn.execute("DELETE FROM strategy_history_v2", [])?;

            // Insert all entries
            let mut stmt = conn.prepare(
                r#"
                INSERT INTO strategy_history_v2 (domain, strategy_id, successes, failures, last_success, last_failure)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )?;

            for (domain, strategy_id, successes, failures, last_success, last_failure) in entries {
                stmt.execute(rusqlite::params![
                    domain,
                    strategy_id,
                    successes as i32,
                    failures as i32,
                    last_success,
                    last_failure
                ])?;
            }

            Ok::<_, rusqlite::Error>(())
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!("Strategy history saved");
        Ok(())
    }

    /// Records a successful strategy application
    pub async fn record_success(&self, domain: &str, strategy_id: &str) -> Result<()> {
        let now = Utc::now();
        
        {
            let mut history = self.history.write().await;
            let stats = history
                .entry(domain.to_string())
                .or_insert_with(HashMap::new)
                .entry(strategy_id.to_string())
                .or_insert_with(StrategyStats::new);

            stats.successes += 1;
            stats.last_success = Some(now);
        }

        // Persist to storage
        self.save_single(domain, strategy_id).await?;

        debug!(domain, strategy_id, "Recorded strategy success");
        Ok(())
    }

    /// Records a failed strategy application
    pub async fn record_failure(&self, domain: &str, strategy_id: &str) -> Result<()> {
        let now = Utc::now();
        
        {
            let mut history = self.history.write().await;
            let stats = history
                .entry(domain.to_string())
                .or_insert_with(HashMap::new)
                .entry(strategy_id.to_string())
                .or_insert_with(StrategyStats::new);

            stats.failures += 1;
            stats.last_failure = Some(now);
        }

        // Persist to storage
        self.save_single(domain, strategy_id).await?;

        debug!(domain, strategy_id, "Recorded strategy failure");
        Ok(())
    }

    /// Saves a single entry to storage (upsert)
    async fn save_single(&self, domain: &str, strategy_id: &str) -> Result<()> {
        let stats = {
            let history = self.history.read().await;
            history
                .get(domain)
                .and_then(|s| s.get(strategy_id))
                .cloned()
        };

        let Some(stats) = stats else {
            return Ok(());
        };

        let conn = self.storage.conn.clone();
        let domain = domain.to_string();
        let strategy_id = strategy_id.to_string();
        let last_success = stats.last_success.map(|dt| dt.to_rfc3339());
        let last_failure = stats.last_failure.map(|dt| dt.to_rfc3339());

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT INTO strategy_history_v2 (domain, strategy_id, successes, failures, last_success, last_failure)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(domain, strategy_id) DO UPDATE SET
                    successes = excluded.successes,
                    failures = excluded.failures,
                    last_success = excluded.last_success,
                    last_failure = excluded.last_failure
                "#,
                rusqlite::params![
                    domain,
                    strategy_id,
                    stats.successes as i32,
                    stats.failures as i32,
                    last_success,
                    last_failure
                ],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(())
    }

    /// Gets statistics for a specific domain/strategy pair
    pub async fn get_stats(&self, domain: &str, strategy_id: &str) -> Option<StrategyStats> {
        let history = self.history.read().await;
        history
            .get(domain)
            .and_then(|s| s.get(strategy_id))
            .cloned()
    }

    /// Gets all strategy statistics for a domain
    pub async fn get_domain_history(&self, domain: &str) -> HashMap<String, StrategyStats> {
        let history = self.history.read().await;
        history.get(domain).cloned().unwrap_or_default()
    }

    /// Gets the best strategy for a domain based on success rate
    /// 
    /// # Arguments
    /// * `domain` - Domain to find best strategy for
    /// * `exclude` - List of strategy IDs to exclude from consideration
    /// 
    /// # Returns
    /// Strategy ID with highest success rate, or None if no history exists
    pub async fn get_best_strategy(&self, domain: &str, exclude: &[String]) -> Option<String> {
        let history = self.history.read().await;
        
        let Some(domain_history) = history.get(domain) else {
            return None;
        };

        domain_history
            .iter()
            .filter(|(id, stats)| {
                !exclude.contains(id) && stats.has_history()
            })
            .max_by(|(_, a), (_, b)| {
                a.success_rate()
                    .partial_cmp(&b.success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| id.clone())
    }

    /// Gets success rate for a specific domain/strategy pair
    pub async fn get_success_rate(&self, domain: &str, strategy_id: &str) -> f64 {
        self.get_stats(domain, strategy_id)
            .await
            .map(|s| s.success_rate())
            .unwrap_or(0.0)
    }

    /// Clears all history
    pub async fn clear(&self) -> Result<()> {
        {
            let mut history = self.history.write().await;
            history.clear();
        }

        let conn = self.storage.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM strategy_history_v2", [])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!("Strategy history cleared");
        Ok(())
    }

    /// Clears history for a specific domain
    pub async fn clear_domain(&self, domain: &str) -> Result<()> {
        {
            let mut history = self.history.write().await;
            history.remove(domain);
        }

        let conn = self.storage.conn.clone();
        let domain_owned = domain.to_string();
        let domain_log = domain.to_string();
        
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM strategy_history_v2 WHERE domain = ?1",
                rusqlite::params![domain_owned],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!(domain = %domain_log, "Domain history cleared");
        Ok(())
    }

    /// Gets all domains with history
    pub async fn get_all_domains(&self) -> Vec<String> {
        let history = self.history.read().await;
        history.keys().cloned().collect()
    }

    /// Gets total number of tracked domain/strategy pairs
    pub async fn get_total_entries(&self) -> usize {
        let history = self.history.read().await;
        history.values().map(|m| m.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_storage() -> Arc<Storage> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let storage = Storage::open(&path).await.unwrap();
        
        // Create the table
        let conn = storage.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS strategy_history_v2 (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    domain TEXT NOT NULL,
                    strategy_id TEXT NOT NULL,
                    successes INTEGER NOT NULL DEFAULT 0,
                    failures INTEGER NOT NULL DEFAULT 0,
                    last_success TEXT,
                    last_failure TEXT,
                    UNIQUE(domain, strategy_id)
                );
                "#,
            ).unwrap();
        })
        .await
        .unwrap();
        
        Arc::new(storage)
    }

    #[tokio::test]
    async fn test_strategy_stats_success_rate() {
        let mut stats = StrategyStats::new();
        assert_eq!(stats.success_rate(), 0.0);

        stats.successes = 3;
        stats.failures = 1;
        assert!((stats.success_rate() - 0.75).abs() < 0.001);

        stats.successes = 10;
        stats.failures = 0;
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[tokio::test]
    async fn test_record_success() {
        let storage = create_test_storage().await;
        let manager = StrategyHistoryManager::new(storage).await.unwrap();

        manager.record_success("youtube.com", "strategy-1").await.unwrap();
        manager.record_success("youtube.com", "strategy-1").await.unwrap();

        let stats = manager.get_stats("youtube.com", "strategy-1").await.unwrap();
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.failures, 0);
        assert!(stats.last_success.is_some());
    }

    #[tokio::test]
    async fn test_record_failure() {
        let storage = create_test_storage().await;
        let manager = StrategyHistoryManager::new(storage).await.unwrap();

        manager.record_failure("discord.com", "strategy-2").await.unwrap();

        let stats = manager.get_stats("discord.com", "strategy-2").await.unwrap();
        assert_eq!(stats.successes, 0);
        assert_eq!(stats.failures, 1);
        assert!(stats.last_failure.is_some());
    }

    #[tokio::test]
    async fn test_get_best_strategy() {
        let storage = create_test_storage().await;
        let manager = StrategyHistoryManager::new(storage).await.unwrap();

        // Strategy 1: 80% success rate
        for _ in 0..8 {
            manager.record_success("youtube.com", "strategy-1").await.unwrap();
        }
        for _ in 0..2 {
            manager.record_failure("youtube.com", "strategy-1").await.unwrap();
        }

        // Strategy 2: 60% success rate
        for _ in 0..6 {
            manager.record_success("youtube.com", "strategy-2").await.unwrap();
        }
        for _ in 0..4 {
            manager.record_failure("youtube.com", "strategy-2").await.unwrap();
        }

        let best = manager.get_best_strategy("youtube.com", &[]).await;
        assert_eq!(best, Some("strategy-1".to_string()));

        // Exclude strategy-1
        let best = manager.get_best_strategy("youtube.com", &["strategy-1".to_string()]).await;
        assert_eq!(best, Some("strategy-2".to_string()));
    }

    #[tokio::test]
    async fn test_get_domain_history() {
        let storage = create_test_storage().await;
        let manager = StrategyHistoryManager::new(storage).await.unwrap();

        manager.record_success("youtube.com", "strategy-1").await.unwrap();
        manager.record_success("youtube.com", "strategy-2").await.unwrap();

        let history = manager.get_domain_history("youtube.com").await;
        assert_eq!(history.len(), 2);
        assert!(history.contains_key("strategy-1"));
        assert!(history.contains_key("strategy-2"));
    }

    #[tokio::test]
    async fn test_clear_domain() {
        let storage = create_test_storage().await;
        let manager = StrategyHistoryManager::new(storage).await.unwrap();

        manager.record_success("youtube.com", "strategy-1").await.unwrap();
        manager.record_success("discord.com", "strategy-1").await.unwrap();

        manager.clear_domain("youtube.com").await.unwrap();

        assert!(manager.get_stats("youtube.com", "strategy-1").await.is_none());
        assert!(manager.get_stats("discord.com", "strategy-1").await.is_some());
    }

    #[tokio::test]
    async fn test_clear_all() {
        let storage = create_test_storage().await;
        let manager = StrategyHistoryManager::new(storage).await.unwrap();

        manager.record_success("youtube.com", "strategy-1").await.unwrap();
        manager.record_success("discord.com", "strategy-2").await.unwrap();

        manager.clear().await.unwrap();

        assert_eq!(manager.get_total_entries().await, 0);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        
        // Create and populate
        {
            let storage = Storage::open(&path).await.unwrap();
            let conn = storage.conn.clone();
            tokio::task::spawn_blocking(move || {
                let conn = conn.blocking_lock();
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS strategy_history_v2 (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        domain TEXT NOT NULL,
                        strategy_id TEXT NOT NULL,
                        successes INTEGER NOT NULL DEFAULT 0,
                        failures INTEGER NOT NULL DEFAULT 0,
                        last_success TEXT,
                        last_failure TEXT,
                        UNIQUE(domain, strategy_id)
                    );
                    "#,
                ).unwrap();
            })
            .await
            .unwrap();
            
            let storage = Arc::new(storage);
            let manager = StrategyHistoryManager::new(storage).await.unwrap();
            manager.record_success("youtube.com", "strategy-1").await.unwrap();
            manager.record_success("youtube.com", "strategy-1").await.unwrap();
            manager.record_failure("youtube.com", "strategy-1").await.unwrap();
        }

        // Reload and verify
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            let manager = StrategyHistoryManager::new(storage).await.unwrap();
            
            let stats = manager.get_stats("youtube.com", "strategy-1").await.unwrap();
            assert_eq!(stats.successes, 2);
            assert_eq!(stats.failures, 1);
        }
    }
}

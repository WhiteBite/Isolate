//! Storage - SQLite хранилище для настроек и кэша стратегий
//!
//! Путь к БД: %APPDATA%/Isolate/data.db
//!
//! # Security
//! Пароли прокси шифруются с помощью Windows DPAPI перед сохранением в БД.
//! Это обеспечивает защиту паролей на уровне пользователя Windows.
//!
//! # Module Structure
//! - `database` - Core Storage struct and basic operations
//! - `migrations` - Database schema initialization
//! - `queries` - Proxy, routing, and learned strategies queries
//! - `routing` - Routing rules operations
//! - `types` - Data types and settings keys
//! - `health_history` - Service health history tracking

mod database;
mod health_history;
mod migrations;
mod queries;
mod routing;
pub mod types;

// Re-export main types
pub use database::Storage;
pub use health_history::{ServiceHealthRecord, ServiceHealthStats};
pub use types::{RoutingRule, StrategyHistoryEntry, StrategyStats};
// Note: These types are part of public API but may not be used internally yet
#[allow(unused_imports)]
pub use types::{CachedStrategy, LearnedStrategy, TestHistoryEntry, settings_keys};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;

    fn create_test_storage() -> Storage {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Storage::open(&path))
            .unwrap()
    }

    #[test]
    fn test_settings_crud() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = create_test_storage();

        rt.block_on(async {
            // Set
            storage.set_setting("test_key", &"test_value").await.unwrap();

            // Get
            let value: Option<String> = storage.get_setting("test_key").await.unwrap();
            assert_eq!(value, Some("test_value".to_string()));

            // Delete
            storage.delete_setting("test_key").await.unwrap();
            let value: Option<String> = storage.get_setting("test_key").await.unwrap();
            assert_eq!(value, None);
        });
    }

    #[test]
    fn test_strategy_cache() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = create_test_storage();

        rt.block_on(async {
            // Cache
            storage
                .cache_strategy("env1", "strategy1", 0.95)
                .await
                .unwrap();

            // Get
            let cached = storage.get_cached_strategy("env1").await.unwrap();
            assert!(cached.is_some());
            let cached = cached.unwrap();
            assert_eq!(cached.strategy_id, "strategy1");
            assert_eq!(cached.score, 0.95);

            // Invalidate
            storage.invalidate_cache("env1").await.unwrap();
            let cached = storage.get_cached_strategy("env1").await.unwrap();
            assert!(cached.is_none());
        });
    }
}

//! LockedStrategiesManager - зафиксированные рабочие стратегии
//!
//! Управляет залоченными стратегиями по протоколам (TLS, HTTP, UDP).
//! Когда стратегия залочена для домена, она используется без повторного тестирования.
//!
//! # Пример использования
//! ```rust,ignore
//! let manager = LockedStrategiesManager::new(storage, blocked_manager).await?;
//!
//! // Зафиксировать стратегию
//! manager.lock("youtube.com", "strategy-1", Protocol::Tls).await?;
//!
//! // Получить залоченную стратегию
//! if let Some(strategy_id) = manager.get_locked("youtube.com", Protocol::Tls).await {
//!     // Использовать strategy_id
//! }
//!
//! // Разблокировать
//! manager.unlock("youtube.com", Protocol::Tls).await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::storage::Storage;

use super::BlockedStrategiesManager;

// ============================================================================
// Types
// ============================================================================

/// Протокол для которого залочена стратегия
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// TLS (HTTPS) трафик
    Tls,
    /// HTTP трафик
    Http,
    /// UDP трафик (для IP адресов)
    Udp,
}

impl Protocol {
    /// Преобразует в строку для SQL
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Tls => "tls",
            Protocol::Http => "http",
            Protocol::Udp => "udp",
        }
    }

    /// Создаёт из строки
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tls" => Some(Protocol::Tls),
            "http" => Some(Protocol::Http),
            "udp" => Some(Protocol::Udp),
            _ => None,
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Запись о залоченной стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedEntry {
    /// Домен или IP
    pub domain: String,
    /// ID стратегии
    pub strategy_id: String,
    /// Протокол
    pub protocol: Protocol,
    /// Время блокировки (ISO 8601)
    pub locked_at: String,
}

// ============================================================================
// LockedStrategiesManager
// ============================================================================

/// Менеджер зафиксированных стратегий
///
/// Thread-safe менеджер для управления залоченными стратегиями по протоколам.
/// При загрузке автоматически очищает конфликты с blocked стратегиями.
pub struct LockedStrategiesManager {
    /// SQLite storage
    storage: Arc<Storage>,
    /// Менеджер заблокированных стратегий (для проверки конфликтов)
    blocked_manager: Arc<BlockedStrategiesManager>,
    /// TLS залоченные: domain -> strategy_id
    tls_locked: RwLock<HashMap<String, String>>,
    /// HTTP залоченные: domain -> strategy_id
    http_locked: RwLock<HashMap<String, String>>,
    /// UDP залоченные: IP -> strategy_id
    udp_locked: RwLock<HashMap<String, String>>,
}

impl LockedStrategiesManager {
    /// Создаёт новый менеджер и загружает данные из Storage
    pub async fn new(
        storage: Arc<Storage>,
        blocked_manager: Arc<BlockedStrategiesManager>,
    ) -> Result<Self> {
        let manager = Self {
            storage,
            blocked_manager,
            tls_locked: RwLock::new(HashMap::new()),
            http_locked: RwLock::new(HashMap::new()),
            udp_locked: RwLock::new(HashMap::new()),
        };

        manager.load().await?;

        Ok(manager)
    }

    /// Загружает залоченные стратегии из SQLite + очищает конфликты с blocked
    pub async fn load(&self) -> Result<()> {
        let conn = self.storage.conn.clone();

        // Загружаем все записи из БД
        let entries: Vec<(String, String, String)> = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt =
                conn.prepare("SELECT domain, strategy_id, protocol FROM locked_strategies")?;

            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok::<_, rusqlite::Error>(result)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        // Очищаем текущие данные
        {
            let mut tls = self.tls_locked.write().await;
            let mut http = self.http_locked.write().await;
            let mut udp = self.udp_locked.write().await;
            tls.clear();
            http.clear();
            udp.clear();
        }

        let mut conflicts = Vec::new();
        let mut loaded_count = 0;

        for (domain, strategy_id, protocol_str) in entries {
            let protocol = match Protocol::from_str(&protocol_str) {
                Some(p) => p,
                None => {
                    warn!(
                        domain = %domain,
                        protocol = %protocol_str,
                        "Unknown protocol in locked_strategies, skipping"
                    );
                    continue;
                }
            };

            // Проверяем конфликт с blocked
            if self
                .blocked_manager
                .is_blocked(&domain, &strategy_id)
                .await
            {
                warn!(
                    domain = %domain,
                    strategy_id = %strategy_id,
                    protocol = %protocol,
                    "Locked strategy conflicts with blocked, removing"
                );
                conflicts.push((domain.clone(), protocol));
                continue;
            }

            // Добавляем в соответствующий map
            match protocol {
                Protocol::Tls => {
                    let mut tls = self.tls_locked.write().await;
                    tls.insert(domain, strategy_id);
                }
                Protocol::Http => {
                    let mut http = self.http_locked.write().await;
                    http.insert(domain, strategy_id);
                }
                Protocol::Udp => {
                    let mut udp = self.udp_locked.write().await;
                    udp.insert(domain, strategy_id);
                }
            }
            loaded_count += 1;
        }

        // Удаляем конфликтующие записи из БД
        if !conflicts.is_empty() {
            self.remove_conflicts(&conflicts).await?;
        }

        let tls_count = self.tls_locked.read().await.len();
        let http_count = self.http_locked.read().await.len();
        let udp_count = self.udp_locked.read().await.len();

        info!(
            loaded = loaded_count,
            conflicts = conflicts.len(),
            tls = tls_count,
            http = http_count,
            udp = udp_count,
            "Loaded locked strategies"
        );

        Ok(())
    }

    /// Удаляет конфликтующие записи из БД
    async fn remove_conflicts(&self, conflicts: &[(String, Protocol)]) -> Result<()> {
        let conn = self.storage.conn.clone();
        let conflicts: Vec<(String, String)> = conflicts
            .iter()
            .map(|(d, p)| (d.clone(), p.as_str().to_string()))
            .collect();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            for (domain, protocol) in conflicts {
                conn.execute(
                    "DELETE FROM locked_strategies WHERE domain = ?1 AND protocol = ?2",
                    params![domain, protocol],
                )?;
            }
            Ok::<_, rusqlite::Error>(())
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        Ok(())
    }

    /// Фиксирует стратегию для домена
    ///
    /// Если стратегия заблокирована для этого домена, возвращает ошибку.
    pub async fn lock(&self, domain: &str, strategy_id: &str, protocol: Protocol) -> Result<()> {
        let domain_lower = domain.to_lowercase();

        // Проверяем конфликт с blocked
        if self
            .blocked_manager
            .is_blocked(&domain_lower, strategy_id)
            .await
        {
            return Err(IsolateError::Validation(format!(
                "Cannot lock strategy '{}' for domain '{}': strategy is blocked",
                strategy_id, domain_lower
            )));
        }

        // Добавляем в memory
        {
            let map = self.get_map_for_protocol(protocol);
            let mut locked = map.write().await;
            locked.insert(domain_lower.clone(), strategy_id.to_string());
        }

        // Сохраняем в БД
        let conn = self.storage.conn.clone();
        let domain_db = domain_lower.clone();
        let strategy_db = strategy_id.to_string();
        let protocol_str = protocol.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                r#"
                INSERT OR REPLACE INTO locked_strategies (domain, strategy_id, protocol, locked_at)
                VALUES (?1, ?2, ?3, datetime('now'))
                "#,
                params![domain_db, strategy_db, protocol_str],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!(
            domain = %domain_lower,
            strategy_id,
            protocol = %protocol,
            "Strategy locked for domain"
        );

        Ok(())
    }

    /// Разблокирует стратегию для домена
    ///
    /// Возвращает ID разблокированной стратегии или None если не была залочена.
    pub async fn unlock(&self, domain: &str, protocol: Protocol) -> Result<Option<String>> {
        let domain_lower = domain.to_lowercase();

        // Удаляем из memory
        let removed = {
            let map = self.get_map_for_protocol(protocol);
            let mut locked = map.write().await;
            locked.remove(&domain_lower)
        };

        if removed.is_none() {
            return Ok(None);
        }

        // Удаляем из БД
        let conn = self.storage.conn.clone();
        let domain_db = domain_lower.clone();
        let protocol_str = protocol.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM locked_strategies WHERE domain = ?1 AND protocol = ?2",
                params![domain_db, protocol_str],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!(
            domain = %domain_lower,
            strategy_id = %removed.as_ref().unwrap(),
            protocol = %protocol,
            "Strategy unlocked for domain"
        );

        Ok(removed)
    }

    /// Получает залоченную стратегию для домена
    pub async fn get_locked(&self, domain: &str, protocol: Protocol) -> Option<String> {
        let domain_lower = domain.to_lowercase();
        let map = self.get_map_for_protocol(protocol);
        let locked = map.read().await;
        locked.get(&domain_lower).cloned()
    }

    /// Получает все залоченные стратегии для протокола
    pub async fn get_all(&self, protocol: Protocol) -> HashMap<String, String> {
        let map = self.get_map_for_protocol(protocol);
        let locked = map.read().await;
        locked.clone()
    }

    /// Получает все залоченные стратегии для всех протоколов
    pub async fn get_all_entries(&self) -> Vec<LockedEntry> {
        let conn = self.storage.conn.clone();

        let entries: Vec<(String, String, String, String)> =
            tokio::task::spawn_blocking(move || {
                let conn = conn.blocking_lock();
                let mut stmt = conn.prepare(
                    "SELECT domain, strategy_id, protocol, locked_at FROM locked_strategies",
                )?;

                let rows =
                    stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;

                let mut result = Vec::new();
                for row in rows {
                    result.push(row?);
                }
                Ok::<_, rusqlite::Error>(result)
            })
            .await
            .unwrap_or_else(|_| Ok(Vec::new()))
            .unwrap_or_default();

        entries
            .into_iter()
            .filter_map(|(domain, strategy_id, protocol_str, locked_at)| {
                Protocol::from_str(&protocol_str).map(|protocol| LockedEntry {
                    domain,
                    strategy_id,
                    protocol,
                    locked_at,
                })
            })
            .collect()
    }

    /// Очищает все залоченные стратегии
    pub async fn clear(&self) -> Result<()> {
        // Очищаем memory
        {
            let mut tls = self.tls_locked.write().await;
            let mut http = self.http_locked.write().await;
            let mut udp = self.udp_locked.write().await;
            tls.clear();
            http.clear();
            udp.clear();
        }

        // Очищаем БД
        let conn = self.storage.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("DELETE FROM locked_strategies", [])
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!("All locked strategies cleared");
        Ok(())
    }

    /// Очищает залоченные стратегии для протокола
    pub async fn clear_protocol(&self, protocol: Protocol) -> Result<u32> {
        let count = {
            let map = self.get_map_for_protocol(protocol);
            let mut locked = map.write().await;
            let count = locked.len();
            locked.clear();
            count as u32
        };

        let conn = self.storage.conn.clone();
        let protocol_str = protocol.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute(
                "DELETE FROM locked_strategies WHERE protocol = ?1",
                params![protocol_str],
            )
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        info!(protocol = %protocol, count, "Cleared locked strategies for protocol");
        Ok(count)
    }

    /// Возвращает количество залоченных стратегий
    pub async fn count(&self) -> usize {
        let tls = self.tls_locked.read().await.len();
        let http = self.http_locked.read().await.len();
        let udp = self.udp_locked.read().await.len();
        tls + http + udp
    }

    /// Возвращает количество залоченных стратегий для протокола
    pub async fn count_for_protocol(&self, protocol: Protocol) -> usize {
        let map = self.get_map_for_protocol(protocol);
        let locked = map.read().await;
        locked.len()
    }

    /// Проверяет, залочена ли стратегия для домена
    pub async fn is_locked(&self, domain: &str, protocol: Protocol) -> bool {
        self.get_locked(domain, protocol).await.is_some()
    }

    /// Возвращает RwLock для указанного протокола
    fn get_map_for_protocol(&self, protocol: Protocol) -> &RwLock<HashMap<String, String>> {
        match protocol {
            Protocol::Tls => &self.tls_locked,
            Protocol::Http => &self.http_locked,
            Protocol::Udp => &self.udp_locked,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_storage() -> Arc<Storage> {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        Arc::new(Storage::open(&path).await.unwrap())
    }

    async fn init_test_tables(storage: &Storage) {
        let conn = storage.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS blocked_strategies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    domain TEXT NOT NULL,
                    strategy_id TEXT NOT NULL,
                    is_user INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(domain, strategy_id)
                );

                CREATE TABLE IF NOT EXISTS locked_strategies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    domain TEXT NOT NULL,
                    strategy_id TEXT NOT NULL,
                    protocol TEXT NOT NULL DEFAULT 'tls',
                    locked_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(domain, protocol)
                );
                "#,
            )
            .unwrap();
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_lock_unlock() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        // Lock
        manager
            .lock("example.com", "strategy-1", Protocol::Tls)
            .await
            .unwrap();

        assert!(manager.is_locked("example.com", Protocol::Tls).await);
        assert_eq!(
            manager.get_locked("example.com", Protocol::Tls).await,
            Some("strategy-1".to_string())
        );

        // Unlock
        let removed = manager.unlock("example.com", Protocol::Tls).await.unwrap();
        assert_eq!(removed, Some("strategy-1".to_string()));
        assert!(!manager.is_locked("example.com", Protocol::Tls).await);
    }

    #[tokio::test]
    async fn test_different_protocols() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        // Lock разные протоколы для одного домена
        manager
            .lock("example.com", "tls-strategy", Protocol::Tls)
            .await
            .unwrap();
        manager
            .lock("example.com", "http-strategy", Protocol::Http)
            .await
            .unwrap();
        manager
            .lock("192.168.1.1", "udp-strategy", Protocol::Udp)
            .await
            .unwrap();

        assert_eq!(
            manager.get_locked("example.com", Protocol::Tls).await,
            Some("tls-strategy".to_string())
        );
        assert_eq!(
            manager.get_locked("example.com", Protocol::Http).await,
            Some("http-strategy".to_string())
        );
        assert_eq!(
            manager.get_locked("192.168.1.1", Protocol::Udp).await,
            Some("udp-strategy".to_string())
        );

        // Разные протоколы независимы
        assert!(manager.get_locked("example.com", Protocol::Udp).await.is_none());
    }

    #[tokio::test]
    async fn test_cannot_lock_blocked_strategy() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());

        // Блокируем стратегию
        blocked.block("example.com", "strategy-1").await.unwrap();

        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        // Попытка залочить заблокированную стратегию должна вернуть ошибку
        let result = manager
            .lock("example.com", "strategy-1", Protocol::Tls)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        manager
            .lock("example1.com", "strategy-1", Protocol::Tls)
            .await
            .unwrap();
        manager
            .lock("example2.com", "strategy-2", Protocol::Tls)
            .await
            .unwrap();

        let all_tls = manager.get_all(Protocol::Tls).await;
        assert_eq!(all_tls.len(), 2);
        assert_eq!(all_tls.get("example1.com"), Some(&"strategy-1".to_string()));
        assert_eq!(all_tls.get("example2.com"), Some(&"strategy-2".to_string()));
    }

    #[tokio::test]
    async fn test_clear() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        manager
            .lock("example.com", "strategy-1", Protocol::Tls)
            .await
            .unwrap();
        manager
            .lock("example.com", "strategy-2", Protocol::Http)
            .await
            .unwrap();

        assert_eq!(manager.count().await, 2);

        manager.clear().await.unwrap();

        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_clear_protocol() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        manager
            .lock("example1.com", "strategy-1", Protocol::Tls)
            .await
            .unwrap();
        manager
            .lock("example2.com", "strategy-2", Protocol::Tls)
            .await
            .unwrap();
        manager
            .lock("example.com", "strategy-3", Protocol::Http)
            .await
            .unwrap();

        let count = manager.clear_protocol(Protocol::Tls).await.unwrap();
        assert_eq!(count, 2);

        assert_eq!(manager.count_for_protocol(Protocol::Tls).await, 0);
        assert_eq!(manager.count_for_protocol(Protocol::Http).await, 1);
    }

    #[tokio::test]
    async fn test_case_insensitive() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        manager
            .lock("EXAMPLE.COM", "strategy-1", Protocol::Tls)
            .await
            .unwrap();

        assert!(manager.is_locked("example.com", Protocol::Tls).await);
        assert!(manager.is_locked("Example.Com", Protocol::Tls).await);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");

        // Создаём и добавляем lock
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            init_test_tables(&storage).await;

            let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
            let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

            manager
                .lock("example.com", "strategy-1", Protocol::Tls)
                .await
                .unwrap();
        }

        // Открываем заново и проверяем
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
            let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

            assert!(manager.is_locked("example.com", Protocol::Tls).await);
            assert_eq!(
                manager.get_locked("example.com", Protocol::Tls).await,
                Some("strategy-1".to_string())
            );
        }
    }

    #[tokio::test]
    async fn test_protocol_from_str() {
        assert_eq!(Protocol::from_str("tls"), Some(Protocol::Tls));
        assert_eq!(Protocol::from_str("TLS"), Some(Protocol::Tls));
        assert_eq!(Protocol::from_str("http"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("HTTP"), Some(Protocol::Http));
        assert_eq!(Protocol::from_str("udp"), Some(Protocol::Udp));
        assert_eq!(Protocol::from_str("UDP"), Some(Protocol::Udp));
        assert_eq!(Protocol::from_str("unknown"), None);
    }

    #[tokio::test]
    async fn test_get_all_entries() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        manager
            .lock("example1.com", "strategy-1", Protocol::Tls)
            .await
            .unwrap();
        manager
            .lock("example2.com", "strategy-2", Protocol::Http)
            .await
            .unwrap();

        let entries = manager.get_all_entries().await;
        assert_eq!(entries.len(), 2);

        let tls_entry = entries.iter().find(|e| e.protocol == Protocol::Tls);
        assert!(tls_entry.is_some());
        assert_eq!(tls_entry.unwrap().domain, "example1.com");
        assert_eq!(tls_entry.unwrap().strategy_id, "strategy-1");
    }

    #[tokio::test]
    async fn test_replace_locked_strategy() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let blocked = Arc::new(BlockedStrategiesManager::new(storage.clone()).await.unwrap());
        let manager = LockedStrategiesManager::new(storage, blocked).await.unwrap();

        // Lock первую стратегию
        manager
            .lock("example.com", "strategy-1", Protocol::Tls)
            .await
            .unwrap();
        assert_eq!(
            manager.get_locked("example.com", Protocol::Tls).await,
            Some("strategy-1".to_string())
        );

        // Lock другую стратегию для того же домена — должна заменить
        manager
            .lock("example.com", "strategy-2", Protocol::Tls)
            .await
            .unwrap();
        assert_eq!(
            manager.get_locked("example.com", Protocol::Tls).await,
            Some("strategy-2".to_string())
        );

        // Должна быть только одна запись
        assert_eq!(manager.count_for_protocol(Protocol::Tls).await, 1);
    }
}

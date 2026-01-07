//! BlockedStrategiesManager - чёрный список стратегий для доменов
//!
//! Управляет блокировками стратегий:
//! - **Default блокировки** — strategy "pass" для заблокированных РКН сайтов (нельзя удалить)
//! - **User блокировки** — добавленные пользователем (можно удалить)
//!
//! # Пример использования
//! ```rust,ignore
//! let manager = BlockedStrategiesManager::new(storage).await?;
//!
//! // Проверка блокировки (учитывает субдомены)
//! if manager.is_blocked("www.youtube.com", "pass").await {
//!     // Стратегия заблокирована для этого домена
//! }
//!
//! // Добавить пользовательскую блокировку
//! manager.block("example.com", "strategy-1").await?;
//!
//! // Удалить (только user блокировки)
//! manager.unblock("example.com", "strategy-1").await?;
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::storage::Storage;

// ============================================================================
// Constants
// ============================================================================

/// Дефолтные блокировки — strategy "pass" (id содержит "pass" или первая стратегия)
/// для заблокированных РКН сайтов. Их нельзя удалить через UI.
const DEFAULT_BLOCKED_PASS_DOMAINS: &[&str] = &[
    // YouTube/Google
    "youtube.com",
    "googlevideo.com",
    "ytimg.com",
    "ggpht.com",
    "youtu.be",
    "google.com",
    "googleapis.com",
    "gstatic.com",
    // Discord
    "discord.com",
    "discordapp.com",
    "discord.gg",
    "discord.media",
    // Twitter/X
    "twitter.com",
    "x.com",
    "twimg.com",
    // Meta
    "instagram.com",
    "facebook.com",
    "fbcdn.net",
    "whatsapp.com",
    // Telegram
    "telegram.org",
    "t.me",
    // Twitch
    "twitch.tv",
    "twitchcdn.net",
    // TikTok
    "tiktok.com",
    "tiktokcdn.com",
    // Spotify
    "spotify.com",
    "spotifycdn.com",
    // Steam
    "steampowered.com",
    "steamcommunity.com",
    // GitHub
    "github.com",
    "githubusercontent.com",
];

/// ID стратегии "pass" (пропуск без обработки)
const PASS_STRATEGY_ID: &str = "pass";

// ============================================================================
// Types
// ============================================================================

/// Запись о блокировке стратегии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedEntry {
    /// Домен
    pub domain: String,
    /// ID стратегии
    pub strategy_id: String,
    /// Это дефолтная блокировка (нельзя удалить)
    pub is_default: bool,
}

// ============================================================================
// BlockedStrategiesManager
// ============================================================================

/// Менеджер чёрного списка стратегий
///
/// Thread-safe менеджер для управления заблокированными стратегиями.
/// Поддерживает дефолтные блокировки (для РКН сайтов) и пользовательские.
pub struct BlockedStrategiesManager {
    /// SQLite storage
    storage: Arc<Storage>,
    /// Заблокированные стратегии: domain -> set of strategy_ids
    blocked: RwLock<HashMap<String, HashSet<String>>>,
    /// Дефолтные блокировки (для быстрой проверки)
    default_domains: HashSet<String>,
}

impl BlockedStrategiesManager {
    /// Создаёт новый менеджер и загружает данные из Storage
    pub async fn new(storage: Arc<Storage>) -> Result<Self> {
        let default_domains: HashSet<String> = DEFAULT_BLOCKED_PASS_DOMAINS
            .iter()
            .map(|s| s.to_string())
            .collect();

        let manager = Self {
            storage,
            blocked: RwLock::new(HashMap::new()),
            default_domains,
        };

        manager.load().await?;

        Ok(manager)
    }

    /// Загружает блокировки из SQLite + добавляет дефолтные
    pub async fn load(&self) -> Result<()> {
        let conn = self.storage.conn.clone();

        // Загружаем user блокировки из БД
        let user_blocks: Vec<(String, String)> = tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT domain, strategy_id FROM blocked_strategies WHERE is_user = 1",
            )?;

            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok::<_, rusqlite::Error>(result)
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        let mut blocked = self.blocked.write().await;
        blocked.clear();

        // Добавляем дефолтные блокировки
        for domain in &self.default_domains {
            blocked
                .entry(domain.clone())
                .or_insert_with(HashSet::new)
                .insert(PASS_STRATEGY_ID.to_string());
        }

        // Добавляем user блокировки
        for (domain, strategy_id) in user_blocks {
            blocked
                .entry(domain)
                .or_insert_with(HashSet::new)
                .insert(strategy_id);
        }

        info!(
            default_count = self.default_domains.len(),
            total_domains = blocked.len(),
            "Loaded blocked strategies"
        );

        Ok(())
    }

    /// Сохраняет только user блокировки в SQLite
    async fn save(&self) -> Result<()> {
        let blocked = self.blocked.read().await;

        // Собираем только user блокировки
        let mut user_blocks: Vec<(String, String)> = Vec::new();
        for (domain, strategies) in blocked.iter() {
            for strategy_id in strategies {
                // Пропускаем дефолтные
                if self.is_default_blocked_internal(domain, strategy_id) {
                    continue;
                }
                user_blocks.push((domain.clone(), strategy_id.clone()));
            }
        }

        drop(blocked);

        let conn = self.storage.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();

            // Очищаем user блокировки
            conn.execute("DELETE FROM blocked_strategies WHERE is_user = 1", [])?;

            // Вставляем новые
            for (domain, strategy_id) in user_blocks {
                conn.execute(
                    "INSERT INTO blocked_strategies (domain, strategy_id, is_user) VALUES (?1, ?2, 1)",
                    params![domain, strategy_id],
                )?;
            }

            Ok::<_, rusqlite::Error>(())
        })
        .await
        .map_err(|e| IsolateError::Storage(e.to_string()))??;

        debug!("Saved blocked strategies to storage");
        Ok(())
    }

    /// Проверяет, заблокирована ли стратегия для домена (учитывает субдомены)
    ///
    /// Например, если заблокирован `youtube.com`, то `www.youtube.com` тоже заблокирован.
    pub async fn is_blocked(&self, domain: &str, strategy_id: &str) -> bool {
        let blocked = self.blocked.read().await;
        self.is_blocked_internal(&blocked, domain, strategy_id)
    }

    /// Внутренняя проверка блокировки (с учётом субдоменов)
    fn is_blocked_internal(
        &self,
        blocked: &HashMap<String, HashSet<String>>,
        domain: &str,
        strategy_id: &str,
    ) -> bool {
        let domain_lower = domain.to_lowercase();

        // Проверяем точное совпадение
        if let Some(strategies) = blocked.get(&domain_lower) {
            if strategies.contains(strategy_id) {
                return true;
            }
        }

        // Проверяем родительские домены (субдомены)
        let parts: Vec<&str> = domain_lower.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if let Some(strategies) = blocked.get(&parent) {
                if strategies.contains(strategy_id) {
                    return true;
                }
            }
        }

        false
    }

    /// Проверяет, является ли это дефолтной блокировкой
    pub fn is_default_blocked(&self, domain: &str, strategy_id: &str) -> bool {
        self.is_default_blocked_internal(domain, strategy_id)
    }

    /// Внутренняя проверка дефолтной блокировки
    fn is_default_blocked_internal(&self, domain: &str, strategy_id: &str) -> bool {
        if strategy_id != PASS_STRATEGY_ID {
            return false;
        }

        let domain_lower = domain.to_lowercase();

        // Проверяем точное совпадение
        if self.default_domains.contains(&domain_lower) {
            return true;
        }

        // Проверяем родительские домены
        let parts: Vec<&str> = domain_lower.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if self.default_domains.contains(&parent) {
                return true;
            }
        }

        false
    }

    /// Добавляет стратегию в чёрный список для домена
    pub async fn block(&self, domain: &str, strategy_id: &str) -> Result<()> {
        let domain_lower = domain.to_lowercase();

        {
            let mut blocked = self.blocked.write().await;
            blocked
                .entry(domain_lower.clone())
                .or_insert_with(HashSet::new)
                .insert(strategy_id.to_string());
        }

        self.save().await?;

        info!(
            domain = %domain_lower,
            strategy_id,
            "Strategy blocked for domain"
        );

        Ok(())
    }

    /// Удаляет стратегию из чёрного списка (только user блокировки)
    ///
    /// Возвращает `true` если блокировка была удалена, `false` если не найдена или дефолтная.
    pub async fn unblock(&self, domain: &str, strategy_id: &str) -> Result<bool> {
        let domain_lower = domain.to_lowercase();

        // Проверяем, не дефолтная ли это блокировка
        if self.is_default_blocked_internal(&domain_lower, strategy_id) {
            warn!(
                domain = %domain_lower,
                strategy_id,
                "Cannot unblock default blocked strategy"
            );
            return Ok(false);
        }

        let removed = {
            let mut blocked = self.blocked.write().await;

            if let Some(strategies) = blocked.get_mut(&domain_lower) {
                let removed = strategies.remove(strategy_id);

                // Удаляем домен если больше нет блокировок
                if strategies.is_empty() {
                    blocked.remove(&domain_lower);
                }

                removed
            } else {
                false
            }
        };

        if removed {
            self.save().await?;
            info!(
                domain = %domain_lower,
                strategy_id,
                "Strategy unblocked for domain"
            );
        }

        Ok(removed)
    }

    /// Возвращает список заблокированных стратегий для домена
    pub async fn get_blocked_for_domain(&self, domain: &str) -> Vec<String> {
        let blocked = self.blocked.read().await;
        let domain_lower = domain.to_lowercase();

        let mut result = HashSet::new();

        // Точное совпадение
        if let Some(strategies) = blocked.get(&domain_lower) {
            result.extend(strategies.iter().cloned());
        }

        // Родительские домены
        let parts: Vec<&str> = domain_lower.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if let Some(strategies) = blocked.get(&parent) {
                result.extend(strategies.iter().cloned());
            }
        }

        result.into_iter().collect()
    }

    /// Возвращает все блокировки
    pub async fn get_all(&self) -> Vec<BlockedEntry> {
        let blocked = self.blocked.read().await;
        let mut result = Vec::new();

        for (domain, strategies) in blocked.iter() {
            for strategy_id in strategies {
                result.push(BlockedEntry {
                    domain: domain.clone(),
                    strategy_id: strategy_id.clone(),
                    is_default: self.is_default_blocked_internal(domain, strategy_id),
                });
            }
        }

        result
    }

    /// Очищает только пользовательские блокировки
    ///
    /// Возвращает количество удалённых блокировок.
    pub async fn clear_user_blocks(&self) -> Result<u32> {
        let mut count = 0u32;

        {
            let mut blocked = self.blocked.write().await;

            // Собираем домены для очистки
            let domains: Vec<String> = blocked.keys().cloned().collect();

            for domain in domains {
                if let Some(strategies) = blocked.get_mut(&domain) {
                    // Удаляем только не-дефолтные
                    let to_remove: Vec<String> = strategies
                        .iter()
                        .filter(|s| !self.is_default_blocked_internal(&domain, s))
                        .cloned()
                        .collect();

                    for strategy_id in to_remove {
                        strategies.remove(&strategy_id);
                        count += 1;
                    }

                    // Удаляем домен если остались только дефолтные или пусто
                    if strategies.is_empty() {
                        blocked.remove(&domain);
                    }
                }
            }

            // Восстанавливаем дефолтные
            for domain in &self.default_domains {
                blocked
                    .entry(domain.clone())
                    .or_insert_with(HashSet::new)
                    .insert(PASS_STRATEGY_ID.to_string());
            }
        }

        self.save().await?;

        info!(count, "Cleared user blocked strategies");
        Ok(count)
    }

    /// Возвращает количество заблокированных доменов
    pub async fn blocked_domains_count(&self) -> usize {
        let blocked = self.blocked.read().await;
        blocked.len()
    }

    /// Возвращает общее количество блокировок
    pub async fn total_blocks_count(&self) -> usize {
        let blocked = self.blocked.read().await;
        blocked.values().map(|s| s.len()).sum()
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
                "#,
            )
            .unwrap();
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_default_blocks_loaded() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Дефолтные домены должны быть заблокированы для "pass"
        assert!(manager.is_blocked("youtube.com", "pass").await);
        assert!(manager.is_blocked("discord.com", "pass").await);
        assert!(manager.is_blocked("twitter.com", "pass").await);

        // Но не для других стратегий
        assert!(!manager.is_blocked("youtube.com", "strategy-1").await);
    }

    #[tokio::test]
    async fn test_subdomain_blocking() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Субдомены должны наследовать блокировку
        assert!(manager.is_blocked("www.youtube.com", "pass").await);
        assert!(manager.is_blocked("m.youtube.com", "pass").await);
        assert!(manager.is_blocked("music.youtube.com", "pass").await);

        // Но не наоборот
        assert!(!manager.is_blocked("youtube.com.fake.com", "pass").await);
    }

    #[tokio::test]
    async fn test_user_block_unblock() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Добавляем user блокировку
        manager.block("example.com", "strategy-1").await.unwrap();
        assert!(manager.is_blocked("example.com", "strategy-1").await);

        // Удаляем
        let removed = manager.unblock("example.com", "strategy-1").await.unwrap();
        assert!(removed);
        assert!(!manager.is_blocked("example.com", "strategy-1").await);
    }

    #[tokio::test]
    async fn test_cannot_unblock_default() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Нельзя удалить дефолтную блокировку
        let removed = manager.unblock("youtube.com", "pass").await.unwrap();
        assert!(!removed);
        assert!(manager.is_blocked("youtube.com", "pass").await);
    }

    #[tokio::test]
    async fn test_is_default_blocked() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        assert!(manager.is_default_blocked("youtube.com", "pass"));
        assert!(manager.is_default_blocked("www.youtube.com", "pass"));
        assert!(!manager.is_default_blocked("youtube.com", "strategy-1"));
        assert!(!manager.is_default_blocked("example.com", "pass"));
    }

    #[tokio::test]
    async fn test_get_blocked_for_domain() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Добавляем user блокировку
        manager.block("youtube.com", "strategy-1").await.unwrap();

        let blocked = manager.get_blocked_for_domain("youtube.com").await;
        assert!(blocked.contains(&"pass".to_string()));
        assert!(blocked.contains(&"strategy-1".to_string()));

        // Субдомен наследует
        let blocked_sub = manager.get_blocked_for_domain("www.youtube.com").await;
        assert!(blocked_sub.contains(&"pass".to_string()));
        assert!(blocked_sub.contains(&"strategy-1".to_string()));
    }

    #[tokio::test]
    async fn test_get_all() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        manager.block("example.com", "strategy-1").await.unwrap();

        let all = manager.get_all().await;

        // Должны быть дефолтные + user
        assert!(all.len() > DEFAULT_BLOCKED_PASS_DOMAINS.len());

        // Проверяем что есть дефолтные
        let youtube_pass = all
            .iter()
            .find(|e| e.domain == "youtube.com" && e.strategy_id == "pass");
        assert!(youtube_pass.is_some());
        assert!(youtube_pass.unwrap().is_default);

        // Проверяем user блокировку
        let example = all
            .iter()
            .find(|e| e.domain == "example.com" && e.strategy_id == "strategy-1");
        assert!(example.is_some());
        assert!(!example.unwrap().is_default);
    }

    #[tokio::test]
    async fn test_clear_user_blocks() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Добавляем user блокировки
        manager.block("example.com", "strategy-1").await.unwrap();
        manager.block("test.com", "strategy-2").await.unwrap();

        // Очищаем
        let count = manager.clear_user_blocks().await.unwrap();
        assert_eq!(count, 2);

        // User блокировки удалены
        assert!(!manager.is_blocked("example.com", "strategy-1").await);
        assert!(!manager.is_blocked("test.com", "strategy-2").await);

        // Дефолтные остались
        assert!(manager.is_blocked("youtube.com", "pass").await);
    }

    #[tokio::test]
    async fn test_case_insensitive() {
        let storage = create_test_storage().await;
        init_test_tables(&storage).await;

        let manager = BlockedStrategiesManager::new(storage).await.unwrap();

        // Регистр не должен влиять
        assert!(manager.is_blocked("YOUTUBE.COM", "pass").await);
        assert!(manager.is_blocked("YouTube.Com", "pass").await);

        manager.block("EXAMPLE.COM", "strategy-1").await.unwrap();
        assert!(manager.is_blocked("example.com", "strategy-1").await);
    }

    #[tokio::test]
    async fn test_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");

        // Создаём и добавляем блокировку
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            init_test_tables(&storage).await;

            let manager = BlockedStrategiesManager::new(storage).await.unwrap();
            manager.block("example.com", "strategy-1").await.unwrap();
        }

        // Открываем заново и проверяем
        {
            let storage = Arc::new(Storage::open(&path).await.unwrap());
            let manager = BlockedStrategiesManager::new(storage).await.unwrap();

            assert!(manager.is_blocked("example.com", "strategy-1").await);
        }
    }
}

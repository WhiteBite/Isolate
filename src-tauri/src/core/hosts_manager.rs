//! Hosts Manager - управление системным файлом hosts
//!
//! Модуль для добавления/удаления записей Discord voice серверов
//! в системный файл hosts для обхода блокировок.
//!
//! ВАЖНО: Требует права администратора для модификации hosts файла.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::paths;

// ============================================================================
// Constants
// ============================================================================

/// Путь к системному hosts файлу
const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

/// Маркер начала блока Isolate
const BEGIN_MARKER: &str = "# BEGIN ISOLATE";

/// Маркер конца блока Isolate
const END_MARKER: &str = "# END ISOLATE";

/// Имя файла бэкапа
const BACKUP_FILENAME: &str = "hosts.backup";

/// Имя файла с Discord hosts записями
const DISCORD_HOSTS_FILENAME: &str = "discord_hosts.txt";

// ============================================================================
// Models
// ============================================================================

/// Статус hosts файла
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostsStatus {
    /// Включены ли Discord hosts записи
    pub enabled: bool,
    /// Количество записей
    pub entries_count: usize,
    /// Есть ли бэкап
    pub backup_exists: bool,
    /// Путь к hosts файлу
    pub hosts_path: String,
    /// Путь к бэкапу
    pub backup_path: String,
}

/// Hosts Manager
#[derive(Debug)]
pub struct HostsManager {
    /// Путь к hosts файлу
    hosts_path: PathBuf,
    /// Путь к бэкапу
    backup_path: PathBuf,
    /// Путь к файлу с Discord hosts
    discord_hosts_path: PathBuf,
}

impl Default for HostsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HostsManager {
    /// Создаёт новый HostsManager
    pub fn new() -> Self {
        let data_dir = paths::get_app_data_dir();
        let configs_dir = paths::get_configs_dir();
        
        Self {
            hosts_path: PathBuf::from(HOSTS_PATH),
            backup_path: data_dir.join(BACKUP_FILENAME),
            discord_hosts_path: configs_dir.join(DISCORD_HOSTS_FILENAME),
        }
    }

    /// Создаёт HostsManager с кастомными путями (для тестов)
    #[cfg(test)]
    pub fn with_paths(hosts_path: PathBuf, backup_path: PathBuf, discord_hosts_path: PathBuf) -> Self {
        Self {
            hosts_path,
            backup_path,
            discord_hosts_path,
        }
    }

    /// Получает статус hosts файла
    pub async fn get_status(&self) -> Result<HostsStatus> {
        debug!("Getting hosts status");

        let hosts_content = self.read_hosts_file().await.unwrap_or_default();
        let (enabled, entries_count) = self.parse_isolate_block(&hosts_content);
        let backup_exists = self.backup_path.exists();

        Ok(HostsStatus {
            enabled,
            entries_count,
            backup_exists,
            hosts_path: self.hosts_path.to_string_lossy().to_string(),
            backup_path: self.backup_path.to_string_lossy().to_string(),
        })
    }

    /// Добавляет Discord hosts записи
    pub async fn add_discord_hosts(&self) -> Result<()> {
        info!("Adding Discord hosts entries");

        // Читаем текущий hosts файл
        let current_content = self.read_hosts_file().await?;

        // Проверяем, не добавлены ли уже записи
        if current_content.contains(BEGIN_MARKER) {
            warn!("Discord hosts entries already exist, removing first");
            self.remove_discord_hosts().await?;
            // Перечитываем после удаления
            let current_content = self.read_hosts_file().await?;
            return self.add_discord_hosts_internal(&current_content).await;
        }

        self.add_discord_hosts_internal(&current_content).await
    }

    /// Внутренняя функция добавления записей
    async fn add_discord_hosts_internal(&self, current_content: &str) -> Result<()> {
        // Создаём бэкап перед изменением
        self.backup_hosts().await?;

        // Читаем Discord hosts записи
        let discord_entries = self.read_discord_hosts().await?;
        
        if discord_entries.is_empty() {
            return Err(IsolateError::Config("Discord hosts file is empty".into()));
        }

        // Формируем новый контент
        let isolate_block = format!(
            "\n{}\n{}\n{}\n",
            BEGIN_MARKER,
            discord_entries.trim(),
            END_MARKER
        );

        let new_content = format!("{}{}", current_content.trim_end(), isolate_block);

        // Записываем обновлённый hosts файл
        self.write_hosts_file(&new_content).await?;

        let entries_count = discord_entries.lines().filter(|l| !l.trim().is_empty()).count();
        info!(entries_count, "Discord hosts entries added successfully");

        Ok(())
    }

    /// Удаляет Discord hosts записи
    pub async fn remove_discord_hosts(&self) -> Result<()> {
        info!("Removing Discord hosts entries");

        let current_content = self.read_hosts_file().await?;

        // Проверяем, есть ли наши записи
        if !current_content.contains(BEGIN_MARKER) {
            debug!("No Discord hosts entries found, nothing to remove");
            return Ok(());
        }

        // Удаляем блок Isolate
        let new_content = self.remove_isolate_block(&current_content);

        // Записываем обновлённый hosts файл
        self.write_hosts_file(&new_content).await?;

        info!("Discord hosts entries removed successfully");

        Ok(())
    }

    /// Создаёт бэкап hosts файла
    pub async fn backup_hosts(&self) -> Result<()> {
        debug!(backup_path = ?self.backup_path, "Creating hosts backup");

        // Создаём директорию для бэкапа если не существует
        if let Some(parent) = self.backup_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                IsolateError::Io(format!("Failed to create backup directory: {}", e))
            })?;
        }

        // Читаем текущий hosts (без блока Isolate)
        let current_content = self.read_hosts_file().await?;
        let clean_content = self.remove_isolate_block(&current_content);

        // Сохраняем бэкап
        fs::write(&self.backup_path, clean_content).await.map_err(|e| {
            IsolateError::Io(format!("Failed to write backup: {}", e))
        })?;

        info!(backup_path = ?self.backup_path, "Hosts backup created");

        Ok(())
    }

    /// Восстанавливает hosts из бэкапа
    pub async fn restore_hosts(&self) -> Result<()> {
        info!(backup_path = ?self.backup_path, "Restoring hosts from backup");

        if !self.backup_path.exists() {
            return Err(IsolateError::Config("Backup file does not exist".into()));
        }

        let backup_content = fs::read_to_string(&self.backup_path).await.map_err(|e| {
            IsolateError::Io(format!("Failed to read backup: {}", e))
        })?;

        self.write_hosts_file(&backup_content).await?;

        info!("Hosts restored from backup successfully");

        Ok(())
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    /// Читает hosts файл
    async fn read_hosts_file(&self) -> Result<String> {
        fs::read_to_string(&self.hosts_path).await.map_err(|e| {
            IsolateError::Io(format!("Failed to read hosts file: {}", e))
        })
    }

    /// Записывает hosts файл
    async fn write_hosts_file(&self, content: &str) -> Result<()> {
        fs::write(&self.hosts_path, content).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                IsolateError::RequiresAdmin
            } else {
                IsolateError::Io(format!("Failed to write hosts file: {}", e))
            }
        })
    }

    /// Читает Discord hosts записи из конфига
    async fn read_discord_hosts(&self) -> Result<String> {
        if !self.discord_hosts_path.exists() {
            return Err(IsolateError::Config(format!(
                "Discord hosts file not found: {:?}",
                self.discord_hosts_path
            )));
        }

        fs::read_to_string(&self.discord_hosts_path).await.map_err(|e| {
            IsolateError::Io(format!("Failed to read Discord hosts file: {}", e))
        })
    }

    /// Парсит блок Isolate из hosts файла
    fn parse_isolate_block(&self, content: &str) -> (bool, usize) {
        let begin_idx = content.find(BEGIN_MARKER);
        let end_idx = content.find(END_MARKER);

        match (begin_idx, end_idx) {
            (Some(begin), Some(end)) if begin < end => {
                let block = &content[begin..end];
                let entries_count = block
                    .lines()
                    .filter(|l| {
                        let trimmed = l.trim();
                        !trimmed.is_empty() 
                            && !trimmed.starts_with('#')
                            && trimmed.contains(' ')
                    })
                    .count();
                (true, entries_count)
            }
            _ => (false, 0),
        }
    }

    /// Удаляет блок Isolate из контента
    fn remove_isolate_block(&self, content: &str) -> String {
        let begin_idx = content.find(BEGIN_MARKER);
        let end_idx = content.find(END_MARKER);

        match (begin_idx, end_idx) {
            (Some(begin), Some(end)) if begin < end => {
                let before = &content[..begin];
                let after = &content[end + END_MARKER.len()..];
                format!("{}{}", before.trim_end(), after.trim_start())
            }
            _ => content.to_string(),
        }
    }
}

// ============================================================================
// Global instance
// ============================================================================

use std::sync::OnceLock;

static HOSTS_MANAGER: OnceLock<HostsManager> = OnceLock::new();

/// Получает глобальный экземпляр HostsManager
pub fn get_hosts_manager() -> &'static HostsManager {
    HOSTS_MANAGER.get_or_init(HostsManager::new)
}

// ============================================================================
// Public API functions
// ============================================================================

/// Добавляет Discord hosts записи
pub async fn add_discord_hosts() -> Result<()> {
    get_hosts_manager().add_discord_hosts().await
}

/// Удаляет Discord hosts записи
pub async fn remove_discord_hosts() -> Result<()> {
    get_hosts_manager().remove_discord_hosts().await
}

/// Получает статус hosts
pub async fn get_hosts_status() -> Result<HostsStatus> {
    get_hosts_manager().get_status().await
}

/// Создаёт бэкап hosts
pub async fn backup_hosts() -> Result<()> {
    get_hosts_manager().backup_hosts().await
}

/// Восстанавливает hosts из бэкапа
pub async fn restore_hosts() -> Result<()> {
    get_hosts_manager().restore_hosts().await
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_env() -> (TempDir, HostsManager) {
        let temp_dir = TempDir::new().unwrap();
        
        let hosts_path = temp_dir.path().join("hosts");
        let backup_path = temp_dir.path().join("hosts.backup");
        let discord_hosts_path = temp_dir.path().join("discord_hosts.txt");

        // Создаём тестовый hosts файл
        fs::write(&hosts_path, "# Test hosts file\n127.0.0.1 localhost\n")
            .await
            .unwrap();

        // Создаём тестовый discord_hosts.txt
        fs::write(
            &discord_hosts_path,
            "104.25.158.178 finland10000.discord.media\n104.25.158.178 finland10001.discord.media\n",
        )
        .await
        .unwrap();

        let manager = HostsManager::with_paths(hosts_path, backup_path, discord_hosts_path);

        (temp_dir, manager)
    }

    #[tokio::test]
    async fn test_get_status_empty() {
        let (_temp_dir, manager) = setup_test_env().await;

        let status = manager.get_status().await.unwrap();

        assert!(!status.enabled);
        assert_eq!(status.entries_count, 0);
        assert!(!status.backup_exists);
    }

    #[tokio::test]
    async fn test_add_discord_hosts() {
        let (_temp_dir, manager) = setup_test_env().await;

        manager.add_discord_hosts().await.unwrap();

        let status = manager.get_status().await.unwrap();
        assert!(status.enabled);
        assert_eq!(status.entries_count, 2);
        assert!(status.backup_exists);

        // Проверяем содержимое hosts
        let content = manager.read_hosts_file().await.unwrap();
        assert!(content.contains(BEGIN_MARKER));
        assert!(content.contains(END_MARKER));
        assert!(content.contains("finland10000.discord.media"));
    }

    #[tokio::test]
    async fn test_remove_discord_hosts() {
        let (_temp_dir, manager) = setup_test_env().await;

        // Сначала добавляем
        manager.add_discord_hosts().await.unwrap();
        
        // Затем удаляем
        manager.remove_discord_hosts().await.unwrap();

        let status = manager.get_status().await.unwrap();
        assert!(!status.enabled);
        assert_eq!(status.entries_count, 0);

        // Проверяем что маркеры удалены
        let content = manager.read_hosts_file().await.unwrap();
        assert!(!content.contains(BEGIN_MARKER));
        assert!(!content.contains(END_MARKER));
    }

    #[tokio::test]
    async fn test_backup_and_restore() {
        let (_temp_dir, manager) = setup_test_env().await;

        // Добавляем записи
        manager.add_discord_hosts().await.unwrap();
        
        // Проверяем что бэкап создан
        let status = manager.get_status().await.unwrap();
        assert!(status.backup_exists);

        // Восстанавливаем из бэкапа
        manager.restore_hosts().await.unwrap();

        // Проверяем что записи удалены
        let status = manager.get_status().await.unwrap();
        assert!(!status.enabled);
        assert_eq!(status.entries_count, 0);
    }

    #[tokio::test]
    async fn test_parse_isolate_block() {
        let manager = HostsManager::new();

        let content = r#"# Test
127.0.0.1 localhost
# BEGIN ISOLATE
104.25.158.178 test1.discord.media
104.25.158.178 test2.discord.media
# END ISOLATE
"#;

        let (enabled, count) = manager.parse_isolate_block(content);
        assert!(enabled);
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_remove_isolate_block() {
        let manager = HostsManager::new();

        let content = r#"# Test
127.0.0.1 localhost
# BEGIN ISOLATE
104.25.158.178 test1.discord.media
# END ISOLATE
# After"#;

        let result = manager.remove_isolate_block(content);
        
        assert!(!result.contains(BEGIN_MARKER));
        assert!(!result.contains(END_MARKER));
        assert!(result.contains("127.0.0.1 localhost"));
        assert!(result.contains("# After"));
    }

    #[tokio::test]
    async fn test_double_add_replaces() {
        let (_temp_dir, manager) = setup_test_env().await;

        // Добавляем дважды
        manager.add_discord_hosts().await.unwrap();
        manager.add_discord_hosts().await.unwrap();

        // Должен быть только один блок
        let content = manager.read_hosts_file().await.unwrap();
        let begin_count = content.matches(BEGIN_MARKER).count();
        let end_count = content.matches(END_MARKER).count();
        
        assert_eq!(begin_count, 1);
        assert_eq!(end_count, 1);
    }
}

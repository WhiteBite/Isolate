//! Управление путями к бинарникам и конфигам
//!
//! Модуль предоставляет функции для получения путей к различным директориям
//! приложения с учётом режима разработки и продакшена.

use std::path::PathBuf;

/// Проверяет, запущено ли приложение в режиме разработки
#[inline]
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}

/// Возвращает путь к директории данных приложения (%APPDATA%/Isolate)
pub fn get_app_data_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join("Isolate")
}

/// Возвращает путь к директории с бинарниками
///
/// - В dev режиме: ./binaries (относительно корня проекта)
/// - В prod режиме: %APPDATA%/Isolate/binaries
pub fn get_binaries_dir() -> PathBuf {
    if is_dev_mode() {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("binaries")
    } else {
        get_app_data_dir().join("binaries")
    }
}

/// Возвращает путь к директории с hostlists
pub fn get_hostlists_dir() -> PathBuf {
    get_binaries_dir().join("hostlists")
}

/// Возвращает путь к директории с конфигами
///
/// - В dev режиме: ./configs (относительно корня проекта)
/// - В prod режиме: %APPDATA%/Isolate/configs
pub fn get_configs_dir() -> PathBuf {
    if is_dev_mode() {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("configs")
    } else {
        get_app_data_dir().join("configs")
    }
}

/// Возвращает путь к директории с логами
pub fn get_logs_dir() -> PathBuf {
    get_app_data_dir().join("logs")
}

/// Создаёт все необходимые директории приложения
pub fn ensure_dirs_exist() -> std::io::Result<()> {
    std::fs::create_dir_all(get_app_data_dir())?;
    std::fs::create_dir_all(get_binaries_dir())?;
    std::fs::create_dir_all(get_hostlists_dir())?;
    std::fs::create_dir_all(get_configs_dir())?;
    std::fs::create_dir_all(get_logs_dir())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dev_mode() {
        // В тестах всегда debug_assertions = true
        assert!(is_dev_mode());
    }

    #[test]
    fn test_paths_not_empty() {
        assert!(!get_app_data_dir().as_os_str().is_empty());
        assert!(!get_binaries_dir().as_os_str().is_empty());
        assert!(!get_hostlists_dir().as_os_str().is_empty());
        assert!(!get_configs_dir().as_os_str().is_empty());
        assert!(!get_logs_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_hostlists_inside_binaries() {
        let binaries = get_binaries_dir();
        let hostlists = get_hostlists_dir();
        assert!(hostlists.starts_with(&binaries));
    }
}

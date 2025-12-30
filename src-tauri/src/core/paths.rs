//! Управление путями к бинарникам и конфигам
//!
//! Модуль предоставляет функции для получения путей к различным директориям
//! приложения с учётом режима разработки, продакшена и portable режима.

use std::path::PathBuf;
use std::sync::OnceLock;

/// Глобальный флаг portable режима
static PORTABLE_MODE: OnceLock<bool> = OnceLock::new();

/// Проверяет, запущено ли приложение в режиме разработки
#[inline]
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}

/// Проверяет, запущено ли приложение в portable режиме
///
/// Portable режим определяется:
/// 1. Флагом командной строки `--portable`
/// 2. Наличием файла `portable.txt` рядом с исполняемым файлом
///
/// В portable режиме все данные хранятся в директории приложения,
/// а не в %APPDATA%
pub fn is_portable_mode() -> bool {
    *PORTABLE_MODE.get_or_init(|| {
        // Проверяем флаг командной строки
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|arg| arg == "--portable") {
            tracing::info!("Portable mode enabled via --portable flag");
            return true;
        }

        // Проверяем наличие portable.txt рядом с exe
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let marker_file = exe_dir.join("portable.txt");
                if marker_file.exists() {
                    tracing::info!("Portable mode enabled via portable.txt marker");
                    return true;
                }
            }
        }

        false
    })
}

/// Возвращает директорию приложения (где находится exe)
fn get_app_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| {
            std::env::current_dir().expect("Failed to get current directory")
        })
}

/// Возвращает путь к директории данных приложения
///
/// - В dev режиме: текущая директория
/// - В portable режиме: директория с exe
/// - В обычном режиме: %APPDATA%/Isolate
pub fn get_app_data_dir() -> PathBuf {
    if is_dev_mode() {
        std::env::current_dir().expect("Failed to get current directory")
    } else if is_portable_mode() {
        get_app_exe_dir().join("data")
    } else {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("Isolate")
    }
}

/// Возвращает путь к директории с бинарниками
///
/// - В dev режиме: ./binaries (относительно корня проекта)
/// - В portable режиме: <exe_dir>/binaries
/// - В prod режиме: %APPDATA%/Isolate/binaries
pub fn get_binaries_dir() -> PathBuf {
    if is_dev_mode() {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("binaries")
    } else if is_portable_mode() {
        get_app_exe_dir().join("binaries")
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
/// - В portable режиме: <exe_dir>/configs
/// - В prod режиме: %APPDATA%/Isolate/configs
pub fn get_configs_dir() -> PathBuf {
    if is_dev_mode() {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("configs")
    } else if is_portable_mode() {
        get_app_exe_dir().join("configs")
    } else {
        get_app_data_dir().join("configs")
    }
}

/// Возвращает путь к директории с логами
pub fn get_logs_dir() -> PathBuf {
    get_app_data_dir().join("logs")
}

/// Возвращает путь к базе данных
pub fn get_database_path() -> PathBuf {
    get_app_data_dir().join("data.db")
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
        assert!(!get_database_path().as_os_str().is_empty());
    }

    #[test]
    fn test_hostlists_inside_binaries() {
        let binaries = get_binaries_dir();
        let hostlists = get_hostlists_dir();
        assert!(hostlists.starts_with(&binaries));
    }

    #[test]
    fn test_database_path_in_data_dir() {
        let data_dir = get_app_data_dir();
        let db_path = get_database_path();
        assert!(db_path.starts_with(&data_dir));
    }
}

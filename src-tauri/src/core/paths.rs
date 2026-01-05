//! Управление путями к бинарникам и конфигам
//!
//! Модуль предоставляет функции для получения путей к различным директориям
//! приложения с учётом режима разработки, продакшена и portable режима.
//!
//! ВАЖНО: Все функции для путей к бинарникам должны использоваться ТОЛЬКО из этого модуля!
//! НЕ дублировать get_singbox_path, get_winws_path и т.д. в других модулях!

use std::path::PathBuf;
use std::sync::OnceLock;

/// Глобальный флаг portable режима
static PORTABLE_MODE: OnceLock<bool> = OnceLock::new();

/// Кэшированный корень проекта для dev режима
static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Проверяет, запущено ли приложение в режиме разработки
#[inline]
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}

/// Возвращает корень проекта в dev режиме
/// 
/// Cargo запускается из src-tauri/, поэтому current_dir() = src-tauri/
/// Нам нужен родительский каталог (корень проекта)
fn get_project_root() -> PathBuf {
    PROJECT_ROOT.get_or_init(|| {
        let current = std::env::current_dir().expect("Failed to get current directory");
        
        // Если мы в src-tauri, поднимаемся на уровень выше
        if current.ends_with("src-tauri") {
            current.parent().unwrap().to_path_buf()
        } else if current.join("src-tauri").exists() {
            // Мы уже в корне проекта
            current
        } else {
            // Fallback - ищем Cargo.toml вверх по дереву
            let mut path = current.clone();
            while path.parent().is_some() {
                if path.join("src-tauri").exists() || path.join("package.json").exists() {
                    return path;
                }
                path = path.parent().unwrap().to_path_buf();
            }
            current
        }
    }).clone()
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
/// - В dev режиме: корень проекта
/// - В portable режиме: директория с exe
/// - В обычном режиме: %APPDATA%/Isolate
pub fn get_app_data_dir() -> PathBuf {
    if is_dev_mode() {
        get_project_root()
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
/// - В dev режиме: <project_root>/bin
/// - В portable режиме: <exe_dir>/bin
/// - В prod режиме: %APPDATA%/Isolate/bin
pub fn get_binaries_dir() -> PathBuf {
    if is_dev_mode() {
        get_project_root().join("bin")
    } else if is_portable_mode() {
        get_app_exe_dir().join("bin")
    } else {
        get_app_data_dir().join("bin")
    }
}

/// Возвращает путь к директории с hostlists
pub fn get_hostlists_dir() -> PathBuf {
    get_binaries_dir().join("hostlists")
}

/// Возвращает путь к директории с конфигами
///
/// - В dev режиме: <project_root>/configs
/// - В portable режиме: <exe_dir>/configs
/// - В prod режиме: %APPDATA%/Isolate/configs
pub fn get_configs_dir() -> PathBuf {
    if is_dev_mode() {
        get_project_root().join("configs")
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

/// Возвращает путь к директории с плагинами
///
/// - В dev режиме: <project_root>/plugins
/// - В portable режиме: <exe_dir>/plugins
/// - В prod режиме: %APPDATA%/Isolate/plugins
pub fn get_plugins_dir() -> PathBuf {
    if is_dev_mode() {
        get_project_root().join("plugins")
    } else if is_portable_mode() {
        get_app_exe_dir().join("plugins")
    } else {
        get_app_data_dir().join("plugins")
    }
}

/// Возвращает путь к базе данных
pub fn get_database_path() -> PathBuf {
    get_app_data_dir().join("data.db")
}

// ============================================================================
// ПУТИ К БИНАРНИКАМ - ЕДИНСТВЕННОЕ МЕСТО ОПРЕДЕЛЕНИЯ!
// НЕ дублировать эти функции в других модулях!
// ============================================================================

/// Возвращает путь к sing-box бинарнику
pub fn get_singbox_path() -> PathBuf {
    let binaries_dir = get_binaries_dir();
    
    #[cfg(windows)]
    {
        binaries_dir.join("sing-box.exe")
    }
    
    #[cfg(not(windows))]
    {
        binaries_dir.join("sing-box")
    }
}

/// Возвращает путь к winws бинарнику
pub fn get_winws_path() -> PathBuf {
    let binaries_dir = get_binaries_dir();
    
    #[cfg(windows)]
    {
        binaries_dir.join("winws.exe")
    }
    
    #[cfg(not(windows))]
    {
        binaries_dir.join("winws")
    }
}

/// Создаёт все необходимые директории приложения
pub fn ensure_dirs_exist() -> std::io::Result<()> {
    std::fs::create_dir_all(get_app_data_dir())?;
    std::fs::create_dir_all(get_binaries_dir())?;
    std::fs::create_dir_all(get_hostlists_dir())?;
    std::fs::create_dir_all(get_configs_dir())?;
    std::fs::create_dir_all(get_logs_dir())?;
    std::fs::create_dir_all(get_plugins_dir())?;
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
        assert!(!get_plugins_dir().as_os_str().is_empty());
        assert!(!get_database_path().as_os_str().is_empty());
        assert!(!get_singbox_path().as_os_str().is_empty());
        assert!(!get_winws_path().as_os_str().is_empty());
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

    #[test]
    fn test_logs_dir_in_data_dir() {
        let data_dir = get_app_data_dir();
        let logs_dir = get_logs_dir();
        assert!(logs_dir.starts_with(&data_dir));
    }

    #[test]
    fn test_plugins_dir_in_dev_mode() {
        // В dev режиме plugins должен быть в корне проекта
        let plugins_dir = get_plugins_dir();
        assert!(plugins_dir.ends_with("plugins"));
        // В dev режиме путь не должен содержать AppData
        let path_str = plugins_dir.to_string_lossy();
        assert!(!path_str.contains("AppData"));
    }

    #[test]
    fn test_dev_mode_paths_not_in_appdata() {
        // В dev режиме все пути должны быть в корне проекта, не в AppData
        let paths = [
            get_app_data_dir(),
            get_binaries_dir(),
            get_hostlists_dir(),
            get_configs_dir(),
            get_logs_dir(),
            get_plugins_dir(),
        ];

        for path in paths {
            let path_str = path.to_string_lossy();
            assert!(
                !path_str.contains("AppData"),
                "Path {} should not contain AppData in dev mode",
                path_str
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_binary_paths_have_exe_extension() {
        let singbox = get_singbox_path();
        let winws = get_winws_path();

        assert!(
            singbox.extension().map_or(false, |ext| ext == "exe"),
            "sing-box path should have .exe extension on Windows: {:?}",
            singbox
        );
        assert!(
            winws.extension().map_or(false, |ext| ext == "exe"),
            "winws path should have .exe extension on Windows: {:?}",
            winws
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_binary_paths_no_exe_extension() {
        let singbox = get_singbox_path();
        let winws = get_winws_path();

        assert!(
            singbox.extension().is_none(),
            "sing-box path should not have extension on non-Windows: {:?}",
            singbox
        );
        assert!(
            winws.extension().is_none(),
            "winws path should not have extension on non-Windows: {:?}",
            winws
        );
    }

    #[test]
    fn test_binary_paths_inside_binaries_dir() {
        let binaries_dir = get_binaries_dir();
        let singbox = get_singbox_path();
        let winws = get_winws_path();

        assert!(
            singbox.starts_with(&binaries_dir),
            "sing-box should be inside binaries dir"
        );
        assert!(
            winws.starts_with(&binaries_dir),
            "winws should be inside binaries dir"
        );
    }

    #[test]
    fn test_binary_filenames() {
        let singbox = get_singbox_path();
        let winws = get_winws_path();

        let singbox_name = singbox.file_stem().unwrap().to_string_lossy();
        let winws_name = winws.file_stem().unwrap().to_string_lossy();

        assert_eq!(singbox_name, "sing-box");
        assert_eq!(winws_name, "winws");
    }

    #[test]
    fn test_database_filename() {
        let db_path = get_database_path();
        let filename = db_path.file_name().unwrap().to_string_lossy();
        assert_eq!(filename, "data.db");
    }

    #[test]
    fn test_directory_names() {
        // Проверяем что директории имеют ожидаемые имена
        assert!(get_binaries_dir().ends_with("bin"));
        assert!(get_hostlists_dir().ends_with("hostlists"));
        assert!(get_configs_dir().ends_with("configs"));
        assert!(get_logs_dir().ends_with("logs"));
        assert!(get_plugins_dir().ends_with("plugins"));
    }
}

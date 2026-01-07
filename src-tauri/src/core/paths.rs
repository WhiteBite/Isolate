//! Управление путями к бинарникам и конфигам
//!
//! Модуль предоставляет функции для получения путей к различным директориям
//! приложения с учётом режима разработки, продакшена и portable режима.
//!
//! ВАЖНО: Все функции для путей к бинарникам должны использоваться ТОЛЬКО из этого модуля!
//! НЕ дублировать get_singbox_path, get_winws_path и т.д. в других модулях!

#![allow(dead_code)] // Public paths API

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
        let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
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
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
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
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
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
// LOG ROTATION
// ============================================================================

/// Максимальное количество файлов логов оркестратора
pub const MAX_ORCHESTRA_LOGS: usize = 10;

/// Ротация логов оркестратора
///
/// Удаляет старые файлы логов, если их количество превышает MAX_ORCHESTRA_LOGS.
/// Файлы сортируются по времени модификации, самые старые удаляются первыми.
///
/// # Arguments
/// * `logs_dir` - Директория с логами
///
/// # Returns
/// * `Ok(deleted_count)` - Количество удалённых файлов
/// * `Err` - Ошибка при работе с файловой системой
pub fn rotate_orchestra_logs(logs_dir: &PathBuf) -> std::io::Result<usize> {
    use std::fs;
    
    // Получаем список файлов логов оркестратора
    let mut log_files: Vec<_> = fs::read_dir(logs_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name()
                .to_string_lossy()
                .starts_with("orchestra_")
        })
        .collect();
    
    // Сортируем по времени модификации (старые первые)
    log_files.sort_by_key(|entry| {
        entry.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    
    let mut deleted_count = 0;
    
    // Удаляем старые если превышен лимит
    while log_files.len() >= MAX_ORCHESTRA_LOGS {
        if let Some(oldest) = log_files.first() {
            let path = oldest.path();
            tracing::info!(
                file = %path.display(),
                "Removing old orchestra log"
            );
            fs::remove_file(&path)?;
            log_files.remove(0);
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}

/// Асинхронная ротация логов оркестратора
///
/// Асинхронная версия `rotate_orchestra_logs` для использования в async контексте.
pub async fn rotate_orchestra_logs_async(logs_dir: &PathBuf) -> std::io::Result<usize> {
    use tokio::fs;
    
    // Получаем список файлов логов оркестратора
    let mut entries = fs::read_dir(logs_dir).await?;
    let mut log_files: Vec<(std::path::PathBuf, std::time::SystemTime)> = Vec::new();
    
    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name();
        if file_name.to_string_lossy().starts_with("orchestra_") {
            let modified = entry.metadata().await
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            log_files.push((entry.path(), modified));
        }
    }
    
    // Сортируем по времени модификации (старые первые)
    log_files.sort_by_key(|(_, modified)| *modified);
    
    let mut deleted_count = 0;
    
    // Удаляем старые если превышен лимит
    while log_files.len() >= MAX_ORCHESTRA_LOGS {
        if let Some((path, _)) = log_files.first() {
            tracing::info!(
                file = %path.display(),
                "Removing old orchestra log"
            );
            fs::remove_file(&path).await?;
            log_files.remove(0);
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}

/// Создаёт путь к новому файлу лога оркестратора с timestamp
///
/// Формат: `orchestra_YYYY-MM-DD_HH-MM-SS.log`
///
/// # Arguments
/// * `logs_dir` - Директория для логов
///
/// # Returns
/// Путь к новому файлу лога
pub fn create_orchestra_log_path(logs_dir: &PathBuf) -> PathBuf {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("orchestra_{}.log", timestamp);
    logs_dir.join(filename)
}

/// Создаёт новый файл лога оркестратора с автоматической ротацией
///
/// 1. Выполняет ротацию старых логов
/// 2. Создаёт путь к новому файлу с timestamp
///
/// # Arguments
/// * `logs_dir` - Директория для логов
///
/// # Returns
/// * `Ok(PathBuf)` - Путь к новому файлу лога
/// * `Err` - Ошибка при ротации или создании
pub fn create_orchestra_log_file(logs_dir: &PathBuf) -> std::io::Result<PathBuf> {
    // Создаём директорию если не существует
    std::fs::create_dir_all(logs_dir)?;
    
    // Сначала ротируем старые логи
    let deleted = rotate_orchestra_logs(logs_dir)?;
    if deleted > 0 {
        tracing::info!(deleted_count = deleted, "Rotated old orchestra logs");
    }
    
    // Создаём путь к новому файлу
    let path = create_orchestra_log_path(logs_dir);
    
    Ok(path)
}

/// Асинхронная версия создания файла лога с ротацией
pub async fn create_orchestra_log_file_async(logs_dir: &PathBuf) -> std::io::Result<PathBuf> {
    // Создаём директорию если не существует
    tokio::fs::create_dir_all(logs_dir).await?;
    
    // Сначала ротируем старые логи
    let deleted = rotate_orchestra_logs_async(logs_dir).await?;
    if deleted > 0 {
        tracing::info!(deleted_count = deleted, "Rotated old orchestra logs");
    }
    
    // Создаём путь к новому файлу
    let path = create_orchestra_log_path(logs_dir);
    
    Ok(path)
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

/// Создаёт все необходимые директории приложения (async)
pub async fn ensure_dirs_exist() -> std::io::Result<()> {
    tokio::fs::create_dir_all(get_app_data_dir()).await?;
    tokio::fs::create_dir_all(get_binaries_dir()).await?;
    tokio::fs::create_dir_all(get_hostlists_dir()).await?;
    tokio::fs::create_dir_all(get_configs_dir()).await?;
    tokio::fs::create_dir_all(get_logs_dir()).await?;
    tokio::fs::create_dir_all(get_plugins_dir()).await?;
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

    #[test]
    fn test_max_orchestra_logs_constant() {
        assert_eq!(MAX_ORCHESTRA_LOGS, 10);
    }

    #[test]
    fn test_create_orchestra_log_path_format() {
        let logs_dir = PathBuf::from("/tmp/logs");
        let path = create_orchestra_log_path(&logs_dir);
        
        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.starts_with("orchestra_"), "Filename should start with 'orchestra_'");
        assert!(filename.ends_with(".log"), "Filename should end with '.log'");
        
        // Проверяем формат даты в имени файла
        // orchestra_YYYY-MM-DD_HH-MM-SS.log
        let name_without_ext = filename.trim_end_matches(".log");
        let parts: Vec<&str> = name_without_ext.split('_').collect();
        assert!(parts.len() >= 3, "Filename should have date and time parts");
    }

    #[test]
    fn test_rotate_orchestra_logs_empty_dir() {
        use std::fs;
        
        // Создаём временную директорию
        let temp_dir = std::env::temp_dir().join(format!("isolate_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        
        // Ротация пустой директории должна вернуть 0
        let result = rotate_orchestra_logs(&temp_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Очистка
        let _ = fs::remove_dir_all(&temp_dir);
    }
}

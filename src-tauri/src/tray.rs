use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    App, AppHandle, Emitter, Manager, Runtime,
};
use tracing::{info, error};

/// ID констант для пунктов меню
mod menu_ids {
    pub const TITLE: &str = "title";
    pub const OPEN: &str = "open";
    pub const TURBO: &str = "turbo";
    pub const DEEP: &str = "deep";
    pub const TOGGLE_BYPASS: &str = "toggle_bypass";
    pub const PANIC_RESET: &str = "panic_reset";
    pub const SETTINGS: &str = "settings";
    pub const QUIT: &str = "quit";
}

/// Создаёт System Tray иконку с меню
pub fn create_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();
    let menu = build_tray_menu(handle)?;
    
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_icon_event)
        .build(app)?;
    
    info!("System tray created successfully");
    Ok(())
}

/// Строит меню для tray
fn build_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, Box<dyn std::error::Error>> {
    // Заголовок (disabled)
    let title = MenuItem::with_id(app, menu_ids::TITLE, "Isolate", false, None::<&str>)?;
    
    // Основные действия
    let open = MenuItem::with_id(app, menu_ids::OPEN, "Открыть Isolate", true, None::<&str>)?;
    let turbo = MenuItem::with_id(app, menu_ids::TURBO, "Оптимизировать (Turbo)", true, None::<&str>)?;
    let deep = MenuItem::with_id(app, menu_ids::DEEP, "Оптимизировать (Deep)", true, None::<&str>)?;
    
    // Toggle bypass - начальное состояние "Включить"
    let toggle_bypass = MenuItem::with_id(app, menu_ids::TOGGLE_BYPASS, "Включить обход", true, None::<&str>)?;
    
    // Panic Reset
    let panic_reset = MenuItem::with_id(app, menu_ids::PANIC_RESET, "Panic Reset", true, None::<&str>)?;
    
    // Настройки и выход
    let settings = MenuItem::with_id(app, menu_ids::SETTINGS, "Настройки", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, menu_ids::QUIT, "Выход", true, None::<&str>)?;
    
    // Разделители
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let sep5 = PredefinedMenuItem::separator(app)?;
    
    let menu = Menu::with_items(app, &[
        &title,
        &sep1,
        &open,
        &turbo,
        &deep,
        &sep2,
        &toggle_bypass,
        &sep3,
        &panic_reset,
        &sep4,
        &settings,
        &sep5,
        &quit,
    ])?;
    
    Ok(menu)
}

/// Обрабатывает события меню tray
fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    info!("Tray menu event: {}", id);
    
    match id {
        menu_ids::OPEN => {
            show_main_window(app);
        }
        menu_ids::TURBO => {
            run_optimization(app, "turbo");
        }
        menu_ids::DEEP => {
            run_optimization(app, "deep");
        }
        menu_ids::TOGGLE_BYPASS => {
            toggle_bypass(app);
        }
        menu_ids::PANIC_RESET => {
            execute_panic_reset(app);
        }
        menu_ids::SETTINGS => {
            open_settings(app);
        }
        menu_ids::QUIT => {
            quit_app(app);
        }
        _ => {
            info!("Unknown menu item: {}", id);
        }
    }
}

/// Обрабатывает события иконки tray (клик по иконке)
fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let app = tray.app_handle();
        show_main_window(app);
    }
}

/// Показывает главное окно
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        info!("Main window shown");
    } else {
        error!("Main window not found");
    }
}

/// Запускает оптимизацию указанного режима
fn run_optimization(app: &AppHandle, mode: &str) {
    info!("Starting optimization: {}", mode);
    
    // Отправляем событие на frontend для запуска оптимизации
    if let Err(e) = app.emit("tray-optimization", mode) {
        error!("Failed to emit optimization event: {}", e);
    }
    
    // Показываем окно чтобы пользователь видел прогресс
    show_main_window(app);
}

/// Переключает состояние обхода
fn toggle_bypass(app: &AppHandle) {
    info!("Toggling bypass state");
    
    // Отправляем событие на frontend
    if let Err(e) = app.emit("tray-toggle-bypass", ()) {
        error!("Failed to emit toggle bypass event: {}", e);
    }
}

/// Выполняет Panic Reset
fn execute_panic_reset(app: &AppHandle) {
    info!("Executing Panic Reset from tray");
    
    // Отправляем событие на frontend
    if let Err(e) = app.emit("tray-panic-reset", ()) {
        error!("Failed to emit panic reset event: {}", e);
    }
    
    show_main_window(app);
}

/// Открывает настройки
fn open_settings(app: &AppHandle) {
    info!("Opening settings");
    
    // Отправляем событие для навигации к настройкам
    if let Err(e) = app.emit("tray-open-settings", ()) {
        error!("Failed to emit open settings event: {}", e);
    }
    
    show_main_window(app);
}

/// Завершает приложение
fn quit_app(app: &AppHandle) {
    info!("Quitting application from tray");
    
    // Отправляем событие для graceful shutdown
    if let Err(e) = app.emit("tray-quit", ()) {
        error!("Failed to emit quit event: {}", e);
    }
    
    // Даём время на cleanup и выходим
    app.exit(0);
}

/// Обновляет текст пункта меню toggle_bypass
pub fn update_bypass_menu_text(_app: &AppHandle, is_active: bool) {
    let text = if is_active {
        "Отключить обход"
    } else {
        "Включить обход"
    };
    
    // В Tauri 2.0 для обновления меню нужно пересоздать его
    // или использовать set_text если доступно
    info!("Bypass menu should show: {}", text);
}

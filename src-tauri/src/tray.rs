//! System Tray implementation for Isolate
//!
//! Provides system tray icon with context menu for quick access to main features.
//! Supports dynamic icon changes based on application state.

use std::sync::Arc;
use parking_lot::RwLock;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};
use tracing::{error, info, warn};

// ============================================================================
// Tray State
// ============================================================================

/// Global tray state for dynamic updates
static TRAY_STATE: once_cell::sync::Lazy<Arc<RwLock<TrayStateData>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(TrayStateData::default())));

/// Internal tray state data
#[derive(Debug, Clone, Default)]
struct TrayStateData {
    pub is_active: bool,
    pub strategy_name: Option<String>,
    pub state: TrayState,
    pub is_system_proxy: bool,
    pub is_tun: bool,
}

/// Tray icon states
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum TrayState {
    /// No bypass active (gray icon)
    #[default]
    Inactive,
    /// Bypass is active (green icon)
    Active,
    /// Optimization in progress (yellow icon)
    Optimizing,
    /// Error state (red icon)
    Error,
}

impl TrayState {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => TrayState::Active,
            "optimizing" => TrayState::Optimizing,
            "error" => TrayState::Error,
            _ => TrayState::Inactive,
        }
    }
    
    /// Get icon filename for this state
    pub fn icon_name(&self) -> &'static str {
        match self {
            TrayState::Inactive => "tray_inactive.ico",
            TrayState::Active => "tray_active.ico",
            TrayState::Optimizing => "tray_optimizing.ico",
            TrayState::Error => "tray_error.ico",
        }
    }
    
    /// Get emoji for this state (used in tooltip)
    pub fn emoji(&self) -> &'static str {
        match self {
            TrayState::Inactive => "⏸️",
            TrayState::Active => "✅",
            TrayState::Optimizing => "🔄",
            TrayState::Error => "❌",
        }
    }
    
    /// Get status text for this state
    pub fn status_text(&self) -> &'static str {
        match self {
            TrayState::Inactive => "Неактивен",
            TrayState::Active => "Активен",
            TrayState::Optimizing => "Оптимизация...",
            TrayState::Error => "Ошибка",
        }
    }
}

// ============================================================================
// Tray Creation
// ============================================================================

/// Creates the system tray with menu
pub fn create_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating system tray");
    
    // Create menu items
    let status_item = MenuItemBuilder::with_id("status", "Статус: Неактивен")
        .enabled(false)
        .build(app)?;
    
    let separator1 = PredefinedMenuItem::separator(app)?;
    
    let open_item = MenuItemBuilder::with_id("open", "Открыть Isolate")
        .build(app)?;
    
    // Optimize submenu
    let optimize_turbo = MenuItemBuilder::with_id("optimize_turbo", "⚡ Turbo (быстро)")
        .build(app)?;
    let optimize_deep = MenuItemBuilder::with_id("optimize_deep", "🔍 Deep (тщательно)")
        .build(app)?;
    
    let optimize_submenu = SubmenuBuilder::with_id(app, "optimize", "🚀 Оптимизировать")
        .items(&[&optimize_turbo, &optimize_deep])
        .build()?;
    
    let separator2 = PredefinedMenuItem::separator(app)?;
    
    let toggle_item = MenuItemBuilder::with_id("toggle", "▶️ Включить обход")
        .build(app)?;
    
    let stop_item = MenuItemBuilder::with_id("stop", "⏹️ Остановить")
        .enabled(false)
        .build(app)?;
    
    let separator3 = PredefinedMenuItem::separator(app)?;
    
    // Quick actions submenu
    let quic_block = MenuItemBuilder::with_id("quic_block", "Блокировать QUIC")
        .build(app)?;
    let quic_unblock = MenuItemBuilder::with_id("quic_unblock", "Разблокировать QUIC")
        .build(app)?;
    
    let quick_submenu = SubmenuBuilder::with_id(app, "quick", "⚙️ Быстрые действия")
        .items(&[&quic_block, &quic_unblock])
        .build()?;
    
    let separator4 = PredefinedMenuItem::separator(app)?;
    
    let panic_item = MenuItemBuilder::with_id("panic_reset", "⚠️ Сброс сети (Panic)")
        .build(app)?;
    
    let separator5 = PredefinedMenuItem::separator(app)?;
    
    let settings_item = MenuItemBuilder::with_id("settings", "⚙️ Настройки")
        .build(app)?;
    
    let logs_item = MenuItemBuilder::with_id("logs", "📋 Логи")
        .build(app)?;
    
    let separator6 = PredefinedMenuItem::separator(app)?;
    
    let quit_item = MenuItemBuilder::with_id("quit", "❌ Выход")
        .build(app)?;

    // Build menu
    let menu = MenuBuilder::new(app)
        .items(&[
            &status_item,
            &separator1,
            &open_item,
            &optimize_submenu,
            &separator2,
            &toggle_item,
            &stop_item,
            &separator3,
            &quick_submenu,
            &separator4,
            &panic_item,
            &separator5,
            &settings_item,
            &logs_item,
            &separator6,
            &quit_item,
        ])
        .build()?;

    // Create tray icon
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Isolate — DPI Bypass\nСтатус: Неактивен")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                show_main_window(app);
            }
        })
        .build(app)?;

    // Store tray reference for later updates
    app.manage(TrayHandle(Arc::new(RwLock::new(Some(tray)))));

    info!("System tray created successfully");
    Ok(())
}

/// Wrapper for tray handle to allow updates
pub struct TrayHandle<R: Runtime>(Arc<RwLock<Option<TrayIcon<R>>>>);

// ============================================================================
// Menu Event Handling
// ============================================================================

/// Handle menu item clicks
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "open" => {
            show_main_window(app);
        }
        "optimize_turbo" => {
            info!("Tray: Optimize Turbo");
            emit_and_show(app, "tray:optimize", "turbo");
        }
        "optimize_deep" => {
            info!("Tray: Optimize Deep");
            emit_and_show(app, "tray:optimize", "deep");
        }
        "toggle" => {
            info!("Tray: Toggle bypass");
            let state = TRAY_STATE.read();
            if state.is_active {
                if let Err(e) = app.emit("tray:stop", ()) {
                    error!("Failed to emit stop event: {}", e);
                }
            } else {
                // Show window to select strategy or run optimization
                emit_and_show(app, "tray:toggle", ());
            }
        }
        "stop" => {
            info!("Tray: Stop bypass");
            if let Err(e) = app.emit("tray:stop", ()) {
                error!("Failed to emit stop event: {}", e);
            }
        }
        "quic_block" => {
            info!("Tray: Block QUIC");
            if let Err(e) = app.emit("tray:quic_block", true) {
                error!("Failed to emit quic_block event: {}", e);
            }
        }
        "quic_unblock" => {
            info!("Tray: Unblock QUIC");
            if let Err(e) = app.emit("tray:quic_block", false) {
                error!("Failed to emit quic_unblock event: {}", e);
            }
        }
        "panic_reset" => {
            info!("Tray: Panic reset");
            emit_and_show(app, "tray:panic_reset", ());
        }
        "settings" => {
            show_main_window(app);
            if let Err(e) = app.emit("tray:navigate", "/settings") {
                error!("Failed to emit navigate event: {}", e);
            }
        }
        "logs" => {
            show_main_window(app);
            if let Err(e) = app.emit("tray:navigate", "/logs") {
                error!("Failed to emit navigate event: {}", e);
            }
        }
        "quit" => {
            info!("Tray: Quit requested");
            // Emit event for graceful shutdown
            if let Err(e) = app.emit("tray:quit", ()) {
                error!("Failed to emit quit event: {}", e);
            }
            // Give time for cleanup
            std::thread::sleep(std::time::Duration::from_millis(500));
            app.exit(0);
        }
        _ => {
            warn!("Unknown tray menu item: {}", id);
        }
    }
}

/// Helper to emit event and show window
fn emit_and_show<R: Runtime, S: serde::Serialize + Clone>(app: &AppHandle<R>, event: &str, payload: S) {
    if let Err(e) = app.emit(event, payload) {
        error!("Failed to emit {} event: {}", event, e);
    }
    show_main_window(app);
}

/// Show and focus main window
fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// ============================================================================
// Tray Updates
// ============================================================================

/// Update tray status and tooltip
pub fn update_tray_status<R: Runtime>(
    app: &AppHandle<R>,
    is_active: bool,
    strategy_name: Option<String>,
) {
    let new_state = if is_active { TrayState::Active } else { TrayState::Inactive };
    update_tray_state(app, new_state, strategy_name);
}

/// Update tray with specific state
pub fn update_tray_state<R: Runtime>(
    app: &AppHandle<R>,
    state: TrayState,
    strategy_name: Option<String>,
) {
    // Update global state
    {
        let mut data = TRAY_STATE.write();
        data.state = state;
        data.is_active = state == TrayState::Active;
        data.strategy_name = strategy_name.clone();
    }
    
    // Build tooltip
    let status_text = match state {
        TrayState::Inactive => "Неактивен".to_string(),
        TrayState::Active => format!("Активен: {}", strategy_name.as_deref().unwrap_or("Unknown")),
        TrayState::Optimizing => "Оптимизация...".to_string(),
        TrayState::Error => "Ошибка".to_string(),
    };
    
    let tooltip = format!("Isolate — DPI Bypass\n{} {}", state.emoji(), status_text);
    
    // Update tray tooltip
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            if let Err(e) = tray.set_tooltip(Some(&tooltip)) {
                error!("Failed to update tray tooltip: {}", e);
            }
        }
    }
    
    // Update icon based on state
    update_tray_icon(app, state);
    
    // Update menu items based on state
    update_tray_menu_items(app, state, strategy_name.as_deref());
    
    info!(
        state = ?state,
        strategy = strategy_name.as_deref().unwrap_or("none"),
        "Tray status updated"
    );
}

/// Update menu items based on current state
fn update_tray_menu_items<R: Runtime>(app: &AppHandle<R>, state: TrayState, strategy_name: Option<&str>) {
    // Update status menu item text
    let status_text = match state {
        TrayState::Inactive => "Статус: Неактивен".to_string(),
        TrayState::Active => format!("Статус: {} ✅", strategy_name.unwrap_or("Активен")),
        TrayState::Optimizing => "Статус: Оптимизация... 🔄".to_string(),
        TrayState::Error => "Статус: Ошибка ❌".to_string(),
    };
    
    // Try to update menu items via tray
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            // Note: Tauri 2.0 doesn't have direct menu item update API
            // Menu items are updated by rebuilding the menu or using menu item IDs
            // For now, we rely on tooltip updates
            let _ = tray.set_tooltip(Some(&format!("Isolate — DPI Bypass\n{}", status_text)));
        }
    }
}

/// Update tray icon based on state
pub fn update_tray_icon<R: Runtime>(app: &AppHandle<R>, state: TrayState) {
    // Update global state
    {
        let mut tray_state = TRAY_STATE.write();
        tray_state.state = state;
    }
    
    // Try to load state-specific icon
    let icon_name = state.icon_name();
    
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            // Try to load custom icon from resources
            match app.path().resource_dir() {
                Ok(resource_dir) => {
                    let icon_path = resource_dir.join("icons").join(icon_name);
                    if icon_path.exists() {
                        // Read icon file and create Image
                        match std::fs::read(&icon_path) {
                            Ok(icon_data) => {
                                match Image::from_bytes(&icon_data) {
                                    Ok(icon) => {
                                        if let Err(e) = tray.set_icon(Some(icon)) {
                                            error!("Failed to set tray icon: {}", e);
                                        } else {
                                            info!(state = ?state, path = %icon_path.display(), "Tray icon updated");
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to create icon from bytes: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to read icon file {}: {}", icon_path.display(), e);
                            }
                        }
                    } else {
                        // Icon file doesn't exist, use default
                        info!(state = ?state, path = %icon_path.display(), "Custom icon not found, using default");
                    }
                }
                Err(e) => {
                    warn!("Failed to get resource dir: {}", e);
                }
            }
            
            // Fall back to default icon
            if let Some(default_icon) = app.default_window_icon() {
                if let Err(e) = tray.set_icon(Some(default_icon.clone())) {
                    error!("Failed to set default tray icon: {}", e);
                }
            }
        }
    }
    
    info!(state = ?state, "Tray icon state changed");
}

/// Set tray to optimizing state
pub fn set_tray_optimizing<R: Runtime>(app: &AppHandle<R>) {
    {
        let mut state = TRAY_STATE.write();
        state.state = TrayState::Optimizing;
    }
    
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            let _ = tray.set_tooltip(Some("Isolate — DPI Bypass\n🔄 Оптимизация..."));
        }
    }
    
    update_tray_icon(app, TrayState::Optimizing);
}

/// Set tray to error state
pub fn set_tray_error<R: Runtime>(app: &AppHandle<R>, error_msg: &str) {
    {
        let mut state = TRAY_STATE.write();
        state.state = TrayState::Error;
    }
    
    let tooltip = format!("Isolate — DPI Bypass\n❌ Ошибка: {}", error_msg);
    
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            let _ = tray.set_tooltip(Some(&tooltip));
        }
    }
    
    update_tray_icon(app, TrayState::Error);
}

/// Get current tray state
pub fn get_tray_state() -> TrayState {
    TRAY_STATE.read().state
}

/// Check if bypass is currently active
pub fn is_bypass_active() -> bool {
    TRAY_STATE.read().is_active
}

/// Get current strategy name if active
pub fn get_active_strategy() -> Option<String> {
    let state = TRAY_STATE.read();
    if state.is_active {
        state.strategy_name.clone()
    } else {
        None
    }
}

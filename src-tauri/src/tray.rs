//! System Tray implementation for Isolate
//!
//! Provides system tray icon with context menu for quick access to main features.
//! Supports dynamic icon changes based on application state.

use std::sync::Arc;
use parking_lot::RwLock;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
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
pub struct TrayStateData {
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
            TrayState::Active => "🟢",
            TrayState::Optimizing => "🔄",
            TrayState::Error => "🔴",
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
    
    let menu = build_tray_menu(app)?;

    // Create tray icon
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Isolate — DPI Bypass\n⏸️ Неактивен")
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

/// Build tray menu based on current state
fn build_tray_menu<R: Runtime>(app: &impl Manager<R>) -> Result<tauri::menu::Menu<R>, Box<dyn std::error::Error>> {
    let state = TRAY_STATE.read();
    
    // Status line (dynamic)
    let status_text = match state.state {
        TrayState::Inactive => "⏸️ Неактивен".to_string(),
        TrayState::Active => format!("🟢 Активен: {}", state.strategy_name.as_deref().unwrap_or("Unknown")),
        TrayState::Optimizing => "🔄 Оптимизация...".to_string(),
        TrayState::Error => "🔴 Ошибка".to_string(),
    };
    
    let status_item = MenuItemBuilder::with_id("status", &status_text)
        .enabled(false)
        .build(app)?;
    
    let separator1 = PredefinedMenuItem::separator(app)?;
    
    // Open Isolate
    let open_item = MenuItemBuilder::with_id("open", "📊 Открыть Isolate")
        .build(app)?;
    
    let separator2 = PredefinedMenuItem::separator(app)?;
    
    // Optimization options
    let optimize_turbo = MenuItemBuilder::with_id("optimize_turbo", "⚡ Turbo оптимизация")
        .build(app)?;
    let optimize_deep = MenuItemBuilder::with_id("optimize_deep", "🔍 Deep оптимизация")
        .build(app)?;
    
    let separator3 = PredefinedMenuItem::separator(app)?;
    
    // Toggle bypass (dynamic text based on state)
    let toggle_text = if state.is_active {
        "❌ Отключить обход"
    } else {
        "✅ Включить обход"
    };
    let toggle_item = MenuItemBuilder::with_id("toggle_bypass", toggle_text)
        .build(app)?;
    
    let separator4 = PredefinedMenuItem::separator(app)?;
    
    // TUN Mode toggle
    let tun_text = if state.is_tun {
        "🔧 TUN Mode: Вкл"
    } else {
        "🔧 TUN Mode: Выкл"
    };
    let tun_item = MenuItemBuilder::with_id("toggle_tun", tun_text)
        .build(app)?;
    
    // System Proxy toggle
    let proxy_text = if state.is_system_proxy {
        "🌐 System Proxy: Вкл"
    } else {
        "🌐 System Proxy: Выкл"
    };
    let proxy_item = MenuItemBuilder::with_id("toggle_proxy", proxy_text)
        .build(app)?;
    
    let separator5 = PredefinedMenuItem::separator(app)?;
    
    // Panic Reset (red/warning)
    let panic_item = MenuItemBuilder::with_id("panic_reset", "🚨 Panic Reset")
        .build(app)?;
    
    let separator6 = PredefinedMenuItem::separator(app)?;
    
    // Settings and Exit
    let settings_item = MenuItemBuilder::with_id("settings", "⚙️ Настройки")
        .build(app)?;
    
    let quit_item = MenuItemBuilder::with_id("quit", "🚪 Выход")
        .build(app)?;

    // Build menu
    let menu = MenuBuilder::new(app)
        .items(&[
            &status_item,
            &separator1,
            &open_item,
            &separator2,
            &optimize_turbo,
            &optimize_deep,
            &separator3,
            &toggle_item,
            &separator4,
            &tun_item,
            &proxy_item,
            &separator5,
            &panic_item,
            &separator6,
            &settings_item,
            &quit_item,
        ])
        .build()?;

    Ok(menu)
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
            info!("Tray: Optimize Turbo requested");
            emit_event(app, "tray:optimize_turbo", ());
            show_main_window(app);
        }
        "optimize_deep" => {
            info!("Tray: Optimize Deep requested");
            emit_event(app, "tray:optimize_deep", ());
            show_main_window(app);
        }
        "toggle_bypass" => {
            info!("Tray: Toggle bypass requested");
            let state = TRAY_STATE.read();
            if state.is_active {
                emit_event(app, "tray:toggle_bypass", "off");
            } else {
                emit_event(app, "tray:toggle_bypass", "on");
                show_main_window(app);
            }
        }
        "toggle_tun" => {
            info!("Tray: Toggle TUN mode requested");
            let state = TRAY_STATE.read();
            emit_event(app, "tray:toggle_tun", !state.is_tun);
        }
        "toggle_proxy" => {
            info!("Tray: Toggle System Proxy requested");
            let state = TRAY_STATE.read();
            emit_event(app, "tray:toggle_proxy", !state.is_system_proxy);
        }
        "panic_reset" => {
            info!("Tray: Panic reset requested");
            emit_event(app, "tray:panic_reset", ());
            show_main_window(app);
        }
        "settings" => {
            show_main_window(app);
            emit_event(app, "tray:navigate", "/settings");
        }
        "quit" => {
            info!("Tray: Quit requested");
            // Emit event for graceful shutdown
            emit_event(app, "tray:quit", ());
            // Give time for cleanup
            std::thread::sleep(std::time::Duration::from_millis(500));
            app.exit(0);
        }
        _ => {
            warn!("Unknown tray menu item: {}", id);
        }
    }
}

/// Helper to emit event
fn emit_event<R: Runtime, S: serde::Serialize + Clone>(app: &AppHandle<R>, event: &str, payload: S) {
    if let Err(e) = app.emit(event, payload) {
        error!("Failed to emit {} event: {}", event, e);
    }
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

/// Rebuild tray menu with current state
/// 
/// Call this function whenever the state changes to update the menu dynamically.
pub fn rebuild_tray_menu<R: Runtime>(app: &AppHandle<R>) {
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            match build_tray_menu(app) {
                Ok(menu) => {
                    if let Err(e) = tray.set_menu(Some(menu)) {
                        error!("Failed to rebuild tray menu: {}", e);
                    } else {
                        info!("Tray menu rebuilt successfully");
                    }
                }
                Err(e) => {
                    error!("Failed to build tray menu: {}", e);
                }
            }
        }
    }
}

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
    
    // Rebuild menu to reflect new state
    rebuild_tray_menu(app);
    
    info!(
        state = ?state,
        strategy = strategy_name.as_deref().unwrap_or("none"),
        "Tray status updated"
    );
}

/// Update TUN mode status in tray
pub fn update_tray_tun_status<R: Runtime>(app: &AppHandle<R>, is_tun: bool) {
    {
        let mut data = TRAY_STATE.write();
        data.is_tun = is_tun;
    }
    
    rebuild_tray_menu(app);
    info!(is_tun, "Tray TUN status updated");
}

/// Update System Proxy status in tray
pub fn update_tray_proxy_status<R: Runtime>(app: &AppHandle<R>, is_proxy: bool) {
    {
        let mut data = TRAY_STATE.write();
        data.is_system_proxy = is_proxy;
    }
    
    rebuild_tray_menu(app);
    info!(is_proxy, "Tray System Proxy status updated");
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
    update_tray_state(app, TrayState::Optimizing, None);
}

/// Set tray to error state
pub fn set_tray_error<R: Runtime>(app: &AppHandle<R>, error_msg: &str) {
    {
        let mut state = TRAY_STATE.write();
        state.state = TrayState::Error;
    }
    
    let tooltip = format!("Isolate — DPI Bypass\n🔴 Ошибка: {}", error_msg);
    
    if let Some(tray_handle) = app.try_state::<TrayHandle<R>>() {
        if let Some(tray) = tray_handle.0.read().as_ref() {
            let _ = tray.set_tooltip(Some(&tooltip));
        }
    }
    
    update_tray_icon(app, TrayState::Error);
    rebuild_tray_menu(app);
}

/// Get current tray state
pub fn get_tray_state() -> TrayState {
    TRAY_STATE.read().state
}

/// Get full tray state data
pub fn get_tray_state_data() -> TrayStateData {
    TRAY_STATE.read().clone()
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

/// Check if TUN mode is active
pub fn is_tun_active() -> bool {
    TRAY_STATE.read().is_tun
}

/// Check if System Proxy is active
pub fn is_system_proxy_active() -> bool {
    TRAY_STATE.read().is_system_proxy
}

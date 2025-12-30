mod commands;
mod core;
pub mod state;
pub mod tray;

use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Isolate...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_strategies,
            commands::get_services,
            commands::run_optimization,
            commands::cancel_optimization,
            commands::apply_strategy,
            commands::stop_strategy,
            commands::diagnose,
            commands::panic_reset,
            // Settings commands
            commands::get_settings,
            commands::save_settings,
            commands::get_services_settings,
            commands::toggle_service,
            commands::get_app_version,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
            #[cfg(debug_assertions)]
            window.open_devtools();
            
            // Initialize AppState asynchronously
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match state::AppState::new().await {
                    Ok(app_state) => {
                        handle.manage(Arc::new(app_state));
                        tracing::info!("AppState initialized successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize AppState: {}", e);
                    }
                }
            });
            
            // Create System Tray
            if let Err(e) = tray::create_tray(app) {
                tracing::error!("Failed to create system tray: {}", e);
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

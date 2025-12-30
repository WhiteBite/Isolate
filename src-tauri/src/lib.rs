mod commands;
mod core;

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
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
            #[cfg(debug_assertions)]
            window.open_devtools();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

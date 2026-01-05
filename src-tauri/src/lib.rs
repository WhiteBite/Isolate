// Allow dead code for modules that are not yet fully connected
#![allow(dead_code)]

mod commands;
mod core;
pub mod plugins;
pub mod services;
pub mod state;
pub mod tray;

use std::sync::Arc;
use tauri::{Emitter, Manager};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Checks if the app was started with --silent flag
fn is_silent_mode() -> bool {
    std::env::args().any(|arg| arg == "--silent")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Setup file logging with rotation
    let log_dir = core::paths::get_logs_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "isolate.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive for the lifetime of the application
    // by storing it in a static or leaking it
    let _file_guard = Box::leak(Box::new(_guard));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
        )
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("isolate=info".parse().unwrap())
            .add_directive("tauri=warn".parse().unwrap())
        )
        .init();

    let silent_mode = is_silent_mode();
    
    tracing::info!(
        silent_mode,
        portable_mode = core::paths::is_portable_mode(),
        "Starting Isolate..."
    );

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }
    
    builder
        .invoke_handler(tauri::generate_handler![
            commands::is_backend_ready,
            commands::get_status,
            commands::get_strategies,
            commands::get_services,
            commands::run_optimization,
            commands::cancel_optimization,
            commands::apply_strategy,
            commands::stop_strategy,
            commands::diagnose,
            commands::panic_reset,
            // Diagnostics commands
            commands::run_diagnostics,
            commands::diagnose_dual_stack,
            commands::check_ipv6,
            // Settings commands
            commands::get_settings,
            commands::save_settings,
            commands::get_services_settings,
            commands::toggle_service,
            commands::get_app_version,
            // Generic setting commands
            commands::get_setting,
            commands::set_setting,
            // Update commands
            commands::check_for_updates,
            commands::install_update,
            // Log commands
            commands::get_logs,
            commands::clear_logs,
            commands::export_logs,
            // VLESS commands
            commands::import_vless,
            commands::get_vless_configs,
            commands::delete_vless_config,
            commands::toggle_vless_config,
            // VLESS proxy control commands
            commands::start_vless_proxy,
            commands::stop_vless_proxy,
            commands::stop_all_vless_proxies,
            commands::get_vless_status,
            commands::get_all_vless_status,
            commands::health_check_vless,
            commands::test_vless_connectivity,
            commands::is_singbox_available,
            commands::get_singbox_version,
            // Binary commands
            commands::check_binaries,
            commands::download_binaries,
            commands::get_binaries_dir,
            // QUIC blocking commands
            commands::enable_quic_block,
            commands::disable_quic_block,
            commands::set_quic_block,
            commands::is_quic_blocked,
            commands::is_admin,
            // Hostlist commands
            commands::get_hostlists,
            commands::get_hostlist,
            commands::add_hostlist_domain,
            commands::remove_hostlist_domain,
            commands::create_hostlist,
            commands::delete_hostlist,
            commands::update_hostlist_from_url,
            commands::save_hostlist,
            // Mode commands
            commands::is_silent_mode,
            commands::is_portable_mode,
            // Proxy management commands
            commands::get_proxies,
            commands::add_proxy,
            commands::update_proxy,
            commands::delete_proxy,
            commands::apply_proxy,
            commands::test_proxy,
            commands::import_proxy_url,
            commands::import_subscription,
            // Routing commands
            commands::get_domain_routes,
            commands::add_domain_route,
            commands::remove_domain_route,
            commands::get_app_routes,
            commands::add_app_route,
            commands::remove_app_route,
            commands::get_installed_apps,
            // Testing commands
            commands::run_tests,
            commands::cancel_tests,
            commands::test_strategy,
            // TUN mode commands
            commands::start_tun,
            commands::stop_tun,
            commands::is_tun_running,
            commands::get_tun_status,
            commands::get_tun_config,
            commands::set_tun_config,
            commands::is_tun_available,
            commands::restart_tun,
            // Tray commands
            commands::update_tray,
            commands::set_tray_optimizing,
            commands::set_tray_error,
            commands::get_tray_state,
            commands::update_tray_tun_status,
            commands::update_tray_proxy_status,
            commands::rebuild_tray_menu,
            // System Proxy commands
            commands::set_system_proxy,
            commands::clear_system_proxy,
            commands::is_system_proxy_set,
            // Monitor commands
            commands::start_monitor,
            commands::stop_monitor,
            commands::is_monitor_running,
            commands::is_strategy_degraded,
            commands::check_strategy_health,
            commands::set_monitor_urls,
            commands::set_monitor_auto_restart,
            // Telemetry commands
            commands::set_telemetry_enabled,
            commands::is_telemetry_enabled,
            commands::get_telemetry_pending_count,
            commands::flush_telemetry,
            commands::clear_telemetry,
            commands::report_optimization_telemetry,
            commands::report_strategy_usage_telemetry,
            // Config updater commands
            commands::check_config_updates,
            commands::download_config_updates,
            // Orchestra commands
            commands::start_orchestra,
            commands::stop_orchestra,
            commands::get_orchestra_results,
            commands::apply_orchestra_results,
            commands::save_orchestra_results,
            commands::load_orchestra_results,
            // Engine mode commands
            commands::get_engine_mode,
            commands::set_engine_mode,
            // AutoRun commands
            commands::get_autorun_status,
            commands::set_autorun,
            // Plugin commands (JS plugins)
            commands::get_plugins_dir,
            commands::scan_plugin_directories,
            commands::load_plugin_manifest,
            commands::get_all_plugins_cmd,
            commands::get_plugin_services,
            commands::check_plugin_service,
            commands::check_all_plugin_services,
            // Service Registry commands (new services system)
            commands::get_registry_services,
            commands::get_service_status,
            commands::check_single_service,
            commands::check_all_registry_services,
            commands::get_services_by_category,
            commands::clear_service_cache,
            commands::register_custom_service,
            commands::unregister_custom_service,
        ])
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();
            
            #[cfg(debug_assertions)]
            window.open_devtools();
            
            // Handle silent mode - start minimized to tray
            if silent_mode {
                tracing::info!("Silent mode: hiding main window");
                let _ = window.hide();
            }
            
            // Initialize AppState asynchronously
            let handle = app.handle().clone();
            let silent = silent_mode;
            tauri::async_runtime::spawn(async move {
                match state::AppState::new().await {
                    Ok(app_state) => {
                        // Check if auto_apply is enabled in silent mode
                        if silent {
                            let settings = app_state.storage.get_settings().await;
                            if let Ok(settings) = settings {
                                if settings.auto_apply {
                                    tracing::info!("Silent mode with auto_apply: applying last strategy");
                                    // Get last strategy and apply it
                                    if let Ok(Some(last_strategy)) = app_state.storage.get_setting::<String>("last_strategy").await {
                                        tracing::info!(strategy = %last_strategy, "Auto-applying last strategy");
                                        // Emit event to frontend to apply strategy
                                        let _ = handle.emit("auto-apply-strategy", &last_strategy);
                                    }
                                }
                            }
                        }
                        
                        handle.manage(Arc::new(app_state));
                        
                        // Start telemetry background flush if enabled
                        // Note: This is done after manage() so we can access state
                        
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

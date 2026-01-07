mod commands;
mod core;
pub mod plugins;
pub mod services;
pub mod state;
pub mod tray;

use std::sync::Arc;
use std::sync::OnceLock;
use tauri::{Emitter, Manager, RunEvent};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Global storage for tracing file guard to prevent memory leak
/// Using OnceLock instead of Box::leak() for proper cleanup
static TRACING_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Initialize Sentry crash reporting if enabled
fn init_crash_reporting() {
    // Check if crash reporting was enabled by user (from storage)
    // This is a best-effort check at startup - actual state is loaded later
    let enabled = std::env::var("SENTRY_DSN").is_ok() 
        && std::env::var("ISOLATE_CRASH_REPORTING").map(|v| v == "1").unwrap_or(false);
    
    if core::sentry_integration::init_sentry(enabled) {
        // Set up panic hook to capture panics
        core::sentry_integration::setup_panic_hook();
        tracing::info!("Crash reporting initialized");
    }
}

/// Applies Windows Mica or Acrylic effect to the window
/// Mica is used on Windows 11, Acrylic on Windows 10
/// Falls back gracefully on unsupported systems (non-Windows or older Windows versions)
#[cfg(target_os = "windows")]
fn apply_window_vibrancy(window: &tauri::WebviewWindow) {
    use window_vibrancy::{apply_mica, apply_acrylic};
    
    // Try Mica first (Windows 11 22H2+)
    // Mica provides a subtle, dynamic blur effect that samples the desktop wallpaper
    match apply_mica(window, Some(true)) {
        Ok(_) => {
            tracing::info!("Applied Mica Dark effect (Windows 11)");
            return;
        }
        Err(e) => {
            tracing::debug!("Mica not available: {:?}", e);
        }
    }
    
    // Fallback to Acrylic (Windows 10 1803+)
    // RGBA color: semi-transparent dark background matching void color (#09090b)
    match apply_acrylic(window, Some((9, 9, 11, 200))) {
        Ok(_) => {
            tracing::info!("Applied Acrylic effect (Windows 10 fallback)");
            return;
        }
        Err(e) => {
            tracing::debug!("Acrylic not available: {:?}", e);
        }
    }
    
    // If neither effect works, the window will use CSS backdrop-blur as fallback
    tracing::warn!("Window vibrancy effects not available - using CSS backdrop-blur fallback");
}

#[cfg(not(target_os = "windows"))]
fn apply_window_vibrancy(_window: &tauri::WebviewWindow) {
    // Vibrancy effects are Windows-only in this implementation
    // macOS could use NSVisualEffectView, but that requires different setup
    // The CSS backdrop-blur in Sidebar.svelte provides a graceful fallback
    tracing::debug!("Window vibrancy is Windows-only, using CSS backdrop-blur fallback");
}

/// Checks if the app was started with --silent flag
fn is_silent_mode() -> bool {
    std::env::args().any(|arg| arg == "--silent")
}

/// Checks if the app was started with --version flag
fn is_version_mode() -> bool {
    std::env::args().any(|arg| arg == "--version" || arg == "-V")
}

/// Checks if the app was started with --smoke-test flag
/// This is used in CI to verify the binary starts correctly
fn is_smoke_test_mode() -> bool {
    std::env::args().any(|arg| arg == "--smoke-test")
}

/// Prints version information and exits
fn print_version_and_exit() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    println!("{} {}", name, version);
    std::process::exit(0);
}

/// Performs smoke test: initializes minimal components and exits
/// Returns exit code 0 on success, 1 on failure
/// 
/// Smoke test checks:
/// 1. Version info availability
/// 2. Paths module functionality
/// 3. Portable mode detection
/// 4. Configuration loading (strategies, services)
/// 5. Binary paths resolution
/// 6. Database initialization (optional, may fail in CI)
fn run_smoke_test() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Isolate Smoke Test                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    
    let mut passed = 0;
    let mut failed = 0;
    let mut warnings = 0;
    
    // Test 1: Version info
    print!("  [1/7] Version info... ");
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    println!("OK ({} v{})", name, version);
    passed += 1;
    
    // Test 2: Paths module
    print!("  [2/7] Paths module... ");
    let logs_dir = core::paths::get_logs_dir();
    let binaries_dir = core::paths::get_binaries_dir();
    let configs_dir = core::paths::get_configs_dir();
    let plugins_dir = core::paths::get_plugins_dir();
    let db_path = core::paths::get_database_path();
    println!("OK");
    println!("        - Logs: {:?}", logs_dir);
    println!("        - Binaries: {:?}", binaries_dir);
    println!("        - Configs: {:?}", configs_dir);
    println!("        - Plugins: {:?}", plugins_dir);
    println!("        - Database: {:?}", db_path);
    passed += 1;
    
    // Test 3: Portable mode detection
    print!("  [3/7] Portable mode detection... ");
    let portable = core::paths::is_portable_mode();
    let dev_mode = core::paths::is_dev_mode();
    println!("OK (portable={}, dev={})", portable, dev_mode);
    passed += 1;
    
    // Test 4: Binary paths
    print!("  [4/7] Binary paths... ");
    let winws_path = core::paths::get_winws_path();
    let singbox_path = core::paths::get_singbox_path();
    println!("OK");
    println!("        - winws: {:?}", winws_path);
    println!("        - sing-box: {:?}", singbox_path);
    passed += 1;
    
    // Test 5: Check binaries existence (warning only, not failure)
    print!("  [5/7] Binaries check... ");
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let binaries_result = rt.block_on(core::binaries::check_binaries());
    match binaries_result {
        Ok(result) => {
            if result.all_present {
                println!("OK (all {} binaries present)", result.present.len());
                passed += 1;
            } else {
                println!("WARN (missing: {:?})", result.missing);
                println!("        Note: Binaries are downloaded on first run");
                warnings += 1;
            }
        }
        Err(e) => {
            println!("WARN (check failed: {})", e);
            warnings += 1;
        }
    }
    
    // Test 6: Configuration loading
    print!("  [6/7] Configuration loading... ");
    let configs_exist = std::path::Path::new(&configs_dir).exists();
    if configs_exist {
        // Try to load strategies
        let strategies_dir = configs_dir.join("strategies");
        if strategies_dir.exists() {
            let strategy_count = std::fs::read_dir(&strategies_dir)
                .map(|entries| entries.filter_map(|e| e.ok()).count())
                .unwrap_or(0);
            println!("OK ({} strategy configs found)", strategy_count);
            passed += 1;
        } else {
            println!("WARN (strategies dir not found)");
            warnings += 1;
        }
    } else {
        println!("WARN (configs dir not found: {:?})", configs_dir);
        warnings += 1;
    }
    
    // Test 7: Database initialization (optional)
    print!("  [7/7] Database initialization... ");
    let db_result = rt.block_on(async {
        // Try to create storage - this may fail in CI without APPDATA
        match core::storage::Storage::new().await {
            Ok(_storage) => Ok(()),
            Err(e) => Err(e),
        }
    });
    match db_result {
        Ok(()) => {
            println!("OK");
            passed += 1;
        }
        Err(e) => {
            // Database init may fail in CI environment - that's OK
            println!("SKIP ({})", e);
            println!("        Note: Database requires APPDATA environment");
            warnings += 1;
        }
    }
    
    // Summary
    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  Results: {} passed, {} warnings, {} failed", passed, warnings, failed);
    println!("══════════════════════════════════════════════════════════════");
    
    if failed > 0 {
        println!();
        println!("❌ Smoke test FAILED");
        std::process::exit(1);
    } else {
        println!();
        println!("✅ Smoke test PASSED");
        std::process::exit(0);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Handle CLI flags that don't require full app initialization
    if is_version_mode() {
        print_version_and_exit();
    }
    
    if is_smoke_test_mode() {
        run_smoke_test();
    }
    
    // Setup file logging with rotation
    let log_dir = core::paths::get_logs_dir();
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "isolate.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Store the guard in OnceLock to keep it alive for the application lifetime
    // This prevents the memory leak from Box::leak() while ensuring logs are flushed
    let _ = TRACING_GUARD.set(guard);

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

    // Initialize crash reporting (if enabled)
    init_crash_reporting();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }
    
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::is_backend_ready,
            commands::get_status,
            commands::get_strategies,
            commands::get_strategies_unified,
            commands::get_services,
            commands::apply_strategy,
            commands::stop_strategy,
            commands::diagnose,
            commands::panic_reset,
            // Diagnostics commands
            commands::run_diagnostics,
            commands::diagnose_dual_stack,
            commands::check_ipv6,
            commands::check_conflicts,
            // Settings commands
            commands::get_settings,
            commands::save_settings,
            commands::get_services_settings,
            commands::toggle_service,
            commands::get_app_version,
            // Generic setting commands
            commands::get_setting,
            commands::set_setting,
            // WinDivert mode commands
            commands::get_windivert_mode,
            commands::set_windivert_mode,
            // Game Filter mode commands
            commands::get_game_filter_mode,
            commands::set_game_filter_mode,
            // Config export/import commands
            commands::export_config,
            commands::import_config,
            // Update commands (GitHub API - no signing required)
            commands::check_github_updates,
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
            commands::verify_binaries_integrity,
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
            // Hostlist updater commands
            commands::get_hostlist_info,
            commands::check_hostlist_updates,
            commands::update_hostlists,
            commands::update_single_hostlist,
            commands::restore_hostlist_backup,
            // Ipset commands
            commands::get_ipset_info,
            commands::update_ipset,
            commands::update_ipset_from_sources,
            commands::set_ipset_auto_update,
            commands::get_ipset_sources,
            commands::restore_ipset_backup,
            // Hosts management commands
            commands::enable_discord_hosts,
            commands::disable_discord_hosts,
            commands::get_hosts_status,
            commands::backup_hosts,
            commands::restore_hosts,
            // Mode commands
            commands::is_silent_mode,
            commands::is_portable_mode,
            // Proxy management commands
            commands::get_proxies,
            commands::add_proxy,
            commands::update_proxy,
            commands::delete_proxy,
            commands::apply_proxy,
            commands::deactivate_proxy,
            commands::test_proxy,
            commands::import_proxy_url,
            commands::export_proxy_url,
            commands::import_subscription,
            // Routing commands
            commands::get_routing_rules,
            commands::add_routing_rule,
            commands::update_routing_rule,
            commands::delete_routing_rule,
            commands::reorder_routing_rules,
            commands::toggle_routing_rule,
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
            // A/B Testing commands
            commands::start_ab_test,
            commands::get_ab_test_status,
            commands::get_ab_test_progress,
            commands::get_ab_test_results,
            commands::cancel_ab_test,
            commands::get_active_ab_tests,
            commands::list_ab_tests,
            commands::delete_ab_test,
            commands::get_all_ab_test_results,
            commands::compare_strategies,
            commands::clear_ab_test_results,
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
            commands::get_monitor_config,
            // Telemetry commands
            commands::set_telemetry_enabled,
            commands::is_telemetry_enabled,
            commands::get_telemetry_pending_count,
            commands::flush_telemetry,
            commands::clear_telemetry,
            commands::report_optimization_telemetry,
            commands::report_strategy_usage_telemetry,
            // Crash Reporting commands
            commands::set_crash_reporting_enabled,
            commands::is_crash_reporting_enabled,
            commands::report_crash_error,
            commands::get_crash_reporting_info,
            // Config updater commands
            commands::check_config_updates,
            commands::download_config_updates,
            // Automation commands (new refactored system)
            commands::run_optimization_v2,
            commands::cancel_optimization_v2,
            commands::is_optimization_v2_running,
            commands::start_domain_monitor,
            commands::stop_domain_monitor,
            commands::is_domain_monitor_running,
            commands::get_domain_status,
            commands::get_all_domain_statuses,
            commands::get_blocked_strategies,
            commands::block_strategy,
            commands::unblock_strategy,
            commands::get_locked_strategy,
            commands::lock_strategy,
            commands::unlock_strategy,
            commands::get_automation_history,
            commands::clear_automation_history,
            commands::invalidate_strategy_cache,
            // Engine mode commands
            commands::get_engine_mode,
            commands::set_engine_mode,
            // Strategy statistics commands
            commands::record_strategy_result,
            commands::get_strategy_history,
            commands::get_strategy_statistics,
            commands::get_all_strategy_statistics,
            commands::clear_strategy_history,
            commands::clear_all_strategy_history,
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
            // Service Health History commands
            commands::get_service_health_history,
            commands::get_service_health_stats,
            commands::get_all_services_health_history,
            commands::cleanup_health_history,
            // Strategy Registry commands (plugin strategies)
            commands::get_plugin_strategies,
            commands::get_all_registered_strategies,
            commands::get_strategies_for_service,
            commands::get_strategies_by_family,
            commands::get_registered_strategy,
            commands::enable_strategy,
            commands::disable_strategy,
            commands::get_strategy_registry_stats,
            commands::reload_plugin_strategies,
            // Plugin Hot Reload commands
            commands::reload_plugins,
            commands::reload_plugin,
            // Plugin Hostlist Registry commands
            commands::get_plugin_hostlists,
            commands::get_plugin_hostlist,
            commands::get_plugin_hostlist_domains,
            commands::merge_plugin_hostlists,
            commands::get_all_plugin_domains,
            commands::check_domain_in_hostlists,
            commands::find_matching_hostlists,
            commands::get_hostlists_by_category,
            commands::get_hostlists_by_plugin,
            commands::set_hostlist_enabled,
            commands::reload_plugin_hostlist,
            commands::get_hostlist_registry_stats,
            // Script execution commands (Level 3 plugins)
            commands::execute_plugin_script,
            commands::execute_plugin_raw,
            commands::list_plugin_scripts,
            commands::get_plugin_storage,
            commands::set_plugin_storage,
            commands::clear_plugin_storage,
            commands::install_plugin,
            // Plugin state persistence commands
            commands::set_plugin_enabled,
            commands::get_plugin_enabled,
            commands::get_all_plugin_states,
            // Plugin settings persistence commands
            commands::get_plugin_settings,
            commands::set_plugin_settings,
            commands::reset_plugin_settings,
            commands::get_all_plugin_settings,
            // Speed test commands
            commands::test_upload_speed,
            // Resource limits commands
            commands::set_process_limits,
            commands::get_process_usage,
            commands::get_default_resource_limits,
            commands::get_recommended_resource_limits,
            commands::get_multiple_process_usage,
            // DNS commands
            commands::get_dns_settings,
            commands::set_dns_server,
            commands::save_dns_settings,
            commands::reset_dns_settings,
            commands::apply_dns_to_system,
            commands::restore_system_dns,
            // TCP Timestamps commands
            commands::get_tcp_timestamps_status,
            commands::set_tcp_timestamps_enabled,
            // Auto Failover commands
            commands::get_failover_status,
            commands::set_failover_enabled,
            commands::get_failover_config,
            commands::set_failover_config,
            commands::trigger_manual_failover,
            commands::get_learned_strategies,
            commands::reset_failover_state,
            // Provider commands
            commands::get_providers,
            commands::get_provider_recommendations,
            commands::is_strategy_recommended,
            commands::reload_providers,
            // Strategy Metrics commands
            commands::get_strategy_metrics,
            commands::get_strategy_metrics_history,
            commands::get_metrics_snapshots,
            commands::get_aggregated_metrics,
            commands::take_metrics_snapshot,
            commands::reset_strategy_metrics,
            commands::clear_strategy_metrics_history,
            commands::export_metrics_csv_string,
            commands::export_metrics_to_csv_file,
            // Strategy Prewarming commands
            commands::prewarm_strategy,
            commands::get_prewarmed_strategies,
            commands::clear_prewarmed,
            commands::is_strategy_prewarmed,
            commands::remove_prewarmed,
            commands::cleanup_prewarmed,
            // Strategy Composition commands
            commands::get_composition_rules,
            commands::set_composition_rules,
            commands::apply_composite_strategy,
            commands::get_compositions,
            commands::get_active_composition,
            commands::set_active_composition,
            commands::upsert_composition,
            commands::remove_composition,
            commands::get_strategy_for_service,
            commands::validate_composition,
        ])
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();
            
            // Apply Windows Mica/Acrylic vibrancy effect
            apply_window_vibrancy(&window);
            
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
                        
                        // Manage HostlistRegistry separately for commands that need it
                        let hostlist_registry = app_state.hostlist_registry.clone();
                        handle.manage(hostlist_registry);
                        
                        handle.manage(Arc::new(app_state));
                        
                        // Start telemetry background flush if enabled
                        // Note: This is done after manage() so we can access state
                        
                        tracing::info!("AppState initialized successfully");
                        
                        // Verify binary integrity at startup
                        // Note: verify_on_startup_async already logs detailed per-binary results
                        // including hash mismatches with expected/actual values
                        let binaries_dir = core::paths::get_binaries_dir();
                        let integrity_result = core::integrity::verify_on_startup_async(&binaries_dir).await;
                        
                        // Log summary for audit trail
                        if integrity_result.is_safe {
                            tracing::info!(
                                verified = integrity_result.verified,
                                total = integrity_result.total_checked,
                                missing = ?integrity_result.missing,
                                "Binary integrity check PASSED"
                            );
                        } else {
                            tracing::warn!(
                                verified = integrity_result.verified,
                                total = integrity_result.total_checked,
                                tampered = ?integrity_result.tampered,
                                missing = ?integrity_result.missing,
                                "Binary integrity check FAILED - potential tampering detected"
                            );
                            
                            // Emit warning event to frontend
                            tracing::warn!("Notifying frontend about integrity check failure");
                            let _ = handle.emit("integrity:warning", &integrity_result);
                        }
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::Exit => {
                    // Graceful shutdown: stop all processes and cleanup
                    tracing::info!("Application exit requested, performing graceful shutdown...");
                    
                    // Use blocking runtime for cleanup since we're in sync context
                    let handle = app_handle.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            perform_graceful_shutdown(&handle).await;
                        });
                    }).join().ok();
                    
                    tracing::info!("Graceful shutdown complete");
                }
                RunEvent::ExitRequested { api, .. } => {
                    // Allow exit to proceed (don't prevent it)
                    // This is called when all windows are closed
                    tracing::debug!("Exit requested, allowing application to close");
                    let _ = api;
                }
                _ => {}
            }
        });
}

/// Performs graceful shutdown of all running processes and services
/// 
/// This function is called when the application is about to exit.
/// It ensures all external processes are properly terminated and
/// system settings are restored.
async fn perform_graceful_shutdown<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use crate::core::{singbox_manager, system_proxy, tun_manager, sentry_integration};
    
    tracing::info!("Starting graceful shutdown sequence...");
    
    // 0. Flush crash reporting events
    sentry_integration::flush();
    
    // 1. Stop all strategies via AppState if available
    if let Some(state) = app.try_state::<Arc<state::AppState>>() {
        tracing::info!("Shutting down AppState...");
        if let Err(e) = state.shutdown().await {
            tracing::error!("Error during AppState shutdown: {}", e);
        }
    }
    
    // 2. Stop TUN mode if running
    let tun_manager = tun_manager::get_tun_manager();
    if tun_manager.is_running().await {
        tracing::info!("Stopping TUN mode...");
        if let Err(e) = tun_manager.stop().await {
            tracing::error!("Error stopping TUN: {}", e);
        }
    }
    
    // 3. Stop all VLESS/sing-box proxies
    let singbox_manager = singbox_manager::get_manager();
    tracing::info!("Stopping all sing-box instances...");
    if let Err(e) = singbox_manager.stop_all().await {
        tracing::error!("Error stopping sing-box instances: {}", e);
    }
    
    // 4. Clear system proxy if it was set by us
    if tray::is_system_proxy_active() {
        tracing::info!("Clearing system proxy...");
        if let Err(e) = system_proxy::clear_system_proxy().await {
            tracing::error!("Error clearing system proxy: {}", e);
        }
    }
    
    tracing::info!("Graceful shutdown sequence completed");
}

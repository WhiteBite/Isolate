//! Core modules for Isolate

pub mod app_routing;
pub mod autorun;
pub mod binaries;
pub mod config;
pub mod config_updater;
pub mod diagnostics;
pub mod global_runner;
pub mod log_capture;
pub mod domain_routing;
pub mod env_info;
pub mod errors;
pub mod hostlists;
pub mod integrity;
pub mod models;
pub mod monitor;
pub mod nodpi_engine;
pub mod orchestra;
pub mod orchestrator;
pub mod paths;
pub mod process_runner;
pub mod proxy_parser;
pub mod quic_blocker;
pub mod scoring;
pub mod singbox_config;
pub mod singbox_manager;
pub mod storage;
pub mod strategy_combiner;
pub mod strategy_engine;
pub mod strategy_loader;
pub mod strategy_tester;
pub mod system_proxy;
pub mod telemetry;
pub mod test_engine;
pub mod tun_manager;
pub mod vless_engine;

// Re-exports removed to avoid unused import warnings
// Use crate::core::strategy_loader::StrategyLoader directly


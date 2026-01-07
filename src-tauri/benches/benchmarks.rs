//! Performance benchmarks for Isolate
//!
//! Run all benchmarks:
//!   cargo bench --manifest-path src-tauri/Cargo.toml
//!
//! Run specific benchmark group:
//!   cargo bench --manifest-path src-tauri/Cargo.toml -- scoring
//!   cargo bench --manifest-path src-tauri/Cargo.toml -- yaml_parsing
//!   cargo bench --manifest-path src-tauri/Cargo.toml -- strategy_ranking
//!
//! These benchmarks measure CPU-bound operations:
//! - Strategy scoring calculations
//! - YAML strategy parsing
//! - Config loading and validation
//! - Strategy lookup and filtering

mod scoring;
mod strategy_engine;

use criterion::criterion_main;

criterion_main!(
    scoring::scoring_benches,
    strategy_engine::strategy_engine_benches,
);

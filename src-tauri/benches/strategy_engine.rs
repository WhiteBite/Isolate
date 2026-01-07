//! Strategy Engine benchmarks for Isolate
//!
//! Benchmarks for strategy loading, validation, and YAML parsing.
//!
//! Run with: cargo bench --manifest-path src-tauri/Cargo.toml strategy_engine

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Strategy Types (mirrors models.rs)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StrategyFamily {
    DnsBypass,
    SniFrag,
    TlsFrag,
    Vless,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StrategyEngine {
    Zapret,
    SingBox,
    Xray,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ModeCapabilities {
    supports_socks: bool,
    supports_global: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct LaunchTemplate {
    binary: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    log_file: Option<String>,
    #[serde(default)]
    requires_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StrategyRequirements {
    #[serde(default)]
    min_rights: String,
    #[serde(default)]
    os: Vec<String>,
    #[serde(default)]
    binaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Strategy {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    family: StrategyFamily,
    engine: StrategyEngine,
    #[serde(default)]
    mode_capabilities: ModeCapabilities,
    socks_template: Option<LaunchTemplate>,
    global_template: Option<LaunchTemplate>,
    #[serde(default)]
    requirements: StrategyRequirements,
    #[serde(default)]
    weight_hint: i32,
    #[serde(default)]
    services: Vec<String>,
}

// ============================================================================
// Test YAML Data
// ============================================================================

/// Simple strategy YAML (like youtube_zapret.yaml)
const SIMPLE_STRATEGY_YAML: &str = r#"
id: "zapret_youtube_disorder"
name: "YouTube Disorder"
description: "Фрагментация SNI с перемешиванием пакетов для YouTube"
family: sni_frag
engine: zapret

mode_capabilities:
  supports_socks: false
  supports_global: true

global_template:
  binary: "binaries/winws.exe"
  args:
    - "--wf-tcp=443"
    - "--filter-tcp=443"
    - "--hostlist=hostlists/youtube.txt"
    - "--dpi-desync=fake,disorder2"
    - "--dpi-desync-autottl=2"
    - "--dpi-desync-fooling=md5sig"
  env: {}
  requires_admin: true

requirements:
  min_rights: "admin"
  os:
    - windows
  binaries:
    - "binaries/winws.exe"

weight_hint: 10
services:
  - youtube
"#;


/// Complex strategy YAML (like general_multisplit.yaml)
const COMPLEX_STRATEGY_YAML: &str = r#"
id: "zapret_general_multisplit"
name: "General Multisplit"
description: "Универсальная стратегия с multisplit фрагментацией"
family: sni_frag
engine: zapret

mode_capabilities:
  supports_socks: false
  supports_global: true

global_template:
  binary: "binaries/winws.exe"
  args:
    - "--wf-tcp=80,443,2053,2083,2087,2096,8443"
    - "--wf-udp=443,19294-19344,50000-50100"
    - "--filter-udp=443"
    - "--hostlist=hostlists/general.txt"
    - "--hostlist-exclude=hostlists/exclude.txt"
    - "--ipset-exclude=hostlists/ipset-exclude.txt"
    - "--dpi-desync=fake"
    - "--dpi-desync-repeats=6"
    - "--new"
    - "--filter-udp=19294-19344,50000-50100"
    - "--filter-l7=discord,stun"
    - "--dpi-desync=fake"
    - "--dpi-desync-repeats=6"
    - "--new"
    - "--filter-tcp=2053,2083,2087,2096,8443"
    - "--hostlist-domains=discord.media"
    - "--dpi-desync=multisplit"
    - "--dpi-desync-split-seqovl=568"
    - "--dpi-desync-split-pos=1"
    - "--new"
    - "--filter-tcp=443"
    - "--hostlist=hostlists/google.txt"
    - "--ip-id=zero"
    - "--dpi-desync=multisplit"
    - "--dpi-desync-split-seqovl=681"
    - "--dpi-desync-split-pos=1"
    - "--new"
    - "--filter-tcp=80,443"
    - "--hostlist=hostlists/general.txt"
    - "--dpi-desync=multisplit"
    - "--dpi-desync-split-seqovl=568"
    - "--dpi-desync-split-pos=1"
  env: {}
  requires_admin: true

requirements:
  min_rights: "admin"
  os:
    - windows
  binaries:
    - "binaries/winws.exe"
    - "binaries/WinDivert64.sys"

weight_hint: 12
services:
  - discord
  - youtube
  - general
"#;


// ============================================================================
// Validation Functions (mirrors config_validation.rs)
// ============================================================================

/// Validate strategy (mirrors config.rs logic)
fn validate_strategy(strategy: &Strategy) -> Result<(), String> {
    if strategy.id.is_empty() {
        return Err("Strategy ID cannot be empty".into());
    }

    if strategy.name.is_empty() {
        return Err(format!("Strategy '{}' has empty name", strategy.id));
    }

    if strategy.socks_template.is_none() && strategy.global_template.is_none() {
        return Err(format!(
            "Strategy '{}' must have at least one launch template",
            strategy.id
        ));
    }

    if let Some(ref template) = strategy.socks_template {
        if template.binary.is_empty() {
            return Err(format!(
                "Strategy '{}' socks template has empty binary path",
                strategy.id
            ));
        }
    }

    if let Some(ref template) = strategy.global_template {
        if template.binary.is_empty() {
            return Err(format!(
                "Strategy '{}' global template has empty binary path",
                strategy.id
            ));
        }
    }

    Ok(())
}

fn create_valid_strategy(id: &str) -> Strategy {
    Strategy {
        id: id.to_string(),
        name: format!("Strategy {}", id),
        description: "Test strategy".to_string(),
        family: StrategyFamily::SniFrag,
        engine: StrategyEngine::Zapret,
        mode_capabilities: ModeCapabilities {
            supports_socks: false,
            supports_global: true,
        },
        socks_template: None,
        global_template: Some(LaunchTemplate {
            binary: "binaries/winws.exe".to_string(),
            args: vec!["--arg1".to_string(), "--arg2".to_string()],
            env: HashMap::new(),
            log_file: None,
            requires_admin: true,
        }),
        requirements: StrategyRequirements {
            min_rights: "admin".to_string(),
            os: vec!["windows".to_string()],
            binaries: vec!["binaries/winws.exe".to_string()],
        },
        weight_hint: 10,
        services: vec!["youtube".to_string()],
    }
}


// ============================================================================
// Benchmarks
// ============================================================================

pub fn bench_yaml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("yaml_parsing");
    
    // Benchmark simple strategy parsing
    group.throughput(Throughput::Bytes(SIMPLE_STRATEGY_YAML.len() as u64));
    group.bench_function("simple_strategy", |b| {
        b.iter(|| {
            let _: Strategy = serde_yaml::from_str(black_box(SIMPLE_STRATEGY_YAML)).unwrap();
        })
    });
    
    // Benchmark complex strategy parsing
    group.throughput(Throughput::Bytes(COMPLEX_STRATEGY_YAML.len() as u64));
    group.bench_function("complex_strategy", |b| {
        b.iter(|| {
            let _: Strategy = serde_yaml::from_str(black_box(COMPLEX_STRATEGY_YAML)).unwrap();
        })
    });
    
    // Benchmark batch parsing (simulating loading multiple strategies)
    let batch_yaml: String = (0..10)
        .map(|i| SIMPLE_STRATEGY_YAML.replace("zapret_youtube_disorder", &format!("strategy_{}", i)))
        .collect::<Vec<_>>()
        .join("\n---\n");
    
    group.throughput(Throughput::Elements(10));
    group.bench_function("batch_10_strategies", |b| {
        b.iter(|| {
            for doc in serde_yaml::Deserializer::from_str(black_box(&batch_yaml)) {
                let _: Strategy = serde::Deserialize::deserialize(doc).unwrap();
            }
        })
    });
    
    group.finish();
}

pub fn bench_config_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_validation");
    
    // Single strategy validation
    let strategy = create_valid_strategy("test_1");
    group.bench_function("single_strategy", |b| {
        b.iter(|| validate_strategy(black_box(&strategy)))
    });
    
    // Batch validation
    for size in [10, 50, 100].iter() {
        let strategies: Vec<Strategy> = (0..*size)
            .map(|i| create_valid_strategy(&format!("strategy_{}", i)))
            .collect();
        
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_validation", size),
            &strategies,
            |b, strategies| {
                b.iter(|| {
                    for strategy in strategies {
                        let _ = validate_strategy(black_box(strategy));
                    }
                })
            },
        );
    }
    
    group.finish();
}

pub fn bench_strategy_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategy_lookup");
    
    // Simulate strategy lookup by ID in a HashMap
    for size in [10, 50, 100, 200].iter() {
        let strategies: HashMap<String, Strategy> = (0..*size)
            .map(|i| {
                let id = format!("strategy_{}", i);
                (id.clone(), create_valid_strategy(&id))
            })
            .collect();
        
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("hashmap_get", size),
            &strategies,
            |b, strategies| {
                b.iter(|| {
                    strategies.get(black_box("strategy_50"))
                })
            },
        );
        
        // Benchmark iteration and filter
        group.bench_with_input(
            BenchmarkId::new("filter_by_engine", size),
            &strategies,
            |b, strategies| {
                b.iter(|| {
                    strategies.values()
                        .filter(|s| matches!(s.engine, StrategyEngine::Zapret))
                        .count()
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    strategy_engine_benches,
    bench_yaml_parsing,
    bench_config_validation,
    bench_strategy_lookup,
);

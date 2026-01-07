//! Scoring benchmarks for Isolate
//!
//! Benchmarks for the scoring module that calculates strategy scores
//! based on test results.
//!
//! Run with: cargo bench --manifest-path src-tauri/Cargo.toml scoring

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};

/// Mock ServiceTestSummary for benchmarks (mirrors the real struct)
#[derive(Clone)]
struct ServiceTestSummary {
    service_id: String,
    total_tests: u32,
    passed_tests: u32,
    success_rate: f64,
    avg_latency_ms: f64,
}

fn create_summary(service_id: &str, total: u32, passed: u32, latency: f64) -> ServiceTestSummary {
    ServiceTestSummary {
        service_id: service_id.to_string(),
        total_tests: total,
        passed_tests: passed,
        success_rate: if total > 0 { passed as f64 / total as f64 } else { 0.0 },
        avg_latency_ms: latency,
    }
}

// ============================================================================
// Scoring Functions (mirrors scoring.rs)
// ============================================================================

/// Calculate success rate (mirrors scoring.rs logic)
fn calculate_success_rate(summaries: &[ServiceTestSummary]) -> f64 {
    if summaries.is_empty() {
        return 0.0;
    }
    let total_tests: u32 = summaries.iter().map(|s| s.total_tests).sum();
    let passed_tests: u32 = summaries.iter().map(|s| s.passed_tests).sum();
    if total_tests == 0 {
        return 0.0;
    }
    passed_tests as f64 / total_tests as f64
}

/// Calculate critical success rate
fn calculate_critical_success_rate(
    summaries: &[ServiceTestSummary],
    critical_service_ids: &[String],
) -> f64 {
    let critical_summaries: Vec<_> = summaries
        .iter()
        .filter(|s| critical_service_ids.contains(&s.service_id))
        .collect();

    if critical_summaries.is_empty() {
        return calculate_success_rate(summaries);
    }
    let total_tests: u32 = critical_summaries.iter().map(|s| s.total_tests).sum();
    let passed_tests: u32 = critical_summaries.iter().map(|s| s.passed_tests).sum();
    if total_tests == 0 {
        return 0.0;
    }
    passed_tests as f64 / total_tests as f64
}

/// Calculate average latency
fn calculate_latency_avg(summaries: &[ServiceTestSummary]) -> f64 {
    let latencies: Vec<f64> = summaries
        .iter()
        .filter(|s| s.passed_tests > 0 && s.avg_latency_ms > 0.0)
        .map(|s| s.avg_latency_ms)
        .collect();
    if latencies.is_empty() {
        return 0.0;
    }
    latencies.iter().sum::<f64>() / latencies.len() as f64
}

/// Calculate jitter (coefficient of variation)
fn calculate_jitter(summaries: &[ServiceTestSummary]) -> f64 {
    let latencies: Vec<f64> = summaries
        .iter()
        .filter(|s| s.passed_tests > 0 && s.avg_latency_ms > 0.0)
        .map(|s| s.avg_latency_ms)
        .collect();
    if latencies.len() < 2 {
        return 0.0;
    }
    let mean = latencies.iter().sum::<f64>() / latencies.len() as f64;
    if mean == 0.0 {
        return 0.0;
    }
    let variance = latencies.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / latencies.len() as f64;
    let std_dev = variance.sqrt();
    let cv = std_dev / mean;
    cv.min(1.0)
}

const WEIGHT_SUCCESS_RATE: f64 = 0.5;
const WEIGHT_CRITICAL_SUCCESS: f64 = 0.3;
const WEIGHT_LATENCY: f64 = 0.15;
const WEIGHT_JITTER: f64 = 0.05;
const MAX_LATENCY_MS: f64 = 5000.0;

/// Calculate full strategy score
fn calculate_score(
    summaries: &[ServiceTestSummary],
    critical_service_ids: &[String],
) -> f64 {
    let success_rate = calculate_success_rate(summaries);
    let critical_success_rate = calculate_critical_success_rate(summaries, critical_service_ids);
    let latency_avg = calculate_latency_avg(summaries);
    let latency_jitter = calculate_jitter(summaries);
    let normalized_latency = (latency_avg / MAX_LATENCY_MS).min(1.0);
    
    (success_rate * WEIGHT_SUCCESS_RATE)
        + (critical_success_rate * WEIGHT_CRITICAL_SUCCESS)
        + ((1.0 - normalized_latency) * WEIGHT_LATENCY)
        + ((1.0 - latency_jitter) * WEIGHT_JITTER)
}

/// Strategy score result
#[derive(Clone)]
struct StrategyScore {
    strategy_id: String,
    success_rate: f64,
    score: f64,
}

/// Filter viable strategies (success_rate >= 0.8)
fn filter_viable_strategies(scores: &[StrategyScore]) -> Vec<&StrategyScore> {
    scores.iter().filter(|s| s.success_rate >= 0.8).collect()
}

/// Get best strategy by score
fn get_best_strategy(scores: &[StrategyScore]) -> Option<&StrategyScore> {
    let viable = filter_viable_strategies(scores);
    viable.into_iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
}

/// Rank strategies by score (descending)
fn rank_strategies(scores: &mut [StrategyScore]) {
    scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
}

// ============================================================================
// Test Data Generators
// ============================================================================

/// Generate test data with N services
fn generate_test_data(n: usize) -> (Vec<ServiceTestSummary>, Vec<String>) {
    let services = ["youtube", "discord", "telegram", "twitter", "instagram", 
                   "facebook", "tiktok", "twitch", "spotify", "netflix"];
    
    let summaries: Vec<ServiceTestSummary> = (0..n)
        .map(|i| {
            let service = services[i % services.len()];
            let passed = (8 + (i % 3)) as u32; // 8-10 passed
            create_summary(service, 10, passed, 100.0 + (i as f64 * 10.0))
        })
        .collect();
    
    let critical = vec!["youtube".to_string(), "discord".to_string()];
    
    (summaries, critical)
}

/// Generate strategy scores for ranking benchmarks
fn generate_strategy_scores(n: usize) -> Vec<StrategyScore> {
    (0..n)
        .map(|i| StrategyScore {
            strategy_id: format!("strategy_{}", i),
            success_rate: 0.7 + (i as f64 * 0.01).min(0.29),
            score: 0.6 + (i as f64 * 0.005).min(0.39),
        })
        .collect()
}

// ============================================================================
// Benchmarks
// ============================================================================

pub fn bench_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("scoring");
    
    // Benchmark with different numbers of services
    for size in [5, 10, 20, 50, 100].iter() {
        let (summaries, critical) = generate_test_data(*size);
        
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("success_rate", size),
            &summaries,
            |b, summaries| {
                b.iter(|| calculate_success_rate(black_box(summaries)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("critical_success_rate", size),
            &(&summaries, &critical),
            |b, (summaries, critical)| {
                b.iter(|| calculate_critical_success_rate(black_box(summaries), black_box(critical)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("latency_avg", size),
            &summaries,
            |b, summaries| {
                b.iter(|| calculate_latency_avg(black_box(summaries)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("jitter", size),
            &summaries,
            |b, summaries| {
                b.iter(|| calculate_jitter(black_box(summaries)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("full_score", size),
            &(&summaries, &critical),
            |b, (summaries, critical)| {
                b.iter(|| calculate_score(black_box(summaries), black_box(critical)))
            },
        );
    }
    
    group.finish();
}

pub fn bench_strategy_ranking(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategy_ranking");
    
    for size in [10, 50, 100, 200].iter() {
        let scores = generate_strategy_scores(*size);
        
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("filter_viable", size),
            &scores,
            |b, scores| {
                b.iter(|| filter_viable_strategies(black_box(scores)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("get_best", size),
            &scores,
            |b, scores| {
                b.iter(|| get_best_strategy(black_box(scores)))
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("rank_all", size),
            &scores,
            |b, scores| {
                b.iter(|| {
                    let mut scores_clone = scores.clone();
                    rank_strategies(black_box(&mut scores_clone))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    scoring_benches,
    bench_scoring,
    bench_strategy_ranking,
);

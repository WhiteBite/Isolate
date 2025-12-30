//! Scoring module for Isolate
//!
//! Calculates strategy scores based on test results.
//! Score formula: (success_rate * 0.5) + (critical_success_rate * 0.3) +
//!                (1.0 - normalized_latency) * 0.15 + (1.0 - jitter) * 0.05

use tracing::{debug, instrument};

use crate::core::models::{ServiceTestSummary, StrategyScore};

/// Weight for success rate in score calculation
const WEIGHT_SUCCESS_RATE: f64 = 0.5;

/// Weight for critical service success rate
const WEIGHT_CRITICAL_SUCCESS: f64 = 0.3;

/// Weight for latency component
const WEIGHT_LATENCY: f64 = 0.15;

/// Weight for jitter component
const WEIGHT_JITTER: f64 = 0.05;

/// Maximum acceptable latency for normalization (ms)
const MAX_LATENCY_MS: f64 = 5000.0;

/// Minimum success rate threshold for strategy to be considered viable
const MIN_SUCCESS_RATE_THRESHOLD: f64 = 0.8;

/// Calculates overall success rate from service test summaries
pub fn calculate_success_rate(summaries: &[ServiceTestSummary]) -> f64 {
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

/// Calculates success rate for critical services only
pub fn calculate_critical_success_rate(
    summaries: &[ServiceTestSummary],
    critical_service_ids: &[String],
) -> f64 {
    let critical_summaries: Vec<_> = summaries
        .iter()
        .filter(|s| critical_service_ids.contains(&s.service_id))
        .collect();

    if critical_summaries.is_empty() {
        // If no critical services defined, use overall success rate
        return calculate_success_rate(summaries);
    }

    let total_tests: u32 = critical_summaries.iter().map(|s| s.total_tests).sum();
    let passed_tests: u32 = critical_summaries.iter().map(|s| s.passed_tests).sum();

    if total_tests == 0 {
        return 0.0;
    }

    passed_tests as f64 / total_tests as f64
}

/// Calculates average latency across all successful tests
pub fn calculate_latency_avg(summaries: &[ServiceTestSummary]) -> f64 {
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

/// Calculates latency jitter (standard deviation / mean)
/// Returns a value between 0.0 (no jitter) and 1.0 (high jitter)
pub fn calculate_jitter(summaries: &[ServiceTestSummary]) -> f64 {
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

    // Calculate standard deviation
    let variance = latencies
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>()
        / latencies.len() as f64;

    let std_dev = variance.sqrt();

    // Coefficient of variation (normalized jitter)
    let cv = std_dev / mean;

    // Clamp to [0, 1] range
    cv.min(1.0)
}

/// Normalizes latency to [0, 1] range where 0 is best (lowest latency)
fn normalize_latency(latency_ms: f64) -> f64 {
    (latency_ms / MAX_LATENCY_MS).min(1.0)
}

/// Calculates the final score for a strategy
///
/// Formula: (success_rate * 0.5) + (critical_success_rate * 0.3) +
///          (1.0 - normalized_latency) * 0.15 + (1.0 - jitter) * 0.05
#[instrument(skip_all, fields(strategy_id = %strategy_id))]
pub fn calculate_score(
    strategy_id: &str,
    summaries: &[ServiceTestSummary],
    critical_service_ids: &[String],
) -> StrategyScore {
    let success_rate = calculate_success_rate(summaries);
    let critical_success_rate = calculate_critical_success_rate(summaries, critical_service_ids);
    let latency_avg = calculate_latency_avg(summaries);
    let latency_jitter = calculate_jitter(summaries);

    let normalized_latency = normalize_latency(latency_avg);

    // Calculate final score using the formula
    let score = (success_rate * WEIGHT_SUCCESS_RATE)
        + (critical_success_rate * WEIGHT_CRITICAL_SUCCESS)
        + ((1.0 - normalized_latency) * WEIGHT_LATENCY)
        + ((1.0 - latency_jitter) * WEIGHT_JITTER);

    debug!(
        "Strategy {} score: {:.3} (sr={:.2}, csr={:.2}, lat={:.0}ms, jit={:.2})",
        strategy_id, score, success_rate, critical_success_rate, latency_avg, latency_jitter
    );

    StrategyScore {
        strategy_id: strategy_id.to_string(),
        success_rate,
        critical_success_rate,
        latency_avg,
        latency_jitter,
        score,
    }
}

/// Filters strategies by minimum success rate threshold
pub fn filter_viable_strategies(scores: &[StrategyScore]) -> Vec<&StrategyScore> {
    scores
        .iter()
        .filter(|s| s.success_rate >= MIN_SUCCESS_RATE_THRESHOLD)
        .collect()
}

/// Filters strategies by custom success rate threshold
pub fn filter_by_threshold(scores: &[StrategyScore], threshold: f64) -> Vec<&StrategyScore> {
    scores
        .iter()
        .filter(|s| s.success_rate >= threshold)
        .collect()
}

/// Sorts strategies by score (descending) and returns the best one
pub fn get_best_strategy(scores: &[StrategyScore]) -> Option<&StrategyScore> {
    let viable = filter_viable_strategies(scores);

    viable
        .into_iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
}

/// Ranks all strategies by score (descending)
pub fn rank_strategies(scores: &mut [StrategyScore]) {
    scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
}

/// Returns top N strategies by score
pub fn get_top_strategies(scores: &[StrategyScore], n: usize) -> Vec<&StrategyScore> {
    let mut sorted: Vec<_> = scores.iter().collect();
    sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    sorted.into_iter().take(n).collect()
}

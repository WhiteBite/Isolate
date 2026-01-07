//! Scoring module for Isolate
//!
//! Calculates strategy scores based on test results.
//! Score formula: (success_rate * 0.5) + (critical_success_rate * 0.3) +
//!                (1.0 - normalized_latency) * 0.15 + (1.0 - jitter) * 0.05
//!
//! Note: This module provides public scoring API for strategy evaluation.

#![allow(dead_code)] // Public scoring API

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
    let variance = latencies.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / latencies.len() as f64;
    let std_dev = variance.sqrt();
    let cv = std_dev / mean;
    cv.min(1.0)
}

/// Normalizes latency to [0, 1] range where 0 is best (lowest latency)
fn normalize_latency(latency_ms: f64) -> f64 {
    (latency_ms / MAX_LATENCY_MS).min(1.0)
}

/// Calculates the final score for a strategy
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
    scores.iter().filter(|s| s.success_rate >= MIN_SUCCESS_RATE_THRESHOLD).collect()
}

/// Filters strategies by custom success rate threshold
pub fn filter_by_threshold(scores: &[StrategyScore], threshold: f64) -> Vec<&StrategyScore> {
    scores.iter().filter(|s| s.success_rate >= threshold).collect()
}

/// Sorts strategies by score (descending) and returns the best one
pub fn get_best_strategy(scores: &[StrategyScore]) -> Option<&StrategyScore> {
    let viable = filter_viable_strategies(scores);
    viable.into_iter().max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
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


#[cfg(test)]
mod tests {
    use super::*;

    fn create_summary(service_id: &str, total: u32, passed: u32, latency: f64) -> ServiceTestSummary {
        ServiceTestSummary {
            service_id: service_id.to_string(),
            total_tests: total,
            passed_tests: passed,
            success_rate: if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            avg_latency_ms: latency,
            errors: vec![],
        }
    }

    fn create_score(id: &str, success_rate: f64, score: f64) -> StrategyScore {
        StrategyScore {
            strategy_id: id.to_string(),
            success_rate,
            critical_success_rate: success_rate,
            latency_avg: 100.0,
            latency_jitter: 0.1,
            score,
        }
    }

    // ==================== calculate_success_rate ====================

    #[test]
    fn test_success_rate_empty_summaries() {
        let summaries: Vec<ServiceTestSummary> = vec![];
        assert_eq!(calculate_success_rate(&summaries), 0.0);
    }

    #[test]
    fn test_success_rate_all_passed() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 5, 5, 150.0),
        ];
        assert_eq!(calculate_success_rate(&summaries), 1.0);
    }

    #[test]
    fn test_success_rate_none_passed() {
        let summaries = vec![create_summary("youtube", 10, 0, 0.0), create_summary("discord", 5, 0, 0.0)];
        assert_eq!(calculate_success_rate(&summaries), 0.0);
    }

    #[test]
    fn test_success_rate_partial() {
        let summaries = vec![create_summary("youtube", 10, 8, 100.0), create_summary("discord", 10, 6, 150.0)];
        assert!((calculate_success_rate(&summaries) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_success_rate_zero_total_tests() {
        let summaries = vec![create_summary("youtube", 0, 0, 0.0), create_summary("discord", 0, 0, 0.0)];
        assert_eq!(calculate_success_rate(&summaries), 0.0);
    }

    #[test]
    fn test_success_rate_single_service() {
        let summaries = vec![create_summary("youtube", 100, 85, 200.0)];
        assert!((calculate_success_rate(&summaries) - 0.85).abs() < 0.001);
    }

    // ==================== calculate_critical_success_rate ====================

    #[test]
    fn test_critical_success_rate_empty_critical_list() {
        let summaries = vec![create_summary("youtube", 10, 8, 100.0), create_summary("discord", 10, 6, 150.0)];
        let critical: Vec<String> = vec![];
        assert!((calculate_critical_success_rate(&summaries, &critical) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_critical_success_rate_with_critical_services() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 10, 5, 150.0),
            create_summary("telegram", 10, 8, 120.0),
        ];
        let critical = vec!["youtube".to_string(), "telegram".to_string()];
        assert!((calculate_critical_success_rate(&summaries, &critical) - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_critical_success_rate_no_matching_services() {
        let summaries = vec![create_summary("youtube", 10, 8, 100.0), create_summary("discord", 10, 6, 150.0)];
        let critical = vec!["nonexistent".to_string()];
        assert!((calculate_critical_success_rate(&summaries, &critical) - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_critical_success_rate_all_critical_failed() {
        let summaries = vec![create_summary("youtube", 10, 0, 0.0), create_summary("discord", 10, 10, 100.0)];
        let critical = vec!["youtube".to_string()];
        assert_eq!(calculate_critical_success_rate(&summaries, &critical), 0.0);
    }

    #[test]
    fn test_critical_success_rate_empty_summaries() {
        let summaries: Vec<ServiceTestSummary> = vec![];
        let critical = vec!["youtube".to_string()];
        assert_eq!(calculate_critical_success_rate(&summaries, &critical), 0.0);
    }

    // ==================== calculate_latency_avg ====================

    #[test]
    fn test_latency_avg_empty_summaries() {
        let summaries: Vec<ServiceTestSummary> = vec![];
        assert_eq!(calculate_latency_avg(&summaries), 0.0);
    }

    #[test]
    fn test_latency_avg_normal() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 10, 10, 200.0),
            create_summary("telegram", 10, 10, 300.0),
        ];
        assert!((calculate_latency_avg(&summaries) - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_latency_avg_excludes_failed_services() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 10, 0, 500.0),
            create_summary("telegram", 10, 10, 200.0),
        ];
        assert!((calculate_latency_avg(&summaries) - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_latency_avg_excludes_zero_latency() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 10, 10, 0.0),
            create_summary("telegram", 10, 10, 200.0),
        ];
        assert!((calculate_latency_avg(&summaries) - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_latency_avg_all_failed() {
        let summaries = vec![create_summary("youtube", 10, 0, 100.0), create_summary("discord", 10, 0, 200.0)];
        assert_eq!(calculate_latency_avg(&summaries), 0.0);
    }

    #[test]
    fn test_latency_avg_single_service() {
        let summaries = vec![create_summary("youtube", 10, 10, 150.0)];
        assert!((calculate_latency_avg(&summaries) - 150.0).abs() < 0.001);
    }

    // ==================== calculate_jitter ====================

    #[test]
    fn test_jitter_empty_summaries() {
        let summaries: Vec<ServiceTestSummary> = vec![];
        assert_eq!(calculate_jitter(&summaries), 0.0);
    }

    #[test]
    fn test_jitter_single_service() {
        let summaries = vec![create_summary("youtube", 10, 10, 100.0)];
        assert_eq!(calculate_jitter(&summaries), 0.0);
    }

    #[test]
    fn test_jitter_identical_latencies() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 10, 10, 100.0),
            create_summary("telegram", 10, 10, 100.0),
        ];
        assert_eq!(calculate_jitter(&summaries), 0.0);
    }

    #[test]
    fn test_jitter_varied_latencies() {
        let summaries = vec![create_summary("youtube", 10, 10, 100.0), create_summary("discord", 10, 10, 200.0)];
        let jitter = calculate_jitter(&summaries);
        assert!(jitter > 0.0 && jitter < 1.0);
        assert!((jitter - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_jitter_high_variation() {
        let summaries = vec![create_summary("youtube", 10, 10, 10.0), create_summary("discord", 10, 10, 1000.0)];
        let jitter = calculate_jitter(&summaries);
        assert!(jitter > 0.5);
    }

    #[test]
    fn test_jitter_excludes_failed_services() {
        let summaries = vec![
            create_summary("youtube", 10, 10, 100.0),
            create_summary("discord", 10, 0, 5000.0),
            create_summary("telegram", 10, 10, 100.0),
        ];
        assert_eq!(calculate_jitter(&summaries), 0.0);
    }

    #[test]
    fn test_jitter_clamped_to_one() {
        let summaries = vec![create_summary("youtube", 10, 10, 1.0), create_summary("discord", 10, 10, 10000.0)];
        let jitter = calculate_jitter(&summaries);
        assert!(jitter <= 1.0);
    }

    // ==================== calculate_score ====================

    #[test]
    fn test_score_perfect() {
        let summaries = vec![create_summary("youtube", 10, 10, 0.0), create_summary("discord", 10, 10, 0.0)];
        let critical = vec!["youtube".to_string()];
        let score = calculate_score("test_strategy", &summaries, &critical);
        assert_eq!(score.success_rate, 1.0);
        assert_eq!(score.critical_success_rate, 1.0);
    }

    #[test]
    fn test_score_empty_summaries() {
        let summaries: Vec<ServiceTestSummary> = vec![];
        let critical: Vec<String> = vec![];
        let score = calculate_score("test_strategy", &summaries, &critical);
        assert_eq!(score.success_rate, 0.0);
        assert_eq!(score.critical_success_rate, 0.0);
        assert_eq!(score.latency_avg, 0.0);
        assert_eq!(score.latency_jitter, 0.0);
        assert!((score.score - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_score_formula_verification() {
        let summaries = vec![create_summary("youtube", 10, 8, 1000.0), create_summary("discord", 10, 8, 1000.0)];
        let critical = vec!["youtube".to_string()];
        let score = calculate_score("test_strategy", &summaries, &critical);
        // Expected: 0.8*0.5 + 0.8*0.3 + (1-0.2)*0.15 + (1-0)*0.05 = 0.81
        assert!((score.score - 0.81).abs() < 0.01);
    }

    #[test]
    fn test_score_high_latency_penalty() {
        let summaries = vec![create_summary("youtube", 10, 10, 5000.0), create_summary("discord", 10, 10, 5000.0)];
        let critical: Vec<String> = vec![];
        let score = calculate_score("test_strategy", &summaries, &critical);
        // Score: 1.0*0.5 + 1.0*0.3 + 0*0.15 + 1.0*0.05 = 0.85
        assert!((score.score - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_score_strategy_id_preserved() {
        let summaries = vec![create_summary("youtube", 10, 10, 100.0)];
        let critical: Vec<String> = vec![];
        let score = calculate_score("my_unique_strategy", &summaries, &critical);
        assert_eq!(score.strategy_id, "my_unique_strategy");
    }

    // ==================== filter_viable_strategies ====================

    #[test]
    fn test_filter_viable_empty() {
        let scores: Vec<StrategyScore> = vec![];
        let viable = filter_viable_strategies(&scores);
        assert!(viable.is_empty());
    }

    #[test]
    fn test_filter_viable_all_pass() {
        let scores = vec![create_score("s1", 0.9, 0.85), create_score("s2", 0.85, 0.80), create_score("s3", 0.8, 0.75)];
        let viable = filter_viable_strategies(&scores);
        assert_eq!(viable.len(), 3);
    }

    #[test]
    fn test_filter_viable_some_fail() {
        let scores = vec![create_score("s1", 0.9, 0.85), create_score("s2", 0.7, 0.65), create_score("s3", 0.5, 0.45)];
        let viable = filter_viable_strategies(&scores);
        assert_eq!(viable.len(), 1);
        assert_eq!(viable[0].strategy_id, "s1");
    }

    #[test]
    fn test_filter_viable_none_pass() {
        let scores = vec![create_score("s1", 0.7, 0.65), create_score("s2", 0.5, 0.45), create_score("s3", 0.3, 0.25)];
        let viable = filter_viable_strategies(&scores);
        assert!(viable.is_empty());
    }

    #[test]
    fn test_filter_viable_boundary() {
        let scores = vec![create_score("s1", 0.8, 0.75), create_score("s2", 0.79, 0.74)];
        let viable = filter_viable_strategies(&scores);
        assert_eq!(viable.len(), 1);
        assert_eq!(viable[0].strategy_id, "s1");
    }

    // ==================== filter_by_threshold ====================

    #[test]
    fn test_filter_by_threshold_custom() {
        let scores = vec![create_score("s1", 0.95, 0.90), create_score("s2", 0.85, 0.80), create_score("s3", 0.75, 0.70)];
        let result = filter_by_threshold(&scores, 0.9);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].strategy_id, "s1");
    }

    #[test]
    fn test_filter_by_threshold_zero() {
        let scores = vec![create_score("s1", 0.5, 0.45), create_score("s2", 0.0, 0.0)];
        let result = filter_by_threshold(&scores, 0.0);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_by_threshold_one() {
        let scores = vec![create_score("s1", 1.0, 0.95), create_score("s2", 0.99, 0.94)];
        let result = filter_by_threshold(&scores, 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].strategy_id, "s1");
    }

    // ==================== get_best_strategy ====================

    #[test]
    fn test_get_best_empty() {
        let scores: Vec<StrategyScore> = vec![];
        assert!(get_best_strategy(&scores).is_none());
    }

    #[test]
    fn test_get_best_single_viable() {
        let scores = vec![create_score("s1", 0.9, 0.85)];
        let best = get_best_strategy(&scores);
        assert!(best.is_some());
        assert_eq!(best.unwrap().strategy_id, "s1");
    }

    #[test]
    fn test_get_best_multiple_viable() {
        let scores = vec![create_score("s1", 0.9, 0.80), create_score("s2", 0.85, 0.90), create_score("s3", 0.8, 0.75)];
        let best = get_best_strategy(&scores);
        assert!(best.is_some());
        assert_eq!(best.unwrap().strategy_id, "s2");
    }

    #[test]
    fn test_get_best_none_viable() {
        let scores = vec![create_score("s1", 0.7, 0.65), create_score("s2", 0.5, 0.45)];
        assert!(get_best_strategy(&scores).is_none());
    }

    #[test]
    fn test_get_best_equal_scores() {
        let scores = vec![create_score("s1", 0.9, 0.85), create_score("s2", 0.9, 0.85)];
        let best = get_best_strategy(&scores);
        assert!(best.is_some());
    }

    #[test]
    fn test_get_best_filters_then_selects() {
        let scores = vec![create_score("s1", 0.7, 0.95), create_score("s2", 0.85, 0.80)];
        let best = get_best_strategy(&scores);
        assert!(best.is_some());
        assert_eq!(best.unwrap().strategy_id, "s2");
    }

    // ==================== rank_strategies ====================

    #[test]
    fn test_rank_empty() {
        let mut scores: Vec<StrategyScore> = vec![];
        rank_strategies(&mut scores);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_rank_single() {
        let mut scores = vec![create_score("s1", 0.9, 0.85)];
        rank_strategies(&mut scores);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].strategy_id, "s1");
    }

    #[test]
    fn test_rank_multiple() {
        let mut scores = vec![create_score("s1", 0.9, 0.70), create_score("s2", 0.85, 0.90), create_score("s3", 0.8, 0.80)];
        rank_strategies(&mut scores);
        assert_eq!(scores[0].strategy_id, "s2");
        assert_eq!(scores[1].strategy_id, "s3");
        assert_eq!(scores[2].strategy_id, "s1");
    }

    #[test]
    fn test_rank_already_sorted() {
        let mut scores = vec![create_score("s1", 0.9, 0.90), create_score("s2", 0.85, 0.80), create_score("s3", 0.8, 0.70)];
        rank_strategies(&mut scores);
        assert_eq!(scores[0].strategy_id, "s1");
        assert_eq!(scores[1].strategy_id, "s2");
        assert_eq!(scores[2].strategy_id, "s3");
    }

    #[test]
    fn test_rank_reverse_sorted() {
        let mut scores = vec![create_score("s1", 0.9, 0.70), create_score("s2", 0.85, 0.80), create_score("s3", 0.8, 0.90)];
        rank_strategies(&mut scores);
        assert_eq!(scores[0].strategy_id, "s3");
        assert_eq!(scores[1].strategy_id, "s2");
        assert_eq!(scores[2].strategy_id, "s1");
    }

    // ==================== get_top_strategies ====================

    #[test]
    fn test_get_top_empty() {
        let scores: Vec<StrategyScore> = vec![];
        let top = get_top_strategies(&scores, 3);
        assert!(top.is_empty());
    }

    #[test]
    fn test_get_top_less_than_n() {
        let scores = vec![create_score("s1", 0.9, 0.85), create_score("s2", 0.85, 0.80)];
        let top = get_top_strategies(&scores, 5);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_get_top_exact_n() {
        let scores = vec![create_score("s1", 0.9, 0.85), create_score("s2", 0.85, 0.80), create_score("s3", 0.8, 0.75)];
        let top = get_top_strategies(&scores, 3);
        assert_eq!(top.len(), 3);
    }

    #[test]
    fn test_get_top_more_than_n() {
        let scores = vec![
            create_score("s1", 0.9, 0.70),
            create_score("s2", 0.85, 0.90),
            create_score("s3", 0.8, 0.80),
            create_score("s4", 0.75, 0.60),
        ];
        let top = get_top_strategies(&scores, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].strategy_id, "s2");
        assert_eq!(top[1].strategy_id, "s3");
    }

    #[test]
    fn test_get_top_zero() {
        let scores = vec![create_score("s1", 0.9, 0.85)];
        let top = get_top_strategies(&scores, 0);
        assert!(top.is_empty());
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_nan_handling_in_jitter() {
        let summaries = vec![create_summary("youtube", 10, 10, 0.0), create_summary("discord", 10, 10, 0.0)];
        let jitter = calculate_jitter(&summaries);
        assert!(!jitter.is_nan());
        assert_eq!(jitter, 0.0);
    }

    #[test]
    fn test_very_small_latencies() {
        let summaries = vec![create_summary("youtube", 10, 10, 0.001), create_summary("discord", 10, 10, 0.002)];
        let avg = calculate_latency_avg(&summaries);
        assert!(avg > 0.0);
        assert!((avg - 0.0015).abs() < 0.0001);
    }

    #[test]
    fn test_very_large_latencies() {
        let summaries = vec![create_summary("youtube", 10, 10, 100000.0), create_summary("discord", 10, 10, 100000.0)];
        let score = calculate_score("test", &summaries, &[]);
        assert!(score.score >= 0.0 && score.score <= 1.0);
    }

    #[test]
    fn test_mixed_zero_and_nonzero_tests() {
        let summaries = vec![create_summary("youtube", 0, 0, 0.0), create_summary("discord", 10, 10, 100.0)];
        let rate = calculate_success_rate(&summaries);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_score_components_sum_to_one_max() {
        let total = WEIGHT_SUCCESS_RATE + WEIGHT_CRITICAL_SUCCESS + WEIGHT_LATENCY + WEIGHT_JITTER;
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_score_bounds() {
        let cases = vec![
            (vec![], vec![]),
            (vec![create_summary("a", 10, 0, 0.0)], vec![]),
            (vec![create_summary("a", 10, 10, 0.0)], vec![]),
            (vec![create_summary("a", 10, 10, 10000.0)], vec![]),
        ];
        for (summaries, critical) in cases {
            let score = calculate_score("test", &summaries, &critical);
            assert!(score.score >= 0.0, "Score should be >= 0");
            assert!(score.score <= 1.0, "Score should be <= 1");
        }
    }

    #[test]
    fn test_unicode_service_ids() {
        let summaries = vec![create_summary("ютуб", 10, 10, 100.0), create_summary("дискорд", 10, 8, 150.0)];
        let critical = vec!["ютуб".to_string()];
        let score = calculate_score("стратегия_1", &summaries, &critical);
        assert_eq!(score.strategy_id, "стратегия_1");
        assert_eq!(score.critical_success_rate, 1.0);
    }
}

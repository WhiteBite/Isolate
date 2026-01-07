//! Score calculator for strategy evaluation
//!
//! Calculates strategy scores based on probe results.
//! The score is a weighted combination of:
//! - Success rate (how many endpoints succeeded)
//! - Critical success rate (how many critical endpoints succeeded)
//! - Latency (lower is better)
//!
//! NOTE: This module is part of the testing infrastructure for strategy validation.

// Public API for strategy scoring
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use super::prober::ProbeResult;

/// Weights for score calculation components
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    /// Weight for overall success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Weight for critical endpoint success rate (0.0 - 1.0)
    pub critical_success: f64,
    /// Weight for latency component (0.0 - 1.0)
    pub latency: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            success_rate: 0.4,
            critical_success: 0.3,
            latency: 0.3,
        }
    }
}

impl ScoreWeights {
    /// Create custom weights
    ///
    /// Note: Weights should sum to 1.0 for normalized scores
    pub fn new(success_rate: f64, critical_success: f64, latency: f64) -> Self {
        Self {
            success_rate,
            critical_success,
            latency,
        }
    }

    /// Create weights that prioritize critical services
    pub fn critical_priority() -> Self {
        Self {
            success_rate: 0.2,
            critical_success: 0.6,
            latency: 0.2,
        }
    }

    /// Create weights that prioritize low latency
    pub fn latency_priority() -> Self {
        Self {
            success_rate: 0.3,
            critical_success: 0.2,
            latency: 0.5,
        }
    }

    /// Validate that weights sum to approximately 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.success_rate + self.critical_success + self.latency;
        (sum - 1.0).abs() < 0.001
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.success_rate + self.critical_success + self.latency;
        if sum > 0.0 {
            self.success_rate /= sum;
            self.critical_success /= sum;
            self.latency /= sum;
        }
    }
}

/// Calculated score for a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyScore {
    /// Strategy identifier
    pub strategy_id: String,
    /// Final weighted score (0.0 - 1.0)
    pub score: f64,
    /// Overall success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Critical endpoint success rate (0.0 - 1.0)
    pub critical_success_rate: f64,
    /// Average latency in milliseconds
    pub latency_avg: f64,
    /// Latency jitter (standard deviation)
    pub latency_jitter: f64,
}

impl StrategyScore {
    /// Check if strategy is viable (has minimum success rate)
    pub fn is_viable(&self, min_success_rate: f64) -> bool {
        self.success_rate >= min_success_rate
    }

    /// Check if all critical endpoints succeeded
    pub fn all_critical_passed(&self) -> bool {
        self.critical_success_rate >= 1.0 - f64::EPSILON
    }
}

/// Calculator for strategy scores
pub struct ScoreCalculator {
    weights: ScoreWeights,
}

impl ScoreCalculator {
    /// Create calculator with custom weights
    pub fn new(weights: ScoreWeights) -> Self {
        Self { weights }
    }

    /// Create calculator with default weights
    pub fn with_default_weights() -> Self {
        Self::new(ScoreWeights::default())
    }

    /// Get current weights
    pub fn weights(&self) -> &ScoreWeights {
        &self.weights
    }

    /// Calculate score for a strategy based on probe results
    #[instrument(skip(self, results), fields(strategy_id = %strategy_id))]
    pub fn calculate(&self, strategy_id: &str, results: &[ProbeResult]) -> StrategyScore {
        if results.is_empty() {
            return StrategyScore {
                strategy_id: strategy_id.to_string(),
                score: 0.0,
                success_rate: 0.0,
                critical_success_rate: 0.0,
                latency_avg: 0.0,
                latency_jitter: 0.0,
            };
        }

        // Calculate success rates
        let success_rate = self.calculate_success_rate(results);
        let critical_success_rate = self.calculate_critical_success_rate(results);

        // Calculate latency metrics
        let latency_avg = self.calculate_latency_avg(results);
        let latency_jitter = self.calculate_latency_jitter(results, latency_avg);

        // Calculate latency score (lower latency = higher score)
        let latency_score = self.calculate_latency_score(latency_avg);

        // Calculate final weighted score
        let score = (success_rate * self.weights.success_rate)
            + (critical_success_rate * self.weights.critical_success)
            + (latency_score * self.weights.latency);

        debug!(
            "Strategy {} score: {:.3} (sr={:.2}, csr={:.2}, lat={:.0}ms, jit={:.1}ms)",
            strategy_id, score, success_rate, critical_success_rate, latency_avg, latency_jitter
        );

        StrategyScore {
            strategy_id: strategy_id.to_string(),
            score,
            success_rate,
            critical_success_rate,
            latency_avg,
            latency_jitter,
        }
    }

    /// Calculate overall success rate
    fn calculate_success_rate(&self, results: &[ProbeResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }

        let successful = results.iter().filter(|r| r.success).count();
        successful as f64 / results.len() as f64
    }

    /// Calculate success rate for critical endpoints only
    fn calculate_critical_success_rate(&self, results: &[ProbeResult]) -> f64 {
        let critical_results: Vec<_> = results.iter().filter(|r| r.is_critical).collect();

        if critical_results.is_empty() {
            // If no critical endpoints, use overall success rate
            return self.calculate_success_rate(results);
        }

        let successful = critical_results.iter().filter(|r| r.success).count();
        successful as f64 / critical_results.len() as f64
    }

    /// Calculate average latency from successful probes
    fn calculate_latency_avg(&self, results: &[ProbeResult]) -> f64 {
        let latencies: Vec<f64> = results
            .iter()
            .filter(|r| r.success)
            .filter_map(|r| r.latency_ms)
            .filter(|&lat| lat > 0.0)
            .collect();

        if latencies.is_empty() {
            return 0.0;
        }

        latencies.iter().sum::<f64>() / latencies.len() as f64
    }

    /// Calculate latency jitter (standard deviation)
    fn calculate_latency_jitter(&self, results: &[ProbeResult], mean: f64) -> f64 {
        let latencies: Vec<f64> = results
            .iter()
            .filter(|r| r.success)
            .filter_map(|r| r.latency_ms)
            .filter(|&lat| lat > 0.0)
            .collect();

        if latencies.len() < 2 {
            return 0.0;
        }

        let variance =
            latencies.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / latencies.len() as f64;

        variance.sqrt()
    }

    /// Calculate latency score (0.0 - 1.0, higher is better)
    ///
    /// Formula:
    /// - < 100ms = 1.0
    /// - 100-2000ms = linear interpolation from 1.0 to 0.0
    /// - > 2000ms = 0.0
    fn calculate_latency_score(&self, latency_ms: f64) -> f64 {
        const MIN_LATENCY: f64 = 100.0;
        const MAX_LATENCY: f64 = 2000.0;

        if latency_ms <= 0.0 {
            // No latency data, assume neutral
            return 0.5;
        }

        if latency_ms < MIN_LATENCY {
            return 1.0;
        }

        if latency_ms > MAX_LATENCY {
            return 0.0;
        }

        // Linear interpolation
        1.0 - (latency_ms - MIN_LATENCY) / (MAX_LATENCY - MIN_LATENCY)
    }
}

/// Compare two strategy scores
pub fn compare_scores(a: &StrategyScore, b: &StrategyScore) -> std::cmp::Ordering {
    b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
}

/// Sort scores by score descending
pub fn sort_scores(scores: &mut [StrategyScore]) {
    scores.sort_by(|a, b| compare_scores(a, b));
}

/// Get the best score from a list
pub fn get_best_score(scores: &[StrategyScore]) -> Option<&StrategyScore> {
    scores
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
}

/// Filter scores by minimum success rate
pub fn filter_viable(scores: &[StrategyScore], min_success_rate: f64) -> Vec<&StrategyScore> {
    scores
        .iter()
        .filter(|s| s.is_viable(min_success_rate))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_probe_result(
        url: &str,
        success: bool,
        latency_ms: Option<f64>,
        is_critical: bool,
    ) -> ProbeResult {
        ProbeResult {
            url: url.to_string(),
            success,
            latency_ms,
            status_code: if success { Some(200) } else { None },
            error: if success {
                None
            } else {
                Some("Error".to_string())
            },
            is_critical,
        }
    }

    // ==================== ScoreWeights tests ====================

    #[test]
    fn test_weights_default() {
        let weights = ScoreWeights::default();
        assert!((weights.success_rate - 0.4).abs() < 0.001);
        assert!((weights.critical_success - 0.3).abs() < 0.001);
        assert!((weights.latency - 0.3).abs() < 0.001);
        assert!(weights.is_valid());
    }

    #[test]
    fn test_weights_critical_priority() {
        let weights = ScoreWeights::critical_priority();
        assert!((weights.critical_success - 0.6).abs() < 0.001);
        assert!(weights.is_valid());
    }

    #[test]
    fn test_weights_latency_priority() {
        let weights = ScoreWeights::latency_priority();
        assert!((weights.latency - 0.5).abs() < 0.001);
        assert!(weights.is_valid());
    }

    #[test]
    fn test_weights_normalize() {
        let mut weights = ScoreWeights::new(0.5, 0.5, 0.5);
        assert!(!weights.is_valid());

        weights.normalize();
        assert!(weights.is_valid());
        assert!((weights.success_rate - 0.333).abs() < 0.01);
    }

    // ==================== ScoreCalculator tests ====================

    #[test]
    fn test_calculator_empty_results() {
        let calc = ScoreCalculator::with_default_weights();
        let score = calc.calculate("test", &[]);

        assert_eq!(score.strategy_id, "test");
        assert_eq!(score.score, 0.0);
        assert_eq!(score.success_rate, 0.0);
        assert_eq!(score.critical_success_rate, 0.0);
    }

    #[test]
    fn test_calculator_all_success() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", true, Some(100.0), true),
            create_probe_result("https://b.com", true, Some(100.0), true),
            create_probe_result("https://c.com", true, Some(100.0), false),
        ];

        let score = calc.calculate("test", &results);

        assert_eq!(score.success_rate, 1.0);
        assert_eq!(score.critical_success_rate, 1.0);
        assert!(score.score > 0.9); // Should be high
    }

    #[test]
    fn test_calculator_all_failure() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", false, None, true),
            create_probe_result("https://b.com", false, None, true),
        ];

        let score = calc.calculate("test", &results);

        assert_eq!(score.success_rate, 0.0);
        assert_eq!(score.critical_success_rate, 0.0);
        assert!(score.score < 0.3); // Should be low (only latency component with neutral value)
    }

    #[test]
    fn test_calculator_partial_success() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", true, Some(150.0), true),
            create_probe_result("https://b.com", false, None, true),
            create_probe_result("https://c.com", true, Some(200.0), false),
            create_probe_result("https://d.com", false, None, false),
        ];

        let score = calc.calculate("test", &results);

        assert!((score.success_rate - 0.5).abs() < 0.001);
        assert!((score.critical_success_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_calculator_critical_vs_optional() {
        let calc = ScoreCalculator::with_default_weights();

        // All critical fail, all optional succeed
        let results = vec![
            create_probe_result("https://critical.com", false, None, true),
            create_probe_result("https://optional.com", true, Some(100.0), false),
        ];

        let score = calc.calculate("test", &results);

        assert_eq!(score.critical_success_rate, 0.0);
        assert!((score.success_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_calculator_no_critical_endpoints() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", true, Some(100.0), false),
            create_probe_result("https://b.com", true, Some(100.0), false),
        ];

        let score = calc.calculate("test", &results);

        // When no critical endpoints, critical_success_rate should equal success_rate
        assert_eq!(score.critical_success_rate, score.success_rate);
    }

    // ==================== Latency score tests ====================

    #[test]
    fn test_latency_score_fast() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![create_probe_result("https://a.com", true, Some(50.0), true)];

        let score = calc.calculate("test", &results);

        // < 100ms should give latency_score = 1.0
        // Total score = 1.0 * 0.4 + 1.0 * 0.3 + 1.0 * 0.3 = 1.0
        assert!(score.score > 0.95);
    }

    #[test]
    fn test_latency_score_slow() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![create_probe_result("https://a.com", true, Some(2500.0), true)];

        let score = calc.calculate("test", &results);

        // > 2000ms should give latency_score = 0.0
        // Total score = 1.0 * 0.4 + 1.0 * 0.3 + 0.0 * 0.3 = 0.7
        assert!((score.score - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_latency_score_medium() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![create_probe_result("https://a.com", true, Some(1050.0), true)];

        let score = calc.calculate("test", &results);

        // 1050ms is roughly in the middle of 100-2000 range
        // latency_score ≈ 0.5
        assert!(score.latency_avg > 1000.0 && score.latency_avg < 1100.0);
    }

    // ==================== Latency jitter tests ====================

    #[test]
    fn test_jitter_identical_latencies() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", true, Some(100.0), true),
            create_probe_result("https://b.com", true, Some(100.0), true),
            create_probe_result("https://c.com", true, Some(100.0), true),
        ];

        let score = calc.calculate("test", &results);

        assert_eq!(score.latency_jitter, 0.0);
    }

    #[test]
    fn test_jitter_varied_latencies() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", true, Some(100.0), true),
            create_probe_result("https://b.com", true, Some(200.0), true),
            create_probe_result("https://c.com", true, Some(300.0), true),
        ];

        let score = calc.calculate("test", &results);

        assert!(score.latency_jitter > 0.0);
        assert!((score.latency_avg - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_jitter_single_result() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![create_probe_result("https://a.com", true, Some(100.0), true)];

        let score = calc.calculate("test", &results);

        // Single result should have 0 jitter
        assert_eq!(score.latency_jitter, 0.0);
    }

    // ==================== StrategyScore tests ====================

    #[test]
    fn test_strategy_score_is_viable() {
        let score = StrategyScore {
            strategy_id: "test".to_string(),
            score: 0.8,
            success_rate: 0.9,
            critical_success_rate: 1.0,
            latency_avg: 100.0,
            latency_jitter: 10.0,
        };

        assert!(score.is_viable(0.8));
        assert!(score.is_viable(0.9));
        assert!(!score.is_viable(0.95));
    }

    #[test]
    fn test_strategy_score_all_critical_passed() {
        let score_passed = StrategyScore {
            strategy_id: "test".to_string(),
            score: 0.8,
            success_rate: 0.9,
            critical_success_rate: 1.0,
            latency_avg: 100.0,
            latency_jitter: 10.0,
        };

        let score_failed = StrategyScore {
            strategy_id: "test".to_string(),
            score: 0.8,
            success_rate: 0.9,
            critical_success_rate: 0.5,
            latency_avg: 100.0,
            latency_jitter: 10.0,
        };

        assert!(score_passed.all_critical_passed());
        assert!(!score_failed.all_critical_passed());
    }

    // ==================== Utility function tests ====================

    #[test]
    fn test_compare_scores() {
        let score_a = StrategyScore {
            strategy_id: "a".to_string(),
            score: 0.8,
            success_rate: 0.9,
            critical_success_rate: 1.0,
            latency_avg: 100.0,
            latency_jitter: 10.0,
        };

        let score_b = StrategyScore {
            strategy_id: "b".to_string(),
            score: 0.9,
            success_rate: 0.95,
            critical_success_rate: 1.0,
            latency_avg: 80.0,
            latency_jitter: 5.0,
        };

        // b should come before a (higher score)
        assert_eq!(
            compare_scores(&score_a, &score_b),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_scores(&score_b, &score_a),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_sort_scores() {
        let mut scores = vec![
            StrategyScore {
                strategy_id: "low".to_string(),
                score: 0.3,
                success_rate: 0.5,
                critical_success_rate: 0.5,
                latency_avg: 500.0,
                latency_jitter: 100.0,
            },
            StrategyScore {
                strategy_id: "high".to_string(),
                score: 0.9,
                success_rate: 1.0,
                critical_success_rate: 1.0,
                latency_avg: 50.0,
                latency_jitter: 5.0,
            },
            StrategyScore {
                strategy_id: "mid".to_string(),
                score: 0.6,
                success_rate: 0.8,
                critical_success_rate: 0.8,
                latency_avg: 200.0,
                latency_jitter: 30.0,
            },
        ];

        sort_scores(&mut scores);

        assert_eq!(scores[0].strategy_id, "high");
        assert_eq!(scores[1].strategy_id, "mid");
        assert_eq!(scores[2].strategy_id, "low");
    }

    #[test]
    fn test_get_best_score() {
        let scores = vec![
            StrategyScore {
                strategy_id: "a".to_string(),
                score: 0.5,
                success_rate: 0.7,
                critical_success_rate: 0.7,
                latency_avg: 200.0,
                latency_jitter: 20.0,
            },
            StrategyScore {
                strategy_id: "b".to_string(),
                score: 0.9,
                success_rate: 1.0,
                critical_success_rate: 1.0,
                latency_avg: 50.0,
                latency_jitter: 5.0,
            },
        ];

        let best = get_best_score(&scores);
        assert!(best.is_some());
        assert_eq!(best.unwrap().strategy_id, "b");
    }

    #[test]
    fn test_get_best_score_empty() {
        let scores: Vec<StrategyScore> = vec![];
        assert!(get_best_score(&scores).is_none());
    }

    #[test]
    fn test_filter_viable() {
        let scores = vec![
            StrategyScore {
                strategy_id: "viable".to_string(),
                score: 0.9,
                success_rate: 0.95,
                critical_success_rate: 1.0,
                latency_avg: 50.0,
                latency_jitter: 5.0,
            },
            StrategyScore {
                strategy_id: "not_viable".to_string(),
                score: 0.3,
                success_rate: 0.5,
                critical_success_rate: 0.5,
                latency_avg: 500.0,
                latency_jitter: 100.0,
            },
        ];

        let viable = filter_viable(&scores, 0.8);
        assert_eq!(viable.len(), 1);
        assert_eq!(viable[0].strategy_id, "viable");
    }

    #[test]
    fn test_score_serialization() {
        let score = StrategyScore {
            strategy_id: "test".to_string(),
            score: 0.85,
            success_rate: 0.9,
            critical_success_rate: 1.0,
            latency_avg: 150.5,
            latency_jitter: 25.3,
        };

        let json = serde_json::to_string(&score).unwrap();
        let deserialized: StrategyScore = serde_json::from_str(&json).unwrap();

        assert_eq!(score.strategy_id, deserialized.strategy_id);
        assert!((score.score - deserialized.score).abs() < 0.001);
        assert!((score.latency_avg - deserialized.latency_avg).abs() < 0.001);
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_zero_latency() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![create_probe_result("https://a.com", true, Some(0.0), true)];

        let score = calc.calculate("test", &results);

        // Zero latency should be treated as no data
        assert_eq!(score.latency_avg, 0.0);
    }

    #[test]
    fn test_negative_latency_ignored() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            ProbeResult {
                url: "https://a.com".to_string(),
                success: true,
                latency_ms: Some(-100.0), // Invalid
                status_code: Some(200),
                error: None,
                is_critical: true,
            },
            create_probe_result("https://b.com", true, Some(200.0), true),
        ];

        let score = calc.calculate("test", &results);

        // Negative latency should be filtered out
        assert!((score.latency_avg - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_mixed_success_failure_latency() {
        let calc = ScoreCalculator::with_default_weights();
        let results = vec![
            create_probe_result("https://a.com", true, Some(100.0), true),
            create_probe_result("https://b.com", false, None, true), // Failed, no latency
            create_probe_result("https://c.com", true, Some(200.0), false),
        ];

        let score = calc.calculate("test", &results);

        // Latency should only consider successful probes
        assert!((score.latency_avg - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_custom_weights() {
        let weights = ScoreWeights::new(0.5, 0.5, 0.0);
        let calc = ScoreCalculator::new(weights);

        let results = vec![
            create_probe_result("https://a.com", true, Some(5000.0), true), // Very slow
        ];

        let score = calc.calculate("test", &results);

        // With 0 latency weight, slow latency shouldn't affect score
        // score = 1.0 * 0.5 + 1.0 * 0.5 + 0 = 1.0
        assert!((score.score - 1.0).abs() < 0.001);
    }
}

//! Retry utilities for Isolate
//!
//! Provides exponential backoff retry logic for network operations
//! and other fallible operations that may succeed on retry.

#![allow(dead_code)] // Public retry API

use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles delay each retry)
    pub backoff_multiplier: f64,
    /// Optional jitter factor (0.0 - 1.0) to add randomness to delays
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with specified max retries
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Create config for network operations (longer delays, more retries)
    pub fn network() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
        }
    }

    /// Create config for quick retries (short delays, few retries)
    pub fn quick() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }

    /// Create config for aggressive retries (many attempts)
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            jitter_factor: 0.3,
        }
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set max delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set jitter factor
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter_factor = jitter.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for a given attempt number (0-indexed)
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        
        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);
        
        // Add jitter
        let jitter = if self.jitter_factor > 0.0 {
            let jitter_range = capped_delay * self.jitter_factor;
            // Simple pseudo-random jitter based on attempt number
            let jitter_value = ((attempt as f64 * 7.0 + 13.0) % 100.0) / 100.0;
            jitter_range * jitter_value - jitter_range / 2.0
        } else {
            0.0
        };
        
        Duration::from_millis((capped_delay + jitter).max(0.0) as u64)
    }
}

/// Result of a retry operation
#[derive(Debug)]
pub struct RetryResult<T, E> {
    /// The final result (success or last error)
    pub result: Result<T, E>,
    /// Number of attempts made (1 = succeeded on first try)
    pub attempts: u32,
    /// Total time spent retrying
    pub total_duration: Duration,
}

impl<T, E> RetryResult<T, E> {
    /// Check if the operation succeeded
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the operation failed after all retries
    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }

    /// Unwrap the result, panicking if it's an error
    pub fn unwrap(self) -> T
    where
        E: std::fmt::Debug,
    {
        self.result.unwrap()
    }

    /// Get the result, converting error if needed
    pub fn into_result(self) -> Result<T, E> {
        self.result
    }
}

/// Execute an async operation with retry logic
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation_name` - Name for logging purposes
/// * `operation` - Async closure that returns Result<T, E>
///
/// # Example
/// ```ignore
/// use crate::core::retry::{retry_async, RetryConfig};
///
/// let result = retry_async(
///     RetryConfig::network(),
///     "fetch_data",
///     || async { fetch_from_server().await }
/// ).await;
/// ```
pub async fn retry_async<T, E, F, Fut>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> RetryResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let start = std::time::Instant::now();
    let mut last_error: Option<E> = None;
    let mut attempts = 0;

    for attempt in 0..=config.max_retries {
        attempts = attempt + 1;

        match operation().await {
            Ok(value) => {
                if attempt > 0 {
                    debug!(
                        operation = %operation_name,
                        attempt = attempts,
                        "Operation succeeded after retry"
                    );
                }
                return RetryResult {
                    result: Ok(value),
                    attempts,
                    total_duration: start.elapsed(),
                };
            }
            Err(e) => {
                if attempt < config.max_retries {
                    let delay = config.calculate_delay(attempt);
                    warn!(
                        operation = %operation_name,
                        attempt = attempts,
                        max_attempts = config.max_retries + 1,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "Operation failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    warn!(
                        operation = %operation_name,
                        attempts = attempts,
                        error = %e,
                        "Operation failed after all retries"
                    );
                }
                last_error = Some(e);
            }
        }
    }

    RetryResult {
        result: Err(last_error.expect("Should have at least one error")),
        attempts,
        total_duration: start.elapsed(),
    }
}

/// Execute an async operation with retry, returning just the Result
///
/// Convenience wrapper around `retry_async` that discards retry metadata.
pub async fn with_retry<T, E, F, Fut>(
    config: RetryConfig,
    operation_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    retry_async(config, operation_name, operation).await.result
}

/// Execute an async operation with default network retry config
pub async fn with_network_retry<T, E, F, Fut>(
    operation_name: &str,
    operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    with_retry(RetryConfig::network(), operation_name, operation).await
}

/// Trait for adding retry capability to Result-returning futures
pub trait RetryExt<T, E>: Sized {
    /// Retry this operation with the given config
    fn with_retry(
        self,
        config: RetryConfig,
        operation_name: &str,
    ) -> impl Future<Output = Result<T, E>>;
}

/// Check if an error is retryable (network-related)
pub fn is_retryable_error(error: &str) -> bool {
    let retryable_patterns = [
        "timeout",
        "timed out",
        "connection refused",
        "connection reset",
        "temporarily unavailable",
        "try again",
        "network unreachable",
        "host unreachable",
        "no route to host",
        "broken pipe",
        "connection aborted",
        "would block",
        "resource temporarily unavailable",
    ];

    let error_lower = error.to_lowercase();
    retryable_patterns.iter().any(|p| error_lower.contains(p))
}

// Note: is_retryable_reqwest_error function was removed as reqwest feature is not used in this project

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_network() {
        let config = RetryConfig::network();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable test
        };

        assert_eq!(config.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(800));
    }

    #[test]
    fn test_calculate_delay_with_cap() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };

        // Should be capped at max_delay
        assert_eq!(config.calculate_delay(5), Duration::from_secs(5));
        assert_eq!(config.calculate_delay(10), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_retry_success_first_try() {
        let result = retry_async(
            RetryConfig::quick(),
            "test_op",
            || async { Ok::<_, String>(42) },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.attempts, 1);
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_async(
            RetryConfig::quick().with_initial_delay(Duration::from_millis(10)),
            "test_op",
            || {
                let counter = counter_clone.clone();
                async move {
                    let attempt = counter.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err("temporary error".to_string())
                    } else {
                        Ok(42)
                    }
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.attempts, 3);
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_all_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_async(
            RetryConfig::new(2).with_initial_delay(Duration::from_millis(10)),
            "test_op",
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("permanent error".to_string())
                }
            },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.attempts, 3); // 1 initial + 2 retries
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error("Connection timeout"));
        assert!(is_retryable_error("connection refused"));
        assert!(is_retryable_error("Network unreachable"));
        assert!(is_retryable_error("Resource temporarily unavailable"));
        
        assert!(!is_retryable_error("Invalid input"));
        assert!(!is_retryable_error("Permission denied"));
        assert!(!is_retryable_error("File not found"));
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::default()
            .with_max_retries(5)
            .with_initial_delay(Duration::from_millis(200))
            .with_max_delay(Duration::from_secs(30))
            .with_backoff_multiplier(1.5)
            .with_jitter(0.2);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 1.5);
        assert_eq!(config.jitter_factor, 0.2);
    }
}

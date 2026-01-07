//! Rate limiter for heavy Tauri commands
//!
//! Prevents abuse of resource-intensive operations like optimization,
//! testing, and speed tests by enforcing cooldown periods.
//!
//! Uses a sliding window algorithm with per-command rate limits.

use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::core::errors::IsolateError;

// ============================================================================
// Rate Limiter Configuration
// ============================================================================

/// Rate limit configuration for a specific command
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed
    pub max_requests: usize,
    /// Time window in seconds
    pub window_secs: u64,
}

impl RateLimitConfig {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
        }
    }
}

/// Predefined rate limits for critical commands
pub mod limits {
    use super::RateLimitConfig;

    /// test_proxy: 10 requests per minute
    pub const TEST_PROXY: RateLimitConfig = RateLimitConfig {
        max_requests: 10,
        window_secs: 60,
    };

    /// import_subscription: 5 requests per minute (SSRF protection)
    pub const IMPORT_SUBSCRIPTION: RateLimitConfig = RateLimitConfig {
        max_requests: 5,
        window_secs: 60,
    };

    /// check_all_registry_services: 2 requests per minute (heavy operation)
    pub const CHECK_ALL_SERVICES: RateLimitConfig = RateLimitConfig {
        max_requests: 2,
        window_secs: 60,
    };

    /// download_config_updates: 3 requests per minute (network operation)
    pub const DOWNLOAD_CONFIG_UPDATES: RateLimitConfig = RateLimitConfig {
        max_requests: 3,
        window_secs: 60,
    };
}

// ============================================================================
// Rate Limiter Implementation
// ============================================================================

/// Sliding window rate limiter for a single command
#[derive(Debug)]
struct CommandRateLimiter {
    /// Maximum requests allowed in the window
    max_requests: usize,
    /// Window duration
    window: Duration,
    /// Timestamps of recent requests (sliding window)
    requests: VecDeque<Instant>,
}

impl CommandRateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            max_requests: config.max_requests,
            window: Duration::from_secs(config.window_secs),
            requests: VecDeque::new(),
        }
    }

    /// Check if a new request is allowed
    fn check_rate_limit(&mut self) -> Result<(), IsolateError> {
        let now = Instant::now();

        // Remove expired requests (outside the window)
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we're at the limit
        if self.requests.len() >= self.max_requests {
            let oldest = self.requests.front().unwrap();
            let time_until_available = self.window
                .checked_sub(now.duration_since(*oldest))
                .unwrap_or(Duration::ZERO);
            let remaining_secs = time_until_available.as_secs()
                + if time_until_available.subsec_millis() > 0 {
                    1
                } else {
                    0
                };

            return Err(IsolateError::Validation(format!(
                "Rate limit exceeded: {} requests per {} seconds. Please wait {} seconds.",
                self.max_requests,
                self.window.as_secs(),
                remaining_secs
            )));
        }

        // Record this request
        self.requests.push_back(now);
        Ok(())
    }

    /// Reset the rate limiter (for testing)
    #[allow(dead_code)]
    fn reset(&mut self) {
        self.requests.clear();
    }
}

/// Global rate limiter state with per-command limiters
static RATE_LIMITERS: Lazy<Mutex<HashMap<String, CommandRateLimiter>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// ============================================================================
// Public API
// ============================================================================

/// Check if an operation is allowed based on rate limiting (legacy API)
///
/// # Arguments
/// * `operation` - Unique identifier for the operation (e.g., "run_optimization")
/// * `cooldown_secs` - Minimum seconds between calls
///
/// # Returns
/// * `Ok(())` - Operation is allowed
/// * `Err(IsolateError)` - Rate limited, contains message with remaining wait time
///
/// # Example
/// ```rust
/// use crate::commands::rate_limiter::check_rate_limit;
///
/// // Allow run_optimization only once per 10 seconds
/// check_rate_limit("run_optimization", 10)?;
/// ```
pub fn check_rate_limit(operation: &str, cooldown_secs: u64) -> Result<(), IsolateError> {
    check_rate_limit_with_config(
        operation,
        RateLimitConfig::new(1, cooldown_secs),
    )
}

/// Check rate limit with custom configuration
///
/// # Arguments
/// * `operation` - Unique identifier for the operation
/// * `config` - Rate limit configuration (max requests, window)
///
/// # Returns
/// * `Ok(())` - Operation is allowed
/// * `Err(IsolateError)` - Rate limited
///
/// # Example
/// ```rust
/// use crate::commands::rate_limiter::{check_rate_limit_with_config, RateLimitConfig};
///
/// // Allow 10 requests per minute
/// check_rate_limit_with_config("test_proxy", RateLimitConfig::new(10, 60))?;
/// ```
pub fn check_rate_limit_with_config(
    operation: &str,
    config: RateLimitConfig,
) -> Result<(), IsolateError> {
    let mut limiters = RATE_LIMITERS
        .lock()
        .map_err(|_| IsolateError::Other("Rate limiter lock poisoned".to_string()))?;

    let limiter = limiters
        .entry(operation.to_string())
        .or_insert_with(|| CommandRateLimiter::new(config.clone()));

    limiter.check_rate_limit()
}

/// Macro for easy rate limiting in commands
///
/// # Example
/// ```rust
/// use crate::rate_limit;
/// use crate::commands::rate_limiter::limits;
///
/// #[tauri::command]
/// pub async fn test_proxy(id: String) -> Result<(), IsolateError> {
///     rate_limit!("test_proxy", limits::TEST_PROXY)?;
///     // ... rest of the command
/// }
/// ```
#[macro_export]
macro_rules! rate_limit {
    ($operation:expr, $config:expr) => {
        $crate::commands::rate_limiter::check_rate_limit_with_config($operation, $config)
    };
}

/// Reset rate limit for a specific operation (useful for testing)
#[allow(dead_code)]
pub fn reset_rate_limit(operation: &str) {
    if let Ok(mut limiters) = RATE_LIMITERS.lock() {
        if let Some(limiter) = limiters.get_mut(operation) {
            limiter.reset();
        }
    }
}

/// Clear all rate limits (useful for testing)
#[allow(dead_code)]
pub fn clear_all_rate_limits() {
    if let Ok(mut limiters) = RATE_LIMITERS.lock() {
        limiters.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_first_call_allowed() {
        clear_all_rate_limits();
        let config = RateLimitConfig::new(5, 10);
        let result = check_rate_limit_with_config("test_op_1", config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_calls_within_limit() {
        clear_all_rate_limits();
        let config = RateLimitConfig::new(3, 10);

        // First 3 calls should succeed
        assert!(check_rate_limit_with_config("test_op_2", config.clone()).is_ok());
        assert!(check_rate_limit_with_config("test_op_2", config.clone()).is_ok());
        assert!(check_rate_limit_with_config("test_op_2", config.clone()).is_ok());

        // 4th call should be blocked
        let result = check_rate_limit_with_config("test_op_2", config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_sliding_window() {
        clear_all_rate_limits();
        let config = RateLimitConfig::new(2, 1); // 2 requests per second

        // First 2 calls succeed
        assert!(check_rate_limit_with_config("test_op_3", config.clone()).is_ok());
        assert!(check_rate_limit_with_config("test_op_3", config.clone()).is_ok());

        // 3rd call blocked
        assert!(check_rate_limit_with_config("test_op_3", config.clone()).is_err());

        // Wait for window to slide
        sleep(Duration::from_millis(1100));

        // Should be allowed now (old requests expired)
        assert!(check_rate_limit_with_config("test_op_3", config).is_ok());
    }

    #[test]
    fn test_different_operations_independent() {
        clear_all_rate_limits();
        let config = RateLimitConfig::new(1, 10);

        // First operation
        assert!(check_rate_limit_with_config("test_op_4a", config.clone()).is_ok());
        assert!(check_rate_limit_with_config("test_op_4a", config.clone()).is_err());

        // Different operation should not be blocked
        assert!(check_rate_limit_with_config("test_op_4b", config.clone()).is_ok());
        assert!(check_rate_limit_with_config("test_op_4b", config).is_err());
    }

    #[test]
    fn test_reset_rate_limit() {
        clear_all_rate_limits();
        let config = RateLimitConfig::new(1, 60);

        // First call succeeds
        assert!(check_rate_limit_with_config("test_op_5", config.clone()).is_ok());

        // Second call blocked
        assert!(check_rate_limit_with_config("test_op_5", config.clone()).is_err());

        // Reset
        reset_rate_limit("test_op_5");

        // Should be allowed now
        assert!(check_rate_limit_with_config("test_op_5", config).is_ok());
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        clear_all_rate_limits();
        let config = RateLimitConfig::new(10, 1);

        let handles: Vec<_> = (0..20)
            .map(|i| {
                let config = config.clone();
                thread::spawn(move || {
                    check_rate_limit_with_config("test_op_6", config)
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Exactly 10 should succeed, 10 should fail
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.iter().filter(|r| r.is_err()).count();

        assert_eq!(success_count, 10);
        assert_eq!(failure_count, 10);
    }

    #[test]
    fn test_predefined_limits() {
        // Test that predefined limits are reasonable
        assert_eq!(limits::TEST_PROXY.max_requests, 10);
        assert_eq!(limits::TEST_PROXY.window_secs, 60);

        assert_eq!(limits::IMPORT_SUBSCRIPTION.max_requests, 5);
        assert_eq!(limits::IMPORT_SUBSCRIPTION.window_secs, 60);

        assert_eq!(limits::CHECK_ALL_SERVICES.max_requests, 2);
        assert_eq!(limits::CHECK_ALL_SERVICES.window_secs, 60);

        assert_eq!(limits::DOWNLOAD_CONFIG_UPDATES.max_requests, 3);
        assert_eq!(limits::DOWNLOAD_CONFIG_UPDATES.window_secs, 60);
    }

    #[test]
    fn test_legacy_api_compatibility() {
        clear_all_rate_limits();

        // Legacy API should still work
        assert!(check_rate_limit("test_op_7", 5).is_ok());
        assert!(check_rate_limit("test_op_7", 5).is_err());

        sleep(Duration::from_millis(5100));
        assert!(check_rate_limit("test_op_7", 5).is_ok());
    }

    #[test]
    fn test_error_message_format() {
        clear_all_rate_limits();
        let config = RateLimitConfig::new(1, 10);

        // First call succeeds
        assert!(check_rate_limit_with_config("test_op_8", config.clone()).is_ok());

        // Second call should have informative error
        let result = check_rate_limit_with_config("test_op_8", config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Rate limit exceeded"));
        assert!(error_msg.contains("1 requests per 10 seconds"));
    }
}

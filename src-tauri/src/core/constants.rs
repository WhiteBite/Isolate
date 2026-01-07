//! Centralized constants for Isolate
//!
//! This module contains all magic numbers and configuration constants
//! used throughout the application.
//!
//! Note: Many constants are defined for future use or external consumers.

#![allow(dead_code)] // Public API constants for library consumers

// ============================================================================
// Timeouts
// ============================================================================

/// HTTP request timeout in seconds
pub const HTTP_TIMEOUT_SECS: u64 = 10;

/// HTTP connection timeout in seconds
pub const HTTP_CONNECT_TIMEOUT_SECS: u64 = 5;

/// Strategy test timeout in milliseconds
pub const STRATEGY_TEST_TIMEOUT_MS: u64 = 10000;

/// Graceful shutdown timeout in milliseconds
pub const SHUTDOWN_TIMEOUT_MS: u64 = 3000;

// ============================================================================
// Ports
// ============================================================================

/// Starting port for SOCKS proxies
pub const SOCKS_PORT_START: u16 = 10800;

/// Maximum number of SOCKS ports to allocate
pub const MAX_SOCKS_PORTS: u16 = 100;

// ============================================================================
// Delays
// ============================================================================

/// Delay after launching Zapret/winws before testing (milliseconds)
pub const ZAPRET_LAUNCH_DELAY_MS: u64 = 2500;

/// Delay for Zapret test operations (milliseconds)
pub const ZAPRET_TEST_DELAY_MS: u64 = 2500;

// ============================================================================
// Scoring
// ============================================================================

/// Minimum acceptable score for a strategy to be considered working
pub const MIN_ACCEPTABLE_SCORE: f64 = 0.6;

// ============================================================================
// Limits
// ============================================================================

/// Maximum number of parallel VLESS strategy tests
pub const MAX_PARALLEL_VLESS_TESTS: usize = 5;

/// Maximum number of retry attempts for operations
pub const MAX_RETRY_ATTEMPTS: u32 = 3;

// ============================================================================
// Mock/Testing
// ============================================================================

/// Minimum mock delay in milliseconds (for testing)
pub const MOCK_DELAY_MIN_MS: u64 = 500;

/// Maximum mock delay in milliseconds (for testing)
pub const MOCK_DELAY_MAX_MS: u64 = 1500;

/// Mock success rate for testing (0.0 - 1.0)
pub const MOCK_SUCCESS_RATE: f64 = 0.8;

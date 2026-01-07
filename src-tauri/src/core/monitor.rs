//! Strategy health monitoring module
//!
//! Monitors active strategy health and detects degradation.
//! Emits events to frontend for status updates.

#![allow(dead_code)] // Public monitoring API

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::core::strategy_engine::StrategyEngine;

// ============================================================================
// Constants
// ============================================================================

/// Default interval between health checks (30 seconds)
const DEFAULT_CHECK_INTERVAL_SECS: u64 = 30;

/// Number of consecutive failures before declaring degradation
const FAILURE_THRESHOLD: u32 = 3;

/// HTTP request timeout for connectivity checks
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Threshold for success rate below which check is considered failed
const SUCCESS_RATE_THRESHOLD: f64 = 0.5;

// ============================================================================
// Event Payloads
// ============================================================================

/// Health check result payload for frontend
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub strategy_id: String,
    pub is_healthy: bool,
    pub success_rate: f64,
    pub consecutive_failures: u32,
    pub timestamp: String,
}

/// Degradation event payload
#[derive(Debug, Clone, Serialize)]
pub struct DegradationEvent {
    pub strategy_id: String,
    pub consecutive_failures: u32,
    pub last_success_rate: f64,
    pub timestamp: String,
}

/// Recovery event payload
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryEvent {
    pub strategy_id: String,
    pub timestamp: String,
}

// ============================================================================
// Monitor State
// ============================================================================

/// Internal state for tracking health
struct MonitorState {
    consecutive_failures: AtomicU32,
    is_degraded: AtomicBool,
    last_success_rate: RwLock<f64>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            consecutive_failures: AtomicU32::new(0),
            is_degraded: AtomicBool::new(false),
            last_success_rate: RwLock::new(1.0),
        }
    }
}

// ============================================================================
// Monitor
// ============================================================================

/// Strategy health monitor
///
/// Periodically checks if the active strategy is working correctly
/// by testing connectivity to configured URLs.
pub struct Monitor {
    /// Check interval
    interval: Duration,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Cancellation token for graceful shutdown
    cancel_token: CancellationToken,
    /// Reference to strategy engine
    strategy_engine: Arc<StrategyEngine>,
    /// Test URLs for health checks
    test_urls: RwLock<Vec<String>>,
    /// Internal state
    state: MonitorState,
    /// Auto-restart on degradation
    auto_restart: AtomicBool,
}

impl Monitor {
    /// Creates a new Monitor instance
    ///
    /// # Arguments
    /// * `strategy_engine` - Reference to the strategy engine for checking active strategy
    pub fn new(strategy_engine: Arc<StrategyEngine>) -> Self {
        Self {
            interval: Duration::from_secs(DEFAULT_CHECK_INTERVAL_SECS),
            running: Arc::new(AtomicBool::new(false)),
            cancel_token: CancellationToken::new(),
            strategy_engine,
            test_urls: RwLock::new(vec![
                "https://www.google.com".to_string(),
                "https://www.youtube.com".to_string(),
                "https://discord.com".to_string(),
            ]),
            state: MonitorState::default(),
            auto_restart: AtomicBool::new(false),
        }
    }

    /// Creates a new Monitor with custom interval
    pub fn with_interval(strategy_engine: Arc<StrategyEngine>, interval_secs: u64) -> Self {
        let mut monitor = Self::new(strategy_engine);
        monitor.interval = Duration::from_secs(interval_secs);
        monitor
    }

    /// Set test URLs for health checks
    pub async fn set_test_urls(&self, urls: Vec<String>) {
        let mut test_urls = self.test_urls.write().await;
        *test_urls = urls;
        debug!(count = test_urls.len(), "Updated monitor test URLs");
    }

    /// Enable or disable auto-restart on degradation
    pub fn set_auto_restart(&self, enabled: bool) {
        self.auto_restart.store(enabled, Ordering::SeqCst);
        info!(enabled, "Auto-restart on degradation");
    }

    /// Start background monitoring
    ///
    /// Spawns a background task that periodically checks strategy health
    /// and emits events to the frontend.
    pub async fn start<R: Runtime>(&self, app_handle: AppHandle<R>) -> crate::core::errors::Result<()> {
        if self.running.load(Ordering::SeqCst) {
            warn!("Monitor is already running");
            return Ok(());
        }

        self.running.store(true, Ordering::SeqCst);
        
        // Reset state
        self.state.consecutive_failures.store(0, Ordering::SeqCst);
        self.state.is_degraded.store(false, Ordering::SeqCst);
        {
            let mut rate = self.state.last_success_rate.write().await;
            *rate = 1.0;
        }

        let running = self.running.clone();
        let cancel_token = self.cancel_token.clone();
        let interval = self.interval;
        let strategy_engine = self.strategy_engine.clone();
        let test_urls = self.test_urls.read().await.clone();
        let state_failures = Arc::new(AtomicU32::new(0));
        let state_degraded = Arc::new(AtomicBool::new(false));
        let auto_restart = self.auto_restart.load(Ordering::SeqCst);

        info!(
            interval_secs = interval.as_secs(),
            test_urls_count = test_urls.len(),
            auto_restart,
            "Starting strategy monitor"
        );

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Strategy monitor stopped by cancellation");
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        // Check if there's an active strategy
                        let strategy_id = match strategy_engine.get_global_strategy().await {
                            Some(id) => id,
                            None => {
                                debug!("No active strategy, skipping health check");
                                continue;
                            }
                        };

                        // Perform health check
                        let success_rate = Self::check_connectivity(&test_urls).await;
                        let is_healthy = success_rate >= SUCCESS_RATE_THRESHOLD;

                        let consecutive_failures = if is_healthy {
                            state_failures.store(0, Ordering::SeqCst);
                            0
                        } else {
                            state_failures.fetch_add(1, Ordering::SeqCst) + 1
                        };

                        let was_degraded = state_degraded.load(Ordering::SeqCst);

                        // Emit health check result
                        let health_result = HealthCheckResult {
                            strategy_id: strategy_id.clone(),
                            is_healthy,
                            success_rate,
                            consecutive_failures,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };

                        // Emit both events for compatibility
                        if let Err(e) = app_handle.emit("monitor:health_check", &health_result) {
                            error!("Failed to emit health_check event: {}", e);
                        }
                        // Also emit strategy:health for frontend convenience
                        if let Err(e) = app_handle.emit("strategy:health", &health_result) {
                            error!("Failed to emit strategy:health event: {}", e);
                        }

                        debug!(
                            strategy_id = %strategy_id,
                            success_rate,
                            is_healthy,
                            consecutive_failures,
                            "Health check completed"
                        );

                        // Check for degradation
                        if consecutive_failures >= FAILURE_THRESHOLD && !was_degraded {
                            state_degraded.store(true, Ordering::SeqCst);

                            warn!(
                                strategy_id = %strategy_id,
                                consecutive_failures,
                                success_rate,
                                "Strategy degradation detected"
                            );

                            let degradation_event = DegradationEvent {
                                strategy_id: strategy_id.clone(),
                                consecutive_failures,
                                last_success_rate: success_rate,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };

                            if let Err(e) = app_handle.emit("strategy:degraded", &degradation_event) {
                                error!("Failed to emit strategy:degraded event: {}", e);
                            }

                            // Auto-restart if enabled
                            if auto_restart {
                                info!(strategy_id = %strategy_id, "Attempting auto-restart");
                                // Note: Auto-restart would require access to strategy config
                                // For now, just emit an event that frontend can handle
                                if let Err(e) = app_handle.emit("strategy:restart_requested", &strategy_id) {
                                    error!("Failed to emit restart_requested event: {}", e);
                                }
                            }
                        }

                        // Check for recovery
                        if is_healthy && was_degraded {
                            state_degraded.store(false, Ordering::SeqCst);

                            info!(
                                strategy_id = %strategy_id,
                                "Strategy recovered"
                            );

                            let recovery_event = RecoveryEvent {
                                strategy_id: strategy_id.clone(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };

                            if let Err(e) = app_handle.emit("strategy:recovered", &recovery_event) {
                                error!("Failed to emit strategy:recovered event: {}", e);
                            }
                        }
                    }
                }
            }

            running.store(false, Ordering::SeqCst);
            info!("Strategy monitor loop exited");
        });

        Ok(())
    }

    /// Stop monitoring
    pub fn stop(&self) {
        if !self.running.load(Ordering::SeqCst) {
            debug!("Monitor is not running");
            return;
        }

        info!("Stopping strategy monitor");
        self.cancel_token.cancel();
    }

    /// Check if monitor is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Check if strategy is currently degraded
    pub fn is_degraded(&self) -> bool {
        self.state.is_degraded.load(Ordering::SeqCst)
    }

    /// Get consecutive failure count
    pub fn get_consecutive_failures(&self) -> u32 {
        self.state.consecutive_failures.load(Ordering::SeqCst)
    }

    /// Perform a manual health check
    ///
    /// Returns true if the strategy is healthy
    pub async fn check_strategy_health(&self) -> crate::core::errors::Result<bool> {
        // Check if there's an active strategy
        let strategy_id = self.strategy_engine.get_global_strategy().await;
        if strategy_id.is_none() {
            debug!("No active strategy for health check");
            return Ok(true); // No strategy = nothing to check
        }

        let test_urls = self.test_urls.read().await;
        let success_rate = Self::check_connectivity(&test_urls).await;
        let is_healthy = success_rate >= SUCCESS_RATE_THRESHOLD;

        debug!(
            strategy_id = ?strategy_id,
            success_rate,
            is_healthy,
            "Manual health check completed"
        );

        Ok(is_healthy)
    }

    /// Check connectivity to the given URLs
    ///
    /// Performs HTTP HEAD requests to all URLs concurrently and returns
    /// the ratio of successful requests.
    pub async fn check_connectivity(urls: &[String]) -> f64 {
        if urls.is_empty() {
            return 1.0;
        }

        let client = match reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .danger_accept_invalid_certs(false)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create HTTP client: {}", e);
                return 0.0;
            }
        };

        let checks: Vec<_> = urls
            .iter()
            .map(|url| {
                let client = client.clone();
                let url = url.clone();
                async move {
                    match client.head(&url).send().await {
                        Ok(response) => {
                            let success = response.status().is_success()
                                || response.status().is_redirection();
                            debug!(
                                url = %url,
                                status = %response.status(),
                                success,
                                "Connectivity check"
                            );
                            success
                        }
                        Err(e) => {
                            debug!(url = %url, error = %e, "Connectivity check failed");
                            false
                        }
                    }
                }
            })
            .collect();

        let results = futures::future::join_all(checks).await;
        let successful = results.iter().filter(|&&r| r).count();

        successful as f64 / urls.len() as f64
    }
}

impl Default for Monitor {
    fn default() -> Self {
        // Create a dummy strategy engine for default
        // In practice, this should always be created with new()
        Self::new(Arc::new(StrategyEngine::new()))
    }
}

// ============================================================================
// Legacy StrategyMonitor (for backward compatibility)
// ============================================================================

/// Legacy strategy monitor (kept for backward compatibility)
///
/// Use `Monitor` for new code.
pub struct StrategyMonitor {
    interval_secs: u64,
    cancel_token: CancellationToken,
    is_running: Arc<tokio::sync::Mutex<bool>>,
}

impl StrategyMonitor {
    /// Creates a new strategy monitor
    pub fn new(interval_secs: u64) -> Self {
        Self {
            interval_secs,
            cancel_token: CancellationToken::new(),
            is_running: Arc::new(tokio::sync::Mutex::new(false)),
        }
    }

    /// Starts background monitoring (legacy API)
    pub async fn start<F>(&self, test_urls: Vec<String>, on_degraded: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            warn!("Monitor is already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        let cancel_token = self.cancel_token.clone();
        let interval = Duration::from_secs(self.interval_secs);
        let is_running = self.is_running.clone();
        let on_degraded = Arc::new(on_degraded);

        info!(
            interval_secs = self.interval_secs,
            test_urls_count = test_urls.len(),
            "Starting legacy strategy monitor"
        );

        tokio::spawn(async move {
            let mut consecutive_failures = 0u32;

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Legacy strategy monitor stopped");
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        let success_rate = Monitor::check_connectivity(&test_urls).await;
                        debug!(success_rate, "Legacy connectivity check");

                        if success_rate < SUCCESS_RATE_THRESHOLD {
                            consecutive_failures += 1;
                            if consecutive_failures >= FAILURE_THRESHOLD {
                                warn!(
                                    consecutive_failures,
                                    success_rate,
                                    "Legacy monitor: degradation detected"
                                );
                                on_degraded();
                                consecutive_failures = 0; // Reset after callback
                            }
                        } else {
                            consecutive_failures = 0;
                        }
                    }
                }
            }

            let mut is_running = is_running.lock().await;
            *is_running = false;
        });
    }

    /// Stops the monitoring
    pub fn stop(&self) {
        info!("Stopping legacy strategy monitor");
        self.cancel_token.cancel();
    }

    /// Returns whether the monitor is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}

impl Default for StrategyMonitor {
    fn default() -> Self {
        Self::new(DEFAULT_CHECK_INTERVAL_SECS)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_connectivity_empty_urls() {
        let rate = Monitor::check_connectivity(&[]).await;
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_monitor_creation() {
        let engine = Arc::new(StrategyEngine::new());
        let monitor = Monitor::new(engine);
        assert!(!monitor.is_running());
        assert!(!monitor.is_degraded());
        assert_eq!(monitor.get_consecutive_failures(), 0);
    }

    #[tokio::test]
    async fn test_monitor_with_interval() {
        let engine = Arc::new(StrategyEngine::new());
        let monitor = Monitor::with_interval(engine, 60);
        assert_eq!(monitor.interval.as_secs(), 60);
    }

    #[tokio::test]
    async fn test_set_test_urls() {
        let engine = Arc::new(StrategyEngine::new());
        let monitor = Monitor::new(engine);
        
        let urls = vec!["https://example.com".to_string()];
        monitor.set_test_urls(urls.clone()).await;
        
        let stored_urls = monitor.test_urls.read().await;
        assert_eq!(*stored_urls, urls);
    }

    #[tokio::test]
    async fn test_legacy_monitor_creation() {
        let monitor = StrategyMonitor::new(60);
        assert_eq!(monitor.interval_secs, 60);
        assert!(!monitor.is_running().await);
    }
}

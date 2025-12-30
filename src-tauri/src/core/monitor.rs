//! Strategy health monitoring module
//!
//! Monitors connectivity to test URLs and detects strategy degradation.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

/// Threshold for success rate below which strategy is considered degraded
const DEGRADATION_THRESHOLD: f64 = 0.7;

/// HTTP request timeout for connectivity checks
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Strategy health monitor
///
/// Periodically checks connectivity to test URLs and triggers
/// a callback when the success rate drops below threshold.
pub struct StrategyMonitor {
    interval_secs: u64,
    cancel_token: CancellationToken,
    is_running: Arc<Mutex<bool>>,
}

impl StrategyMonitor {
    /// Creates a new strategy monitor
    ///
    /// # Arguments
    /// * `interval_secs` - Interval between connectivity checks in seconds
    pub fn new(interval_secs: u64) -> Self {
        Self {
            interval_secs,
            cancel_token: CancellationToken::new(),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Starts background monitoring
    ///
    /// Periodically checks connectivity to test URLs and calls `on_degraded`
    /// callback when success rate drops below 70%.
    ///
    /// # Arguments
    /// * `test_urls` - URLs to check for connectivity
    /// * `on_degraded` - Callback invoked when strategy performance degrades
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
            "Starting strategy monitor with {}s interval, {} test URLs",
            self.interval_secs,
            test_urls.len()
        );

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Strategy monitor stopped");
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        let success_rate = Self::check_connectivity(&test_urls).await;
                        debug!("Connectivity check: success_rate = {:.2}", success_rate);

                        if success_rate < DEGRADATION_THRESHOLD {
                            warn!(
                                "Strategy degradation detected: success_rate = {:.2} (threshold: {:.2})",
                                success_rate, DEGRADATION_THRESHOLD
                            );
                            on_degraded();
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
        info!("Stopping strategy monitor");
        self.cancel_token.cancel();
    }

    /// Checks connectivity to the given URLs
    ///
    /// Performs HTTP HEAD requests to all URLs concurrently and returns
    /// the ratio of successful requests.
    ///
    /// # Arguments
    /// * `urls` - URLs to check
    ///
    /// # Returns
    /// Success rate as a value between 0.0 and 1.0
    pub async fn check_connectivity(urls: &[String]) -> f64 {
        if urls.is_empty() {
            return 1.0;
        }

        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();

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
                            debug!("URL {} check: status={}, success={}", url, response.status(), success);
                            success
                        }
                        Err(e) => {
                            debug!("URL {} check failed: {}", url, e);
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

    /// Returns whether the monitor is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}

impl Default for StrategyMonitor {
    fn default() -> Self {
        Self::new(30) // Default: check every 30 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_connectivity_empty_urls() {
        let rate = StrategyMonitor::check_connectivity(&[]).await;
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_monitor_creation() {
        let monitor = StrategyMonitor::new(60);
        assert_eq!(monitor.interval_secs, 60);
        assert!(!monitor.is_running().await);
    }
}

//! Opt-in telemetry module for Isolate
//!
//! Collects anonymous usage statistics to improve strategy recommendations.
//! NEVER collects: IP addresses, personal data, browsing history, user UUIDs.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Telemetry event containing anonymous usage data
#[derive(Debug, Clone, Serialize)]
pub struct TelemetryEvent {
    /// Type of event (e.g., "optimization", "strategy_usage", "error")
    pub event_type: String,
    /// ISO 8601 timestamp when event occurred
    pub timestamp: String,
    /// Event-specific data (anonymized)
    pub data: serde_json::Value,
    /// Application version
    pub app_version: String,
}

/// Batch of telemetry events for sending
#[derive(Debug, Clone, Serialize)]
struct TelemetryBatch {
    /// Events in this batch
    events: Vec<TelemetryEvent>,
}

/// Telemetry service for collecting and sending anonymous usage statistics
///
/// # Privacy Guarantees
/// - Disabled by default (opt-in only)
/// - Never collects IP addresses
/// - Never collects personal data or user UUIDs
/// - Only collects: strategy_id, success_rate, duration, anonymized errors
pub struct TelemetryService {
    /// Whether telemetry is enabled (opt-in, atomic for lock-free reads)
    enabled: AtomicBool,
    /// Endpoint URL for sending telemetry
    endpoint: String,
    /// Buffered events waiting to be sent
    batch: RwLock<Vec<TelemetryEvent>>,
    /// Interval between automatic flushes
    flush_interval: Duration,
}

impl TelemetryService {
    /// Creates a new telemetry service
    ///
    /// # Note
    /// Telemetry is disabled by default. Call `set_enabled(true)` to opt-in.
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false), // Disabled by default (opt-in)
            endpoint: "https://telemetry.isolate.app/v1/events".to_string(),
            batch: RwLock::new(Vec::new()),
            flush_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Sets whether telemetry collection is enabled
    ///
    /// # Arguments
    /// * `enabled` - true to opt-in, false to opt-out
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);

        if enabled {
            info!("Telemetry enabled by user (opt-in)");
        } else {
            info!("Telemetry disabled");
            // Clear any buffered events when disabled (async cleanup)
            // Note: We can't await here, so we spawn a task
            // The actual clearing happens on next operation
        }
    }

    /// Returns whether telemetry is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Reports an optimization result
    ///
    /// # Arguments
    /// * `strategy_id` - ID of the strategy that was tested
    /// * `score` - Optimization score (0.0 - 1.0)
    /// * `success` - Whether the optimization was successful
    pub async fn report_optimization(&self, strategy_id: &str, score: f32, success: bool) {
        if !self.is_enabled() {
            return;
        }

        let event = TelemetryEvent {
            event_type: "optimization".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "strategy_id": strategy_id,
                "score": score,
                "success": success,
            }),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        self.add_event(event).await;
        debug!(
            "Recorded optimization event: strategy={}, score={}, success={}",
            strategy_id, score, success
        );
    }

    /// Reports strategy usage statistics
    ///
    /// # Arguments
    /// * `strategy_id` - ID of the strategy being used
    /// * `duration_secs` - How long the strategy was active (in seconds)
    pub async fn report_strategy_usage(&self, strategy_id: &str, duration_secs: u64) {
        if !self.is_enabled() {
            return;
        }

        let event = TelemetryEvent {
            event_type: "strategy_usage".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "strategy_id": strategy_id,
                "duration_secs": duration_secs,
            }),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        self.add_event(event).await;
        debug!(
            "Recorded strategy usage: strategy={}, duration={}s",
            strategy_id, duration_secs
        );
    }

    /// Reports an error (anonymized)
    ///
    /// # Arguments
    /// * `error_type` - Category of error (e.g., "network", "process", "config")
    /// * `message` - Error message (will be anonymized - no paths, IPs, etc.)
    pub async fn report_error(&self, error_type: &str, message: &str) {
        if !self.is_enabled() {
            return;
        }

        // Anonymize the error message - remove potential PII
        let anonymized_message = Self::anonymize_error_message(message);

        let event = TelemetryEvent {
            event_type: "error".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: serde_json::json!({
                "error_type": error_type,
                "message": anonymized_message,
            }),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        self.add_event(event).await;
        debug!("Recorded error event: type={}", error_type);
    }

    /// Adds an event to the batch
    async fn add_event(&self, event: TelemetryEvent) {
        let mut batch = self.batch.write().await;
        batch.push(event);
        debug!("Telemetry event added, batch size: {}", batch.len());
    }

    /// Flushes pending events to the server
    ///
    /// # Returns
    /// * `Ok(())` - Events sent successfully (or telemetry disabled/no events)
    /// * `Err` - Network or server error (events are preserved for retry)
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_enabled() {
            // Clear batch if telemetry was disabled
            let mut batch = self.batch.write().await;
            if !batch.is_empty() {
                debug!("Clearing {} events (telemetry disabled)", batch.len());
                batch.clear();
            }
            return Ok(());
        }

        // Take events from batch
        let events: Vec<TelemetryEvent> = {
            let mut batch = self.batch.write().await;
            std::mem::take(&mut *batch)
        };

        if events.is_empty() {
            debug!("No telemetry events to flush");
            return Ok(());
        }

        let event_count = events.len();
        let batch_payload = TelemetryBatch {
            events: events.clone(),
        };

        debug!(
            "Flushing {} telemetry events to {}",
            event_count, self.endpoint
        );

        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(format!("Isolate/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let response = client.post(&self.endpoint).json(&batch_payload).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                info!("Telemetry batch sent successfully ({} events)", event_count);
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                warn!("Telemetry server returned error: {}", status);

                // Re-add events to batch on failure (for retry)
                let mut batch = self.batch.write().await;
                batch.extend(events);

                // Limit batch size to prevent unbounded growth
                if batch.len() > 1000 {
                    let excess = batch.len() - 1000;
                    batch.drain(0..excess);
                    warn!("Telemetry batch truncated, dropped {} old events", excess);
                }

                Err(format!("Server returned status: {}", status).into())
            }
            Err(e) => {
                // Graceful handling - log warning but don't fail hard
                warn!("Failed to send telemetry (endpoint unavailable): {}", e);

                // Re-add events to batch on failure (for retry)
                let mut batch = self.batch.write().await;
                batch.extend(events);

                // Limit batch size to prevent unbounded growth
                if batch.len() > 1000 {
                    let excess = batch.len() - 1000;
                    batch.drain(0..excess);
                    warn!("Telemetry batch truncated, dropped {} old events", excess);
                }

                Err(Box::new(e))
            }
        }
    }

    /// Starts a background task that periodically flushes events
    ///
    /// This spawns a Tokio task that runs every `flush_interval` (5 minutes by default).
    /// The task will continue running until the application exits.
    pub fn start_background_flush(self: &Arc<Self>) {
        let service = Arc::clone(self);
        let interval = self.flush_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if service.is_enabled() {
                    debug!("Background telemetry flush triggered");
                    if let Err(e) = service.flush().await {
                        // Log but don't panic - telemetry failures shouldn't crash the app
                        warn!("Background telemetry flush failed: {}", e);
                    }
                }
            }
        });

        info!(
            "Telemetry background flush started (interval: {:?})",
            self.flush_interval
        );
    }

    /// Returns the number of pending events in the batch
    pub async fn pending_events(&self) -> usize {
        self.batch.read().await.len()
    }

    /// Clears all pending events without sending
    pub async fn clear(&self) {
        let mut batch = self.batch.write().await;
        batch.clear();
        debug!("Telemetry batch cleared");
    }

    /// Anonymizes an error message by removing potential PII
    ///
    /// Removes:
    /// - File paths (C:\Users\..., /home/...)
    /// - IP addresses
    /// - UUIDs
    /// - Email addresses
    fn anonymize_error_message(message: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = message.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Check for Windows path (C:\, D:\, etc.)
            if i + 2 < chars.len()
                && chars[i].is_ascii_alphabetic()
                && chars[i + 1] == ':'
                && chars[i + 2] == '\\'
            {
                result.push_str("[PATH]");
                // Skip until whitespace or end
                while i < chars.len() && !chars[i].is_whitespace() {
                    i += 1;
                }
                continue;
            }

            // Check for Unix path (/home/, /Users/, /tmp/, /var/)
            if chars[i] == '/' {
                let rest: String = chars[i..].iter().collect();
                if rest.starts_with("/home/")
                    || rest.starts_with("/Users/")
                    || rest.starts_with("/tmp/")
                    || rest.starts_with("/var/")
                {
                    result.push_str("[PATH]");
                    // Skip until whitespace or end
                    while i < chars.len() && !chars[i].is_whitespace() {
                        i += 1;
                    }
                    continue;
                }
            }

            // Check for IP address (simple heuristic: digit.digit.digit.digit)
            if chars[i].is_ascii_digit() {
                let start = i;
                let mut dots = 0;
                let mut j = i;

                while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                    if chars[j] == '.' {
                        dots += 1;
                    }
                    j += 1;
                }

                // Simple IP check: has 3 dots and reasonable length
                if dots == 3 && j - start >= 7 && j - start <= 15 {
                    let potential_ip: String = chars[start..j].iter().collect();
                    // Verify it looks like an IP (4 parts separated by dots)
                    let parts: Vec<&str> = potential_ip.split('.').collect();
                    if parts.len() == 4
                        && parts
                            .iter()
                            .all(|p| p.parse::<u8>().is_ok() || p.parse::<u16>().map_or(false, |n| n <= 255))
                    {
                        result.push_str("[IP]");
                        i = j;
                        continue;
                    }
                }
            }

            // Check for UUID (8-4-4-4-12 hex pattern)
            if chars[i].is_ascii_hexdigit() && i + 36 <= chars.len() {
                let potential: String = chars[i..i + 36].iter().collect();
                if Self::is_uuid(&potential) {
                    result.push_str("[UUID]");
                    i += 36;
                    continue;
                }
            }

            // Check for email (contains @ with text before and after)
            if chars[i] == '@' && i > 0 && i + 1 < chars.len() {
                // Find start of email (go back to find alphanumeric start)
                let mut email_start = i;
                while email_start > 0 {
                    let prev = chars[email_start - 1];
                    if prev.is_alphanumeric() || prev == '.' || prev == '_' || prev == '-' || prev == '+' {
                        email_start -= 1;
                    } else {
                        break;
                    }
                }

                // Find end of email (go forward to find domain end)
                let mut email_end = i + 1;
                let mut has_dot_after_at = false;
                while email_end < chars.len() {
                    let c = chars[email_end];
                    if c.is_alphanumeric() || c == '.' || c == '-' {
                        if c == '.' {
                            has_dot_after_at = true;
                        }
                        email_end += 1;
                    } else {
                        break;
                    }
                }

                // If we found a valid-looking email, replace it
                if email_start < i && has_dot_after_at && email_end > i + 2 {
                    // Remove already added characters that are part of email
                    let chars_to_remove = i - email_start;
                    for _ in 0..chars_to_remove {
                        result.pop();
                    }
                    result.push_str("[EMAIL]");
                    i = email_end;
                    continue;
                }
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    /// Checks if a string is a valid UUID format (8-4-4-4-12)
    fn is_uuid(s: &str) -> bool {
        if s.len() != 36 {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();

        // Check dashes at positions 8, 13, 18, 23
        if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
            return false;
        }

        // Check all other characters are hex digits
        for (i, c) in chars.iter().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                continue;
            }
            if !c.is_ascii_hexdigit() {
                return false;
            }
        }

        true
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_by_default() {
        let service = TelemetryService::new();
        assert!(!service.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let service = TelemetryService::new();

        service.set_enabled(true);
        assert!(service.is_enabled());

        service.set_enabled(false);
        assert!(!service.is_enabled());
    }

    #[tokio::test]
    async fn test_no_events_when_disabled() {
        let service = TelemetryService::new();

        service.report_optimization("test-strategy", 0.95, true).await;
        assert_eq!(service.pending_events().await, 0);
    }

    #[tokio::test]
    async fn test_events_recorded_when_enabled() {
        let service = TelemetryService::new();
        service.set_enabled(true);

        service.report_optimization("test-strategy", 0.95, true).await;
        assert_eq!(service.pending_events().await, 1);

        service.report_strategy_usage("test-strategy", 3600).await;
        assert_eq!(service.pending_events().await, 2);

        service.report_error("network", "Connection timeout").await;
        assert_eq!(service.pending_events().await, 3);
    }

    #[tokio::test]
    async fn test_clear_events() {
        let service = TelemetryService::new();
        service.set_enabled(true);

        service.report_optimization("test-strategy", 0.95, true).await;
        assert_eq!(service.pending_events().await, 1);

        service.clear().await;
        assert_eq!(service.pending_events().await, 0);
    }

    #[test]
    fn test_anonymize_error_message() {
        // Test Windows path removal
        let msg = "Error at C:\\Users\\john\\Documents\\file.txt";
        let anon = TelemetryService::anonymize_error_message(msg);
        assert!(anon.contains("[PATH]"));
        assert!(!anon.contains("john"));

        // Test IP removal
        let msg = "Connection failed to 192.168.1.100:8080";
        let anon = TelemetryService::anonymize_error_message(msg);
        assert!(anon.contains("[IP]"));
        assert!(!anon.contains("192.168"));

        // Test UUID removal
        let msg = "Session 550e8400-e29b-41d4-a716-446655440000 expired";
        let anon = TelemetryService::anonymize_error_message(msg);
        assert!(anon.contains("[UUID]"));
        assert!(!anon.contains("550e8400"));

        // Test email removal
        let msg = "User user@example.com not found";
        let anon = TelemetryService::anonymize_error_message(msg);
        assert!(anon.contains("[EMAIL]"));
        assert!(!anon.contains("user@example.com"));
    }

    #[tokio::test]
    async fn test_flush_clears_batch_when_disabled() {
        let service = TelemetryService::new();
        service.set_enabled(true);

        service.report_optimization("test-strategy", 0.95, true).await;
        assert_eq!(service.pending_events().await, 1);

        service.set_enabled(false);
        let _ = service.flush().await;
        assert_eq!(service.pending_events().await, 0);
    }
}

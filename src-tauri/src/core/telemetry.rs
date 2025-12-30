//! Opt-in telemetry module for Isolate
//!
//! Collects anonymous usage statistics to improve strategy recommendations.
//! NEVER collects: IP addresses, personal data, browsing history.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// Telemetry event containing anonymous usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Autonomous System Number (identifies ISP, not user)
    pub asn: Option<u32>,
    /// Country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// Strategy identifier that was tested
    pub strategy_id: String,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Unix timestamp when event occurred
    pub timestamp: i64,
}

/// Batch of telemetry events for sending
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TelemetryBatch {
    /// Application version
    app_version: String,
    /// Events in this batch
    events: Vec<TelemetryEvent>,
}

/// Telemetry client for collecting and sending anonymous usage statistics
///
/// # Privacy Guarantees
/// - Disabled by default (opt-in only)
/// - Never collects IP addresses
/// - Never collects personal data
/// - Only collects: ASN, country, strategy_id, success_rate
pub struct TelemetryClient {
    /// Whether telemetry is enabled (opt-in)
    enabled: Arc<RwLock<bool>>,
    /// Endpoint URL for sending telemetry
    endpoint: String,
    /// Buffered events waiting to be sent
    events: Arc<RwLock<Vec<TelemetryEvent>>>,
    /// HTTP client for sending requests
    client: reqwest::Client,
    /// Maximum events to buffer before auto-flush
    max_buffer_size: usize,
}

impl TelemetryClient {
    /// Creates a new telemetry client
    ///
    /// # Arguments
    /// * `endpoint` - URL to POST telemetry data to
    ///
    /// # Note
    /// Telemetry is disabled by default. Call `set_enabled(true)` to opt-in.
    pub fn new(endpoint: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .user_agent(format!("Isolate/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();

        Self {
            enabled: Arc::new(RwLock::new(false)), // Disabled by default
            endpoint: endpoint.to_string(),
            events: Arc::new(RwLock::new(Vec::new())),
            client,
            max_buffer_size: 100,
        }
    }

    /// Sets whether telemetry collection is enabled
    ///
    /// # Arguments
    /// * `enabled` - true to opt-in, false to opt-out
    pub async fn set_enabled(&self, enabled: bool) {
        let mut guard = self.enabled.write().await;
        *guard = enabled;
        
        if enabled {
            debug!("Telemetry enabled by user");
        } else {
            debug!("Telemetry disabled");
            // Clear any buffered events when disabled
            let mut events = self.events.write().await;
            events.clear();
        }
    }

    /// Returns whether telemetry is currently enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Records a telemetry event
    ///
    /// Events are buffered and sent periodically via `flush()`.
    /// If telemetry is disabled, this is a no-op.
    ///
    /// # Arguments
    /// * `event` - The telemetry event to record
    pub async fn record_event(&self, event: TelemetryEvent) {
        if !self.is_enabled().await {
            return;
        }

        let mut events = self.events.write().await;
        events.push(event);
        
        debug!("Telemetry event recorded, buffer size: {}", events.len());

        // Auto-flush if buffer is full
        if events.len() >= self.max_buffer_size {
            drop(events); // Release lock before flush
            if let Err(e) = self.flush().await {
                warn!("Auto-flush failed: {}", e);
            }
        }
    }

    /// Sends all buffered events to the telemetry endpoint
    ///
    /// # Returns
    /// * `Ok(())` - Events sent successfully (or telemetry disabled/no events)
    /// * `Err` - Network or server error
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_enabled().await {
            return Ok(());
        }

        let events: Vec<TelemetryEvent> = {
            let mut guard = self.events.write().await;
            std::mem::take(&mut *guard)
        };

        if events.is_empty() {
            debug!("No telemetry events to flush");
            return Ok(());
        }

        let batch = TelemetryBatch {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            events: events.clone(),
        };

        debug!("Flushing {} telemetry events to {}", batch.events.len(), self.endpoint);

        let response = self
            .client
            .post(&self.endpoint)
            .json(&batch)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                debug!("Telemetry batch sent successfully");
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                error!("Telemetry server returned error: {}", status);
                
                // Re-add events to buffer on failure
                let mut guard = self.events.write().await;
                guard.extend(events);
                
                Err(format!("Server returned status: {}", status).into())
            }
            Err(e) => {
                error!("Failed to send telemetry: {}", e);
                
                // Re-add events to buffer on failure
                let mut guard = self.events.write().await;
                guard.extend(events);
                
                Err(Box::new(e))
            }
        }
    }

    /// Returns the number of buffered events
    pub async fn pending_events(&self) -> usize {
        self.events.read().await.len()
    }

    /// Clears all buffered events without sending
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        events.clear();
        debug!("Telemetry buffer cleared");
    }
}

impl Default for TelemetryClient {
    fn default() -> Self {
        Self::new("https://telemetry.isolate.app/v1/events")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disabled_by_default() {
        let client = TelemetryClient::new("http://localhost:8080/telemetry");
        assert!(!client.is_enabled().await);
    }

    #[tokio::test]
    async fn test_enable_disable() {
        let client = TelemetryClient::new("http://localhost:8080/telemetry");
        
        client.set_enabled(true).await;
        assert!(client.is_enabled().await);
        
        client.set_enabled(false).await;
        assert!(!client.is_enabled().await);
    }

    #[tokio::test]
    async fn test_no_events_when_disabled() {
        let client = TelemetryClient::new("http://localhost:8080/telemetry");
        
        let event = TelemetryEvent {
            asn: Some(12345),
            country: Some("RU".to_string()),
            strategy_id: "test-strategy".to_string(),
            success_rate: 0.95,
            timestamp: 1234567890,
        };
        
        client.record_event(event).await;
        assert_eq!(client.pending_events().await, 0);
    }

    #[tokio::test]
    async fn test_events_recorded_when_enabled() {
        let client = TelemetryClient::new("http://localhost:8080/telemetry");
        client.set_enabled(true).await;
        
        let event = TelemetryEvent {
            asn: Some(12345),
            country: Some("RU".to_string()),
            strategy_id: "test-strategy".to_string(),
            success_rate: 0.95,
            timestamp: 1234567890,
        };
        
        client.record_event(event).await;
        assert_eq!(client.pending_events().await, 1);
    }

    #[tokio::test]
    async fn test_clear_on_disable() {
        let client = TelemetryClient::new("http://localhost:8080/telemetry");
        client.set_enabled(true).await;
        
        let event = TelemetryEvent {
            asn: Some(12345),
            country: Some("RU".to_string()),
            strategy_id: "test-strategy".to_string(),
            success_rate: 0.95,
            timestamp: 1234567890,
        };
        
        client.record_event(event).await;
        assert_eq!(client.pending_events().await, 1);
        
        client.set_enabled(false).await;
        assert_eq!(client.pending_events().await, 0);
    }
}

//! Centralized Event Bus for Isolate
//!
//! Provides typed events and pub/sub pattern for decoupling components.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

/// All application events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    // Strategy events
    StrategyApplied {
        strategy_id: String,
        strategy_name: String,
    },
    StrategyStopped,
    StrategyFailed {
        strategy_id: String,
        error: String,
    },

    // Optimization events
    OptimizationStarted {
        domain: Option<String>,
    },
    OptimizationProgress {
        current: u32,
        total: u32,
        strategy_id: String,
    },
    OptimizationCompleted {
        strategy_id: String,
        score: f64,
    },
    OptimizationFailed {
        error: String,
    },

    // Monitor events
    DomainLocked {
        domain: String,
        strategy_id: String,
    },
    DomainUnlocked {
        domain: String,
    },
    HealthCheckFailed {
        domain: String,
    },

    // Service events
    ServiceStatusChanged {
        service_id: String,
        available: bool,
    },

    // System events
    BinaryIntegrityWarning {
        tampered: Vec<String>,
    },
    ConfigReloaded,
}

/// Event Bus with broadcast channels
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish event to all subscribers
    pub fn publish(&self, event: AppEvent) {
        debug!(event = ?event, "Publishing event");
        let _ = self.sender.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    /// Get sender for cloning into other components
    pub fn sender(&self) -> broadcast::Sender<AppEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

// Shared type alias
pub type SharedEventBus = Arc<EventBus>;

pub fn create_event_bus() -> SharedEventBus {
    Arc::new(EventBus::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = create_event_bus();
        let mut rx = bus.subscribe();

        bus.publish(AppEvent::StrategyApplied {
            strategy_id: "test".into(),
            strategy_name: "Test Strategy".into(),
        });

        let event = rx.recv().await.unwrap();
        match event {
            AppEvent::StrategyApplied { strategy_id, .. } => {
                assert_eq!(strategy_id, "test");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = create_event_bus();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(AppEvent::ConfigReloaded);

        assert!(matches!(
            rx1.recv().await.unwrap(),
            AppEvent::ConfigReloaded
        ));
        assert!(matches!(
            rx2.recv().await.unwrap(),
            AppEvent::ConfigReloaded
        ));
    }
}

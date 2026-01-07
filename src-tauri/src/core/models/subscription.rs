//! Subscription models for proxy subscription management
//!
//! Supports automatic updates of proxy lists from subscription URLs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Subscription configuration for automatic proxy updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Subscription URL
    pub url: String,

    /// Update interval in seconds (0 = manual only)
    pub update_interval: u64,

    /// Last successful update timestamp
    pub last_updated: Option<DateTime<Utc>>,

    /// Last update error (if any)
    pub last_error: Option<String>,

    /// Number of proxies from this subscription
    pub proxy_count: u32,

    /// Whether auto-update is enabled
    pub auto_update: bool,

    /// User-Agent header for requests (optional)
    pub user_agent: Option<String>,

    /// Subscription format hint
    pub format: SubscriptionFormat,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            url: String::new(),
            update_interval: 86400, // 24 hours
            last_updated: None,
            last_error: None,
            proxy_count: 0,
            auto_update: true,
            user_agent: None,
            format: SubscriptionFormat::Auto,
            created_at: Utc::now(),
        }
    }
}

impl Subscription {
    /// Create a new subscription with generated ID
    pub fn new(name: String, url: String) -> Self {
        let id = format!(
            "sub_{}",
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        Self {
            id,
            name,
            url,
            ..Default::default()
        }
    }

    /// Check if subscription needs update based on interval
    pub fn needs_update(&self) -> bool {
        if !self.auto_update || self.update_interval == 0 {
            return false;
        }

        match self.last_updated {
            Some(last) => {
                let elapsed = Utc::now().signed_duration_since(last);
                elapsed.num_seconds() as u64 >= self.update_interval
            }
            None => true, // Never updated
        }
    }
}

/// Subscription format for parsing
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionFormat {
    /// Auto-detect format
    #[default]
    Auto,
    /// V2Ray subscription (base64 encoded proxy URLs)
    V2ray,
    /// Clash YAML format
    Clash,
    /// Plain text (one URL per line)
    Plain,
    /// JSON array of ProxyConfig
    Json,
}

impl std::fmt::Display for SubscriptionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionFormat::Auto => write!(f, "auto"),
            SubscriptionFormat::V2ray => write!(f, "v2ray"),
            SubscriptionFormat::Clash => write!(f, "clash"),
            SubscriptionFormat::Plain => write!(f, "plain"),
            SubscriptionFormat::Json => write!(f, "json"),
        }
    }
}

impl std::str::FromStr for SubscriptionFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(SubscriptionFormat::Auto),
            "v2ray" => Ok(SubscriptionFormat::V2ray),
            "clash" => Ok(SubscriptionFormat::Clash),
            "plain" => Ok(SubscriptionFormat::Plain),
            "json" => Ok(SubscriptionFormat::Json),
            _ => Err(format!("Unknown subscription format: {}", s)),
        }
    }
}

/// Result of subscription update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionUpdateResult {
    /// Subscription ID that was updated
    pub subscription_id: String,

    /// Whether update was successful
    pub success: bool,

    /// Number of new proxies added
    pub added_count: u32,

    /// Number of existing proxies updated
    pub updated_count: u32,

    /// Number of proxies removed (no longer in subscription)
    pub removed_count: u32,

    /// Error message if update failed
    pub error: Option<String>,
}

impl SubscriptionUpdateResult {
    /// Create a successful result
    pub fn success(subscription_id: String, added: u32, updated: u32, removed: u32) -> Self {
        Self {
            subscription_id,
            success: true,
            added_count: added,
            updated_count: updated,
            removed_count: removed,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(subscription_id: String, error: String) -> Self {
        Self {
            subscription_id,
            success: false,
            added_count: 0,
            updated_count: 0,
            removed_count: 0,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_default() {
        let sub = Subscription::default();
        assert!(sub.id.is_empty());
        assert!(sub.name.is_empty());
        assert_eq!(sub.update_interval, 86400);
        assert!(sub.auto_update);
        assert_eq!(sub.format, SubscriptionFormat::Auto);
    }

    #[test]
    fn test_subscription_new() {
        let sub = Subscription::new("Test".to_string(), "https://example.com".to_string());
        assert!(sub.id.starts_with("sub_"));
        assert_eq!(sub.name, "Test");
        assert_eq!(sub.url, "https://example.com");
    }

    #[test]
    fn test_subscription_needs_update_never_updated() {
        let sub = Subscription {
            auto_update: true,
            update_interval: 3600,
            last_updated: None,
            ..Default::default()
        };
        assert!(sub.needs_update());
    }

    #[test]
    fn test_subscription_needs_update_disabled() {
        let sub = Subscription {
            auto_update: false,
            update_interval: 3600,
            last_updated: None,
            ..Default::default()
        };
        assert!(!sub.needs_update());
    }

    #[test]
    fn test_subscription_needs_update_manual_only() {
        let sub = Subscription {
            auto_update: true,
            update_interval: 0,
            last_updated: None,
            ..Default::default()
        };
        assert!(!sub.needs_update());
    }

    #[test]
    fn test_subscription_format_display() {
        assert_eq!(SubscriptionFormat::Auto.to_string(), "auto");
        assert_eq!(SubscriptionFormat::V2ray.to_string(), "v2ray");
        assert_eq!(SubscriptionFormat::Clash.to_string(), "clash");
    }

    #[test]
    fn test_subscription_format_from_str() {
        assert_eq!(
            "auto".parse::<SubscriptionFormat>().unwrap(),
            SubscriptionFormat::Auto
        );
        assert_eq!(
            "V2RAY".parse::<SubscriptionFormat>().unwrap(),
            SubscriptionFormat::V2ray
        );
        assert!("invalid".parse::<SubscriptionFormat>().is_err());
    }

    #[test]
    fn test_update_result_success() {
        let result = SubscriptionUpdateResult::success("sub_123".to_string(), 5, 2, 1);
        assert!(result.success);
        assert_eq!(result.added_count, 5);
        assert_eq!(result.updated_count, 2);
        assert_eq!(result.removed_count, 1);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_update_result_failure() {
        let result =
            SubscriptionUpdateResult::failure("sub_123".to_string(), "Network error".to_string());
        assert!(!result.success);
        assert_eq!(result.added_count, 0);
        assert_eq!(result.error, Some("Network error".to_string()));
    }
}

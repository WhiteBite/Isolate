//! Dashboard commands — live connections, traffic stats, and protection issues
//!
//! These commands provide real-time monitoring data for the Dashboard UI.
//! Currently returns mock data; real implementation will integrate with
//! WinDivert/network monitoring in future versions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Dashboard-specific errors
#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Failed to get connections: {0}")]
    ConnectionError(String),
    
    #[error("Failed to get traffic stats: {0}")]
    TrafficError(String),
    
    #[error("Issue not found: {0}")]
    IssueNotFound(String),
    
    #[error("Failed to fix issue: {0}")]
    FixError(String),
}

impl Serialize for DashboardError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Represents a live network connection being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveConnection {
    /// Unique connection identifier
    pub id: String,
    /// Domain name (if resolved)
    pub domain: String,
    /// IP address
    pub ip: String,
    /// Port number
    pub port: u16,
    /// Protocol (TCP/UDP)
    pub protocol: String,
    /// Connection status (active/closed)
    pub status: String,
    /// Bytes sent through this connection
    pub bytes_sent: u64,
    /// Bytes received through this connection
    pub bytes_received: u64,
    /// Unix timestamp when connection started
    pub started_at: i64,
}

/// Aggregated traffic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    /// Total bytes sent
    pub total_sent: u64,
    /// Total bytes received
    pub total_received: u64,
    /// Number of currently active connections
    pub active_connections: u32,
    /// Number of connections going through DPI bypass
    pub protected_connections: u32,
    /// Unix timestamp of last update
    pub last_updated: i64,
}

/// Protection issue detected by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionIssue {
    /// Unique issue identifier
    pub id: String,
    /// Severity level: warning, error, critical
    pub severity: String,
    /// Human-readable description
    pub message: String,
    /// Related service ID (if applicable)
    pub service_id: Option<String>,
    /// Unix timestamp when issue was detected
    pub detected_at: i64,
}

/// Result of attempting to fix an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    /// Whether the fix was successful
    pub success: bool,
    /// Description of what was done
    pub message: String,
    /// ID of the fixed issue
    pub issue_id: String,
}

/// Get list of live network connections
///
/// Returns mock data for now. Real implementation will use WinDivert
/// or similar to monitor actual network traffic.
#[tauri::command]
pub async fn get_live_connections() -> Result<Vec<LiveConnection>, DashboardError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    // Mock data for development
    let connections = vec![
        LiveConnection {
            id: "conn-001".to_string(),
            domain: "youtube.com".to_string(),
            ip: "142.250.185.206".to_string(),
            port: 443,
            protocol: "TCP".to_string(),
            status: "active".to_string(),
            bytes_sent: 15_420,
            bytes_received: 2_458_000,
            started_at: now - 120,
        },
        LiveConnection {
            id: "conn-002".to_string(),
            domain: "discord.com".to_string(),
            ip: "162.159.135.234".to_string(),
            port: 443,
            protocol: "TCP".to_string(),
            status: "active".to_string(),
            bytes_sent: 8_200,
            bytes_received: 45_600,
            started_at: now - 300,
        },
        LiveConnection {
            id: "conn-003".to_string(),
            domain: "twitch.tv".to_string(),
            ip: "151.101.66.167".to_string(),
            port: 443,
            protocol: "TCP".to_string(),
            status: "active".to_string(),
            bytes_sent: 12_100,
            bytes_received: 5_120_000,
            started_at: now - 60,
        },
        LiveConnection {
            id: "conn-004".to_string(),
            domain: "googlevideo.com".to_string(),
            ip: "142.250.185.110".to_string(),
            port: 443,
            protocol: "UDP".to_string(),
            status: "active".to_string(),
            bytes_sent: 5_000,
            bytes_received: 1_200_000,
            started_at: now - 45,
        },
    ];
    
    tracing::debug!("Returning {} mock live connections", connections.len());
    Ok(connections)
}

/// Get aggregated traffic statistics
///
/// Returns mock data for now. Real implementation will aggregate
/// data from connection monitoring.
#[tauri::command]
pub async fn get_traffic_stats() -> Result<TrafficStats, DashboardError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    // Mock data for development
    let stats = TrafficStats {
        total_sent: 156_789_000,      // ~150 MB
        total_received: 2_458_900_000, // ~2.3 GB
        active_connections: 4,
        protected_connections: 3,
        last_updated: now,
    };
    
    tracing::debug!(
        "Traffic stats: sent={}, received={}, active={}, protected={}",
        stats.total_sent,
        stats.total_received,
        stats.active_connections,
        stats.protected_connections
    );
    
    Ok(stats)
}

/// Get list of protection issues
///
/// Returns mock data for now. Real implementation will detect
/// actual issues like blocked connections, strategy failures, etc.
#[tauri::command]
pub async fn get_protection_issues() -> Result<Vec<ProtectionIssue>, DashboardError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    // Mock data for development - return some example issues
    let issues = vec![
        ProtectionIssue {
            id: "issue-001".to_string(),
            severity: "warning".to_string(),
            message: "QUIC blocking is disabled. Some services may not work optimally.".to_string(),
            service_id: None,
            detected_at: now - 3600,
        },
        ProtectionIssue {
            id: "issue-002".to_string(),
            severity: "error".to_string(),
            message: "Discord voice connection failed. Try switching strategy.".to_string(),
            service_id: Some("discord".to_string()),
            detected_at: now - 600,
        },
    ];
    
    tracing::debug!("Returning {} mock protection issues", issues.len());
    Ok(issues)
}

/// Attempt to fix a protection issue
///
/// Returns mock result for now. Real implementation will perform
/// actual fixes like restarting strategies, enabling QUIC blocking, etc.
#[tauri::command]
pub async fn fix_issue(issue_id: String) -> Result<FixResult, DashboardError> {
    tracing::info!("Attempting to fix issue: {}", issue_id);
    
    // Mock implementation - simulate fixing different issues
    let result = match issue_id.as_str() {
        "issue-001" => FixResult {
            success: true,
            message: "QUIC blocking has been enabled.".to_string(),
            issue_id: issue_id.clone(),
        },
        "issue-002" => FixResult {
            success: true,
            message: "Strategy restarted for Discord.".to_string(),
            issue_id: issue_id.clone(),
        },
        _ => {
            return Err(DashboardError::IssueNotFound(issue_id));
        }
    };
    
    tracing::info!("Issue {} fix result: success={}", result.issue_id, result.success);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_live_connections() {
        let connections = get_live_connections().await.unwrap();
        assert!(!connections.is_empty());
        
        // Check first connection has valid data
        let first = &connections[0];
        assert!(!first.id.is_empty());
        assert!(!first.domain.is_empty());
        assert!(first.port > 0);
    }
    
    #[tokio::test]
    async fn test_get_traffic_stats() {
        let stats = get_traffic_stats().await.unwrap();
        assert!(stats.last_updated > 0);
        assert!(stats.active_connections > 0);
    }
    
    #[tokio::test]
    async fn test_get_protection_issues() {
        let issues = get_protection_issues().await.unwrap();
        // Mock returns some issues
        assert!(!issues.is_empty());
    }
    
    #[tokio::test]
    async fn test_fix_issue_success() {
        let result = fix_issue("issue-001".to_string()).await.unwrap();
        assert!(result.success);
        assert_eq!(result.issue_id, "issue-001");
    }
    
    #[tokio::test]
    async fn test_fix_issue_not_found() {
        let result = fix_issue("nonexistent".to_string()).await;
        assert!(result.is_err());
    }
}

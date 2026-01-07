//! Configuration and settings models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IP stack type for dual-stack support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IpStack {
    /// IPv4 only
    V4Only,
    /// IPv6 only
    V6Only,
    /// Both IPv4 and IPv6
    #[default]
    DualStack,
}

impl std::fmt::Display for IpStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpStack::V4Only => write!(f, "IPv4"),
            IpStack::V6Only => write!(f, "IPv6"),
            IpStack::DualStack => write!(f, "Dual-Stack"),
        }
    }
}

/// WinDivert operation mode for Zapret strategies
/// 
/// Controls how winws handles TTL and hostlist management:
/// - Normal: Standard operation with fixed parameters
/// - AutoTTL: Automatically determines optimal TTL values
/// - AutoHostlist: Automatically manages hostlist based on blocked domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WinDivertMode {
    /// Standard mode - uses fixed parameters from strategy
    #[default]
    Normal,
    /// Auto TTL mode - automatically determines optimal TTL values
    #[serde(rename = "autottl")]
    AutoTTL,
    /// Auto hostlist mode - automatically manages hostlist
    #[serde(rename = "autohostlist")]
    AutoHostlist,
}

/// Game Filter Mode for port-based traffic filtering
/// 
/// Controls which ports winws intercepts:
/// - Normal: Standard web ports (80, 443) for regular browsing
/// - Gaming: Extended port range (1024-65535) for game traffic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameFilterMode {
    /// Normal mode - intercepts standard web ports (80, 443)
    #[default]
    Normal,
    /// Gaming mode - intercepts extended port range for games
    Gaming,
}

impl GameFilterMode {
    /// Returns the port filter string for winws --wf-tcp parameter
    pub fn to_port_filter(&self) -> &'static str {
        match self {
            GameFilterMode::Normal => "80,443",
            GameFilterMode::Gaming => "1024-65535",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gaming" => GameFilterMode::Gaming,
            _ => GameFilterMode::Normal,
        }
    }
    
    /// Check if gaming mode is active
    pub fn is_gaming(&self) -> bool {
        matches!(self, GameFilterMode::Gaming)
    }
}

impl std::fmt::Display for GameFilterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameFilterMode::Normal => write!(f, "normal"),
            GameFilterMode::Gaming => write!(f, "gaming"),
        }
    }
}

impl WinDivertMode {
    /// Returns the winws command line flag for this mode
    pub fn to_winws_flag(&self) -> Option<&'static str> {
        match self {
            WinDivertMode::Normal => None,
            WinDivertMode::AutoTTL => Some("--autottl"),
            WinDivertMode::AutoHostlist => Some("--autohostlist"),
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "autottl" => WinDivertMode::AutoTTL,
            "autohostlist" => WinDivertMode::AutoHostlist,
            _ => WinDivertMode::Normal,
        }
    }
}

impl std::fmt::Display for WinDivertMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinDivertMode::Normal => write!(f, "normal"),
            WinDivertMode::AutoTTL => write!(f, "autottl"),
            WinDivertMode::AutoHostlist => write!(f, "autohostlist"),
        }
    }
}

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_start: bool,
    pub auto_apply: bool,
    pub minimize_to_tray: bool,
    pub block_quic: bool,
    pub default_mode: String, // "turbo" or "deep"
    /// Portable mode - store data in app directory
    #[serde(default)]
    pub portable_mode: bool,
    /// WinDivert operation mode for Zapret strategies
    #[serde(default)]
    pub windivert_mode: WinDivertMode,
    /// Game filter mode for port-based traffic filtering
    #[serde(default)]
    pub game_filter_mode: GameFilterMode,
    /// Auto-update hostlists from remote sources
    #[serde(default)]
    pub auto_update_hostlists: bool,
    /// Hostlist update interval in hours (default: 24)
    #[serde(default = "default_hostlist_update_interval")]
    pub hostlist_update_interval_hours: u32,
    /// Auto failover - automatically switch to backup strategy on failure
    #[serde(default)]
    pub auto_failover_enabled: bool,
    /// Maximum failures before failover (default: 3)
    #[serde(default = "default_failover_max_failures")]
    pub failover_max_failures: u32,
    /// Cooldown in seconds before retry (default: 60)
    #[serde(default = "default_failover_cooldown_secs")]
    pub failover_cooldown_secs: u32,
}

/// Default max failures for failover
fn default_failover_max_failures() -> u32 {
    3
}

/// Default cooldown for failover (60 seconds)
fn default_failover_cooldown_secs() -> u32 {
    60
}

/// Default hostlist update interval (24 hours)
fn default_hostlist_update_interval() -> u32 {
    24
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_start: false,
            auto_apply: false,
            minimize_to_tray: true,
            block_quic: false,  // ВАЖНО: false по умолчанию, иначе ломает Discord voice
            default_mode: "turbo".to_string(),
            portable_mode: false,
            windivert_mode: WinDivertMode::Normal,
            game_filter_mode: GameFilterMode::Normal,
            auto_update_hostlists: false,
            hostlist_update_interval_hours: 24,
            auto_failover_enabled: false,
            failover_max_failures: 3,
            failover_cooldown_secs: 60,
        }
    }
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvInfo {
    pub asn: Option<String>,
    pub country: Option<String>,
    pub wifi_ssid: Option<String>,
    pub is_admin: bool,
    pub os_version: String,
}

impl EnvInfo {
    /// Generate cache key from environment
    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.asn.as_deref().unwrap_or("unknown"),
            self.country.as_deref().unwrap_or("unknown"),
            self.wifi_ssid.as_deref().unwrap_or("unknown")
        )
    }
}

/// Current application status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppStatus {
    pub is_active: bool,
    pub current_strategy: Option<String>,
    pub current_strategy_name: Option<String>,
    pub services_status: HashMap<String, bool>,
}

/// Information about available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}

/// Log entry for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_stack_display() {
        assert_eq!(IpStack::V4Only.to_string(), "IPv4");
        assert_eq!(IpStack::V6Only.to_string(), "IPv6");
        assert_eq!(IpStack::DualStack.to_string(), "Dual-Stack");
    }

    #[test]
    fn test_ip_stack_default() {
        let stack: IpStack = Default::default();
        assert_eq!(stack, IpStack::DualStack);
    }

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(!settings.auto_start);
        assert!(!settings.auto_apply);
        assert!(settings.minimize_to_tray);
        assert!(!settings.block_quic);
        assert_eq!(settings.default_mode, "turbo");
        assert!(!settings.portable_mode);
        assert_eq!(settings.game_filter_mode, GameFilterMode::Normal);
        assert!(!settings.auto_update_hostlists);
        assert_eq!(settings.hostlist_update_interval_hours, 24);
        assert!(!settings.auto_failover_enabled);
        assert_eq!(settings.failover_max_failures, 3);
        assert_eq!(settings.failover_cooldown_secs, 60);
    }

    #[test]
    fn test_game_filter_mode() {
        // Test default
        let mode: GameFilterMode = Default::default();
        assert_eq!(mode, GameFilterMode::Normal);
        
        // Test port filters
        assert_eq!(GameFilterMode::Normal.to_port_filter(), "80,443");
        assert_eq!(GameFilterMode::Gaming.to_port_filter(), "1024-65535");
        
        // Test from_str
        assert_eq!(GameFilterMode::from_str("gaming"), GameFilterMode::Gaming);
        assert_eq!(GameFilterMode::from_str("Gaming"), GameFilterMode::Gaming);
        assert_eq!(GameFilterMode::from_str("normal"), GameFilterMode::Normal);
        assert_eq!(GameFilterMode::from_str("unknown"), GameFilterMode::Normal);
        
        // Test is_gaming
        assert!(!GameFilterMode::Normal.is_gaming());
        assert!(GameFilterMode::Gaming.is_gaming());
        
        // Test display
        assert_eq!(GameFilterMode::Normal.to_string(), "normal");
        assert_eq!(GameFilterMode::Gaming.to_string(), "gaming");
    }

    #[test]
    fn test_env_info_cache_key() {
        let env = EnvInfo {
            asn: Some("AS12345".to_string()),
            country: Some("RU".to_string()),
            wifi_ssid: Some("MyWiFi".to_string()),
            is_admin: false,
            os_version: "Windows 11".to_string(),
        };
        
        assert_eq!(env.cache_key(), "AS12345:RU:MyWiFi");
        
        let env_partial = EnvInfo {
            asn: None,
            country: Some("US".to_string()),
            wifi_ssid: None,
            is_admin: true,
            os_version: "Windows 10".to_string(),
        };
        
        assert_eq!(env_partial.cache_key(), "unknown:US:unknown");
    }

    #[test]
    fn test_app_status_default() {
        let status = AppStatus::default();
        assert!(!status.is_active);
        assert!(status.current_strategy.is_none());
        assert!(status.services_status.is_empty());
    }
}

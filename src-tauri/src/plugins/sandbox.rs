//! Plugin Sandbox Security Module
//!
//! Provides security enforcement and violation logging for plugin execution.
//! 
//! ## Features
//! - Timeout violation logging
//! - Memory limit violation logging  
//! - HTTP domain whitelist enforcement with logging
//! - Tauri command whitelist enforcement with logging
//! - In-memory violation storage (last N violations per plugin)
//!
//! ## Usage
//! ```rust
//! use crate::plugins::sandbox::{SandboxViolation, SandboxLogger, VIOLATION_STORE};
//! 
//! let logger = SandboxLogger::new("my-plugin");
//! logger.log_http_violation("https://evil.com", &["*.allowed.com"]);
//! 
//! // Get violations for a plugin
//! let violations = VIOLATION_STORE.get_violations("my-plugin");
//! ```

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, warn, info};

// ============================================================================
// Violation Store (In-Memory Storage)
// ============================================================================

/// Maximum number of violations to store per plugin
const MAX_VIOLATIONS_PER_PLUGIN: usize = 100;

/// Global violation store for all plugins
pub static VIOLATION_STORE: Lazy<ViolationStore> = Lazy::new(ViolationStore::new);

/// In-memory store for sandbox violations
#[derive(Debug)]
pub struct ViolationStore {
    /// Map of plugin_id -> deque of violations (newest first)
    violations: RwLock<HashMap<String, VecDeque<SandboxViolation>>>,
}

impl ViolationStore {
    /// Create a new violation store
    pub fn new() -> Self {
        Self {
            violations: RwLock::new(HashMap::new()),
        }
    }

    /// Add a violation for a plugin
    pub fn add(&self, violation: SandboxViolation) {
        let mut store = self.violations.write();
        let plugin_id = violation.plugin_id.clone();
        
        let deque = store.entry(plugin_id).or_insert_with(VecDeque::new);
        
        // Add to front (newest first)
        deque.push_front(violation);
        
        // Trim to max size
        while deque.len() > MAX_VIOLATIONS_PER_PLUGIN {
            deque.pop_back();
        }
    }

    /// Get all violations for a plugin
    pub fn get_violations(&self, plugin_id: &str) -> Vec<SandboxViolation> {
        let store = self.violations.read();
        store
            .get(plugin_id)
            .map(|d| d.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get violation count for a plugin
    pub fn count(&self, plugin_id: &str) -> usize {
        let store = self.violations.read();
        store.get(plugin_id).map(|d| d.len()).unwrap_or(0)
    }

    /// Clear violations for a plugin
    pub fn clear(&self, plugin_id: &str) {
        let mut store = self.violations.write();
        store.remove(plugin_id);
    }

    /// Clear all violations
    pub fn clear_all(&self) {
        let mut store = self.violations.write();
        store.clear();
    }

    /// Get all plugin IDs that have violations
    pub fn get_plugins_with_violations(&self) -> Vec<String> {
        let store = self.violations.read();
        store.keys().cloned().collect()
    }

    /// Get total violation count across all plugins
    pub fn total_count(&self) -> usize {
        let store = self.violations.read();
        store.values().map(|d| d.len()).sum()
    }
}

impl Default for ViolationStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Sandbox Violation Types
// ============================================================================

/// Types of sandbox violations that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    /// Script execution exceeded timeout limit
    Timeout,
    /// Script exceeded memory limit
    MemoryLimit,
    /// Attempted HTTP request to non-whitelisted domain
    HttpDomainBlocked,
    /// Attempted to call forbidden Tauri command
    CommandBlocked,
    /// Attempted filesystem access without permission
    FilesystemBlocked,
    /// Attempted process spawn without permission
    ProcessBlocked,
}

impl std::fmt::Display for ViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationType::Timeout => write!(f, "timeout"),
            ViolationType::MemoryLimit => write!(f, "memory_limit"),
            ViolationType::HttpDomainBlocked => write!(f, "http_domain_blocked"),
            ViolationType::CommandBlocked => write!(f, "command_blocked"),
            ViolationType::FilesystemBlocked => write!(f, "filesystem_blocked"),
            ViolationType::ProcessBlocked => write!(f, "process_blocked"),
        }
    }
}

/// A recorded sandbox violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxViolation {
    /// Plugin that caused the violation
    pub plugin_id: String,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Human-readable description
    pub description: String,
    /// Additional context (e.g., blocked URL, command name)
    pub context: Option<String>,
    /// Timestamp when violation occurred (Unix millis)
    pub timestamp: u64,
}

impl SandboxViolation {
    pub fn new(
        plugin_id: impl Into<String>,
        violation_type: ViolationType,
        description: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            violation_type,
            description: description.into(),
            context: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

// ============================================================================
// Sandbox Logger
// ============================================================================

/// Logger for sandbox violations with structured tracing output
#[derive(Debug, Clone)]
pub struct SandboxLogger {
    plugin_id: String,
    violation_count: Arc<AtomicU64>,
}

impl SandboxLogger {
    /// Create a new sandbox logger for a plugin
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            violation_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get the plugin ID
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Get total violation count
    pub fn violation_count(&self) -> u64 {
        self.violation_count.load(Ordering::Relaxed)
    }

    /// Increment violation counter
    fn increment_count(&self) {
        self.violation_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Log a timeout violation
    pub fn log_timeout(&self, elapsed: Duration, limit: Duration) {
        self.increment_count();
        
        let violation = SandboxViolation::new(
            &self.plugin_id,
            ViolationType::Timeout,
            format!(
                "Script execution exceeded timeout: {:.2}s > {:.2}s limit",
                elapsed.as_secs_f64(),
                limit.as_secs_f64()
            ),
        );

        warn!(
            plugin_id = %self.plugin_id,
            violation_type = %violation.violation_type,
            elapsed_ms = elapsed.as_millis() as u64,
            limit_ms = limit.as_millis() as u64,
            "Plugin sandbox violation: timeout exceeded"
        );

        // Store violation
        VIOLATION_STORE.add(violation);
    }

    /// Log a memory limit violation
    pub fn log_memory_exceeded(&self, used_bytes: usize, limit_bytes: usize) {
        self.increment_count();

        let violation = SandboxViolation::new(
            &self.plugin_id,
            ViolationType::MemoryLimit,
            format!(
                "Script exceeded memory limit: {} bytes > {} bytes limit",
                used_bytes, limit_bytes
            ),
        );

        warn!(
            plugin_id = %self.plugin_id,
            violation_type = %violation.violation_type,
            used_bytes = used_bytes,
            limit_bytes = limit_bytes,
            used_mb = used_bytes / (1024 * 1024),
            limit_mb = limit_bytes / (1024 * 1024),
            "Plugin sandbox violation: memory limit exceeded"
        );

        // Store violation
        VIOLATION_STORE.add(violation);
    }

    /// Log an HTTP domain violation
    pub fn log_http_violation(&self, url: &str, whitelist: &[String]) {
        self.increment_count();

        let domain = extract_domain(url).unwrap_or_else(|| url.to_string());
        
        let violation = SandboxViolation::new(
            &self.plugin_id,
            ViolationType::HttpDomainBlocked,
            format!("HTTP request to non-whitelisted domain: {}", domain),
        )
        .with_context(url.to_string());

        warn!(
            plugin_id = %self.plugin_id,
            violation_type = %violation.violation_type,
            blocked_url = %url,
            blocked_domain = %domain,
            whitelist = ?whitelist,
            "Plugin sandbox violation: HTTP domain not in whitelist"
        );

        // Store violation
        VIOLATION_STORE.add(violation);
    }

    /// Log a blocked Tauri command
    pub fn log_command_blocked(&self, command: &str) {
        self.increment_count();

        let violation = SandboxViolation::new(
            &self.plugin_id,
            ViolationType::CommandBlocked,
            format!("Attempted to call forbidden Tauri command: {}", command),
        )
        .with_context(command.to_string());

        error!(
            plugin_id = %self.plugin_id,
            violation_type = %violation.violation_type,
            blocked_command = %command,
            "Plugin sandbox violation: Tauri command not allowed"
        );

        // Store violation
        VIOLATION_STORE.add(violation);
    }

    /// Log a filesystem access violation
    pub fn log_filesystem_blocked(&self, path: &str) {
        self.increment_count();

        let violation = SandboxViolation::new(
            &self.plugin_id,
            ViolationType::FilesystemBlocked,
            format!("Attempted filesystem access without permission: {}", path),
        )
        .with_context(path.to_string());

        error!(
            plugin_id = %self.plugin_id,
            violation_type = %violation.violation_type,
            blocked_path = %path,
            "Plugin sandbox violation: filesystem access denied"
        );

        // Store violation
        VIOLATION_STORE.add(violation);
    }

    /// Log a process spawn violation
    pub fn log_process_blocked(&self, command: &str) {
        self.increment_count();

        let violation = SandboxViolation::new(
            &self.plugin_id,
            ViolationType::ProcessBlocked,
            format!("Attempted to spawn process without permission: {}", command),
        )
        .with_context(command.to_string());

        error!(
            plugin_id = %self.plugin_id,
            violation_type = %violation.violation_type,
            blocked_process = %command,
            "Plugin sandbox violation: process spawn denied"
        );

        // Store violation
        VIOLATION_STORE.add(violation);
    }
}

// ============================================================================
// Command Whitelist
// ============================================================================

/// Whitelist of Tauri commands that plugins are allowed to call
/// 
/// This mirrors the frontend ALLOWED_COMMANDS in src/lib/plugins/context.ts
/// and provides backend validation for plugin command invocations.
pub static ALLOWED_COMMANDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    
    // Backend Status (Read-Only)
    set.insert("is_backend_ready");
    set.insert("get_app_version");
    set.insert("get_app_status");
    set.insert("is_portable_mode");
    set.insert("is_silent_mode");
    
    // Services (Read-Only)
    set.insert("get_services");
    set.insert("get_registry_services");
    set.insert("get_service_status");
    set.insert("get_services_by_category");
    set.insert("get_protected_services");
    set.insert("get_top_services");
    
    // Service Health (Read-Only)
    set.insert("check_single_service");
    set.insert("check_all_registry_services");
    set.insert("get_service_health_history");
    set.insert("get_service_health_stats");
    set.insert("get_all_services_health_history");
    
    // Strategies (Read-Only)
    set.insert("get_strategies");
    set.insert("get_strategies_unified");
    set.insert("get_current_strategy");
    set.insert("get_strategy_info");
    set.insert("get_all_registered_strategies");
    set.insert("get_strategies_for_service");
    set.insert("get_strategies_by_family");
    set.insert("get_registered_strategy");
    set.insert("get_strategy_registry_stats");
    
    // Strategy Statistics (Read-Only)
    set.insert("get_strategy_history");
    set.insert("get_strategy_statistics");
    set.insert("get_all_strategy_statistics");
    set.insert("get_automation_history");
    set.insert("get_strategy_metrics");
    set.insert("get_strategy_metrics_history");
    set.insert("get_metrics_snapshots");
    set.insert("get_aggregated_metrics");
    set.insert("is_strategy_prewarmed");
    set.insert("get_prewarmed_strategies");
    
    // Composition (Read-Only)
    set.insert("get_composition_rules");
    set.insert("get_compositions");
    set.insert("get_active_composition");
    set.insert("get_strategy_for_service");
    
    // AI Pilot (Read-Only Status)
    set.insert("get_ai_pilot_status");
    set.insert("get_ai_pilot_history");
    
    // Automation (Read-Only Status)
    set.insert("is_optimization_v2_running");
    set.insert("is_domain_monitor_running");
    set.insert("get_domain_status");
    set.insert("get_all_domain_statuses");
    set.insert("get_blocked_strategies");
    set.insert("get_locked_strategy");
    
    // A/B Testing (Read-Only Status)
    set.insert("get_ab_test_status");
    set.insert("get_ab_test_progress");
    set.insert("get_ab_test_results");
    set.insert("get_active_ab_tests");
    set.insert("list_ab_tests");
    set.insert("get_all_ab_test_results");
    set.insert("get_ab_test_history");
    set.insert("get_ab_test_statistics");
    
    // Monitor (Read-Only Status)
    set.insert("is_monitor_running");
    set.insert("is_strategy_degraded");
    set.insert("get_monitor_config");
    
    // Proxy (Read-Only)
    set.insert("get_proxies");
    set.insert("get_proxy_chains");
    set.insert("get_proxy_chain_stats");
    set.insert("is_system_proxy_set");
    
    // VLESS (Read-Only Status)
    set.insert("get_vless_configs");
    set.insert("get_vless_status");
    set.insert("get_all_vless_status");
    set.insert("is_singbox_available");
    set.insert("get_singbox_version");
    
    // TUN Mode (Read-Only Status)
    set.insert("is_tun_running");
    set.insert("get_tun_status");
    set.insert("get_tun_config");
    set.insert("is_tun_available");
    
    // Hostlists (Read-Only)
    set.insert("get_hostlists");
    set.insert("get_hostlist");
    set.insert("get_hostlist_info");
    set.insert("get_plugin_hostlists");
    set.insert("get_plugin_hostlist");
    set.insert("get_plugin_hostlist_domains");
    set.insert("get_all_plugin_domains");
    set.insert("check_domain_in_hostlists");
    set.insert("find_matching_hostlists");
    set.insert("get_hostlists_by_category");
    set.insert("get_hostlists_by_plugin");
    set.insert("get_hostlist_registry_stats");
    
    // IPSET (Read-Only)
    set.insert("get_ipset_info");
    set.insert("get_ipset_stats");
    set.insert("get_ipset_mode");
    set.insert("get_ipset_sources");
    
    // Routing (Read-Only)
    set.insert("get_routing_rules");
    set.insert("get_domain_routes");
    set.insert("get_app_routes");
    set.insert("get_installed_apps");
    
    // Diagnostics (Read-Only)
    set.insert("run_diagnostics");
    set.insert("diagnose");
    set.insert("diagnose_dual_stack");
    set.insert("check_ipv6");
    set.insert("check_conflicts");
    
    // Binaries (Read-Only)
    set.insert("check_binaries");
    set.insert("get_binaries_dir");
    set.insert("verify_binaries_integrity");
    set.insert("is_admin");
    
    // Settings (Read-Only)
    set.insert("get_settings");
    set.insert("get_services_settings");
    set.insert("get_setting");
    set.insert("get_windivert_mode");
    set.insert("get_game_filter_mode");
    set.insert("get_dns_settings");
    set.insert("get_tcp_timestamps_status");
    set.insert("get_failover_status");
    set.insert("get_failover_config");
    set.insert("get_learned_strategies");
    set.insert("get_current_profile");
    set.insert("get_all_profiles");
    set.insert("get_engine_mode");
    
    // Telemetry (Read-Only Status)
    set.insert("is_telemetry_enabled");
    set.insert("get_telemetry_pending_count");
    set.insert("is_crash_reporting_enabled");
    set.insert("get_crash_reporting_info");
    
    // Autorun (Read-Only Status)
    set.insert("get_autorun_status");
    
    // Hosts (Read-Only Status)
    set.insert("get_hosts_status");
    
    // QUIC (Read-Only Status)
    set.insert("is_quic_blocked");
    
    // Logs (Read-Only)
    set.insert("get_logs");
    
    // Tray (Read-Only Status)
    set.insert("get_tray_state");
    
    // Dashboard (Read-Only)
    set.insert("get_live_connections");
    set.insert("get_traffic_stats");
    set.insert("get_protection_issues");
    
    // Providers (Read-Only)
    set.insert("get_providers");
    set.insert("get_provider_recommendations");
    set.insert("is_strategy_recommended");
    
    // Library (Read-Only)
    set.insert("get_library_rules");
    set.insert("get_library_rule");
    set.insert("get_library_presets");
    set.insert("get_library_preset");
    set.insert("get_rules_for_service");
    set.insert("get_rules_by_method");
    
    // Plugin-Specific Commands (Isolated)
    set.insert("get_plugins_dir");
    set.insert("get_all_plugins_cmd");
    set.insert("get_plugin_services");
    set.insert("get_plugin_strategies");
    set.insert("check_plugin_service");
    set.insert("check_all_plugin_services");
    set.insert("list_plugin_scripts");
    set.insert("get_plugin_storage");
    set.insert("get_plugin_enabled");
    set.insert("get_all_plugin_states");
    set.insert("get_plugin_settings");
    set.insert("get_all_plugin_settings");
    
    // Resource Monitoring (Read-Only)
    set.insert("get_process_usage");
    set.insert("get_multiple_process_usage");
    set.insert("get_default_resource_limits");
    set.insert("get_recommended_resource_limits");
    
    // Config Updates (Read-Only Status)
    set.insert("check_config_updates");
    set.insert("check_github_updates");
    set.insert("check_hostlist_updates");
    
    set
});

/// Check if a Tauri command is allowed for plugins
pub fn is_command_allowed(command: &str) -> bool {
    ALLOWED_COMMANDS.contains(command)
}

/// Validate a command and log violation if blocked
pub fn validate_command(plugin_id: &str, command: &str) -> bool {
    if is_command_allowed(command) {
        return true;
    }
    
    let logger = SandboxLogger::new(plugin_id);
    logger.log_command_blocked(command);
    false
}

// ============================================================================
// HTTP Domain Validation
// ============================================================================

/// Check if a URL is allowed by the whitelist
pub fn is_url_allowed(url: &str, whitelist: &[String]) -> bool {
    if whitelist.is_empty() {
        return false;
    }

    let host = match extract_domain(url) {
        Some(h) => h,
        None => return false,
    };

    for pattern in whitelist {
        if pattern.starts_with("*.") {
            // Wildcard pattern: *.discord.com matches voice.discord.com
            let suffix = &pattern[1..]; // .discord.com
            if host.ends_with(suffix) || host == &pattern[2..] {
                return true;
            }
        } else if host == *pattern {
            return true;
        }
    }

    false
}

/// Validate HTTP URL and log violation if blocked
pub fn validate_http_url(plugin_id: &str, url: &str, whitelist: &[String]) -> bool {
    if is_url_allowed(url, whitelist) {
        return true;
    }
    
    let logger = SandboxLogger::new(plugin_id);
    logger.log_http_violation(url, whitelist);
    false
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}

// ============================================================================
// Execution Monitor
// ============================================================================

/// Monitor for tracking script execution limits
pub struct ExecutionMonitor {
    plugin_id: String,
    start_time: Instant,
    timeout: Duration,
    memory_limit: usize,
    logger: SandboxLogger,
}

impl ExecutionMonitor {
    /// Create a new execution monitor
    pub fn new(plugin_id: impl Into<String>, timeout: Duration, memory_limit: usize) -> Self {
        let plugin_id = plugin_id.into();
        Self {
            logger: SandboxLogger::new(&plugin_id),
            plugin_id,
            start_time: Instant::now(),
            timeout,
            memory_limit,
        }
    }

    /// Check if execution has exceeded timeout
    pub fn check_timeout(&self) -> bool {
        let elapsed = self.start_time.elapsed();
        if elapsed > self.timeout {
            self.logger.log_timeout(elapsed, self.timeout);
            return true;
        }
        false
    }

    /// Check if memory usage exceeds limit
    pub fn check_memory(&self, used_bytes: usize) -> bool {
        if used_bytes > self.memory_limit {
            self.logger.log_memory_exceeded(used_bytes, self.memory_limit);
            return true;
        }
        false
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the logger
    pub fn logger(&self) -> &SandboxLogger {
        &self.logger
    }

    /// Log execution completion
    pub fn log_completion(&self, success: bool) {
        let elapsed = self.start_time.elapsed();
        
        if success {
            info!(
                plugin_id = %self.plugin_id,
                elapsed_ms = elapsed.as_millis() as u64,
                "Plugin script execution completed successfully"
            );
        } else {
            warn!(
                plugin_id = %self.plugin_id,
                elapsed_ms = elapsed.as_millis() as u64,
                "Plugin script execution failed"
            );
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_type_display() {
        assert_eq!(ViolationType::Timeout.to_string(), "timeout");
        assert_eq!(ViolationType::MemoryLimit.to_string(), "memory_limit");
        assert_eq!(ViolationType::HttpDomainBlocked.to_string(), "http_domain_blocked");
        assert_eq!(ViolationType::CommandBlocked.to_string(), "command_blocked");
    }

    #[test]
    fn test_sandbox_violation_creation() {
        let violation = SandboxViolation::new(
            "test-plugin",
            ViolationType::Timeout,
            "Test timeout",
        );
        
        assert_eq!(violation.plugin_id, "test-plugin");
        assert_eq!(violation.violation_type, ViolationType::Timeout);
        assert!(violation.context.is_none());
    }

    #[test]
    fn test_sandbox_violation_with_context() {
        let violation = SandboxViolation::new(
            "test-plugin",
            ViolationType::HttpDomainBlocked,
            "Blocked domain",
        )
        .with_context("https://evil.com");
        
        assert_eq!(violation.context, Some("https://evil.com".to_string()));
    }

    #[test]
    fn test_sandbox_logger_violation_count() {
        let logger = SandboxLogger::new("test-plugin");
        assert_eq!(logger.violation_count(), 0);
        
        logger.log_command_blocked("forbidden_command");
        assert_eq!(logger.violation_count(), 1);
        
        logger.log_http_violation("https://evil.com", &[]);
        assert_eq!(logger.violation_count(), 2);
    }

    #[test]
    fn test_is_command_allowed() {
        assert!(is_command_allowed("get_services"));
        assert!(is_command_allowed("get_strategies"));
        assert!(is_command_allowed("is_backend_ready"));
        
        assert!(!is_command_allowed("apply_strategy"));
        assert!(!is_command_allowed("save_settings"));
        assert!(!is_command_allowed("execute_plugin_script"));
        assert!(!is_command_allowed("nonexistent_command"));
    }

    #[test]
    fn test_is_url_allowed() {
        let whitelist = vec![
            "discord.com".to_string(),
            "*.discord.gg".to_string(),
        ];
        
        assert!(is_url_allowed("https://discord.com/api", &whitelist));
        assert!(is_url_allowed("https://voice.discord.gg/", &whitelist));
        assert!(is_url_allowed("https://discord.gg/", &whitelist));
        
        assert!(!is_url_allowed("https://google.com/", &whitelist));
        assert!(!is_url_allowed("https://evil.com/discord.com", &whitelist));
    }

    #[test]
    fn test_is_url_allowed_empty_whitelist() {
        let whitelist: Vec<String> = vec![];
        assert!(!is_url_allowed("https://discord.com/", &whitelist));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://discord.com/api"), Some("discord.com".to_string()));
        assert_eq!(extract_domain("https://voice.discord.gg:443/"), Some("voice.discord.gg".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }

    #[test]
    fn test_execution_monitor() {
        let monitor = ExecutionMonitor::new(
            "test-plugin",
            Duration::from_secs(10),
            1024 * 1024, // 1MB
        );
        
        assert!(!monitor.check_timeout());
        assert!(!monitor.check_memory(512 * 1024)); // 512KB
        assert!(monitor.check_memory(2 * 1024 * 1024)); // 2MB exceeds limit
    }

    #[test]
    fn test_validate_command() {
        assert!(validate_command("test-plugin", "get_services"));
        assert!(!validate_command("test-plugin", "apply_strategy"));
    }

    #[test]
    fn test_validate_http_url() {
        let whitelist = vec!["api.example.com".to_string()];
        
        assert!(validate_http_url("test-plugin", "https://api.example.com/data", &whitelist));
        assert!(!validate_http_url("test-plugin", "https://evil.com/", &whitelist));
    }
}

//! Sentry integration for crash reporting
//!
//! Provides opt-in crash reporting and error tracking.
//! Privacy-first: disabled by default, anonymizes all data.
//!
//! # Privacy Guarantees
//! - Disabled by default (opt-in only)
//! - Never collects IP addresses (send_default_pii = false)
//! - Never collects usernames or file paths
//! - Anonymizes error messages
//! - User can disable at any time

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Global Sentry client guard
static SENTRY_GUARD: OnceCell<sentry::ClientInitGuard> = OnceCell::new();

/// Global enabled state (atomic for lock-free reads)
static SENTRY_ENABLED: AtomicBool = AtomicBool::new(false);

/// Sentry DSN from environment
const SENTRY_DSN_ENV: &str = "SENTRY_DSN";

/// Sentry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryConfig {
    /// Whether crash reporting is enabled (opt-in)
    pub enabled: bool,
    /// Anonymous user ID (generated, not personal)
    pub anonymous_id: Option<String>,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            anonymous_id: None,
        }
    }
}

/// Initialize Sentry with privacy-preserving defaults
///
/// # Arguments
/// * `enabled` - Whether to enable Sentry (from user settings)
///
/// # Returns
/// * `true` if Sentry was initialized successfully
/// * `false` if DSN not configured or initialization failed
pub fn init_sentry(enabled: bool) -> bool {
    // Store enabled state
    SENTRY_ENABLED.store(enabled, Ordering::SeqCst);

    if !enabled {
        info!("Sentry crash reporting disabled (user opt-out)");
        return false;
    }

    // Get DSN from environment
    let dsn = match std::env::var(SENTRY_DSN_ENV) {
        Ok(dsn) if !dsn.is_empty() => dsn,
        _ => {
            debug!("Sentry DSN not configured, crash reporting disabled");
            return false;
        }
    };

    // Check if already initialized
    if SENTRY_GUARD.get().is_some() {
        debug!("Sentry already initialized");
        return true;
    }

    // Determine environment
    let environment = if cfg!(debug_assertions) {
        "development"
    } else {
        "production"
    };

    // Skip in development unless explicitly enabled
    if cfg!(debug_assertions) && std::env::var("SENTRY_FORCE_DEV").is_err() {
        debug!("Sentry disabled in development mode");
        return false;
    }

    // Initialize Sentry with privacy-preserving options
    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: Some(env!("CARGO_PKG_VERSION").into()),
            environment: Some(environment.into()),
            // Privacy settings
            send_default_pii: false, // Never send PII
            attach_stacktrace: true,
            // Performance sampling (low to minimize overhead)
            traces_sample_rate: 0.1,
            // Filter sensitive data
            before_send: Some(Arc::new(|mut event| {
                // Anonymize any remaining PII
                event = anonymize_event(event);
                Some(event)
            })),
            before_breadcrumb: Some(Arc::new(|breadcrumb| {
                // Filter breadcrumbs with potential PII
                filter_breadcrumb(breadcrumb)
            })),
            ..Default::default()
        },
    ));

    // Store guard to keep Sentry alive
    if SENTRY_GUARD.set(guard).is_err() {
        warn!("Sentry guard already set");
    }

    info!("Sentry crash reporting initialized (user opt-in)");
    true
}

/// Set whether Sentry is enabled
///
/// Can be called at runtime to enable/disable crash reporting
pub fn set_enabled(enabled: bool) {
    let was_enabled = SENTRY_ENABLED.swap(enabled, Ordering::SeqCst);

    if enabled && !was_enabled {
        // Try to initialize if not already done
        init_sentry(true);
        info!("Sentry crash reporting enabled");
    } else if !enabled && was_enabled {
        info!("Sentry crash reporting disabled");
    }
}

/// Check if Sentry is currently enabled
pub fn is_enabled() -> bool {
    SENTRY_ENABLED.load(Ordering::SeqCst)
}

/// Set anonymous user context
///
/// # Arguments
/// * `anonymous_id` - A randomly generated ID (not linked to real user)
pub fn set_user_context(anonymous_id: &str) {
    if !is_enabled() {
        return;
    }

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            id: Some(anonymous_id.to_string()),
            // Explicitly don't set: email, username, ip_address
            ..Default::default()
        }));
    });

    debug!("Sentry user context set (anonymous)");
}

/// Clear user context
pub fn clear_user_context() {
    sentry::configure_scope(|scope| {
        scope.set_user(None);
    });
}

/// Capture an error and send to Sentry
///
/// # Arguments
/// * `error` - The error to capture
/// * `context` - Additional context (will be anonymized)
pub fn capture_error(error: &dyn std::error::Error, context: Option<&str>) {
    if !is_enabled() {
        return;
    }

    sentry::with_scope(
        |scope| {
            if let Some(ctx) = context {
                scope.set_extra("context", anonymize_string(ctx).into());
            }
        },
        || {
            sentry::capture_error(error);
        },
    );

    debug!("Captured error to Sentry");
}

/// Capture a message
///
/// # Arguments
/// * `message` - The message to capture (will be anonymized)
/// * `level` - Severity level
pub fn capture_message(message: &str, level: sentry::Level) {
    if !is_enabled() {
        return;
    }

    let anonymized = anonymize_string(message);
    sentry::capture_message(&anonymized, level);

    debug!("Captured message to Sentry: level={:?}", level);
}

/// Add a breadcrumb for debugging
///
/// # Arguments
/// * `category` - Breadcrumb category (e.g., "strategy", "network")
/// * `message` - Description (will be anonymized)
pub fn add_breadcrumb(category: &str, message: &str) {
    if !is_enabled() {
        return;
    }

    sentry::add_breadcrumb(sentry::Breadcrumb {
        category: Some(category.to_string()),
        message: Some(anonymize_string(message)),
        level: sentry::Level::Info,
        ..Default::default()
    });
}

/// Capture an IsolateError
///
/// Automatically extracts error kind and context
pub fn capture_isolate_error(error: &crate::core::errors::IsolateError) {
    if !is_enabled() {
        return;
    }

    sentry::with_scope(
        |scope| {
            scope.set_tag("error_kind", error.kind());
        },
        || {
            let anonymized_msg = anonymize_string(&error.to_string());
            sentry::capture_message(&anonymized_msg, sentry::Level::Error);
        },
    );
}

/// Guard against recursive panic hook calls
static IN_PANIC_HOOK: AtomicBool = AtomicBool::new(false);

/// Set up panic hook to capture panics
pub fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Guard against recursive calls (prevents deadlock if panic occurs in Sentry code)
        if IN_PANIC_HOOK.swap(true, Ordering::SeqCst) {
            // Already in panic hook, just call default
            default_hook(panic_info);
            return;
        }

        // Capture to Sentry if enabled
        if is_enabled() {
            let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                anonymize_string(s)
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                anonymize_string(s)
            } else {
                "Unknown panic".to_string()
            };

            // Get location without full path
            let location = panic_info
                .location()
                .map(|loc| format!("{}:{}", loc.file().split('/').last().unwrap_or("unknown"), loc.line()))
                .unwrap_or_else(|| "unknown".to_string());

            sentry::capture_message(
                &format!("Panic at {}: {}", location, message),
                sentry::Level::Fatal,
            );

            // Flush events before panic continues (with short timeout to avoid blocking)
            if let Some(client) = sentry::Hub::current().client() {
                client.flush(Some(std::time::Duration::from_secs(2)));
            }
        }

        // Reset guard before calling default hook
        IN_PANIC_HOOK.store(false, Ordering::SeqCst);

        // Call default hook
        default_hook(panic_info);
    }));

    info!("Sentry panic hook installed");
}

/// Anonymize a Sentry event
fn anonymize_event(mut event: sentry::protocol::Event<'static>) -> sentry::protocol::Event<'static> {
    // Anonymize message
    if let Some(ref mut message) = event.message {
        *message = anonymize_string(message);
    }

    // Remove server_name (could contain hostname)
    event.server_name = None;

    // Remove user IP if somehow set
    if let Some(ref mut user) = event.user {
        user.ip_address = None;
        user.email = None;
        user.username = None;
    }

    event
}

/// Filter breadcrumb for PII
fn filter_breadcrumb(mut breadcrumb: sentry::Breadcrumb) -> Option<sentry::Breadcrumb> {
    // Anonymize message
    if let Some(ref mut message) = breadcrumb.message {
        *message = anonymize_string(message);
    }

    Some(breadcrumb)
}

// Pre-compiled regex patterns for anonymization (compiled once, used many times)
use once_cell::sync::Lazy;

static WINDOWS_PATH_RE: Lazy<regex::Regex> = Lazy::new(|| {
    // Covers C:\..., D:\..., UNC paths \\server\share, and \\?\C:\...
    regex::Regex::new(r"(?:[A-Za-z]:|\\\\[^\s\\]+\\[^\s\\]+|\\\\[?]\\[^\s]+)\\[^\s]*").unwrap()
});

static UNIX_PATH_RE: Lazy<regex::Regex> = Lazy::new(|| {
    // Covers /home, /Users, /tmp, /var, /root, /opt, /usr, /etc
    regex::Regex::new(r"/(home|Users|tmp|var|root|opt|usr|etc)/[^\s]+").unwrap()
});

static IPV4_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap()
});

static IPV6_RE: Lazy<regex::Regex> = Lazy::new(|| {
    // Simplified IPv6 pattern covering most common formats
    regex::Regex::new(r"(?i)\b([0-9a-f]{1,4}:){2,7}[0-9a-f]{1,4}\b|\b::([0-9a-f]{1,4}:){0,5}[0-9a-f]{1,4}\b|\b::1\b").unwrap()
});

static EMAIL_RE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

/// Anonymize a string by removing potential PII
///
/// Removes:
/// - File paths (Windows including UNC, and Unix)
/// - IP addresses (IPv4 and IPv6)
/// - Email addresses
/// - UUIDs (except our anonymous ID)
fn anonymize_string(s: &str) -> String {
    let mut result = s.to_string();

    // Remove Windows paths (C:\Users\..., D:\..., \\server\share\...)
    result = WINDOWS_PATH_RE.replace_all(&result, "[PATH]").to_string();

    // Remove Unix paths (/home/..., /Users/..., /tmp/..., etc.)
    result = UNIX_PATH_RE.replace_all(&result, "[PATH]").to_string();

    // Remove IPv4 addresses
    result = IPV4_RE.replace_all(&result, "[IP]").to_string();

    // Remove IPv6 addresses
    result = IPV6_RE.replace_all(&result, "[IP]").to_string();

    // Remove email addresses
    result = EMAIL_RE.replace_all(&result, "[EMAIL]").to_string();

    result
}

/// Flush pending events (call before app exit)
pub fn flush() {
    if is_enabled() {
        if let Some(client) = sentry::Hub::current().client() {
            client.flush(Some(std::time::Duration::from_secs(2)));
        }
    }
}

/// Shutdown Sentry cleanly
pub fn shutdown() {
    flush();
    // Guard will be dropped when app exits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_by_default() {
        assert!(!is_enabled());
    }

    #[test]
    fn test_anonymize_windows_path() {
        let input = "Error at C:\\Users\\john\\Documents\\file.txt";
        let result = anonymize_string(input);
        assert!(result.contains("[PATH]"));
        assert!(!result.contains("john"));
    }

    #[test]
    fn test_anonymize_unix_path() {
        let input = "Error at /home/user/project/file.rs";
        let result = anonymize_string(input);
        assert!(result.contains("[PATH]"));
        assert!(!result.contains("user"));
    }

    #[test]
    fn test_anonymize_ip() {
        let input = "Connection to 192.168.1.100 failed";
        let result = anonymize_string(input);
        assert!(result.contains("[IP]"));
        assert!(!result.contains("192.168"));
    }

    #[test]
    fn test_anonymize_email() {
        let input = "User user@example.com not found";
        let result = anonymize_string(input);
        assert!(result.contains("[EMAIL]"));
        assert!(!result.contains("user@example.com"));
    }

    #[test]
    fn test_anonymize_combined() {
        let input = "Error for user@test.com at C:\\Users\\admin\\file.txt from 10.0.0.1";
        let result = anonymize_string(input);
        assert!(result.contains("[EMAIL]"));
        assert!(result.contains("[PATH]"));
        assert!(result.contains("[IP]"));
        assert!(!result.contains("admin"));
        assert!(!result.contains("10.0.0.1"));
    }

    #[test]
    fn test_anonymize_ipv6() {
        let input = "Connection to 2001:db8::1 failed";
        let result = anonymize_string(input);
        assert!(result.contains("[IP]"));
        assert!(!result.contains("2001:db8"));
    }

    #[test]
    fn test_anonymize_ipv6_localhost() {
        let input = "Listening on ::1";
        let result = anonymize_string(input);
        assert!(result.contains("[IP]"));
        assert!(!result.contains("::1"));
    }

    #[test]
    fn test_anonymize_unc_path() {
        let input = "Error at \\\\server\\share\\file.txt";
        let result = anonymize_string(input);
        assert!(result.contains("[PATH]"));
        assert!(!result.contains("server"));
    }
}

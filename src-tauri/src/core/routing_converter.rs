//! Routing Rules Converter
//!
//! Converts high-level RoutingRule (from UI) to low-level DomainRoute/AppRoute
//! for sing-box configuration generation.
//!
//! ## Action Handling
//!
//! - `direct` → DomainRoute with proxy_id = "direct"
//! - `proxy` → DomainRoute with proxy_id from rule
//! - `block` → DomainRoute with proxy_id = "block"
//! - `dpi-bypass` → DomainRoute with proxy_id = "direct" + domain added to dpi_bypass_domains
//!
//! Note: DPI bypass works at WinDivert level (winws), not at sing-box level.
//! For dpi-bypass action, traffic goes direct through sing-box, while winws
//! modifies packets at kernel level using hostlist.

use crate::core::models::{AppRoute, DomainRoute};
use crate::core::storage::RoutingRule;
use tracing::debug;

/// Result of routing rules conversion
#[derive(Debug, Clone, Default)]
pub struct ConvertedRoutes {
    /// Domain-based routes for sing-box
    pub domain_routes: Vec<DomainRoute>,
    /// Application-based routes for sing-box
    pub app_routes: Vec<AppRoute>,
    /// Domains that need DPI bypass (for winws hostlist)
    pub dpi_bypass_domains: Vec<String>,
}

/// Convert RoutingRule array to sing-box compatible routes
///
/// # Arguments
/// * `rules` - High-level routing rules from UI/storage
///
/// # Returns
/// ConvertedRoutes containing domain_routes, app_routes, and dpi_bypass_domains
///
/// # Example
/// ```ignore
/// let rules = storage.get_routing_rules().await?;
/// let converted = convert_routing_rules(&rules);
/// 
/// // Use domain_routes and app_routes for sing-box config
/// let config = generate_singbox_config(proxies, &converted.domain_routes, &converted.app_routes, ...);
/// 
/// // Use dpi_bypass_domains for winws hostlist
/// create_dynamic_hostlist(&converted.dpi_bypass_domains).await?;
/// ```
pub fn convert_routing_rules(rules: &[RoutingRule]) -> ConvertedRoutes {
    let mut result = ConvertedRoutes::default();

    for rule in rules.iter().filter(|r| r.enabled) {
        match rule.source.as_str() {
            "domain" => {
                if let Some(ref domain) = rule.source_value {
                    let proxy_id = get_proxy_id_for_action(&rule.action, rule.proxy_id.as_deref());
                    
                    result.domain_routes.push(DomainRoute {
                        domain: normalize_domain(domain),
                        proxy_id,
                    });

                    // Track dpi-bypass domains for winws hostlist
                    if rule.action == "dpi-bypass" {
                        result.dpi_bypass_domains.push(normalize_domain(domain));
                    }

                    debug!(
                        rule_id = %rule.id,
                        domain = %domain,
                        action = %rule.action,
                        "Converted domain routing rule"
                    );
                }
            }
            "app" => {
                if let Some(ref app_path) = rule.source_value {
                    let proxy_id = get_proxy_id_for_action(&rule.action, rule.proxy_id.as_deref());
                    let app_name = extract_app_name(app_path);

                    result.app_routes.push(AppRoute {
                        app_name,
                        app_path: app_path.clone(),
                        proxy_id,
                    });

                    debug!(
                        rule_id = %rule.id,
                        app_path = %app_path,
                        action = %rule.action,
                        "Converted app routing rule"
                    );
                }
            }
            "ip" => {
                // IP-based routing is handled differently in sing-box
                // For now, we skip it as it requires ip_cidr rules
                debug!(
                    rule_id = %rule.id,
                    action = %rule.action,
                    "Skipping IP-based routing rule (not yet implemented)"
                );
            }
            "all" => {
                // "all" source means default route - handled by sing-box's "final" field
                debug!(
                    rule_id = %rule.id,
                    action = %rule.action,
                    "Skipping 'all' source rule (handled by default route)"
                );
            }
            _ => {
                debug!(
                    rule_id = %rule.id,
                    source = %rule.source,
                    "Unknown routing rule source type"
                );
            }
        }
    }

    debug!(
        domain_routes = result.domain_routes.len(),
        app_routes = result.app_routes.len(),
        dpi_bypass_domains = result.dpi_bypass_domains.len(),
        "Routing rules converted"
    );

    result
}

/// Get proxy_id for sing-box based on action type
fn get_proxy_id_for_action(action: &str, proxy_id: Option<&str>) -> String {
    match action {
        "direct" => "direct".to_string(),
        "block" => "block".to_string(),
        "dpi-bypass" => {
            // DPI bypass traffic goes direct through sing-box
            // winws handles the actual DPI bypass at kernel level
            "direct".to_string()
        }
        "proxy" => {
            // Use specified proxy_id or fallback to first available proxy
            proxy_id.unwrap_or("direct").to_string()
        }
        _ => "direct".to_string(),
    }
}

/// Normalize domain for consistent matching
fn normalize_domain(domain: &str) -> String {
    let domain = domain.trim().to_lowercase();

    // Remove protocol if present
    let domain = domain
        .strip_prefix("https://")
        .or_else(|| domain.strip_prefix("http://"))
        .unwrap_or(&domain);

    // Remove path if present
    let domain = domain.split('/').next().unwrap_or(domain);

    // Remove port if present
    let domain = domain.split(':').next().unwrap_or(domain);

    domain.to_string()
}

/// Extract application name from path
fn extract_app_name(app_path: &str) -> String {
    std::path::Path::new(app_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rule(
        id: &str,
        source: &str,
        source_value: Option<&str>,
        action: &str,
        proxy_id: Option<&str>,
    ) -> RoutingRule {
        RoutingRule {
            id: id.to_string(),
            name: format!("Test Rule {}", id),
            enabled: true,
            source: source.to_string(),
            source_value: source_value.map(|s| s.to_string()),
            action: action.to_string(),
            proxy_id: proxy_id.map(|s| s.to_string()),
            priority: 0,
        }
    }

    #[test]
    fn test_convert_direct_rule() {
        let rules = vec![create_test_rule(
            "1",
            "domain",
            Some("example.com"),
            "direct",
            None,
        )];

        let result = convert_routing_rules(&rules);

        assert_eq!(result.domain_routes.len(), 1);
        assert_eq!(result.domain_routes[0].domain, "example.com");
        assert_eq!(result.domain_routes[0].proxy_id, "direct");
        assert!(result.dpi_bypass_domains.is_empty());
    }

    #[test]
    fn test_convert_proxy_rule() {
        let rules = vec![create_test_rule(
            "1",
            "domain",
            Some("discord.com"),
            "proxy",
            Some("vless-1"),
        )];

        let result = convert_routing_rules(&rules);

        assert_eq!(result.domain_routes.len(), 1);
        assert_eq!(result.domain_routes[0].domain, "discord.com");
        assert_eq!(result.domain_routes[0].proxy_id, "vless-1");
    }

    #[test]
    fn test_convert_block_rule() {
        let rules = vec![create_test_rule(
            "1",
            "domain",
            Some("ads.example.com"),
            "block",
            None,
        )];

        let result = convert_routing_rules(&rules);

        assert_eq!(result.domain_routes.len(), 1);
        assert_eq!(result.domain_routes[0].proxy_id, "block");
    }

    #[test]
    fn test_convert_dpi_bypass_rule() {
        let rules = vec![create_test_rule(
            "1",
            "domain",
            Some("youtube.com"),
            "dpi-bypass",
            None,
        )];

        let result = convert_routing_rules(&rules);

        assert_eq!(result.domain_routes.len(), 1);
        assert_eq!(result.domain_routes[0].domain, "youtube.com");
        assert_eq!(result.domain_routes[0].proxy_id, "direct");
        
        // DPI bypass domain should be tracked
        assert_eq!(result.dpi_bypass_domains.len(), 1);
        assert_eq!(result.dpi_bypass_domains[0], "youtube.com");
    }

    #[test]
    fn test_convert_app_rule() {
        let rules = vec![create_test_rule(
            "1",
            "app",
            Some("C:\\Program Files\\Discord\\Discord.exe"),
            "proxy",
            Some("vless-1"),
        )];

        let result = convert_routing_rules(&rules);

        assert_eq!(result.app_routes.len(), 1);
        assert_eq!(result.app_routes[0].app_name, "Discord");
        assert_eq!(result.app_routes[0].proxy_id, "vless-1");
    }

    #[test]
    fn test_disabled_rules_ignored() {
        let mut rule = create_test_rule("1", "domain", Some("example.com"), "direct", None);
        rule.enabled = false;

        let result = convert_routing_rules(&[rule]);

        assert!(result.domain_routes.is_empty());
    }

    #[test]
    fn test_normalize_domain() {
        assert_eq!(normalize_domain("YouTube.com"), "youtube.com");
        assert_eq!(normalize_domain("https://youtube.com/watch"), "youtube.com");
        assert_eq!(normalize_domain("youtube.com:443"), "youtube.com");
        assert_eq!(normalize_domain("  youtube.com  "), "youtube.com");
    }

    #[test]
    fn test_mixed_rules() {
        let rules = vec![
            create_test_rule("1", "domain", Some("youtube.com"), "dpi-bypass", None),
            create_test_rule("2", "domain", Some("discord.com"), "proxy", Some("vless-1")),
            create_test_rule("3", "domain", Some("ads.com"), "block", None),
            create_test_rule("4", "app", Some("/usr/bin/firefox"), "direct", None),
        ];

        let result = convert_routing_rules(&rules);

        assert_eq!(result.domain_routes.len(), 3);
        assert_eq!(result.app_routes.len(), 1);
        assert_eq!(result.dpi_bypass_domains.len(), 1);
        assert_eq!(result.dpi_bypass_domains[0], "youtube.com");
    }
}

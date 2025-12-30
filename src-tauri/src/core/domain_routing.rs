//! Domain-based routing module
//!
//! Manages per-domain routing rules for sing-box configuration

use crate::core::models::DomainRoute;
use crate::core::storage::Storage;
use crate::core::errors::{IsolateError, Result};
use std::sync::Arc;
use tracing::{debug, info};

// ============================================================================
// DomainRouter
// ============================================================================

/// Domain-based routing manager
pub struct DomainRouter {
    storage: Arc<Storage>,
}

impl DomainRouter {
    /// Create a new domain router
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Add a domain route
    pub async fn add_route(&self, domain: &str, proxy_id: &str) -> Result<()> {
        // Validate domain
        let domain = Self::validate_domain(domain)?;
        
        // Validate proxy_id is not empty
        if proxy_id.trim().is_empty() {
            return Err(IsolateError::Validation("proxy_id cannot be empty".into()));
        }

        let route = DomainRoute {
            domain: domain.clone(),
            proxy_id: proxy_id.to_string(),
        };

        self.storage.save_domain_route(&route)?;
        info!(domain = %domain, proxy_id = %proxy_id, "Domain route added");
        
        Ok(())
    }

    /// Remove a domain route
    pub async fn remove_route(&self, domain: &str) -> Result<()> {
        let domain = Self::normalize_domain(domain);
        self.storage.delete_domain_route(&domain)?;
        info!(domain = %domain, "Domain route removed");
        Ok(())
    }

    /// Get all domain routes
    pub async fn get_routes(&self) -> Result<Vec<DomainRoute>> {
        self.storage.get_domain_routes()
    }

    /// Generate sing-box routing rules from domain routes
    pub fn generate_rules(&self, routes: &[DomainRoute]) -> Vec<serde_json::Value> {
        let mut rules = Vec::new();

        // Group domains by proxy_id for efficient rules
        let mut proxy_domains: std::collections::HashMap<String, Vec<String>> = 
            std::collections::HashMap::new();

        for route in routes {
            proxy_domains
                .entry(route.proxy_id.clone())
                .or_default()
                .push(route.domain.clone());
        }

        // Generate rule for each proxy
        for (proxy_id, domains) in proxy_domains {
            let rule = serde_json::json!({
                "domain_suffix": domains,
                "outbound": proxy_id
            });
            rules.push(rule);
            debug!(proxy_id = %proxy_id, domains_count = domains.len(), "Generated domain rule");
        }

        rules
    }

    /// Check if a domain matches any route
    pub async fn find_route(&self, domain: &str) -> Result<Option<DomainRoute>> {
        let routes = self.get_routes().await?;
        let domain = Self::normalize_domain(domain);

        // Check exact match first
        if let Some(route) = routes.iter().find(|r| r.domain == domain) {
            return Ok(Some(route.clone()));
        }

        // Check suffix match (e.g., "youtube.com" matches "www.youtube.com")
        for route in &routes {
            if domain.ends_with(&format!(".{}", route.domain)) {
                return Ok(Some(route.clone()));
            }
        }

        Ok(None)
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Validate and normalize domain
    fn validate_domain(domain: &str) -> Result<String> {
        let domain = Self::normalize_domain(domain);

        if domain.is_empty() {
            return Err(IsolateError::Validation("Domain cannot be empty".into()));
        }

        // Basic domain validation
        if domain.contains(' ') {
            return Err(IsolateError::Validation("Domain cannot contain spaces".into()));
        }

        // Check for valid characters
        let valid_chars = domain.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'
        });

        if !valid_chars {
            return Err(IsolateError::Validation(
                "Domain contains invalid characters".into()
            ));
        }

        // Check domain structure
        if domain.starts_with('.') || domain.ends_with('.') {
            return Err(IsolateError::Validation(
                "Domain cannot start or end with a dot".into()
            ));
        }

        if domain.contains("..") {
            return Err(IsolateError::Validation(
                "Domain cannot contain consecutive dots".into()
            ));
        }

        Ok(domain)
    }

    /// Normalize domain (lowercase, trim, remove protocol)
    fn normalize_domain(domain: &str) -> String {
        let domain = domain.trim().to_lowercase();
        
        // Remove protocol if present
        let domain = domain
            .strip_prefix("https://")
            .or_else(|| domain.strip_prefix("http://"))
            .unwrap_or(&domain);

        // Remove path if present
        let domain = domain.split('/').next().unwrap_or(&domain);

        // Remove port if present
        let domain = domain.split(':').next().unwrap_or(domain);

        domain.to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_domain() {
        assert_eq!(DomainRouter::normalize_domain("YouTube.com"), "youtube.com");
        assert_eq!(DomainRouter::normalize_domain("https://youtube.com/watch"), "youtube.com");
        assert_eq!(DomainRouter::normalize_domain("youtube.com:443"), "youtube.com");
        assert_eq!(DomainRouter::normalize_domain("  youtube.com  "), "youtube.com");
    }

    #[test]
    fn test_validate_domain() {
        assert!(DomainRouter::validate_domain("youtube.com").is_ok());
        assert!(DomainRouter::validate_domain("www.youtube.com").is_ok());
        assert!(DomainRouter::validate_domain("sub-domain.example.com").is_ok());
        
        assert!(DomainRouter::validate_domain("").is_err());
        assert!(DomainRouter::validate_domain("invalid domain").is_err());
        assert!(DomainRouter::validate_domain(".youtube.com").is_err());
        assert!(DomainRouter::validate_domain("youtube..com").is_err());
    }

    #[test]
    fn test_generate_rules() {
        let storage = Arc::new(Storage::new().unwrap());
        let router = DomainRouter::new(storage);

        let routes = vec![
            DomainRoute {
                domain: "youtube.com".to_string(),
                proxy_id: "proxy1".to_string(),
            },
            DomainRoute {
                domain: "google.com".to_string(),
                proxy_id: "proxy1".to_string(),
            },
            DomainRoute {
                domain: "discord.com".to_string(),
                proxy_id: "proxy2".to_string(),
            },
        ];

        let rules = router.generate_rules(&routes);
        assert_eq!(rules.len(), 2); // Two proxies
    }
}

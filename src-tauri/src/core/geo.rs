//! Proxy geolocation module
//!
//! Provides functionality to detect the country of a proxy server
//! using IP geolocation APIs with caching support.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache entry for geo lookup results
#[derive(Debug, Clone)]
struct GeoCacheEntry {
    country_code: Option<String>,
    timestamp: Instant,
}

/// Geo lookup cache with TTL
pub struct GeoCache {
    cache: RwLock<HashMap<String, GeoCacheEntry>>,
    ttl: Duration,
}

impl GeoCache {
    /// Create new cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// Get cached country code for IP
    pub async fn get(&self, ip: &str) -> Option<Option<String>> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(ip) {
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.country_code.clone());
            }
        }
        None
    }

    /// Store country code in cache
    pub async fn set(&self, ip: String, country_code: Option<String>) {
        let mut cache = self.cache.write().await;
        cache.insert(
            ip,
            GeoCacheEntry {
                country_code,
                timestamp: Instant::now(),
            },
        );
    }

    /// Clear expired entries
    pub async fn cleanup(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, entry| entry.timestamp.elapsed() < self.ttl);
    }

    /// Get cache size
    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }
}

/// Global geo cache instance
static GEO_CACHE: std::sync::OnceLock<Arc<GeoCache>> = std::sync::OnceLock::new();

/// Get or initialize the global geo cache
pub fn get_geo_cache() -> Arc<GeoCache> {
    GEO_CACHE
        .get_or_init(|| {
            // Cache TTL: 24 hours
            Arc::new(GeoCache::new(Duration::from_secs(24 * 60 * 60)))
        })
        .clone()
}

/// Response from ip-api.com
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    country: Option<String>,
    message: Option<String>,
}

/// Detect country of a proxy server by its IP/hostname
///
/// # Arguments
/// * `proxy_server` - Server address (IP or hostname)
///
/// # Returns
/// * `Ok(Some(code))` - ISO 3166-1 alpha-2 country code (e.g., "US", "RU", "DE")
/// * `Ok(None)` - Could not determine country
/// * `Err(_)` - Error during lookup
///
/// # Example
/// ```ignore
/// let country = detect_proxy_country("1.2.3.4").await?;
/// assert_eq!(country, Some("US".to_string()));
/// ```
pub async fn detect_proxy_country(proxy_server: &str) -> Result<Option<String>> {
    // Resolve hostname to IP if needed
    let ip = resolve_to_ip(proxy_server).await?;
    
    // Check cache first
    let cache = get_geo_cache();
    if let Some(cached) = cache.get(&ip).await {
        debug!(ip = %ip, cached = ?cached, "Using cached geo result");
        return Ok(cached);
    }

    // Query ip-api.com (free tier: 45 requests/minute)
    let result = query_ip_api(&ip).await;
    
    // Cache the result (even if None)
    match &result {
        Ok(country_code) => {
            cache.set(ip.clone(), country_code.clone()).await;
            info!(ip = %ip, country = ?country_code, "Geo lookup successful");
        }
        Err(e) => {
            warn!(ip = %ip, error = %e, "Geo lookup failed");
            // Cache None to avoid repeated failed requests
            cache.set(ip.clone(), None).await;
        }
    }

    result
}

/// Resolve hostname to IP address
async fn resolve_to_ip(server: &str) -> Result<String> {
    // Check if already an IP
    if server.parse::<IpAddr>().is_ok() {
        return Ok(server.to_string());
    }

    // Resolve hostname
    let addrs = tokio::net::lookup_host(format!("{}:0", server))
        .await
        .map_err(|e| anyhow!("Failed to resolve hostname '{}': {}", server, e))?;

    // Get first IP (prefer IPv4)
    let mut ipv4 = None;
    let mut ipv6 = None;

    for addr in addrs {
        match addr.ip() {
            IpAddr::V4(ip) => {
                if ipv4.is_none() {
                    ipv4 = Some(ip.to_string());
                }
            }
            IpAddr::V6(ip) => {
                if ipv6.is_none() {
                    ipv6 = Some(ip.to_string());
                }
            }
        }
    }

    ipv4.or(ipv6)
        .ok_or_else(|| anyhow!("No IP addresses found for hostname '{}'", server))
}

/// Query ip-api.com for geolocation
async fn query_ip_api(ip: &str) -> Result<Option<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

    // ip-api.com free API (no key required, 45 req/min limit)
    // Using fields parameter to minimize response size
    let url = format!(
        "http://ip-api.com/json/{}?fields=status,countryCode,country,message",
        ip
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("HTTP error: {}", response.status()));
    }

    let data: IpApiResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    if data.status == "success" {
        Ok(data.country_code)
    } else {
        let msg = data.message.unwrap_or_else(|| "Unknown error".to_string());
        debug!(ip = %ip, message = %msg, "ip-api.com returned failure status");
        Ok(None)
    }
}

/// Batch detect countries for multiple proxies
///
/// # Arguments
/// * `servers` - List of server addresses
///
/// # Returns
/// HashMap of server -> country code
pub async fn detect_proxy_countries_batch(
    servers: &[String],
) -> HashMap<String, Option<String>> {
    let mut results = HashMap::new();

    // Process in parallel with rate limiting (max 10 concurrent)
    let semaphore = Arc::new(tokio::sync::Semaphore::new(10));
    let mut handles = Vec::new();

    for server in servers {
        let server = server.clone();
        let permit = semaphore.clone().acquire_owned().await;

        let handle = tokio::spawn(async move {
            let _permit = permit;
            let result = detect_proxy_country(&server).await.unwrap_or(None);
            (server, result)
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Ok((server, country)) = handle.await {
            results.insert(server, country);
        }
    }

    results
}

/// Get country name from ISO code
pub fn country_code_to_name(code: &str) -> Option<&'static str> {
    match code.to_uppercase().as_str() {
        "US" => Some("United States"),
        "RU" => Some("Russia"),
        "DE" => Some("Germany"),
        "NL" => Some("Netherlands"),
        "GB" => Some("United Kingdom"),
        "FR" => Some("France"),
        "JP" => Some("Japan"),
        "SG" => Some("Singapore"),
        "HK" => Some("Hong Kong"),
        "KR" => Some("South Korea"),
        "CA" => Some("Canada"),
        "AU" => Some("Australia"),
        "CH" => Some("Switzerland"),
        "SE" => Some("Sweden"),
        "FI" => Some("Finland"),
        "NO" => Some("Norway"),
        "PL" => Some("Poland"),
        "UA" => Some("Ukraine"),
        "TR" => Some("Turkey"),
        "IN" => Some("India"),
        "BR" => Some("Brazil"),
        "AR" => Some("Argentina"),
        "MX" => Some("Mexico"),
        "ZA" => Some("South Africa"),
        "AE" => Some("United Arab Emirates"),
        "IL" => Some("Israel"),
        "TW" => Some("Taiwan"),
        "VN" => Some("Vietnam"),
        "TH" => Some("Thailand"),
        "ID" => Some("Indonesia"),
        "MY" => Some("Malaysia"),
        "PH" => Some("Philippines"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_code_to_name() {
        assert_eq!(country_code_to_name("US"), Some("United States"));
        assert_eq!(country_code_to_name("us"), Some("United States"));
        assert_eq!(country_code_to_name("RU"), Some("Russia"));
        assert_eq!(country_code_to_name("XX"), None);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let cache = GeoCache::new(Duration::from_secs(60));
        
        // Initially empty
        assert!(cache.get("1.2.3.4").await.is_none());
        
        // Set and get
        cache.set("1.2.3.4".to_string(), Some("US".to_string())).await;
        assert_eq!(cache.get("1.2.3.4").await, Some(Some("US".to_string())));
        
        // Cache None
        cache.set("5.6.7.8".to_string(), None).await;
        assert_eq!(cache.get("5.6.7.8").await, Some(None));
        
        // Size
        assert_eq!(cache.len().await, 2);
    }

    #[tokio::test]
    async fn test_resolve_ip_passthrough() {
        // IP should pass through unchanged
        let result = resolve_to_ip("8.8.8.8").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8.8.8.8");
    }
}

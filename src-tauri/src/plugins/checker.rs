use serde::{Deserialize, Serialize};
use crate::plugins::manifest::ServiceEndpoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointCheckResult {
    pub url: String,
    pub name: String,
    pub accessible: bool,
    pub latency_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service_id: String,
    pub accessible: bool,
    pub avg_latency_ms: Option<u64>,
    pub endpoints: Vec<EndpointCheckResult>,
    pub last_check: String,
    pub error: Option<String>,
}

/// Check a single endpoint
pub async fn check_endpoint(endpoint: &ServiceEndpoint) -> EndpointCheckResult {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    let start = std::time::Instant::now();
    
    let result = match endpoint.method.to_uppercase().as_str() {
        "HEAD" => client.head(&endpoint.url).send().await,
        _ => client.get(&endpoint.url).send().await,
    };

    match result {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;
            let status = response.status().as_u16();
            let accessible = response.status().is_success() || response.status().is_redirection();
            
            EndpointCheckResult {
                url: endpoint.url.clone(),
                name: endpoint.name.clone(),
                accessible,
                latency_ms: Some(latency),
                status_code: Some(status),
                error: None,
            }
        }
        Err(e) => EndpointCheckResult {
            url: endpoint.url.clone(),
            name: endpoint.name.clone(),
            accessible: false,
            latency_ms: None,
            status_code: None,
            error: Some(e.to_string()),
        }
    }
}

/// Check all endpoints for a service
pub async fn check_service_endpoints(endpoints: &[ServiceEndpoint]) -> ServiceStatus {
    let mut results = Vec::new();
    
    for endpoint in endpoints {
        results.push(check_endpoint(endpoint).await);
    }
    
    let accessible_count = results.iter().filter(|r| r.accessible).count();
    let accessible = accessible_count > 0;
    
    let avg_latency = if accessible_count > 0 {
        let total: u64 = results.iter()
            .filter_map(|r| r.latency_ms)
            .sum();
        Some(total / accessible_count as u64)
    } else {
        None
    };
    
    ServiceStatus {
        service_id: String::new(),
        accessible,
        avg_latency_ms: avg_latency,
        endpoints: results,
        last_check: chrono::Utc::now().to_rfc3339(),
        error: None,
    }
}

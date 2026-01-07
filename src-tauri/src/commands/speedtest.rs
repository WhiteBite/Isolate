//! Speed test commands - upload/download speed measurement via Cloudflare
//! 
//! Upload test is done via Tauri backend to bypass CORS restrictions.

use crate::commands::rate_limiter;
use crate::core::errors::{IsolateError, TypedResultExt};
use tracing::info;

const CLOUDFLARE_UP_URL: &str = "https://speed.cloudflare.com/__up";
const UPLOAD_SIZE: usize = 5_000_000; // 5MB

/// Result of upload speed test
#[derive(Debug, Clone, serde::Serialize)]
pub struct UploadTestResult {
    pub speed_mbps: f64,
    pub bytes_sent: usize,
    pub duration_ms: u64,
}

/// Test upload speed via Cloudflare
/// 
/// This bypasses CORS by making the request from Rust backend.
#[tauri::command]
pub async fn test_upload_speed() -> Result<UploadTestResult, IsolateError> {
    // Rate limit: max once per 60 seconds
    rate_limiter::check_rate_limit("test_upload_speed", 60)?;
    
    info!("Starting upload speed test");
    
    // Generate random data
    let data: Vec<u8> = (0..UPLOAD_SIZE).map(|_| rand::random::<u8>()).collect();
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .network_context("Failed to create HTTP client")?;
    
    let start = std::time::Instant::now();
    
    let response = client
        .post(CLOUDFLARE_UP_URL)
        .body(data)
        .send()
        .await
        .network_context("Upload failed")?;
    
    if !response.status().is_success() {
        return Err(IsolateError::Network(format!("Upload failed with status: {}", response.status())));
    }
    
    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;
    let duration_secs = duration.as_secs_f64();
    
    // Calculate speed: (bytes * 8 bits) / seconds / 1_000_000 = Mbps
    let speed_mbps = (UPLOAD_SIZE as f64 * 8.0) / duration_secs / 1_000_000.0;
    
    info!(
        speed_mbps = format!("{:.1}", speed_mbps),
        duration_ms,
        "Upload speed test completed"
    );
    
    Ok(UploadTestResult {
        speed_mbps,
        bytes_sent: UPLOAD_SIZE,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_result_serialization() {
        let result = UploadTestResult {
            speed_mbps: 100.5,
            bytes_sent: 5_000_000,
            duration_ms: 400,
        };

        let json = serde_json::to_string(&result).expect("Should serialize");
        
        assert!(json.contains("\"speed_mbps\":100.5"));
        assert!(json.contains("\"bytes_sent\":5000000"));
        assert!(json.contains("\"duration_ms\":400"));
    }

    #[test]
    fn test_upload_result_deserialization_roundtrip() {
        let original = UploadTestResult {
            speed_mbps: 50.25,
            bytes_sent: 1_000_000,
            duration_ms: 160,
        };

        let json = serde_json::to_string(&original).expect("Should serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should parse as JSON");

        assert_eq!(parsed["speed_mbps"], 50.25);
        assert_eq!(parsed["bytes_sent"], 1_000_000);
        assert_eq!(parsed["duration_ms"], 160);
    }

    #[test]
    fn test_constants_are_valid() {
        // UPLOAD_SIZE should be 5MB
        assert_eq!(UPLOAD_SIZE, 5_000_000);
        
        // URL should be Cloudflare speed test endpoint
        assert_eq!(CLOUDFLARE_UP_URL, "https://speed.cloudflare.com/__up");
        assert!(CLOUDFLARE_UP_URL.starts_with("https://"));
    }

    #[test]
    fn test_speed_calculation_formula() {
        // Verify the speed calculation formula:
        // speed_mbps = (bytes * 8 bits) / seconds / 1_000_000
        
        let bytes = 5_000_000usize;
        let duration_secs = 1.0f64; // 1 second
        
        let speed_mbps = (bytes as f64 * 8.0) / duration_secs / 1_000_000.0;
        
        // 5MB in 1 second = 40 Mbps
        assert!((speed_mbps - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_speed_calculation_various_durations() {
        let bytes = UPLOAD_SIZE;
        
        // 0.5 seconds = 80 Mbps
        let speed_half_sec = (bytes as f64 * 8.0) / 0.5 / 1_000_000.0;
        assert!((speed_half_sec - 80.0).abs() < 0.001);
        
        // 2 seconds = 20 Mbps
        let speed_two_sec = (bytes as f64 * 8.0) / 2.0 / 1_000_000.0;
        assert!((speed_two_sec - 20.0).abs() < 0.001);
        
        // 0.1 seconds = 400 Mbps
        let speed_fast = (bytes as f64 * 8.0) / 0.1 / 1_000_000.0;
        assert!((speed_fast - 400.0).abs() < 0.001);
    }

    #[test]
    fn test_upload_result_debug_impl() {
        let result = UploadTestResult {
            speed_mbps: 100.0,
            bytes_sent: 5_000_000,
            duration_ms: 400,
        };
        
        let debug_str = format!("{:?}", result);
        
        assert!(debug_str.contains("UploadTestResult"));
        assert!(debug_str.contains("speed_mbps"));
        assert!(debug_str.contains("bytes_sent"));
        assert!(debug_str.contains("duration_ms"));
    }

    #[test]
    fn test_upload_result_clone() {
        let original = UploadTestResult {
            speed_mbps: 75.5,
            bytes_sent: 5_000_000,
            duration_ms: 530,
        };
        
        let cloned = original.clone();
        
        assert!((cloned.speed_mbps - original.speed_mbps).abs() < f64::EPSILON);
        assert_eq!(cloned.bytes_sent, original.bytes_sent);
        assert_eq!(cloned.duration_ms, original.duration_ms);
    }

    /// Integration test - requires network access
    /// Run with: cargo test test_real_upload -- --ignored
    #[test]
    #[ignore]
    fn test_real_upload() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = test_upload_speed().await;
            assert!(result.is_ok(), "Upload should succeed: {:?}", result);
            
            let result = result.unwrap();
            assert!(result.speed_mbps > 0.0, "Speed should be positive");
            assert_eq!(result.bytes_sent, UPLOAD_SIZE);
            assert!(result.duration_ms > 0, "Duration should be positive");
        });
    }
}

//! Environment information detection for Isolate
//!
//! Detects ASN, country, Wi-Fi SSID, and admin privileges.

use std::time::Duration;

use serde::Deserialize;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::core::errors::{IsolateError, Result};
use crate::core::models::EnvInfo;

/// IP-API response structure
#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    #[serde(rename = "as")]
    as_info: Option<String>,
    isp: Option<String>,
    query: Option<String>,
}

/// Default timeout for HTTP requests (5 seconds as per project rules)
const HTTP_TIMEOUT_MS: u64 = 5000;

/// Collect all environment information
pub async fn collect_env_info() -> EnvInfo {
    info!("Collecting environment information");

    let (asn, country) = get_network_info().await.unwrap_or((None, None));
    let wifi_ssid = get_wifi_ssid().await.ok().flatten();
    let is_admin = check_admin_privileges();
    let os_version = get_os_version();

    let env_info = EnvInfo {
        asn,
        country,
        wifi_ssid,
        is_admin,
        os_version,
    };

    debug!(?env_info, "Environment info collected");
    env_info
}


/// Get ASN and country from IP-API
pub async fn get_network_info() -> Result<(Option<String>, Option<String>)> {
    debug!("Fetching network info from IP-API");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(HTTP_TIMEOUT_MS))
        .build()
        .map_err(|e| IsolateError::Network(format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get("http://ip-api.com/json")
        .send()
        .await
        .map_err(|e| IsolateError::Network(format!("IP-API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(IsolateError::Network(format!(
            "IP-API returned status: {}",
            response.status()
        )));
    }

    let data: IpApiResponse = response
        .json()
        .await
        .map_err(|e| IsolateError::Network(format!("Failed to parse IP-API response: {}", e)))?;

    if data.status != "success" {
        warn!("IP-API returned non-success status");
        return Ok((None, None));
    }

    // Extract ASN number from "AS12345 Provider Name" format
    let asn = data.as_info.as_ref().and_then(|as_str| {
        as_str
            .split_whitespace()
            .next()
            .map(|s| s.to_string())
    });

    let country = data.country_code.or(data.country);

    debug!(asn = ?asn, country = ?country, "Network info retrieved");
    Ok((asn, country))
}


/// Get current Wi-Fi SSID on Windows using netsh
#[cfg(windows)]
pub async fn get_wifi_ssid() -> Result<Option<String>> {
    debug!("Getting Wi-Fi SSID via netsh");

    let output = Command::new("netsh")
        .args(["wlan", "show", "interfaces"])
        .output()
        .await
        .map_err(|e| IsolateError::Process(format!("Failed to run netsh: {}", e)))?;

    if !output.status.success() {
        debug!("netsh command failed, Wi-Fi might not be available");
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse SSID from netsh output
    // Looking for line like "    SSID                   : NetworkName"
    for line in stdout.lines() {
        let line = line.trim();

        // Handle both English and Russian locales
        if line.starts_with("SSID") && line.contains(':') {
            if let Some(ssid) = line.split(':').nth(1) {
                let ssid = ssid.trim();
                if !ssid.is_empty() {
                    debug!(ssid = %ssid, "Wi-Fi SSID detected");
                    return Ok(Some(ssid.to_string()));
                }
            }
        }
    }

    debug!("No Wi-Fi SSID found");
    Ok(None)
}

/// Stub for non-Windows platforms
#[cfg(not(windows))]
pub async fn get_wifi_ssid() -> Result<Option<String>> {
    debug!("Wi-Fi SSID detection not implemented for this platform");
    Ok(None)
}


/// Check if running with administrator privileges on Windows
#[cfg(windows)]
pub fn check_admin_privileges() -> bool {
    use std::ptr;

    // Use Windows API to check if running as admin
    unsafe {
        let mut token_handle: windows_sys::Win32::Foundation::HANDLE = ptr::null_mut();
        let process = windows_sys::Win32::System::Threading::GetCurrentProcess();

        if windows_sys::Win32::Security::OpenProcessToken(
            process,
            windows_sys::Win32::Security::TOKEN_QUERY,
            &mut token_handle,
        ) == 0
        {
            return false;
        }

        let mut elevation = windows_sys::Win32::Security::TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = std::mem::size_of::<windows_sys::Win32::Security::TOKEN_ELEVATION>() as u32;

        let result = windows_sys::Win32::Security::GetTokenInformation(
            token_handle,
            windows_sys::Win32::Security::TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        windows_sys::Win32::Foundation::CloseHandle(token_handle);

        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// Stub for non-Windows platforms
#[cfg(not(windows))]
pub fn check_admin_privileges() -> bool {
    // On Unix, check if running as root
    unsafe { libc::geteuid() == 0 }
}


/// Get OS version string
#[cfg(windows)]
pub fn get_os_version() -> String {
    use std::process::Command as StdCommand;

    // Try to get Windows version via systeminfo or ver
    if let Ok(output) = StdCommand::new("cmd").args(["/c", "ver"]).output() {
        let version = String::from_utf8_lossy(&output.stdout);
        let version = version.trim();
        if !version.is_empty() {
            return version.to_string();
        }
    }

    // Fallback
    format!("Windows {}", std::env::consts::ARCH)
}

/// Get OS version for non-Windows platforms
#[cfg(not(windows))]
pub fn get_os_version() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get ASN only (convenience function)
pub async fn get_asn() -> Option<String> {
    get_network_info().await.ok().and_then(|(asn, _)| asn)
}

/// Get country only (convenience function)
pub async fn get_country() -> Option<String> {
    get_network_info().await.ok().and_then(|(_, country)| country)
}

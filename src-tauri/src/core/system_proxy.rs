//! System Proxy configuration for Windows
//!
//! Sets/clears Windows system proxy settings via WinINet API.
//! Based on Qv2ray's QvProxyConfigurator implementation.

use tracing::{info, error, debug};
use crate::core::errors::{Result, IsolateError};

/// Set system proxy to use SOCKS5 or HTTP proxy
pub async fn set_system_proxy(host: &str, port: u16, scheme: &str) -> Result<()> {
    info!(host, port, scheme, "Setting system proxy");
    
    #[cfg(windows)]
    {
        set_windows_proxy(host, port, scheme)?;
    }
    
    #[cfg(not(windows))]
    {
        let _ = (host, port, scheme);
        error!("System proxy is only supported on Windows");
    }
    
    Ok(())
}

/// Clear system proxy settings
pub async fn clear_system_proxy() -> Result<()> {
    info!("Clearing system proxy");
    
    #[cfg(windows)]
    {
        clear_windows_proxy()?;
    }
    
    #[cfg(not(windows))]
    {
        error!("System proxy is only supported on Windows");
    }
    
    Ok(())
}

/// Check if system proxy is currently set
pub async fn is_system_proxy_set() -> Result<bool> {
    #[cfg(windows)]
    {
        query_windows_proxy_enabled()
    }
    
    #[cfg(not(windows))]
    Ok(false)
}

/// Get current system proxy settings
pub async fn get_system_proxy() -> Result<Option<SystemProxyInfo>> {
    #[cfg(windows)]
    {
        query_windows_proxy_info()
    }
    
    #[cfg(not(windows))]
    Ok(None)
}

/// System proxy information
#[derive(Debug, Clone)]
pub struct SystemProxyInfo {
    pub server: String,
    pub enabled: bool,
}

// ============================================================================
// Windows Implementation
// ============================================================================

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::ptr::null_mut;
    use windows_sys::Win32::Networking::WinInet::{
        InternetSetOptionW, InternetQueryOptionW,
        INTERNET_OPTION_PER_CONNECTION_OPTION, INTERNET_OPTION_SETTINGS_CHANGED,
        INTERNET_OPTION_REFRESH, INTERNET_PER_CONN_OPTION_LISTW,
        INTERNET_PER_CONN_OPTIONW, INTERNET_PER_CONN_FLAGS,
        INTERNET_PER_CONN_PROXY_SERVER, PROXY_TYPE_DIRECT, PROXY_TYPE_PROXY,
    };
    use windows_sys::Win32::Foundation::GetLastError;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    /// Convert Rust string to wide string (null-terminated)
    fn to_wide_string(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn set_windows_proxy(host: &str, port: u16, scheme: &str) -> Result<()> {
        // Build proxy string based on scheme
        // Windows format: "socks=host:port" for SOCKS, "http://host:port" for HTTP
        let proxy_str = match scheme {
            "socks" | "socks5" => format!("socks={}:{}", host, port),
            "http" => format!("http://{}:{}", host, port),
            _ => format!("{}:{}", host, port),
        };
        
        info!(proxy_str = %proxy_str, "Windows proxy string");
        
        // Convert to wide string
        let proxy_wide = to_wide_string(&proxy_str);
        
        unsafe {
            // Allocate options array
            let mut options: [INTERNET_PER_CONN_OPTIONW; 2] = std::mem::zeroed();
            
            // Option 1: Set flags to enable proxy
            options[0].dwOption = INTERNET_PER_CONN_FLAGS;
            options[0].Value.dwValue = PROXY_TYPE_DIRECT | PROXY_TYPE_PROXY;
            
            // Option 2: Set proxy server
            options[1].dwOption = INTERNET_PER_CONN_PROXY_SERVER;
            options[1].Value.pszValue = proxy_wide.as_ptr() as *mut u16;
            
            // Build option list
            let mut list = INTERNET_PER_CONN_OPTION_LISTW {
                dwSize: std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
                pszConnection: null_mut(), // NULL = LAN settings
                dwOptionCount: 2,
                dwOptionError: 0,
                pOptions: options.as_mut_ptr(),
            };
            
            // Set options for LAN
            let result = InternetSetOptionW(
                null_mut(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                &mut list as *mut _ as *mut _,
                std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            );
            
            if result == 0 {
                let err = GetLastError();
                error!(error_code = err, "InternetSetOptionW failed");
                return Err(IsolateError::SystemProxy(
                    format!("Failed to set system proxy, error code: {}", err)
                ));
            }
            
            // Notify system of changes
            InternetSetOptionW(null_mut(), INTERNET_OPTION_SETTINGS_CHANGED, null_mut(), 0);
            InternetSetOptionW(null_mut(), INTERNET_OPTION_REFRESH, null_mut(), 0);
        }
        
        info!("System proxy set successfully");
        Ok(())
    }

    pub fn clear_windows_proxy() -> Result<()> {
        unsafe {
            let mut options: [INTERNET_PER_CONN_OPTIONW; 1] = std::mem::zeroed();
            
            // Set flags to direct (no proxy)
            options[0].dwOption = INTERNET_PER_CONN_FLAGS;
            options[0].Value.dwValue = PROXY_TYPE_DIRECT;
            
            let mut list = INTERNET_PER_CONN_OPTION_LISTW {
                dwSize: std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
                pszConnection: null_mut(),
                dwOptionCount: 1,
                dwOptionError: 0,
                pOptions: options.as_mut_ptr(),
            };
            
            let result = InternetSetOptionW(
                null_mut(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                &mut list as *mut _ as *mut _,
                std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            );
            
            if result == 0 {
                let err = GetLastError();
                error!(error_code = err, "InternetSetOptionW failed when clearing proxy");
                return Err(IsolateError::SystemProxy(
                    format!("Failed to clear system proxy, error code: {}", err)
                ));
            }
            
            InternetSetOptionW(null_mut(), INTERNET_OPTION_SETTINGS_CHANGED, null_mut(), 0);
            InternetSetOptionW(null_mut(), INTERNET_OPTION_REFRESH, null_mut(), 0);
        }
        
        info!("System proxy cleared");
        Ok(())
    }

    pub fn query_windows_proxy_enabled() -> Result<bool> {
        unsafe {
            let mut options: [INTERNET_PER_CONN_OPTIONW; 1] = std::mem::zeroed();
            options[0].dwOption = INTERNET_PER_CONN_FLAGS;
            
            let mut list = INTERNET_PER_CONN_OPTION_LISTW {
                dwSize: std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
                pszConnection: null_mut(),
                dwOptionCount: 1,
                dwOptionError: 0,
                pOptions: options.as_mut_ptr(),
            };
            
            let mut size = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            
            let result = InternetQueryOptionW(
                null_mut(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                &mut list as *mut _ as *mut _,
                &mut size,
            );
            
            if result == 0 {
                let err = GetLastError();
                debug!(error_code = err, "InternetQueryOptionW failed");
                return Ok(false);
            }
            
            let flags = options[0].Value.dwValue;
            let is_proxy_enabled = (flags & PROXY_TYPE_PROXY) == PROXY_TYPE_PROXY;
            
            debug!(flags, is_proxy_enabled, "Proxy flags queried");
            Ok(is_proxy_enabled)
        }
    }

    pub fn query_windows_proxy_info() -> Result<Option<SystemProxyInfo>> {
        unsafe {
            let mut options: [INTERNET_PER_CONN_OPTIONW; 2] = std::mem::zeroed();
            options[0].dwOption = INTERNET_PER_CONN_FLAGS;
            options[1].dwOption = INTERNET_PER_CONN_PROXY_SERVER;
            
            let mut list = INTERNET_PER_CONN_OPTION_LISTW {
                dwSize: std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
                pszConnection: null_mut(),
                dwOptionCount: 2,
                dwOptionError: 0,
                pOptions: options.as_mut_ptr(),
            };
            
            let mut size = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            
            let result = InternetQueryOptionW(
                null_mut(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                &mut list as *mut _ as *mut _,
                &mut size,
            );
            
            if result == 0 {
                let err = GetLastError();
                debug!(error_code = err, "InternetQueryOptionW failed");
                return Ok(None);
            }
            
            let flags = options[0].Value.dwValue;
            let is_proxy_enabled = (flags & PROXY_TYPE_PROXY) == PROXY_TYPE_PROXY;
            
            // Get proxy server string
            let server = if !options[1].Value.pszValue.is_null() {
                let ptr = options[1].Value.pszValue;
                let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
                let slice = std::slice::from_raw_parts(ptr, len);
                String::from_utf16_lossy(slice)
            } else {
                String::new()
            };
            
            if is_proxy_enabled && !server.is_empty() {
                Ok(Some(SystemProxyInfo {
                    server,
                    enabled: true,
                }))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(windows)]
fn set_windows_proxy(host: &str, port: u16, scheme: &str) -> Result<()> {
    windows_impl::set_windows_proxy(host, port, scheme)
}

#[cfg(windows)]
fn clear_windows_proxy() -> Result<()> {
    windows_impl::clear_windows_proxy()
}

#[cfg(windows)]
fn query_windows_proxy_enabled() -> Result<bool> {
    windows_impl::query_windows_proxy_enabled()
}

#[cfg(windows)]
fn query_windows_proxy_info() -> Result<Option<SystemProxyInfo>> {
    windows_impl::query_windows_proxy_info()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Windows and may modify system settings
    async fn test_set_and_clear_proxy() {
        // Set proxy
        set_system_proxy("127.0.0.1", 1080, "socks5").await.unwrap();
        
        // Verify it's set
        let is_set = is_system_proxy_set().await.unwrap();
        assert!(is_set);
        
        // Clear proxy
        clear_system_proxy().await.unwrap();
        
        // Verify it's cleared
        let is_set = is_system_proxy_set().await.unwrap();
        assert!(!is_set);
    }
}

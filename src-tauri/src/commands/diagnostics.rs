//! Diagnostics commands
//!
//! Commands for DPI diagnostics, IPv6 checking, and emergency network reset.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use tokio::process::Command;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error};

use crate::core::models::DiagnosticResult;
use crate::core::diagnostics::{DualStackResult, Ipv6Status};
use crate::core::paths::{get_binaries_dir, get_winws_path, get_singbox_path};
use crate::state::AppState;

/// Component check result for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentCheck {
    pub status: String,  // "healthy", "warning", "error"
    pub details: String,
}

/// Run system diagnostics for UI
/// Returns status of each component: network, dns, windivert, winws, singbox, firewall
#[tauri::command]
pub async fn run_diagnostics() -> Result<HashMap<String, ComponentCheck>, String> {
    info!("Running system diagnostics for UI");
    
    let mut results = HashMap::new();
    
    // 1. Network check - ping google.com
    let network_result = check_network_connectivity().await;
    results.insert("network".to_string(), network_result);
    
    // 2. DNS check
    let dns_result = check_dns_resolution().await;
    results.insert("dns".to_string(), dns_result);
    
    // 3. WinDivert check
    let windivert_result = check_windivert().await;
    results.insert("windivert".to_string(), windivert_result);
    
    // 4. winws binary check
    let winws_result = check_winws_binary().await;
    results.insert("winws".to_string(), winws_result);
    
    // 5. sing-box binary check
    let singbox_result = check_singbox_binary().await;
    results.insert("singbox".to_string(), singbox_result);
    
    // 6. Firewall check
    let firewall_result = check_firewall().await;
    results.insert("firewall".to_string(), firewall_result);
    
    info!("System diagnostics completed");
    Ok(results)
}

/// Check network connectivity by connecting to google.com:443
async fn check_network_connectivity() -> ComponentCheck {
    let start = Instant::now();
    
    match timeout(Duration::from_secs(5), TcpStream::connect("www.google.com:443")).await {
        Ok(Ok(_)) => {
            let latency = start.elapsed().as_millis();
            ComponentCheck {
                status: "healthy".to_string(),
                details: format!("Connected ({}ms latency)", latency),
            }
        }
        Ok(Err(e)) => {
            ComponentCheck {
                status: "error".to_string(),
                details: format!("Connection failed: {}", e),
            }
        }
        Err(_) => {
            ComponentCheck {
                status: "error".to_string(),
                details: "Connection timeout (5s)".to_string(),
            }
        }
    }
}

/// Check DNS resolution
async fn check_dns_resolution() -> ComponentCheck {
    let start = Instant::now();
    
    match timeout(Duration::from_secs(5), tokio::net::lookup_host("www.google.com:443")).await {
        Ok(Ok(addrs)) => {
            let count = addrs.count();
            let latency = start.elapsed().as_millis();
            if count > 0 {
                ComponentCheck {
                    status: "healthy".to_string(),
                    details: format!("Resolving correctly ({}ms, {} addresses)", latency, count),
                }
            } else {
                ComponentCheck {
                    status: "warning".to_string(),
                    details: "No addresses resolved".to_string(),
                }
            }
        }
        Ok(Err(e)) => {
            ComponentCheck {
                status: "error".to_string(),
                details: format!("DNS failed: {}", e),
            }
        }
        Err(_) => {
            ComponentCheck {
                status: "error".to_string(),
                details: "DNS timeout (5s)".to_string(),
            }
        }
    }
}

/// Check WinDivert driver
async fn check_windivert() -> ComponentCheck {
    let binaries_dir = get_binaries_dir();
    let dll_path = binaries_dir.join("WinDivert.dll");
    let sys_path = binaries_dir.join("WinDivert64.sys");
    
    let dll_exists = dll_path.exists();
    let sys_exists = sys_path.exists();
    
    if dll_exists && sys_exists {
        // Try to get version from DLL
        #[cfg(windows)]
        {
            use std::process::Command as StdCommand;
            if let Ok(output) = StdCommand::new("powershell")
                .args(["-Command", &format!(
                    "(Get-Item '{}').VersionInfo.FileVersion",
                    dll_path.display()
                )])
                .output()
            {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return ComponentCheck {
                        status: "healthy".to_string(),
                        details: format!("Driver loaded (v{})", version),
                    };
                }
            }
        }
        
        ComponentCheck {
            status: "healthy".to_string(),
            details: "Driver files present".to_string(),
        }
    } else {
        let missing: Vec<&str> = [
            (!dll_exists).then_some("WinDivert.dll"),
            (!sys_exists).then_some("WinDivert64.sys"),
        ].into_iter().flatten().collect();
        
        ComponentCheck {
            status: "error".to_string(),
            details: format!("Missing: {}", missing.join(", ")),
        }
    }
}

/// Check winws binary
async fn check_winws_binary() -> ComponentCheck {
    let winws_path = get_winws_path();
    
    if winws_path.exists() {
        // Try to get version
        match tokio::process::Command::new(&winws_path)
            .arg("--help")
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Extract version if present
                if stdout.contains("zapret") || stdout.contains("winws") {
                    ComponentCheck {
                        status: "healthy".to_string(),
                        details: "Binary found and working".to_string(),
                    }
                } else {
                    ComponentCheck {
                        status: "healthy".to_string(),
                        details: "Binary found".to_string(),
                    }
                }
            }
            Err(_) => {
                ComponentCheck {
                    status: "warning".to_string(),
                    details: "Binary found but not executable".to_string(),
                }
            }
        }
    } else {
        ComponentCheck {
            status: "error".to_string(),
            details: format!("Not found at {}", winws_path.display()),
        }
    }
}

/// Check sing-box binary
async fn check_singbox_binary() -> ComponentCheck {
    let singbox_path = get_singbox_path();
    
    if singbox_path.exists() {
        // Try to get version
        match tokio::process::Command::new(&singbox_path)
            .arg("version")
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Extract version
                if let Some(line) = stdout.lines().next() {
                    let version = line.trim();
                    if !version.is_empty() {
                        return ComponentCheck {
                            status: "healthy".to_string(),
                            details: format!("v{}", version.replace("sing-box version ", "")),
                        };
                    }
                }
                ComponentCheck {
                    status: "healthy".to_string(),
                    details: "Binary found".to_string(),
                }
            }
            Err(_) => {
                ComponentCheck {
                    status: "warning".to_string(),
                    details: "Binary found but not executable".to_string(),
                }
            }
        }
    } else {
        ComponentCheck {
            status: "warning".to_string(),
            details: "Not installed (optional for VLESS)".to_string(),
        }
    }
}

/// Check Windows Firewall status
async fn check_firewall() -> ComponentCheck {
    #[cfg(windows)]
    {
        match tokio::process::Command::new("netsh")
            .args(["advfirewall", "show", "currentprofile", "state"])
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("ON") {
                    ComponentCheck {
                        status: "healthy".to_string(),
                        details: "Firewall active".to_string(),
                    }
                } else if stdout.contains("OFF") {
                    ComponentCheck {
                        status: "warning".to_string(),
                        details: "Firewall disabled".to_string(),
                    }
                } else {
                    ComponentCheck {
                        status: "healthy".to_string(),
                        details: "Status checked".to_string(),
                    }
                }
            }
            Err(e) => {
                ComponentCheck {
                    status: "warning".to_string(),
                    details: format!("Could not check: {}", e),
                }
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        ComponentCheck {
            status: "healthy".to_string(),
            details: "N/A (non-Windows)".to_string(),
        }
    }
}

/// Run DPI diagnostics
#[tauri::command]
pub async fn diagnose() -> Result<DiagnosticResult, String> {
    info!("Running DPI diagnostics");
    
    let profile = crate::core::diagnostics::diagnose()
        .await
        .map_err(|e| format!("Diagnostics failed: {}", e))?;
    
    Ok(DiagnosticResult {
        profile,
        tested_services: vec![],
        blocked_services: vec![],
    })
}

/// Run dual-stack (IPv4/IPv6) diagnostics
#[tauri::command]
pub async fn diagnose_dual_stack() -> Result<DualStackResult, String> {
    info!("Running dual-stack diagnostics");
    
    crate::core::diagnostics::diagnose_dual_stack()
        .await
        .map_err(|e| format!("Dual-stack diagnostics failed: {}", e))
}

/// Check IPv6 availability
#[tauri::command]
pub async fn check_ipv6() -> Result<Ipv6Status, String> {
    info!("Checking IPv6 availability");
    
    Ok(crate::core::diagnostics::check_ipv6_availability().await)
}

/// Emergency network reset
#[tauri::command]
pub async fn panic_reset(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    warn!("Panic reset triggered!");
    
    // 1. Stop all running strategies
    if let Err(e) = state.strategy_engine.shutdown_all().await {
        error!("Failed to shutdown strategies: {}", e);
    }
    
    // 2. Reset network (Windows specific)
    #[cfg(windows)]
    {
        // Winsock reset
        let _ = Command::new("netsh")
            .args(["winsock", "reset"])
            .output()
            .await;
        
        // Flush DNS
        let _ = Command::new("ipconfig")
            .args(["/flushdns"])
            .output()
            .await;
        
        info!("Network reset commands executed");
    }
    
    Ok(())
}

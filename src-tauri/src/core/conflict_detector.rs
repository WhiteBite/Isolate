//! Conflict detector module for Isolate
//!
//! Detects software that may conflict with WinDivert/winws:
//! - Network filtering software (Adguard, etc.)
//! - VPN clients (OpenVPN, WireGuard, NordVPN, etc.)
//! - Network optimization services (Killer Network, SmartByte, etc.)
//! - Security software with network filtering

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// Severity level of a conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSeverity {
    /// Critical - will likely cause BSOD or complete failure
    Critical,
    /// High - will likely cause winws to not work
    High,
    /// Medium - may cause intermittent issues
    Medium,
    /// Low - might cause minor issues
    Low,
}

/// Category of conflicting software
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictCategory {
    /// Network filtering software (Adguard, etc.)
    NetworkFilter,
    /// VPN clients
    Vpn,
    /// Network optimization/gaming services
    NetworkOptimization,
    /// Security software with network filtering
    Security,
    /// Other WinDivert-based software
    WinDivert,
}

/// Information about a detected conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Name of the conflicting software
    pub name: String,
    /// Category of the conflict
    pub category: ConflictCategory,
    /// Severity level
    pub severity: ConflictSeverity,
    /// Description of the conflict
    pub description: String,
    /// Recommendation for resolving the conflict
    pub recommendation: String,
    /// Process names that were detected
    pub detected_processes: Vec<String>,
    /// Service names that were detected
    pub detected_services: Vec<String>,
}

/// Definition of a known conflicting software
struct ConflictDefinition {
    name: &'static str,
    category: ConflictCategory,
    severity: ConflictSeverity,
    description: &'static str,
    recommendation: &'static str,
    /// Process names to look for (case-insensitive)
    processes: &'static [&'static str],
    /// Windows service names to look for
    services: &'static [&'static str],
}

/// List of known conflicting software
const KNOWN_CONFLICTS: &[ConflictDefinition] = &[
    // Network Filtering Software
    ConflictDefinition {
        name: "AdGuard",
        category: ConflictCategory::NetworkFilter,
        severity: ConflictSeverity::Critical,
        description: "AdGuard uses WinDivert for traffic filtering, which conflicts with winws",
        recommendation: "Disable AdGuard or add Isolate to AdGuard's exclusions",
        processes: &["Adguard.exe", "AdguardSvc.exe"],
        services: &["AdguardSvc"],
    },
    ConflictDefinition {
        name: "Simplewall",
        category: ConflictCategory::NetworkFilter,
        severity: ConflictSeverity::Critical,
        description: "Simplewall uses WinDivert for firewall functionality",
        recommendation: "Disable Simplewall while using Isolate",
        processes: &["simplewall.exe"],
        services: &[],
    },
    ConflictDefinition {
        name: "GoodbyeDPI",
        category: ConflictCategory::WinDivert,
        severity: ConflictSeverity::Critical,
        description: "GoodbyeDPI uses WinDivert and cannot run simultaneously with winws",
        recommendation: "Stop GoodbyeDPI before using Isolate",
        processes: &["goodbyedpi.exe"],
        services: &["GoodbyeDPI"],
    },
    ConflictDefinition {
        name: "Zapret (other instance)",
        category: ConflictCategory::WinDivert,
        severity: ConflictSeverity::Critical,
        description: "Another instance of zapret/winws is already running",
        recommendation: "Stop the other winws instance before using Isolate",
        processes: &["winws.exe", "nfqws.exe"],
        services: &[],
    },
    
    // Network Optimization Services
    ConflictDefinition {
        name: "Killer Network Service",
        category: ConflictCategory::NetworkOptimization,
        severity: ConflictSeverity::High,
        description: "Killer Network Service can interfere with packet filtering",
        recommendation: "Disable Killer Network Service in Windows Services",
        processes: &["KillerNetworkService.exe"],
        services: &["Killer Network Service", "KillerNetworkService"],
    },
    ConflictDefinition {
        name: "Intel Connectivity Performance Suite",
        category: ConflictCategory::NetworkOptimization,
        severity: ConflictSeverity::High,
        description: "Intel network optimization can interfere with WinDivert",
        recommendation: "Disable Intel Connectivity services",
        processes: &["IntelConnectivityService.exe"],
        services: &["IntelConnectivityService"],
    },
    ConflictDefinition {
        name: "SmartByte Network Service",
        category: ConflictCategory::NetworkOptimization,
        severity: ConflictSeverity::High,
        description: "Dell SmartByte can interfere with network packet handling",
        recommendation: "Disable SmartByte in Windows Services or uninstall it",
        processes: &["SmartByteNetworkService.exe", "SmartByte.exe"],
        services: &["SmartByteNetworkService"],
    },
    ConflictDefinition {
        name: "cFosSpeed",
        category: ConflictCategory::NetworkOptimization,
        severity: ConflictSeverity::Medium,
        description: "cFosSpeed traffic shaping may interfere with packet modification",
        recommendation: "Disable cFosSpeed or add exclusions for Isolate",
        processes: &["cfosspeed.exe", "cfosspeed64.exe"],
        services: &["cFosSpeedS"],
    },
    
    // VPN Clients
    ConflictDefinition {
        name: "OpenVPN",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "OpenVPN may route traffic differently, bypassing winws",
        recommendation: "Configure split tunneling or disable VPN for target services",
        processes: &["openvpn.exe", "openvpn-gui.exe", "openvpnserv.exe"],
        services: &["OpenVPNService", "OpenVPNServiceInteractive"],
    },
    ConflictDefinition {
        name: "WireGuard",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "WireGuard tunnels traffic, which may bypass winws",
        recommendation: "Configure split tunneling or disable WireGuard for target services",
        processes: &["wireguard.exe", "wg.exe"],
        services: &["WireGuardTunnel$*", "WireGuardManager"],
    },
    ConflictDefinition {
        name: "NordVPN",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "NordVPN tunnels traffic, which may bypass winws",
        recommendation: "Use split tunneling in NordVPN or disable it for target services",
        processes: &["NordVPN.exe", "nordvpn-service.exe"],
        services: &["nordvpn-service", "NordVPN Service"],
    },
    ConflictDefinition {
        name: "ExpressVPN",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "ExpressVPN tunnels traffic, which may bypass winws",
        recommendation: "Use split tunneling in ExpressVPN or disable it",
        processes: &["expressvpn.exe", "expressvpnd.exe"],
        services: &["ExpressVpnService"],
    },
    ConflictDefinition {
        name: "ProtonVPN",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "ProtonVPN tunnels traffic, which may bypass winws",
        recommendation: "Use split tunneling in ProtonVPN or disable it",
        processes: &["ProtonVPN.exe", "ProtonVPNService.exe"],
        services: &["ProtonVPN Service"],
    },
    ConflictDefinition {
        name: "Mullvad VPN",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "Mullvad VPN tunnels traffic, which may bypass winws",
        recommendation: "Configure split tunneling or disable Mullvad",
        processes: &["mullvad-vpn.exe", "mullvad-daemon.exe"],
        services: &["mullvad-daemon"],
    },
    ConflictDefinition {
        name: "Cloudflare WARP",
        category: ConflictCategory::Vpn,
        severity: ConflictSeverity::Medium,
        description: "Cloudflare WARP tunnels traffic, which may bypass winws",
        recommendation: "Disable WARP or use it in proxy mode only",
        processes: &["Cloudflare WARP.exe", "warp-svc.exe"],
        services: &["CloudflareWARP"],
    },
    
    // Security Software
    ConflictDefinition {
        name: "Check Point VPN",
        category: ConflictCategory::Security,
        severity: ConflictSeverity::High,
        description: "Check Point VPN uses kernel-level filtering that conflicts with WinDivert",
        recommendation: "Disable Check Point VPN or contact IT for exclusions",
        processes: &["TracSrvWrapper.exe", "trac.exe", "cpd.exe"],
        services: &["TracSrvWrapper", "Check Point VPN"],
    },
    ConflictDefinition {
        name: "Kaspersky",
        category: ConflictCategory::Security,
        severity: ConflictSeverity::Medium,
        description: "Kaspersky network protection may interfere with packet filtering",
        recommendation: "Add Isolate to Kaspersky exclusions or disable network protection",
        processes: &["avp.exe", "avpui.exe"],
        services: &["AVP", "klnagent"],
    },
    ConflictDefinition {
        name: "ESET",
        category: ConflictCategory::Security,
        severity: ConflictSeverity::Low,
        description: "ESET firewall may interfere with WinDivert",
        recommendation: "Add Isolate to ESET exclusions",
        processes: &["ekrn.exe", "egui.exe"],
        services: &["ekrn", "ESET Service"],
    },
    ConflictDefinition {
        name: "Bitdefender",
        category: ConflictCategory::Security,
        severity: ConflictSeverity::Low,
        description: "Bitdefender firewall may interfere with packet filtering",
        recommendation: "Add Isolate to Bitdefender exclusions",
        processes: &["bdagent.exe", "vsserv.exe"],
        services: &["VSSERV", "bdredline"],
    },
];

/// Detects running processes that may conflict with WinDivert/winws
/// 
/// Returns a list of detected conflicts with severity and recommendations.
pub async fn detect_conflicts() -> Vec<ConflictInfo> {
    info!("Starting conflict detection");
    
    let running_processes = get_running_processes().await;
    let running_services = get_running_services().await;
    
    debug!(
        processes_count = running_processes.len(),
        services_count = running_services.len(),
        "Got system state for conflict detection"
    );
    
    let mut conflicts = Vec::new();
    
    for def in KNOWN_CONFLICTS {
        let mut detected_processes = Vec::new();
        let mut detected_services = Vec::new();
        
        // Check processes (case-insensitive)
        for proc in def.processes {
            let proc_lower = proc.to_lowercase();
            if running_processes.contains(&proc_lower) {
                detected_processes.push(proc.to_string());
            }
        }
        
        // Check services
        for svc in def.services {
            // Handle wildcard services (e.g., "WireGuardTunnel$*")
            if svc.contains('*') {
                let prefix = svc.trim_end_matches("$*");
                for running_svc in &running_services {
                    if running_svc.starts_with(prefix) {
                        detected_services.push(running_svc.clone());
                    }
                }
            } else if running_services.contains(&svc.to_string()) {
                detected_services.push(svc.to_string());
            }
        }
        
        // If any process or service was detected, add to conflicts
        if !detected_processes.is_empty() || !detected_services.is_empty() {
            // Skip self-detection for winws if it's our own process
            if def.name == "Zapret (other instance)" {
                // Only report if there's more than one winws process
                // This is a simplified check - in production we'd check PIDs
                let winws_count = detected_processes.iter()
                    .filter(|p| p.to_lowercase() == "winws.exe")
                    .count();
                if winws_count <= 1 {
                    continue;
                }
            }
            
            warn!(
                name = def.name,
                processes = ?detected_processes,
                services = ?detected_services,
                "Detected conflicting software"
            );
            
            conflicts.push(ConflictInfo {
                name: def.name.to_string(),
                category: def.category,
                severity: def.severity,
                description: def.description.to_string(),
                recommendation: def.recommendation.to_string(),
                detected_processes,
                detected_services,
            });
        }
    }
    
    // Sort by severity (critical first)
    conflicts.sort_by(|a, b| {
        let severity_order = |s: &ConflictSeverity| match s {
            ConflictSeverity::Critical => 0,
            ConflictSeverity::High => 1,
            ConflictSeverity::Medium => 2,
            ConflictSeverity::Low => 3,
        };
        severity_order(&a.severity).cmp(&severity_order(&b.severity))
    });
    
    info!(
        conflicts_count = conflicts.len(),
        critical = conflicts.iter().filter(|c| c.severity == ConflictSeverity::Critical).count(),
        high = conflicts.iter().filter(|c| c.severity == ConflictSeverity::High).count(),
        "Conflict detection completed"
    );
    
    conflicts
}

/// Gets list of running process names (lowercase)
#[cfg(windows)]
async fn get_running_processes() -> HashSet<String> {
    use std::process::Command;
    
    let mut processes = HashSet::new();
    
    // Use tasklist to get running processes
    let output = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // CSV format: "process.exe","PID","Session Name","Session#","Mem Usage"
                if let Some(name) = line.split(',').next() {
                    let name = name.trim_matches('"').to_lowercase();
                    if !name.is_empty() {
                        processes.insert(name);
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to get process list: {}", e);
        }
    }
    
    processes
}

#[cfg(not(windows))]
async fn get_running_processes() -> HashSet<String> {
    // Non-Windows: return empty set
    HashSet::new()
}

/// Gets list of running Windows service names
#[cfg(windows)]
async fn get_running_services() -> HashSet<String> {
    use std::process::Command;
    
    let mut services = HashSet::new();
    
    // Use sc query to get running services
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Service | Where-Object {$_.Status -eq 'Running'} | Select-Object -ExpandProperty Name"
        ])
        .output();
    
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() {
                    services.insert(name);
                }
            }
        }
        Err(e) => {
            warn!("Failed to get service list: {}", e);
        }
    }
    
    services
}

#[cfg(not(windows))]
async fn get_running_services() -> HashSet<String> {
    // Non-Windows: return empty set
    HashSet::new()
}

/// Checks if there are any critical conflicts
pub fn has_critical_conflicts(conflicts: &[ConflictInfo]) -> bool {
    conflicts.iter().any(|c| c.severity == ConflictSeverity::Critical)
}

/// Checks if there are any high or critical conflicts
pub fn has_blocking_conflicts(conflicts: &[ConflictInfo]) -> bool {
    conflicts.iter().any(|c| {
        matches!(c.severity, ConflictSeverity::Critical | ConflictSeverity::High)
    })
}

/// Gets a summary of conflicts for display
pub fn get_conflict_summary(conflicts: &[ConflictInfo]) -> String {
    if conflicts.is_empty() {
        return "No conflicts detected".to_string();
    }
    
    let critical = conflicts.iter().filter(|c| c.severity == ConflictSeverity::Critical).count();
    let high = conflicts.iter().filter(|c| c.severity == ConflictSeverity::High).count();
    let medium = conflicts.iter().filter(|c| c.severity == ConflictSeverity::Medium).count();
    let low = conflicts.iter().filter(|c| c.severity == ConflictSeverity::Low).count();
    
    let mut parts = Vec::new();
    if critical > 0 { parts.push(format!("{} critical", critical)); }
    if high > 0 { parts.push(format!("{} high", high)); }
    if medium > 0 { parts.push(format!("{} medium", medium)); }
    if low > 0 { parts.push(format!("{} low", low)); }
    
    format!("Detected {} conflicts: {}", conflicts.len(), parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_conflict_severity_ordering() {
        let mut conflicts = vec![
            ConflictInfo {
                name: "Low".to_string(),
                category: ConflictCategory::Security,
                severity: ConflictSeverity::Low,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
            ConflictInfo {
                name: "Critical".to_string(),
                category: ConflictCategory::NetworkFilter,
                severity: ConflictSeverity::Critical,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
            ConflictInfo {
                name: "Medium".to_string(),
                category: ConflictCategory::Vpn,
                severity: ConflictSeverity::Medium,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
        ];
        
        conflicts.sort_by(|a, b| {
            let severity_order = |s: &ConflictSeverity| match s {
                ConflictSeverity::Critical => 0,
                ConflictSeverity::High => 1,
                ConflictSeverity::Medium => 2,
                ConflictSeverity::Low => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });
        
        assert_eq!(conflicts[0].name, "Critical");
        assert_eq!(conflicts[1].name, "Medium");
        assert_eq!(conflicts[2].name, "Low");
    }
    
    #[test]
    fn test_has_critical_conflicts() {
        let conflicts = vec![
            ConflictInfo {
                name: "Test".to_string(),
                category: ConflictCategory::Vpn,
                severity: ConflictSeverity::Medium,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
        ];
        
        assert!(!has_critical_conflicts(&conflicts));
        
        let conflicts_with_critical = vec![
            ConflictInfo {
                name: "Test".to_string(),
                category: ConflictCategory::NetworkFilter,
                severity: ConflictSeverity::Critical,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
        ];
        
        assert!(has_critical_conflicts(&conflicts_with_critical));
    }
    
    #[test]
    fn test_get_conflict_summary() {
        let conflicts: Vec<ConflictInfo> = vec![];
        assert_eq!(get_conflict_summary(&conflicts), "No conflicts detected");
        
        let conflicts = vec![
            ConflictInfo {
                name: "Test".to_string(),
                category: ConflictCategory::NetworkFilter,
                severity: ConflictSeverity::Critical,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
            ConflictInfo {
                name: "Test2".to_string(),
                category: ConflictCategory::Vpn,
                severity: ConflictSeverity::Medium,
                description: String::new(),
                recommendation: String::new(),
                detected_processes: vec![],
                detected_services: vec![],
            },
        ];
        
        let summary = get_conflict_summary(&conflicts);
        assert!(summary.contains("2 conflicts"));
        assert!(summary.contains("1 critical"));
        assert!(summary.contains("1 medium"));
    }
}

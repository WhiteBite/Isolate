//! Strategy Analyzer for Isolate
//!
//! Analyzes winws arguments to extract features, validate configuration,
//! and determine strategy complexity and service compatibility.
//!
//! NOTE: This module is prepared for future integration with strategy optimization.
//! Currently not used in production but has comprehensive tests.

// Public API for future strategy optimization features
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::debug;

/// Result of strategy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAnalysis {
    /// Strategy identifier
    pub strategy_id: String,
    /// Whether the strategy configuration is valid
    pub is_valid: bool,
    /// Warning messages (non-critical issues)
    pub warnings: Vec<String>,
    /// Error messages (critical issues)
    pub errors: Vec<String>,
    /// Detected features in the strategy
    pub features: StrategyFeatures,
}

/// Features detected in a strategy's winws arguments
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyFeatures {
    /// Uses fake packet injection (--dpi-desync-fake-*)
    pub uses_fake: bool,
    /// Uses packet splitting (--dpi-desync-split*)
    pub uses_split: bool,
    /// Uses packet disorder (--dpi-desync=disorder*)
    pub uses_disorder: bool,
    /// Uses fooling techniques (--dpi-desync-fooling)
    pub uses_fooling: bool,
    /// Uses hostlist filtering (--hostlist, --hostlist-domains)
    pub uses_hostlist: bool,
    /// Uses auto hostlist (--hostlist-auto)
    pub uses_auto_hostlist: bool,
    /// TCP ports being filtered
    pub tcp_ports: Vec<u16>,
    /// UDP ports being filtered
    pub udp_ports: Vec<u16>,
    /// Desync methods used
    pub desync_methods: Vec<String>,
    /// TTL value if specified
    pub ttl: Option<u8>,
    /// Window size if specified
    pub window_size: Option<u32>,
}

/// Strategy analyzer for winws arguments
pub struct StrategyAnalyzer;

impl StrategyAnalyzer {
    /// Creates a new StrategyAnalyzer instance
    pub fn new() -> Self {
        Self
    }

    /// Analyzes winws arguments and returns detailed analysis
    pub fn analyze(&self, strategy_id: &str, args: &str) -> StrategyAnalysis {
        let mut analysis = StrategyAnalysis {
            strategy_id: strategy_id.to_string(),
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            features: StrategyFeatures::default(),
        };

        let args_lower = args.to_lowercase();
        let tokens: Vec<&str> = args.split_whitespace().collect();

        // Parse features
        self.parse_features(&mut analysis, &args_lower, &tokens);

        // Validate configuration
        self.validate(&mut analysis, &tokens);

        debug!(
            "Analyzed strategy {}: valid={}, warnings={}, errors={}",
            strategy_id,
            analysis.is_valid,
            analysis.warnings.len(),
            analysis.errors.len()
        );

        analysis
    }

    /// Parses features from winws arguments
    fn parse_features(&self, analysis: &mut StrategyAnalysis, args_lower: &str, tokens: &[&str]) {
        let features = &mut analysis.features;

        // Detect fake packet usage
        features.uses_fake = args_lower.contains("--dpi-desync-fake")
            || args_lower.contains("-fake-");

        // Detect split usage
        features.uses_split = args_lower.contains("--dpi-desync-split")
            || args_lower.contains("-split-")
            || args_lower.contains("-split2");

        // Detect disorder usage
        features.uses_disorder = args_lower.contains("disorder")
            || args_lower.contains("disorder2");

        // Detect fooling techniques
        features.uses_fooling = args_lower.contains("--dpi-desync-fooling");

        // Detect hostlist usage
        features.uses_hostlist = args_lower.contains("--hostlist")
            || args_lower.contains("--hostlist-domains");
        features.uses_auto_hostlist = args_lower.contains("--hostlist-auto");

        // Parse ports
        for token in tokens.iter() {
            let token_lower = token.to_lowercase();
            
            // TCP ports: --wf-tcp=80,443 or --filter-tcp=80,443
            if token_lower.starts_with("--wf-tcp=") || token_lower.starts_with("--filter-tcp=") {
                if let Some(ports_str) = token.split('=').nth(1) {
                    features.tcp_ports = Self::parse_ports(ports_str);
                }
            }
            
            // UDP ports: --wf-udp=443,80 or --filter-udp=443
            if token_lower.starts_with("--wf-udp=") || token_lower.starts_with("--filter-udp=") {
                if let Some(ports_str) = token.split('=').nth(1) {
                    features.udp_ports = Self::parse_ports(ports_str);
                }
            }

            // Desync methods: --dpi-desync=fake,disorder2
            if token_lower.starts_with("--dpi-desync=") {
                if let Some(methods_str) = token.split('=').nth(1) {
                    features.desync_methods = methods_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }

            // TTL: --dpi-desync-ttl=5
            if token_lower.starts_with("--dpi-desync-ttl=") {
                if let Some(ttl_str) = token.split('=').nth(1) {
                    features.ttl = ttl_str.parse().ok();
                }
            }

            // Window size: --wssize=1:6 or --wsize=40
            if token_lower.starts_with("--wssize=") || token_lower.starts_with("--wsize=") {
                if let Some(size_str) = token.split('=').nth(1) {
                    // Handle format like "1:6" - take first number
                    let first_part = size_str.split(':').next().unwrap_or(size_str);
                    features.window_size = first_part.parse().ok();
                }
            }
        }
    }

    /// Validates strategy configuration
    fn validate(&self, analysis: &mut StrategyAnalysis, tokens: &[&str]) {
        // Check for empty args
        if tokens.is_empty() {
            analysis.errors.push("Empty arguments".to_string());
            analysis.is_valid = false;
            return;
        }

        // Check for required port filters
        if analysis.features.tcp_ports.is_empty() && analysis.features.udp_ports.is_empty() {
            analysis.warnings.push("No port filters specified (--wf-tcp or --wf-udp)".to_string());
        }

        // Check for desync method
        if analysis.features.desync_methods.is_empty() {
            analysis.warnings.push("No desync method specified (--dpi-desync)".to_string());
        }

        // Validate TTL range
        if let Some(ttl) = analysis.features.ttl {
            if ttl == 0 {
                analysis.errors.push("TTL cannot be 0".to_string());
                analysis.is_valid = false;
            } else if ttl > 64 {
                analysis.warnings.push(format!("High TTL value ({}) may cause issues", ttl));
            }
        }

        // Check for conflicting options
        if analysis.features.uses_disorder && analysis.features.uses_split {
            analysis.warnings.push("Using both disorder and split may be redundant".to_string());
        }

        // Validate port ranges
        for port in &analysis.features.tcp_ports {
            if *port == 0 {
                analysis.errors.push("Invalid TCP port: 0".to_string());
                analysis.is_valid = false;
            }
        }
        for port in &analysis.features.udp_ports {
            if *port == 0 {
                analysis.errors.push("Invalid UDP port: 0".to_string());
                analysis.is_valid = false;
            }
        }
    }

    /// Parses port string like "80,443" or "80-443" or "80,443,8080-8090"
    pub fn parse_ports(ports_str: &str) -> Vec<u16> {
        let mut ports = Vec::new();
        
        for part in ports_str.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.contains('-') {
                // Range: "80-443"
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (
                        range_parts[0].trim().parse::<u16>(),
                        range_parts[1].trim().parse::<u16>(),
                    ) {
                        if start <= end {
                            for port in start..=end {
                                ports.push(port);
                            }
                        }
                    }
                }
            } else {
                // Single port: "80"
                if let Ok(port) = part.parse::<u16>() {
                    ports.push(port);
                }
            }
        }

        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        ports.retain(|p| seen.insert(*p));
        
        ports
    }

    /// Checks if strategy is suitable for a service based on port requirements
    pub fn is_suitable_for_service(&self, analysis: &StrategyAnalysis, service_ports: &ServicePorts) -> bool {
        // If strategy has no port filters, it might work for any service (global mode)
        if analysis.features.tcp_ports.is_empty() && analysis.features.udp_ports.is_empty() {
            return true;
        }

        // Check TCP ports
        for required_port in &service_ports.tcp {
            if !analysis.features.tcp_ports.contains(required_port) {
                return false;
            }
        }

        // Check UDP ports
        for required_port in &service_ports.udp {
            if !analysis.features.udp_ports.contains(required_port) {
                return false;
            }
        }

        true
    }

    /// Calculates complexity score for a strategy (0.0 - 1.0)
    /// Higher score = more complex strategy
    pub fn complexity_score(&self, analysis: &StrategyAnalysis) -> f64 {
        let features = &analysis.features;
        let mut score = 0.0;

        // Base complexity from features
        if features.uses_fake {
            score += 0.15;
        }
        if features.uses_split {
            score += 0.15;
        }
        if features.uses_disorder {
            score += 0.15;
        }
        if features.uses_fooling {
            score += 0.10;
        }
        if features.uses_hostlist {
            score += 0.05;
        }
        if features.uses_auto_hostlist {
            score += 0.10;
        }

        // Complexity from number of desync methods
        let methods_count = features.desync_methods.len();
        score += (methods_count as f64 * 0.05).min(0.15);

        // Complexity from port count (more ports = more complex filtering)
        let total_ports = features.tcp_ports.len() + features.udp_ports.len();
        if total_ports > 10 {
            score += 0.10;
        } else if total_ports > 5 {
            score += 0.05;
        }

        // TTL manipulation adds complexity
        if features.ttl.is_some() {
            score += 0.05;
        }

        // Window size manipulation
        if features.window_size.is_some() {
            score += 0.05;
        }

        // Clamp to [0.0, 1.0]
        score.min(1.0)
    }
}

impl Default for StrategyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Service port requirements
#[derive(Debug, Clone, Default)]
pub struct ServicePorts {
    /// Required TCP ports
    pub tcp: Vec<u16>,
    /// Required UDP ports
    pub udp: Vec<u16>,
}

impl ServicePorts {
    /// Creates new ServicePorts with specified TCP and UDP ports
    pub fn new(tcp: Vec<u16>, udp: Vec<u16>) -> Self {
        Self { tcp, udp }
    }

    /// Creates ServicePorts for common HTTPS service
    pub fn https() -> Self {
        Self {
            tcp: vec![443],
            udp: vec![],
        }
    }

    /// Creates ServicePorts for HTTP + HTTPS
    pub fn web() -> Self {
        Self {
            tcp: vec![80, 443],
            udp: vec![],
        }
    }

    /// Creates ServicePorts for QUIC (UDP 443)
    pub fn quic() -> Self {
        Self {
            tcp: vec![],
            udp: vec![443],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_ports ====================

    #[test]
    fn test_parse_ports_single() {
        let ports = StrategyAnalyzer::parse_ports("443");
        assert_eq!(ports, vec![443]);
    }

    #[test]
    fn test_parse_ports_multiple() {
        let ports = StrategyAnalyzer::parse_ports("80,443,8080");
        assert_eq!(ports, vec![80, 443, 8080]);
    }

    #[test]
    fn test_parse_ports_range() {
        let ports = StrategyAnalyzer::parse_ports("80-83");
        assert_eq!(ports, vec![80, 81, 82, 83]);
    }

    #[test]
    fn test_parse_ports_mixed() {
        let ports = StrategyAnalyzer::parse_ports("80,443,8080-8082");
        assert_eq!(ports, vec![80, 443, 8080, 8081, 8082]);
    }

    #[test]
    fn test_parse_ports_with_spaces() {
        let ports = StrategyAnalyzer::parse_ports(" 80 , 443 ");
        assert_eq!(ports, vec![80, 443]);
    }

    #[test]
    fn test_parse_ports_empty() {
        let ports = StrategyAnalyzer::parse_ports("");
        assert!(ports.is_empty());
    }

    #[test]
    fn test_parse_ports_invalid() {
        let ports = StrategyAnalyzer::parse_ports("abc,xyz");
        assert!(ports.is_empty());
    }

    #[test]
    fn test_parse_ports_duplicates_removed() {
        let ports = StrategyAnalyzer::parse_ports("80,443,80,443");
        assert_eq!(ports, vec![80, 443]);
    }

    // ==================== analyze ====================

    #[test]
    fn test_analyze_basic_strategy() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=80,443 --dpi-desync=fake,disorder2";
        let analysis = analyzer.analyze("test_strategy", args);

        assert!(analysis.is_valid);
        assert_eq!(analysis.features.tcp_ports, vec![80, 443]);
        assert!(analysis.features.desync_methods.contains(&"fake".to_string()));
        assert!(analysis.features.desync_methods.contains(&"disorder2".to_string()));
    }

    #[test]
    fn test_analyze_detects_fake() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync-fake-tls=0x00000000";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.features.uses_fake);
    }

    #[test]
    fn test_analyze_detects_split() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync-split-pos=2";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.features.uses_split);
    }

    #[test]
    fn test_analyze_detects_disorder() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync=disorder2";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.features.uses_disorder);
    }

    #[test]
    fn test_analyze_detects_hostlist() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --hostlist=youtube.txt";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.features.uses_hostlist);
    }

    #[test]
    fn test_analyze_empty_args_invalid() {
        let analyzer = StrategyAnalyzer::new();
        let analysis = analyzer.analyze("test", "");

        assert!(!analysis.is_valid);
        assert!(!analysis.errors.is_empty());
    }

    #[test]
    fn test_analyze_ttl_parsing() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync-ttl=5";
        let analysis = analyzer.analyze("test", args);

        assert_eq!(analysis.features.ttl, Some(5));
    }

    #[test]
    fn test_analyze_invalid_ttl_zero() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync-ttl=0";
        let analysis = analyzer.analyze("test", args);

        assert!(!analysis.is_valid);
        assert!(analysis.errors.iter().any(|e| e.contains("TTL")));
    }

    #[test]
    fn test_analyze_udp_ports() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-udp=443,80";
        let analysis = analyzer.analyze("test", args);

        assert_eq!(analysis.features.udp_ports, vec![443, 80]);
    }

    // ==================== is_suitable_for_service ====================

    #[test]
    fn test_suitable_for_service_matching_ports() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=80,443";
        let analysis = analyzer.analyze("test", args);
        let service = ServicePorts::web();

        assert!(analyzer.is_suitable_for_service(&analysis, &service));
    }

    #[test]
    fn test_suitable_for_service_missing_port() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443";
        let analysis = analyzer.analyze("test", args);
        let service = ServicePorts::web(); // requires 80 and 443

        assert!(!analyzer.is_suitable_for_service(&analysis, &service));
    }

    #[test]
    fn test_suitable_for_service_no_filters_global() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--dpi-desync=fake"; // no port filters
        let analysis = analyzer.analyze("test", args);
        let service = ServicePorts::https();

        // No port filters = global mode, suitable for any service
        assert!(analyzer.is_suitable_for_service(&analysis, &service));
    }

    #[test]
    fn test_suitable_for_service_udp_required() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443"; // only TCP
        let analysis = analyzer.analyze("test", args);
        let service = ServicePorts::quic(); // requires UDP 443

        assert!(!analyzer.is_suitable_for_service(&analysis, &service));
    }

    // ==================== complexity_score ====================

    #[test]
    fn test_complexity_score_minimal() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443";
        let analysis = analyzer.analyze("test", args);
        let score = analyzer.complexity_score(&analysis);

        assert!(score < 0.2);
    }

    #[test]
    fn test_complexity_score_moderate() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=80,443 --dpi-desync=fake,disorder2 --dpi-desync-split-pos=2";
        let analysis = analyzer.analyze("test", args);
        let score = analyzer.complexity_score(&analysis);

        assert!(score > 0.3 && score < 0.7);
    }

    #[test]
    fn test_complexity_score_high() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=80,443,8080,8443 --wf-udp=443 \
                   --dpi-desync=fake,disorder2,split2 \
                   --dpi-desync-fake-tls=0x00 \
                   --dpi-desync-split-pos=2 \
                   --dpi-desync-fooling=md5sig \
                   --dpi-desync-ttl=5 \
                   --hostlist-auto=auto.txt";
        let analysis = analyzer.analyze("test", args);
        let score = analyzer.complexity_score(&analysis);

        assert!(score > 0.6);
    }

    #[test]
    fn test_complexity_score_bounded() {
        let analyzer = StrategyAnalyzer::new();
        // Maximum complexity args
        let args = "--wf-tcp=1-1000 --wf-udp=1-1000 \
                   --dpi-desync=fake,disorder,disorder2,split,split2 \
                   --dpi-desync-fake-tls=0x00 \
                   --dpi-desync-split-pos=2 \
                   --dpi-desync-fooling=md5sig \
                   --dpi-desync-ttl=5 \
                   --wssize=1:6 \
                   --hostlist=list.txt \
                   --hostlist-auto=auto.txt";
        let analysis = analyzer.analyze("test", args);
        let score = analyzer.complexity_score(&analysis);

        assert!(score <= 1.0);
        assert!(score >= 0.0);
    }

    // ==================== warnings ====================

    #[test]
    fn test_warning_no_port_filters() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--dpi-desync=fake";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.warnings.iter().any(|w| w.contains("port")));
    }

    #[test]
    fn test_warning_no_desync_method() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.warnings.iter().any(|w| w.contains("desync")));
    }

    #[test]
    fn test_warning_high_ttl() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync-ttl=100";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.warnings.iter().any(|w| w.contains("TTL")));
    }

    #[test]
    fn test_warning_disorder_and_split() {
        let analyzer = StrategyAnalyzer::new();
        let args = "--wf-tcp=443 --dpi-desync=disorder2 --dpi-desync-split-pos=2";
        let analysis = analyzer.analyze("test", args);

        assert!(analysis.warnings.iter().any(|w| w.contains("disorder") && w.contains("split")));
    }
}

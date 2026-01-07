//! Strategy-related models

#![allow(dead_code)] // Public strategy models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy family/type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyFamily {
    DnsBypass,
    SniFrag,
    TlsFrag,
    Vless,
    Hybrid,
}

/// Engine type for strategy execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyEngine {
    Zapret,
    SingBox,
    Xray,
    Hybrid,
}

/// Strategy execution mode capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModeCapabilities {
    pub supports_socks: bool,
    pub supports_global: bool,
}

/// Template for launching a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchTemplate {
    pub binary: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub log_file: Option<String>,
    #[serde(default)]
    pub requires_admin: bool,
}

/// Strategy requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRequirements {
    #[serde(default = "default_min_rights")]
    pub min_rights: String,
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub binaries: Vec<String>,
}

impl Default for StrategyRequirements {
    fn default() -> Self {
        Self {
            min_rights: default_min_rights(),
            os: Vec::new(),
            binaries: Vec::new(),
        }
    }
}

fn default_min_rights() -> String {
    "user".to_string()
}

/// Strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub family: StrategyFamily,
    pub engine: StrategyEngine,
    #[serde(default)]
    pub mode_capabilities: ModeCapabilities,
    pub socks_template: Option<LaunchTemplate>,
    pub global_template: Option<LaunchTemplate>,
    #[serde(default)]
    pub requirements: StrategyRequirements,
    #[serde(default)]
    pub weight_hint: i32,
    #[serde(default)]
    pub services: Vec<String>,
}

/// Strategy score after testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyScore {
    pub strategy_id: String,
    pub success_rate: f64,
    pub critical_success_rate: f64,
    pub latency_avg: f64,
    pub latency_jitter: f64,
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_serialization() {
        let strategy = Strategy {
            id: "test-strategy".to_string(),
            name: "Test Strategy".to_string(),
            description: "A test strategy".to_string(),
            family: StrategyFamily::DnsBypass,
            engine: StrategyEngine::Zapret,
            mode_capabilities: ModeCapabilities {
                supports_socks: true,
                supports_global: true,
            },
            socks_template: Some(LaunchTemplate {
                binary: "winws.exe".to_string(),
                args: vec!["--arg1".to_string(), "--arg2".to_string()],
                env: HashMap::new(),
                log_file: Some("log.txt".to_string()),
                requires_admin: true,
            }),
            global_template: None,
            requirements: StrategyRequirements {
                min_rights: "admin".to_string(),
                os: vec!["windows".to_string()],
                binaries: vec!["winws.exe".to_string()],
            },
            weight_hint: 100,
            services: vec!["youtube".to_string(), "discord".to_string()],
        };

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: Strategy = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.id, "test-strategy");
        assert_eq!(deserialized.name, "Test Strategy");
        assert_eq!(deserialized.family, StrategyFamily::DnsBypass);
    }

    #[test]
    fn test_strategy_family_serialization() {
        let families = vec![
            (StrategyFamily::DnsBypass, "\"dns_bypass\""),
            (StrategyFamily::SniFrag, "\"sni_frag\""),
            (StrategyFamily::TlsFrag, "\"tls_frag\""),
            (StrategyFamily::Vless, "\"vless\""),
            (StrategyFamily::Hybrid, "\"hybrid\""),
        ];

        for (family, expected_json) in families {
            let json = serde_json::to_string(&family).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_engine_enum() {
        let engines = vec![
            (StrategyEngine::Zapret, "\"zapret\""),
            (StrategyEngine::SingBox, "\"sing_box\""),
            (StrategyEngine::Xray, "\"xray\""),
            (StrategyEngine::Hybrid, "\"hybrid\""),
        ];

        for (engine, expected_json) in engines {
            let json = serde_json::to_string(&engine).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_mode_capabilities_default() {
        let caps = ModeCapabilities::default();
        assert!(!caps.supports_socks);
        assert!(!caps.supports_global);
    }

    #[test]
    fn test_strategy_requirements_default() {
        let reqs = StrategyRequirements::default();
        assert_eq!(reqs.min_rights, "user");
        assert!(reqs.os.is_empty());
        assert!(reqs.binaries.is_empty());
    }
}

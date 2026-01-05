//! Plugin manifest schema for Isolate plugins
//!
//! Defines types for plugin.json manifest files.

use serde::{Deserialize, Serialize};

/// Plugin types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginType {
    ServiceChecker,
    StrategyProvider,
    HostlistProvider,
}

/// Plugin manifest (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    /// Service definition (for service-checker type)
    pub service: Option<ServiceDefinition>,
    /// Strategy definition (for strategy-provider type)
    pub strategy: Option<StrategyDefinition>,
    /// Hostlist definition (for hostlist-provider type)
    pub hostlist: Option<HostlistDefinition>,
    #[serde(default)]
    pub permissions: Permissions,
}

/// Service definition for checker plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub category: String,
    pub description: Option<String>,
    pub endpoints: Vec<ServiceEndpoint>,
}

/// Service endpoint to check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
}

fn default_method() -> String {
    "GET".to_string()
}

/// Strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDefinition {
    pub id: String,
    pub name: String,
    pub family: String,
    pub config_file: String,
}

/// Hostlist definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostlistDefinition {
    pub id: String,
    pub name: String,
    pub file: String,
}

/// Plugin permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub http: Vec<String>,
    #[serde(default)]
    pub filesystem: bool,
    #[serde(default)]
    pub process: bool,
}

/// Loaded plugin info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedPluginInfo {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub path: String,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== PluginType Tests ====================

    #[test]
    fn test_plugin_type_service_checker_serialization() {
        let plugin_type = PluginType::ServiceChecker;
        let json = serde_json::to_string(&plugin_type).unwrap();
        assert_eq!(json, "\"service-checker\"");
    }

    #[test]
    fn test_plugin_type_strategy_provider_serialization() {
        let plugin_type = PluginType::StrategyProvider;
        let json = serde_json::to_string(&plugin_type).unwrap();
        assert_eq!(json, "\"strategy-provider\"");
    }

    #[test]
    fn test_plugin_type_hostlist_provider_serialization() {
        let plugin_type = PluginType::HostlistProvider;
        let json = serde_json::to_string(&plugin_type).unwrap();
        assert_eq!(json, "\"hostlist-provider\"");
    }

    #[test]
    fn test_plugin_type_deserialization() {
        let service: PluginType = serde_json::from_str("\"service-checker\"").unwrap();
        assert_eq!(service, PluginType::ServiceChecker);

        let strategy: PluginType = serde_json::from_str("\"strategy-provider\"").unwrap();
        assert_eq!(strategy, PluginType::StrategyProvider);

        let hostlist: PluginType = serde_json::from_str("\"hostlist-provider\"").unwrap();
        assert_eq!(hostlist, PluginType::HostlistProvider);
    }

    #[test]
    fn test_plugin_type_invalid_deserialization() {
        let result: Result<PluginType, _> = serde_json::from_str("\"invalid-type\"");
        assert!(result.is_err());
    }

    // ==================== ServiceEndpoint Tests ====================

    #[test]
    fn test_service_endpoint_default_method() {
        let json = r#"{
            "id": "test-endpoint",
            "name": "Test Endpoint",
            "url": "https://example.com"
        }"#;
        let endpoint: ServiceEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.method, "GET");
    }

    #[test]
    fn test_service_endpoint_custom_method() {
        let json = r#"{
            "id": "test-endpoint",
            "name": "Test Endpoint",
            "url": "https://example.com",
            "method": "POST"
        }"#;
        let endpoint: ServiceEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.method, "POST");
    }

    #[test]
    fn test_service_endpoint_missing_required_field() {
        let json = r#"{
            "id": "test-endpoint",
            "name": "Test Endpoint"
        }"#;
        let result: Result<ServiceEndpoint, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ==================== ServiceDefinition Tests ====================

    #[test]
    fn test_service_definition_full() {
        let json = r#"{
            "id": "youtube",
            "name": "YouTube",
            "icon": "youtube.svg",
            "category": "video",
            "description": "YouTube video service",
            "endpoints": [
                {
                    "id": "main",
                    "name": "Main Site",
                    "url": "https://youtube.com"
                }
            ]
        }"#;
        let service: ServiceDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(service.id, "youtube");
        assert_eq!(service.name, "YouTube");
        assert_eq!(service.icon, "youtube.svg");
        assert_eq!(service.category, "video");
        assert_eq!(service.description, Some("YouTube video service".to_string()));
        assert_eq!(service.endpoints.len(), 1);
    }

    #[test]
    fn test_service_definition_optional_description() {
        let json = r#"{
            "id": "test",
            "name": "Test",
            "icon": "test.svg",
            "category": "test",
            "endpoints": []
        }"#;
        let service: ServiceDefinition = serde_json::from_str(json).unwrap();
        assert!(service.description.is_none());
    }

    // ==================== StrategyDefinition Tests ====================

    #[test]
    fn test_strategy_definition_parsing() {
        let json = r#"{
            "id": "zapret-youtube",
            "name": "Zapret YouTube",
            "family": "zapret",
            "config_file": "youtube.yaml"
        }"#;
        let strategy: StrategyDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(strategy.id, "zapret-youtube");
        assert_eq!(strategy.name, "Zapret YouTube");
        assert_eq!(strategy.family, "zapret");
        assert_eq!(strategy.config_file, "youtube.yaml");
    }

    #[test]
    fn test_strategy_definition_missing_field() {
        let json = r#"{
            "id": "test",
            "name": "Test"
        }"#;
        let result: Result<StrategyDefinition, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // ==================== HostlistDefinition Tests ====================

    #[test]
    fn test_hostlist_definition_parsing() {
        let json = r#"{
            "id": "youtube-hosts",
            "name": "YouTube Hosts",
            "file": "youtube.txt"
        }"#;
        let hostlist: HostlistDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(hostlist.id, "youtube-hosts");
        assert_eq!(hostlist.name, "YouTube Hosts");
        assert_eq!(hostlist.file, "youtube.txt");
    }

    // ==================== Permissions Tests ====================

    #[test]
    fn test_permissions_default() {
        let permissions = Permissions::default();
        assert!(permissions.http.is_empty());
        assert!(!permissions.filesystem);
        assert!(!permissions.process);
    }

    #[test]
    fn test_permissions_parsing_full() {
        let json = r#"{
            "http": ["https://api.example.com", "https://cdn.example.com"],
            "filesystem": true,
            "process": true
        }"#;
        let permissions: Permissions = serde_json::from_str(json).unwrap();
        assert_eq!(permissions.http.len(), 2);
        assert!(permissions.filesystem);
        assert!(permissions.process);
    }

    #[test]
    fn test_permissions_parsing_partial() {
        let json = r#"{
            "http": ["https://api.example.com"]
        }"#;
        let permissions: Permissions = serde_json::from_str(json).unwrap();
        assert_eq!(permissions.http.len(), 1);
        assert!(!permissions.filesystem);
        assert!(!permissions.process);
    }

    #[test]
    fn test_permissions_parsing_empty() {
        let json = "{}";
        let permissions: Permissions = serde_json::from_str(json).unwrap();
        assert!(permissions.http.is_empty());
        assert!(!permissions.filesystem);
        assert!(!permissions.process);
    }

    // ==================== PluginManifest Tests ====================

    #[test]
    fn test_plugin_manifest_service_checker() {
        let json = r#"{
            "id": "youtube-checker",
            "name": "YouTube Checker",
            "version": "1.0.0",
            "author": "Test Author",
            "description": "Checks YouTube availability",
            "type": "service-checker",
            "service": {
                "id": "youtube",
                "name": "YouTube",
                "icon": "youtube.svg",
                "category": "video",
                "endpoints": [
                    {
                        "id": "main",
                        "name": "Main",
                        "url": "https://youtube.com"
                    }
                ]
            }
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "youtube-checker");
        assert_eq!(manifest.name, "YouTube Checker");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.author, "Test Author");
        assert_eq!(manifest.description, Some("Checks YouTube availability".to_string()));
        assert_eq!(manifest.plugin_type, PluginType::ServiceChecker);
        assert!(manifest.service.is_some());
        assert!(manifest.strategy.is_none());
        assert!(manifest.hostlist.is_none());
    }

    #[test]
    fn test_plugin_manifest_strategy_provider() {
        let json = r#"{
            "id": "zapret-strategy",
            "name": "Zapret Strategy",
            "version": "1.0.0",
            "author": "Test Author",
            "type": "strategy-provider",
            "strategy": {
                "id": "zapret-youtube",
                "name": "Zapret YouTube",
                "family": "zapret",
                "config_file": "youtube.yaml"
            }
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.plugin_type, PluginType::StrategyProvider);
        assert!(manifest.strategy.is_some());
        assert!(manifest.service.is_none());
        assert!(manifest.description.is_none());
    }

    #[test]
    fn test_plugin_manifest_hostlist_provider() {
        let json = r#"{
            "id": "custom-hostlist",
            "name": "Custom Hostlist",
            "version": "1.0.0",
            "author": "Test Author",
            "type": "hostlist-provider",
            "hostlist": {
                "id": "custom-hosts",
                "name": "Custom Hosts",
                "file": "hosts.txt"
            }
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.plugin_type, PluginType::HostlistProvider);
        assert!(manifest.hostlist.is_some());
    }

    #[test]
    fn test_plugin_manifest_with_permissions() {
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "type": "service-checker",
            "permissions": {
                "http": ["https://api.example.com"],
                "filesystem": true,
                "process": false
            }
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.permissions.http.len(), 1);
        assert!(manifest.permissions.filesystem);
        assert!(!manifest.permissions.process);
    }

    #[test]
    fn test_plugin_manifest_default_permissions() {
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "type": "service-checker"
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert!(manifest.permissions.http.is_empty());
        assert!(!manifest.permissions.filesystem);
        assert!(!manifest.permissions.process);
    }

    #[test]
    fn test_plugin_manifest_missing_required_field() {
        // Missing "author" field
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "type": "service-checker"
        }"#;
        let result: Result<PluginManifest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_manifest_invalid_type() {
        let json = r#"{
            "id": "test-plugin",
            "name": "Test Plugin",
            "version": "1.0.0",
            "author": "Test Author",
            "type": "unknown-type"
        }"#;
        let result: Result<PluginManifest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_manifest_empty_strings() {
        let json = r#"{
            "id": "",
            "name": "",
            "version": "",
            "author": "",
            "type": "service-checker"
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "");
        assert_eq!(manifest.name, "");
        // Note: Empty strings are valid JSON, validation logic would be separate
    }

    // ==================== LoadedPluginInfo Tests ====================

    #[test]
    fn test_loaded_plugin_info_enabled() {
        let manifest = PluginManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: "Author".to_string(),
            description: None,
            plugin_type: PluginType::ServiceChecker,
            service: None,
            strategy: None,
            hostlist: None,
            permissions: Permissions::default(),
        };
        let info = LoadedPluginInfo {
            manifest,
            enabled: true,
            path: "/plugins/test".to_string(),
            error: None,
        };
        assert!(info.enabled);
        assert!(info.error.is_none());
    }

    #[test]
    fn test_loaded_plugin_info_with_error() {
        let manifest = PluginManifest {
            id: "broken".to_string(),
            name: "Broken Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Author".to_string(),
            description: None,
            plugin_type: PluginType::ServiceChecker,
            service: None,
            strategy: None,
            hostlist: None,
            permissions: Permissions::default(),
        };
        let info = LoadedPluginInfo {
            manifest,
            enabled: false,
            path: "/plugins/broken".to_string(),
            error: Some("Failed to load plugin".to_string()),
        };
        assert!(!info.enabled);
        assert_eq!(info.error, Some("Failed to load plugin".to_string()));
    }

    #[test]
    fn test_loaded_plugin_info_serialization_roundtrip() {
        let manifest = PluginManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: "Author".to_string(),
            description: Some("Description".to_string()),
            plugin_type: PluginType::StrategyProvider,
            service: None,
            strategy: Some(StrategyDefinition {
                id: "strat".to_string(),
                name: "Strategy".to_string(),
                family: "zapret".to_string(),
                config_file: "config.yaml".to_string(),
            }),
            hostlist: None,
            permissions: Permissions {
                http: vec!["https://example.com".to_string()],
                filesystem: true,
                process: false,
            },
        };
        let info = LoadedPluginInfo {
            manifest,
            enabled: true,
            path: "/plugins/test".to_string(),
            error: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: LoadedPluginInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.manifest.id, "test");
        assert_eq!(deserialized.manifest.plugin_type, PluginType::StrategyProvider);
        assert!(deserialized.manifest.strategy.is_some());
        assert!(deserialized.enabled);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_manifest_with_unicode() {
        let json = r#"{
            "id": "unicode-plugin",
            "name": "Плагин с юникодом 🚀",
            "version": "1.0.0",
            "author": "Автор",
            "description": "Описание на русском",
            "type": "service-checker"
        }"#;
        let manifest: PluginManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.name, "Плагин с юникодом 🚀");
        assert_eq!(manifest.author, "Автор");
    }

    #[test]
    fn test_manifest_with_special_characters_in_url() {
        let json = r#"{
            "id": "endpoint",
            "name": "Endpoint",
            "url": "https://example.com/path?query=value&foo=bar"
        }"#;
        let endpoint: ServiceEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.url, "https://example.com/path?query=value&foo=bar");
    }

    #[test]
    fn test_permissions_with_many_http_urls() {
        let json = r#"{
            "http": [
                "https://api1.example.com",
                "https://api2.example.com",
                "https://api3.example.com",
                "https://api4.example.com",
                "https://api5.example.com"
            ],
            "filesystem": false,
            "process": false
        }"#;
        let permissions: Permissions = serde_json::from_str(json).unwrap();
        assert_eq!(permissions.http.len(), 5);
    }

    #[test]
    fn test_service_definition_multiple_endpoints() {
        let json = r#"{
            "id": "multi-service",
            "name": "Multi Service",
            "icon": "icon.svg",
            "category": "test",
            "endpoints": [
                {"id": "ep1", "name": "Endpoint 1", "url": "https://ep1.example.com"},
                {"id": "ep2", "name": "Endpoint 2", "url": "https://ep2.example.com", "method": "POST"},
                {"id": "ep3", "name": "Endpoint 3", "url": "https://ep3.example.com", "method": "HEAD"}
            ]
        }"#;
        let service: ServiceDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(service.endpoints.len(), 3);
        assert_eq!(service.endpoints[0].method, "GET");
        assert_eq!(service.endpoints[1].method, "POST");
        assert_eq!(service.endpoints[2].method, "HEAD");
    }
}

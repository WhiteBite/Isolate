//! Service Registry - manages registered services
//!
//! Services are loaded from plugin.json files in the plugins directory.
//! Each plugin can contribute service checkers via the `contributes.checkers` field.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Service not found: {0}")]
    NotFound(String),

    #[error("Service already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Failed to load plugin manifest: {0}")]
    ManifestError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// HTTP method for endpoint checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    GET,
    HEAD,
    POST,
}

/// Service endpoint for availability checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// URL to check
    pub url: String,
    /// Human-readable name for this endpoint
    pub name: String,
    /// HTTP method to use
    #[serde(default)]
    pub method: HttpMethod,
    /// Expected status codes (empty = any 2xx/3xx)
    #[serde(default)]
    pub expected_status: Vec<u16>,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

/// Service category for grouping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCategory {
    #[default]
    Social,
    Video,
    Gaming,
    Messaging,
    Streaming,
    Other,
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Unique service ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Icon (emoji or icon name)
    #[serde(default)]
    pub icon: Option<String>,
    /// Service category
    #[serde(default)]
    pub category: ServiceCategory,
    /// Endpoints to check for availability
    pub endpoints: Vec<ServiceEndpoint>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Plugin ID that contributed this service
    #[serde(default)]
    pub plugin_id: Option<String>,
}

impl Service {
    /// Create a new service with basic info
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: None,
            category: ServiceCategory::default(),
            endpoints: Vec::new(),
            description: None,
            plugin_id: None,
        }
    }

    /// Add an endpoint to check
    pub fn with_endpoint(mut self, url: impl Into<String>, name: impl Into<String>) -> Self {
        self.endpoints.push(ServiceEndpoint {
            url: url.into(),
            name: name.into(),
            method: HttpMethod::GET,
            expected_status: Vec::new(),
            timeout_ms: default_timeout(),
        });
        self
    }

    /// Set the icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: ServiceCategory) -> Self {
        self.category = category;
        self
    }

    /// Set the plugin ID
    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        self.plugin_id = Some(plugin_id.into());
        self
    }
}

/// Service Registry - stores and manages all registered services
pub struct ServiceRegistry {
    /// Registered services by ID
    services: RwLock<HashMap<String, Service>>,
}

impl ServiceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Create registry and load services from plugins directory
    pub async fn from_plugins_dir(plugins_dir: &Path) -> Result<Self, RegistryError> {
        let registry = Self::new();
        registry.load_from_plugins(plugins_dir).await?;
        Ok(registry)
    }

    /// Register a service
    pub async fn register(&self, service: Service) -> Result<(), RegistryError> {
        let mut services = self.services.write().await;
        
        if services.contains_key(&service.id) {
            return Err(RegistryError::AlreadyRegistered(service.id));
        }

        info!(service_id = %service.id, name = %service.name, "Registering service");
        services.insert(service.id.clone(), service);
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister(&self, service_id: &str) -> Result<(), RegistryError> {
        let mut services = self.services.write().await;
        
        services
            .remove(service_id)
            .ok_or_else(|| RegistryError::NotFound(service_id.to_string()))?;

        info!(service_id = %service_id, "Unregistered service");
        Ok(())
    }

    /// Get all registered services
    pub async fn get_all(&self) -> Vec<Service> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Get a service by ID
    pub async fn get(&self, service_id: &str) -> Option<Service> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// Get services by category
    pub async fn get_by_category(&self, category: ServiceCategory) -> Vec<Service> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.category == category)
            .cloned()
            .collect()
    }

    /// Get services contributed by a specific plugin
    pub async fn get_by_plugin(&self, plugin_id: &str) -> Vec<Service> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|s| s.plugin_id.as_deref() == Some(plugin_id))
            .cloned()
            .collect()
    }

    /// Load services from plugin.json files in plugins directory
    pub async fn load_from_plugins(&self, plugins_dir: &Path) -> Result<usize, RegistryError> {
        if !plugins_dir.exists() {
            debug!(path = %plugins_dir.display(), "Plugins directory does not exist");
            return Ok(0);
        }

        let mut loaded_count = 0;
        let mut entries = tokio::fs::read_dir(plugins_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Look for plugin directories containing plugin.json
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");

                if manifest_path.exists() {
                    match self.load_plugin_services(&manifest_path).await {
                        Ok(count) => {
                            loaded_count += count;
                        }
                        Err(e) => {
                            warn!(
                                path = %manifest_path.display(),
                                error = %e,
                                "Failed to load services from plugin"
                            );
                        }
                    }
                }
            }
        }

        info!(count = loaded_count, "Loaded services from plugins");
        Ok(loaded_count)
    }

    /// Load services from a single plugin.json file
    async fn load_plugin_services(&self, manifest_path: &Path) -> Result<usize, RegistryError> {
        let content = tokio::fs::read_to_string(manifest_path).await?;
        
        // Try new format first (service-checker type)
        if let Ok(manifest) = serde_json::from_str::<NewPluginManifest>(&content) {
            if manifest.plugin_type == "service-checker" {
                if let Some(service_def) = manifest.service {
                    let category = match service_def.category.to_lowercase().as_str() {
                        "communication" | "messaging" => ServiceCategory::Messaging,
                        "media" | "video" => ServiceCategory::Video,
                        "gaming" => ServiceCategory::Gaming,
                        "streaming" => ServiceCategory::Streaming,
                        "social" => ServiceCategory::Social,
                        _ => ServiceCategory::Other,
                    };
                    
                    let service = Service {
                        id: service_def.id.clone(),
                        name: service_def.name,
                        icon: Some(service_def.icon),
                        category,
                        endpoints: service_def
                            .endpoints
                            .into_iter()
                            .map(|ep| ServiceEndpoint {
                                url: ep.url,
                                name: ep.name,
                                method: if ep.method.to_uppercase() == "HEAD" {
                                    HttpMethod::HEAD
                                } else {
                                    HttpMethod::GET
                                },
                                expected_status: Vec::new(),
                                timeout_ms: default_timeout(),
                            })
                            .collect(),
                        description: service_def.description,
                        plugin_id: Some(manifest.id),
                    };

                    if self.register(service).await.is_ok() {
                        return Ok(1);
                    }
                }
            }
            return Ok(0);
        }
        
        // Fallback to old format (contributes.checkers)
        let manifest: PluginManifestPartial = serde_json::from_str(&content)
            .map_err(|e| RegistryError::ParseError(e.to_string()))?;

        let plugin_id = manifest.id.clone();
        let mut count = 0;

        // Load checkers as services
        if let Some(contributes) = manifest.contributes {
            for checker in contributes.checkers {
                let service = Service {
                    id: checker.id.clone(),
                    name: checker.name,
                    icon: checker.icon,
                    category: ServiceCategory::Other, // Could be extended in manifest
                    endpoints: checker
                        .urls
                        .into_iter()
                        .enumerate()
                        .map(|(i, url)| ServiceEndpoint {
                            url: url.clone(),
                            name: format!("Endpoint {}", i + 1),
                            method: HttpMethod::GET,
                            expected_status: Vec::new(),
                            timeout_ms: default_timeout(),
                        })
                        .collect(),
                    description: None,
                    plugin_id: Some(plugin_id.clone()),
                };

                // Try to register, ignore if already exists
                if self.register(service).await.is_ok() {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Register built-in services (Discord, YouTube, etc.)
    pub async fn register_builtin_services(&self) {
        // Discord
        let discord = Service::new("discord", "Discord")
            .with_icon("💬")
            .with_category(ServiceCategory::Messaging)
            .with_endpoint("https://discord.com/api/v10/gateway", "API Gateway")
            .with_endpoint("https://cdn.discordapp.com/", "CDN")
            .with_endpoint("https://discordstatus.com/api/v2/status.json", "Status");

        let _ = self.register(discord).await;

        // YouTube
        let youtube = Service::new("youtube", "YouTube")
            .with_icon("📺")
            .with_category(ServiceCategory::Video)
            .with_endpoint("https://www.youtube.com/", "Main")
            .with_endpoint("https://i.ytimg.com/", "Images")
            .with_endpoint("https://www.googleapis.com/youtube/v3/", "API");

        let _ = self.register(youtube).await;

        // Twitch
        let twitch = Service::new("twitch", "Twitch")
            .with_icon("🎮")
            .with_category(ServiceCategory::Streaming)
            .with_endpoint("https://www.twitch.tv/", "Main")
            .with_endpoint("https://api.twitch.tv/helix/", "API");

        let _ = self.register(twitch).await;

        // Telegram
        let telegram = Service::new("telegram", "Telegram")
            .with_icon("✈️")
            .with_category(ServiceCategory::Messaging)
            .with_endpoint("https://web.telegram.org/", "Web")
            .with_endpoint("https://api.telegram.org/", "API");

        let _ = self.register(telegram).await;

        // Twitter/X
        let twitter = Service::new("twitter", "Twitter/X")
            .with_icon("🐦")
            .with_category(ServiceCategory::Social)
            .with_endpoint("https://twitter.com/", "Main")
            .with_endpoint("https://x.com/", "X.com")
            .with_endpoint("https://api.twitter.com/", "API");

        let _ = self.register(twitter).await;

        // Instagram
        let instagram = Service::new("instagram", "Instagram")
            .with_icon("📷")
            .with_category(ServiceCategory::Social)
            .with_endpoint("https://www.instagram.com/", "Main")
            .with_endpoint("https://i.instagram.com/", "API");

        let _ = self.register(instagram).await;

        // Spotify
        let spotify = Service::new("spotify", "Spotify")
            .with_icon("🎵")
            .with_category(ServiceCategory::Streaming)
            .with_endpoint("https://open.spotify.com/", "Web Player")
            .with_endpoint("https://api.spotify.com/", "API");

        let _ = self.register(spotify).await;

        // Netflix
        let netflix = Service::new("netflix", "Netflix")
            .with_icon("🎬")
            .with_category(ServiceCategory::Streaming)
            .with_endpoint("https://www.netflix.com/", "Main")
            .with_endpoint("https://api.netflix.com/", "API");

        let _ = self.register(netflix).await;

        // TikTok
        let tiktok = Service::new("tiktok", "TikTok")
            .with_icon("🎵")
            .with_category(ServiceCategory::Social)
            .with_endpoint("https://www.tiktok.com/", "Main");

        let _ = self.register(tiktok).await;

        // Facebook
        let facebook = Service::new("facebook", "Facebook")
            .with_icon("👤")
            .with_category(ServiceCategory::Social)
            .with_endpoint("https://www.facebook.com/", "Main")
            .with_endpoint("https://graph.facebook.com/", "API");

        let _ = self.register(facebook).await;

        // WhatsApp
        let whatsapp = Service::new("whatsapp", "WhatsApp")
            .with_icon("💬")
            .with_category(ServiceCategory::Messaging)
            .with_endpoint("https://web.whatsapp.com/", "Web")
            .with_endpoint("https://www.whatsapp.com/", "Main");

        let _ = self.register(whatsapp).await;

        // Reddit
        let reddit = Service::new("reddit", "Reddit")
            .with_icon("🔴")
            .with_category(ServiceCategory::Social)
            .with_endpoint("https://www.reddit.com/", "Main")
            .with_endpoint("https://oauth.reddit.com/", "API");

        let _ = self.register(reddit).await;

        info!("Registered built-in services");
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Partial manifest structure for loading checkers
#[derive(Debug, Deserialize)]
struct PluginManifestPartial {
    id: String,
    contributes: Option<ContributesPartial>,
}

#[derive(Debug, Deserialize)]
struct ContributesPartial {
    #[serde(default)]
    checkers: Vec<CheckerContribution>,
}

#[derive(Debug, Deserialize)]
struct CheckerContribution {
    id: String,
    name: String,
    icon: Option<String>,
    urls: Vec<String>,
}

// New plugin manifest format (service-checker type)
#[derive(Debug, Deserialize)]
struct NewPluginManifest {
    id: String,
    #[serde(rename = "type")]
    plugin_type: String,
    service: Option<NewServiceDefinition>,
}

#[derive(Debug, Deserialize)]
struct NewServiceDefinition {
    id: String,
    name: String,
    icon: String,
    category: String,
    description: Option<String>,
    endpoints: Vec<NewEndpointDefinition>,
}

#[derive(Debug, Deserialize)]
struct NewEndpointDefinition {
    #[allow(dead_code)]
    id: String,
    name: String,
    url: String,
    #[serde(default = "default_method_str")]
    method: String,
}

fn default_method_str() -> String {
    "GET".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_register() {
        let registry = ServiceRegistry::new();
        
        let service = Service::new("test", "Test Service")
            .with_endpoint("https://example.com", "Main");

        registry.register(service).await.unwrap();

        let services = registry.get_all().await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "test");
    }

    #[tokio::test]
    async fn test_registry_get_by_id() {
        let registry = ServiceRegistry::new();
        
        let service = Service::new("my-service", "My Service")
            .with_icon("🔧")
            .with_category(ServiceCategory::Other)
            .with_endpoint("https://example.com/api", "API");

        registry.register(service).await.unwrap();

        // Test successful get
        let found = registry.get("my-service").await;
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "my-service");
        assert_eq!(found.name, "My Service");
        assert_eq!(found.icon, Some("🔧".to_string()));
        assert_eq!(found.category, ServiceCategory::Other);
        assert_eq!(found.endpoints.len(), 1);

        // Test get non-existent
        let not_found = registry.get("non-existent").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = ServiceRegistry::new();
        
        let service = Service::new("to-remove", "Service to Remove");
        registry.register(service).await.unwrap();

        // Verify it exists
        assert!(registry.get("to-remove").await.is_some());
        assert_eq!(registry.get_all().await.len(), 1);

        // Unregister
        registry.unregister("to-remove").await.unwrap();

        // Verify it's gone
        assert!(registry.get("to-remove").await.is_none());
        assert_eq!(registry.get_all().await.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_unregister_not_found() {
        let registry = ServiceRegistry::new();
        
        let result = registry.unregister("non-existent").await;
        assert!(matches!(result, Err(RegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_registry_duplicate() {
        let registry = ServiceRegistry::new();
        
        let service1 = Service::new("test", "Test 1");
        let service2 = Service::new("test", "Test 2");

        registry.register(service1).await.unwrap();
        let result = registry.register(service2).await;

        assert!(matches!(result, Err(RegistryError::AlreadyRegistered(_))));
        
        // Verify original is unchanged
        let service = registry.get("test").await.unwrap();
        assert_eq!(service.name, "Test 1");
    }

    #[tokio::test]
    async fn test_registry_get_by_category() {
        let registry = ServiceRegistry::new();
        
        let service1 = Service::new("s1", "Service 1").with_category(ServiceCategory::Video);
        let service2 = Service::new("s2", "Service 2").with_category(ServiceCategory::Messaging);
        let service3 = Service::new("s3", "Service 3").with_category(ServiceCategory::Video);
        let service4 = Service::new("s4", "Service 4").with_category(ServiceCategory::Gaming);

        registry.register(service1).await.unwrap();
        registry.register(service2).await.unwrap();
        registry.register(service3).await.unwrap();
        registry.register(service4).await.unwrap();

        let video_services = registry.get_by_category(ServiceCategory::Video).await;
        assert_eq!(video_services.len(), 2);
        assert!(video_services.iter().all(|s| s.category == ServiceCategory::Video));

        let messaging_services = registry.get_by_category(ServiceCategory::Messaging).await;
        assert_eq!(messaging_services.len(), 1);
        assert_eq!(messaging_services[0].id, "s2");

        let gaming_services = registry.get_by_category(ServiceCategory::Gaming).await;
        assert_eq!(gaming_services.len(), 1);

        // Empty category
        let streaming_services = registry.get_by_category(ServiceCategory::Streaming).await;
        assert_eq!(streaming_services.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_get_by_plugin() {
        let registry = ServiceRegistry::new();
        
        let service1 = Service::new("s1", "Service 1").with_plugin_id("plugin-a");
        let service2 = Service::new("s2", "Service 2").with_plugin_id("plugin-b");
        let service3 = Service::new("s3", "Service 3").with_plugin_id("plugin-a");
        let service4 = Service::new("s4", "Service 4"); // No plugin

        registry.register(service1).await.unwrap();
        registry.register(service2).await.unwrap();
        registry.register(service3).await.unwrap();
        registry.register(service4).await.unwrap();

        let plugin_a_services = registry.get_by_plugin("plugin-a").await;
        assert_eq!(plugin_a_services.len(), 2);

        let plugin_b_services = registry.get_by_plugin("plugin-b").await;
        assert_eq!(plugin_b_services.len(), 1);

        let unknown_plugin = registry.get_by_plugin("unknown").await;
        assert_eq!(unknown_plugin.len(), 0);
    }

    #[tokio::test]
    async fn test_builtin_services() {
        let registry = ServiceRegistry::new();
        registry.register_builtin_services().await;

        let services = registry.get_all().await;
        // Discord, YouTube, Twitch, Telegram, Twitter, Instagram, Spotify, Netflix, TikTok, Facebook, WhatsApp, Reddit
        assert!(services.len() >= 12);

        // Check Discord
        let discord = registry.get("discord").await;
        assert!(discord.is_some());
        let discord = discord.unwrap();
        assert_eq!(discord.name, "Discord");
        assert_eq!(discord.category, ServiceCategory::Messaging);
        assert!(!discord.endpoints.is_empty());

        // Check YouTube
        let youtube = registry.get("youtube").await;
        assert!(youtube.is_some());
        let youtube = youtube.unwrap();
        assert_eq!(youtube.category, ServiceCategory::Video);

        // Check Twitch
        let twitch = registry.get("twitch").await;
        assert!(twitch.is_some());
        assert_eq!(twitch.unwrap().category, ServiceCategory::Streaming);

        // Check categories are properly assigned
        let messaging = registry.get_by_category(ServiceCategory::Messaging).await;
        assert!(messaging.len() >= 3); // Discord, Telegram, WhatsApp

        let video = registry.get_by_category(ServiceCategory::Video).await;
        assert!(video.len() >= 1); // YouTube

        let social = registry.get_by_category(ServiceCategory::Social).await;
        assert!(social.len() >= 4); // Twitter, Instagram, TikTok, Facebook, Reddit
    }

    #[tokio::test]
    async fn test_builtin_services_no_duplicates() {
        let registry = ServiceRegistry::new();
        
        // Register builtin twice
        registry.register_builtin_services().await;
        registry.register_builtin_services().await;

        // Should still have same count (duplicates ignored)
        let services = registry.get_all().await;
        let discord_count = services.iter().filter(|s| s.id == "discord").count();
        assert_eq!(discord_count, 1);
    }

    #[test]
    fn test_service_builder() {
        let service = Service::new("test-id", "Test Name")
            .with_icon("🎯")
            .with_category(ServiceCategory::Gaming)
            .with_plugin_id("my-plugin")
            .with_endpoint("https://api.example.com", "API")
            .with_endpoint("https://cdn.example.com", "CDN");

        assert_eq!(service.id, "test-id");
        assert_eq!(service.name, "Test Name");
        assert_eq!(service.icon, Some("🎯".to_string()));
        assert_eq!(service.category, ServiceCategory::Gaming);
        assert_eq!(service.plugin_id, Some("my-plugin".to_string()));
        assert_eq!(service.endpoints.len(), 2);
        assert_eq!(service.endpoints[0].url, "https://api.example.com");
        assert_eq!(service.endpoints[1].name, "CDN");
    }

    #[test]
    fn test_http_method_default() {
        let method: HttpMethod = Default::default();
        assert_eq!(method, HttpMethod::GET);
    }

    #[test]
    fn test_service_category_default() {
        let category: ServiceCategory = Default::default();
        assert_eq!(category, ServiceCategory::Social);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(default_timeout(), 5000);
    }

    #[test]
    fn test_service_endpoint_creation() {
        let endpoint = ServiceEndpoint {
            url: "https://api.test.com".to_string(),
            name: "Test API".to_string(),
            method: HttpMethod::HEAD,
            expected_status: vec![200, 204],
            timeout_ms: 3000,
        };

        assert_eq!(endpoint.url, "https://api.test.com");
        assert_eq!(endpoint.name, "Test API");
        assert_eq!(endpoint.method, HttpMethod::HEAD);
        assert_eq!(endpoint.expected_status, vec![200, 204]);
        assert_eq!(endpoint.timeout_ms, 3000);
    }

    #[test]
    fn test_service_endpoint_default_values() {
        // Test that default method is GET
        let endpoint = ServiceEndpoint {
            url: "https://example.com".to_string(),
            name: "Test".to_string(),
            method: HttpMethod::default(),
            expected_status: Vec::new(),
            timeout_ms: default_timeout(),
        };

        assert_eq!(endpoint.method, HttpMethod::GET);
        assert!(endpoint.expected_status.is_empty());
        assert_eq!(endpoint.timeout_ms, 5000);
    }

    #[test]
    fn test_service_registry_default() {
        let registry = ServiceRegistry::default();
        // Default registry should be empty - need async context to verify
        // Just verify it creates without panic
        assert!(std::mem::size_of_val(&registry) > 0);
    }

    #[tokio::test]
    async fn test_empty_registry() {
        let registry = ServiceRegistry::new();

        let all = registry.get_all().await;
        assert!(all.is_empty());

        let by_category = registry.get_by_category(ServiceCategory::Video).await;
        assert!(by_category.is_empty());

        let by_plugin = registry.get_by_plugin("any").await;
        assert!(by_plugin.is_empty());

        let by_id = registry.get("any").await;
        assert!(by_id.is_none());
    }

    #[test]
    fn test_service_new_defaults() {
        let service = Service::new("id", "name");

        assert_eq!(service.id, "id");
        assert_eq!(service.name, "name");
        assert!(service.icon.is_none());
        assert_eq!(service.category, ServiceCategory::Social); // default
        assert!(service.endpoints.is_empty());
        assert!(service.description.is_none());
        assert!(service.plugin_id.is_none());
    }

    #[test]
    fn test_http_method_variants() {
        assert_eq!(HttpMethod::GET, HttpMethod::GET);
        assert_eq!(HttpMethod::HEAD, HttpMethod::HEAD);
        assert_eq!(HttpMethod::POST, HttpMethod::POST);
        assert_ne!(HttpMethod::GET, HttpMethod::HEAD);
        assert_ne!(HttpMethod::GET, HttpMethod::POST);
    }

    #[test]
    fn test_service_category_variants() {
        assert_eq!(ServiceCategory::Social, ServiceCategory::Social);
        assert_eq!(ServiceCategory::Video, ServiceCategory::Video);
        assert_eq!(ServiceCategory::Gaming, ServiceCategory::Gaming);
        assert_eq!(ServiceCategory::Messaging, ServiceCategory::Messaging);
        assert_eq!(ServiceCategory::Streaming, ServiceCategory::Streaming);
        assert_eq!(ServiceCategory::Other, ServiceCategory::Other);
        
        // All variants are different
        assert_ne!(ServiceCategory::Social, ServiceCategory::Video);
        assert_ne!(ServiceCategory::Gaming, ServiceCategory::Messaging);
    }

    #[test]
    fn test_service_with_endpoint_creates_correct_defaults() {
        let service = Service::new("test", "Test")
            .with_endpoint("https://example.com", "Main");

        assert_eq!(service.endpoints.len(), 1);
        let ep = &service.endpoints[0];
        assert_eq!(ep.url, "https://example.com");
        assert_eq!(ep.name, "Main");
        assert_eq!(ep.method, HttpMethod::GET);
        assert!(ep.expected_status.is_empty());
        assert_eq!(ep.timeout_ms, 5000);
    }

    #[tokio::test]
    async fn test_registry_multiple_operations() {
        let registry = ServiceRegistry::new();

        // Register multiple services
        for i in 0..5 {
            let service = Service::new(format!("service-{}", i), format!("Service {}", i))
                .with_category(if i % 2 == 0 { ServiceCategory::Video } else { ServiceCategory::Gaming });
            registry.register(service).await.unwrap();
        }

        assert_eq!(registry.get_all().await.len(), 5);
        assert_eq!(registry.get_by_category(ServiceCategory::Video).await.len(), 3);
        assert_eq!(registry.get_by_category(ServiceCategory::Gaming).await.len(), 2);

        // Unregister some
        registry.unregister("service-0").await.unwrap();
        registry.unregister("service-2").await.unwrap();

        assert_eq!(registry.get_all().await.len(), 3);
        assert_eq!(registry.get_by_category(ServiceCategory::Video).await.len(), 1);
        assert_eq!(registry.get_by_category(ServiceCategory::Gaming).await.len(), 2);
    }

    #[tokio::test]
    async fn test_load_from_nonexistent_plugins_dir() {
        let registry = ServiceRegistry::new();
        let result = registry.load_from_plugins(Path::new("/nonexistent/path")).await;
        
        // Should return Ok(0) for non-existent directory
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_registry_error_display() {
        let not_found = RegistryError::NotFound("test".to_string());
        assert!(not_found.to_string().contains("test"));

        let already_registered = RegistryError::AlreadyRegistered("dup".to_string());
        assert!(already_registered.to_string().contains("dup"));

        let manifest_error = RegistryError::ManifestError("bad json".to_string());
        assert!(manifest_error.to_string().contains("bad json"));

        let parse_error = RegistryError::ParseError("syntax".to_string());
        assert!(parse_error.to_string().contains("syntax"));
    }

    #[test]
    fn test_service_clone() {
        let service = Service::new("clone-test", "Clone Test")
            .with_icon("🔄")
            .with_category(ServiceCategory::Other)
            .with_plugin_id("plugin")
            .with_endpoint("https://example.com", "Main");

        let cloned = service.clone();

        assert_eq!(cloned.id, service.id);
        assert_eq!(cloned.name, service.name);
        assert_eq!(cloned.icon, service.icon);
        assert_eq!(cloned.category, service.category);
        assert_eq!(cloned.plugin_id, service.plugin_id);
        assert_eq!(cloned.endpoints.len(), service.endpoints.len());
    }

    #[test]
    fn test_service_endpoint_clone() {
        let endpoint = ServiceEndpoint {
            url: "https://test.com".to_string(),
            name: "Test".to_string(),
            method: HttpMethod::POST,
            expected_status: vec![200, 201],
            timeout_ms: 1000,
        };

        let cloned = endpoint.clone();

        assert_eq!(cloned.url, endpoint.url);
        assert_eq!(cloned.name, endpoint.name);
        assert_eq!(cloned.method, endpoint.method);
        assert_eq!(cloned.expected_status, endpoint.expected_status);
        assert_eq!(cloned.timeout_ms, endpoint.timeout_ms);
    }
}

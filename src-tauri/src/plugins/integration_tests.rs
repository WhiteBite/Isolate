//! Plugin System Integration Tests
//!
//! Comprehensive tests for the Isolate plugin system:
//! - PluginManager: init, load, unload, list, get_by_type
//! - HostlistRegistry: register, get, merge, domains
//! - StrategyRegistry: register, get, enable/disable
//! - LuaRuntime: execute, check, permissions, storage
//! - ScriptExecutor: execute_check, execute_raw, list_scripts

#[cfg(test)]
mod plugin_manager_tests {
    use crate::plugins::{PluginManager, PluginType};
    use std::path::Path;
    use tempfile::TempDir;

    async fn create_test_plugin(plugins_dir: &Path, id: &str, plugin_type: &str) {
        let plugin_dir = plugins_dir.join(id);
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        let manifest = format!(
            r#"{{
                "id": "{}",
                "name": "Test Plugin {}",
                "version": "1.0.0",
                "author": "Test Author",
                "type": "{}"
            }}"#,
            id, id, plugin_type
        );

        tokio::fs::write(plugin_dir.join("plugin.json"), manifest)
            .await
            .unwrap();
    }

    async fn create_plugin_with_service(plugins_dir: &Path, id: &str) {
        let plugin_dir = plugins_dir.join(id);
        tokio::fs::create_dir_all(&plugin_dir).await.unwrap();

        let manifest = format!(
            r#"{{
                "id": "{}",
                "name": "Service Plugin {}",
                "version": "1.0.0",
                "author": "Test Author",
                "type": "service-checker",
                "service": {{
                    "id": "{}-service",
                    "name": "Test Service",
                    "icon": "test.svg",
                    "category": "test",
                    "endpoints": [
                        {{
                            "id": "main",
                            "name": "Main",
                            "url": "https://example.com"
                        }}
                    ]
                }}
            }}"#,
            id, id, id
        );

        tokio::fs::write(plugin_dir.join("plugin.json"), manifest)
            .await
            .unwrap();
    }


    #[tokio::test]
    async fn test_init_creates_plugins_dir() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");

        let manager = PluginManager::new(&plugins_dir);
        let count = manager.init().await.unwrap();

        assert_eq!(count, 0);
        assert!(plugins_dir.exists());
    }

    #[tokio::test]
    async fn test_init_loads_existing_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "plugin-a", "service-checker").await;
        create_test_plugin(plugins_dir, "plugin-b", "strategy-provider").await;
        create_test_plugin(plugins_dir, "plugin-c", "hostlist-provider").await;

        let manager = PluginManager::new(plugins_dir);
        let count = manager.init().await.unwrap();

        assert_eq!(count, 3);
        assert_eq!(manager.loaded_count().await, 3);
    }

    #[tokio::test]
    async fn test_reload_all_updates_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "plugin-a", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();
        assert_eq!(manager.count().await, 1);

        create_test_plugin(plugins_dir, "plugin-b", "service-checker").await;

        let count = manager.reload_all().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_load_already_loaded_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "test-plugin", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let result = manager.load("test-plugin").await;
        assert!(result.is_ok());
        assert!(manager.is_loaded("test-plugin").await);
    }

    #[tokio::test]
    async fn test_unload_and_reload_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "test-plugin", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        manager.unload("test-plugin").await.unwrap();
        assert!(!manager.is_loaded("test-plugin").await);
        assert_eq!(manager.loaded_count().await, 0);

        manager.load("test-plugin").await.unwrap();
        assert!(manager.is_loaded("test-plugin").await);
        assert_eq!(manager.loaded_count().await, 1);
    }

    #[tokio::test]
    async fn test_unload_already_unloaded() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "test-plugin", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        manager.unload("test-plugin").await.unwrap();
        let result = manager.unload("test-plugin").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_nonexistent_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path());
        manager.init().await.unwrap();

        let result = manager.load("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unload_nonexistent_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path());
        manager.init().await.unwrap();

        let result = manager.unload("nonexistent").await;
        assert!(result.is_err());
    }


    #[tokio::test]
    async fn test_list_returns_all_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "plugin-a", "service-checker").await;
        create_test_plugin(plugins_dir, "plugin-b", "strategy-provider").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let plugins = manager.list().await;
        assert_eq!(plugins.len(), 2);

        let ids: Vec<_> = plugins.iter().map(|p| p.info.manifest.id.as_str()).collect();
        assert!(ids.contains(&"plugin-a"));
        assert!(ids.contains(&"plugin-b"));
    }

    #[tokio::test]
    async fn test_list_loaded_excludes_unloaded() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "plugin-a", "service-checker").await;
        create_test_plugin(plugins_dir, "plugin-b", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        manager.unload("plugin-a").await.unwrap();

        let loaded = manager.list_loaded().await;
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].info.manifest.id, "plugin-b");
    }

    #[tokio::test]
    async fn test_get_plugin_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "test-plugin", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let plugin = manager.get("test-plugin").await;
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().info.manifest.id, "test-plugin");

        let nonexistent = manager.get("nonexistent").await;
        assert!(nonexistent.is_none());
    }

    #[tokio::test]
    async fn test_get_by_type_service_checker() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "checker-1", "service-checker").await;
        create_test_plugin(plugins_dir, "checker-2", "service-checker").await;
        create_test_plugin(plugins_dir, "strategy-1", "strategy-provider").await;
        create_test_plugin(plugins_dir, "hostlist-1", "hostlist-provider").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let checkers = manager.get_by_type(PluginType::ServiceChecker).await;
        assert_eq!(checkers.len(), 2);

        let strategies = manager.get_by_type(PluginType::StrategyProvider).await;
        assert_eq!(strategies.len(), 1);

        let hostlists = manager.get_by_type(PluginType::HostlistProvider).await;
        assert_eq!(hostlists.len(), 1);
    }

    #[tokio::test]
    async fn test_get_by_type_excludes_unloaded() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "checker-1", "service-checker").await;
        create_test_plugin(plugins_dir, "checker-2", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        manager.unload("checker-1").await.unwrap();

        let checkers = manager.get_by_type(PluginType::ServiceChecker).await;
        assert_eq!(checkers.len(), 1);
        assert_eq!(checkers[0].info.manifest.id, "checker-2");
    }

    #[tokio::test]
    async fn test_get_services_from_loaded_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_plugin_with_service(plugins_dir, "service-plugin").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        let services = manager.get_services().await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "service-plugin-service");
    }

    #[tokio::test]
    async fn test_count_methods() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        create_test_plugin(plugins_dir, "plugin-a", "service-checker").await;
        create_test_plugin(plugins_dir, "plugin-b", "service-checker").await;
        create_test_plugin(plugins_dir, "plugin-c", "service-checker").await;

        let manager = PluginManager::new(plugins_dir);
        manager.init().await.unwrap();

        assert_eq!(manager.count().await, 3);
        assert_eq!(manager.loaded_count().await, 3);

        manager.unload("plugin-a").await.unwrap();

        assert_eq!(manager.count().await, 3);
        assert_eq!(manager.loaded_count().await, 2);
    }

    #[tokio::test]
    async fn test_plugins_dir_accessor() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("my-plugins");

        let manager = PluginManager::new(&plugins_dir);
        assert_eq!(manager.plugins_dir(), plugins_dir);
    }

    #[tokio::test]
    async fn test_default_plugin_manager() {
        let manager = PluginManager::default();
        assert_eq!(manager.plugins_dir().to_str().unwrap(), "plugins");
    }
}


#[cfg(test)]
mod hostlist_registry_tests {
    use crate::plugins::hostlist_registry::{HostlistRegistry, HostlistRegistryError};
    use crate::plugins::manifest::{HostlistDefinition, HostlistFormat};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_definition(id: &str, domains: Vec<String>) -> HostlistDefinition {
        HostlistDefinition {
            id: id.to_string(),
            name: format!("Test Hostlist {}", id),
            description: Some(format!("Description for {}", id)),
            format: HostlistFormat::Plain,
            domains,
            file: None,
            update_url: None,
            update_interval: None,
            category: Some("test".to_string()),
            tags: vec!["test".to_string()],
        }
    }

    #[tokio::test]
    async fn test_register_and_get_hostlist() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-1", vec!["example.com".to_string()]);

        registry
            .register("plugin-1", PathBuf::from("."), definition)
            .await
            .unwrap();

        let hostlist = registry.get("test-1").await;
        assert!(hostlist.is_some());

        let hostlist = hostlist.unwrap();
        assert_eq!(hostlist.definition.id, "test-1");
        assert_eq!(hostlist.plugin_id, "plugin-1");
        assert!(hostlist.enabled);
    }

    #[tokio::test]
    async fn test_register_duplicate_fails() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-1", vec!["example.com".to_string()]);

        registry
            .register("plugin-1", PathBuf::from("."), definition.clone())
            .await
            .unwrap();

        let result = registry
            .register("plugin-2", PathBuf::from("."), definition)
            .await;

        assert!(matches!(result, Err(HostlistRegistryError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_unregister_hostlist() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition("test-1", vec!["example.com".to_string()]);

        registry
            .register("plugin-1", PathBuf::from("."), definition)
            .await
            .unwrap();

        assert_eq!(registry.count().await, 1);

        registry.unregister("test-1").await.unwrap();

        assert_eq!(registry.count().await, 0);
        assert!(registry.get("test-1").await.is_none());
    }

    #[tokio::test]
    async fn test_unregister_not_found() {
        let registry = HostlistRegistry::new();

        let result = registry.unregister("nonexistent").await;
        assert!(matches!(result, Err(HostlistRegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_unregister_plugin_removes_all_hostlists() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["b.com".to_string()]);
        let def3 = create_test_definition("hostlist-3", vec!["c.com".to_string()]);

        registry.register("plugin-a", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin-a", PathBuf::from("."), def2).await.unwrap();
        registry.register("plugin-b", PathBuf::from("."), def3).await.unwrap();

        assert_eq!(registry.count().await, 3);

        let removed = registry.unregister_plugin("plugin-a").await;
        assert_eq!(removed, 2);
        assert_eq!(registry.count().await, 1);

        assert!(registry.get("hostlist-3").await.is_some());
    }

    #[tokio::test]
    async fn test_get_domains() {
        let registry = HostlistRegistry::new();
        let definition = create_test_definition(
            "test-1",
            vec!["example.com".to_string(), "test.com".to_string()],
        );

        registry.register("plugin-1", PathBuf::from("."), definition).await.unwrap();

        let domains = registry.get_domains("test-1").await.unwrap();
        assert_eq!(domains.len(), 2);
        assert!(domains.contains(&"example.com".to_string()));
        assert!(domains.contains(&"test.com".to_string()));
    }

    #[tokio::test]
    async fn test_get_domains_not_found() {
        let registry = HostlistRegistry::new();

        let result = registry.get_domains("nonexistent").await;
        assert!(matches!(result, Err(HostlistRegistryError::NotFound(_))));
    }


    #[tokio::test]
    async fn test_merge_hostlists_deduplicates() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string(), "b.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["b.com".to_string(), "c.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();

        let merged = registry.merge_hostlists(&["hostlist-1", "hostlist-2"]).await.unwrap();

        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"a.com".to_string()));
        assert!(merged.contains(&"b.com".to_string()));
        assert!(merged.contains(&"c.com".to_string()));
    }

    #[tokio::test]
    async fn test_merge_hostlists_sorted() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["z.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["a.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();

        let merged = registry.merge_hostlists(&["hostlist-1", "hostlist-2"]).await.unwrap();

        assert_eq!(merged[0], "a.com");
        assert_eq!(merged[1], "z.com");
    }

    #[tokio::test]
    async fn test_merge_hostlists_not_found() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string()]);
        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();

        let result = registry.merge_hostlists(&["hostlist-1", "nonexistent"]).await;

        assert!(matches!(result, Err(HostlistRegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_merge_all_enabled() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["b.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();

        registry.set_enabled("hostlist-1", false).await.unwrap();

        let merged = registry.merge_all().await;
        assert_eq!(merged.len(), 1);
        assert!(merged.contains(&"b.com".to_string()));
    }

    #[tokio::test]
    async fn test_list_hostlists() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["b.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();

        let list = registry.list().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_plugin() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["b.com".to_string()]);
        let def3 = create_test_definition("hostlist-3", vec!["c.com".to_string()]);

        registry.register("plugin-a", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin-a", PathBuf::from("."), def2).await.unwrap();
        registry.register("plugin-b", PathBuf::from("."), def3).await.unwrap();

        let plugin_a_lists = registry.list_by_plugin("plugin-a").await;
        assert_eq!(plugin_a_lists.len(), 2);

        let plugin_b_lists = registry.list_by_plugin("plugin-b").await;
        assert_eq!(plugin_b_lists.len(), 1);
    }

    #[tokio::test]
    async fn test_list_by_category() {
        let registry = HostlistRegistry::new();

        let mut def1 = create_test_definition("hostlist-1", vec!["a.com".to_string()]);
        def1.category = Some("social".to_string());

        let mut def2 = create_test_definition("hostlist-2", vec!["b.com".to_string()]);
        def2.category = Some("video".to_string());

        let mut def3 = create_test_definition("hostlist-3", vec!["c.com".to_string()]);
        def3.category = Some("social".to_string());

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def3).await.unwrap();

        let social = registry.list_by_category("social").await;
        assert_eq!(social.len(), 2);

        let video = registry.list_by_category("video").await;
        assert_eq!(video.len(), 1);

        let gaming = registry.list_by_category("gaming").await;
        assert_eq!(gaming.len(), 0);
    }


    #[tokio::test]
    async fn test_domain_matches_any() {
        let registry = HostlistRegistry::new();

        let definition = create_test_definition(
            "test-1",
            vec!["example.com".to_string(), "test.com".to_string()],
        );

        registry.register("plugin", PathBuf::from("."), definition).await.unwrap();

        assert!(registry.domain_matches_any("example.com").await);
        assert!(registry.domain_matches_any("test.com").await);
        assert!(!registry.domain_matches_any("other.com").await);
    }

    #[tokio::test]
    async fn test_domain_matches_disabled_hostlist() {
        let registry = HostlistRegistry::new();

        let definition = create_test_definition("test-1", vec!["example.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), definition).await.unwrap();

        assert!(registry.domain_matches_any("example.com").await);

        registry.set_enabled("test-1", false).await.unwrap();

        assert!(!registry.domain_matches_any("example.com").await);
    }

    #[tokio::test]
    async fn test_find_matching_hostlists() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["example.com".to_string(), "test.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["example.com".to_string()]);
        let def3 = create_test_definition("hostlist-3", vec!["other.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def3).await.unwrap();

        let matching = registry.find_matching_hostlists("example.com").await;
        assert_eq!(matching.len(), 2);
        assert!(matching.contains(&"hostlist-1".to_string()));
        assert!(matching.contains(&"hostlist-2".to_string()));
    }

    #[tokio::test]
    async fn test_set_enabled() {
        let registry = HostlistRegistry::new();

        let definition = create_test_definition("test-1", vec!["example.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), definition).await.unwrap();

        let hostlist = registry.get("test-1").await.unwrap();
        assert!(hostlist.enabled);

        registry.set_enabled("test-1", false).await.unwrap();
        let hostlist = registry.get("test-1").await.unwrap();
        assert!(!hostlist.enabled);

        registry.set_enabled("test-1", true).await.unwrap();
        let hostlist = registry.get("test-1").await.unwrap();
        assert!(hostlist.enabled);
    }

    #[tokio::test]
    async fn test_set_enabled_not_found() {
        let registry = HostlistRegistry::new();

        let result = registry.set_enabled("nonexistent", true).await;
        assert!(matches!(result, Err(HostlistRegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_stats() {
        let registry = HostlistRegistry::new();

        let def1 = create_test_definition("hostlist-1", vec!["a.com".to_string(), "b.com".to_string()]);
        let def2 = create_test_definition("hostlist-2", vec!["b.com".to_string(), "c.com".to_string()]);

        registry.register("plugin", PathBuf::from("."), def1).await.unwrap();
        registry.register("plugin", PathBuf::from("."), def2).await.unwrap();

        let stats = registry.stats().await;

        assert_eq!(stats.total_hostlists, 2);
        assert_eq!(stats.enabled_hostlists, 2);
        assert_eq!(stats.total_domains, 4);
        assert_eq!(stats.unique_domains, 3);
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("domains.txt");

        tokio::fs::write(&file_path, "domain1.com\ndomain2.com\n# comment\n\ndomain3.com")
            .await
            .unwrap();

        let registry = HostlistRegistry::new();

        let definition = HostlistDefinition {
            id: "file-hostlist".to_string(),
            name: "File Hostlist".to_string(),
            description: None,
            format: HostlistFormat::Plain,
            domains: vec![],
            file: Some("domains.txt".to_string()),
            update_url: None,
            update_interval: None,
            category: None,
            tags: vec![],
        };

        registry.register("plugin", temp_dir.path().to_path_buf(), definition).await.unwrap();

        let domains = registry.get_domains("file-hostlist").await.unwrap();

        assert_eq!(domains.len(), 3);
        assert!(domains.contains(&"domain1.com".to_string()));
        assert!(domains.contains(&"domain2.com".to_string()));
        assert!(domains.contains(&"domain3.com".to_string()));
    }

    #[tokio::test]
    async fn test_wildcard_format_matching() {
        let registry = HostlistRegistry::new();

        let definition = HostlistDefinition {
            id: "wildcard-hostlist".to_string(),
            name: "Wildcard Hostlist".to_string(),
            description: None,
            format: HostlistFormat::Wildcard,
            domains: vec!["*.discord.com".to_string(), "discord.gg".to_string()],
            file: None,
            update_url: None,
            update_interval: None,
            category: None,
            tags: vec![],
        };

        registry.register("plugin", PathBuf::from("."), definition).await.unwrap();

        assert!(registry.domain_matches_any("cdn.discord.com").await);
        assert!(registry.domain_matches_any("voice.discord.com").await);
        assert!(registry.domain_matches_any("discord.com").await);
        assert!(registry.domain_matches_any("discord.gg").await);
        assert!(!registry.domain_matches_any("other.com").await);
    }

    #[tokio::test]
    async fn test_default_registry() {
        let registry = HostlistRegistry::default();
        assert_eq!(registry.count().await, 0);
    }
}


#[cfg(test)]
mod strategy_registry_tests {
    use crate::plugins::manifest::{PluginStrategyConfig, PluginStrategyDefinition, StrategyFamily};
    use crate::plugins::strategy_registry::{RegistryError, StrategyRegistry, StrategySource};

    fn create_test_strategy(id: &str, name: &str) -> PluginStrategyDefinition {
        PluginStrategyDefinition {
            id: id.to_string(),
            name: name.to_string(),
            description: Some(format!("Description for {}", name)),
            family: StrategyFamily::Zapret,
            engine: "winws".to_string(),
            target_services: vec!["youtube".to_string(), "discord".to_string()],
            priority: 10,
            config: PluginStrategyConfig::default(),
            author: Some("Test Author".to_string()),
            label: Some("test".to_string()),
            source_plugin: None,
        }
    }

    #[tokio::test]
    async fn test_register_and_get_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = create_test_strategy("test-1", "Test Strategy 1");

        registry.register(strategy, StrategySource::Builtin).await.unwrap();

        assert_eq!(registry.count().await, 1);

        let retrieved = registry.get("test-1").await.unwrap();
        assert_eq!(retrieved.definition.name, "Test Strategy 1");
        assert!(matches!(retrieved.source, StrategySource::Builtin));
        assert!(retrieved.enabled);
    }

    #[tokio::test]
    async fn test_register_duplicate_fails() {
        let registry = StrategyRegistry::new();
        let strategy1 = create_test_strategy("test-1", "Test Strategy 1");
        let strategy2 = create_test_strategy("test-1", "Test Strategy 1 Duplicate");

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();

        let result = registry.register(strategy2, StrategySource::Builtin).await;
        assert!(matches!(result, Err(RegistryError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_unregister_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = create_test_strategy("test-1", "Test Strategy 1");

        registry.register(strategy, StrategySource::Builtin).await.unwrap();

        assert_eq!(registry.count().await, 1);

        let removed = registry.unregister("test-1").await.unwrap();
        assert_eq!(removed.definition.id, "test-1");
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_not_found() {
        let registry = StrategyRegistry::new();

        let result = registry.unregister("nonexistent").await;
        assert!(matches!(result, Err(RegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_register_from_plugin() {
        let registry = StrategyRegistry::new();

        let strategies = vec![
            create_test_strategy("strat-1", "Strategy 1"),
            create_test_strategy("strat-2", "Strategy 2"),
        ];

        let count = registry.register_from_plugin("test-plugin", strategies).await.unwrap();

        assert_eq!(count, 2);

        assert!(registry.get("test-plugin:strat-1").await.is_some());
        assert!(registry.get("test-plugin:strat-2").await.is_some());
    }

    #[tokio::test]
    async fn test_unregister_plugin_strategies() {
        let registry = StrategyRegistry::new();

        let strategies = vec![
            create_test_strategy("strat-1", "Strategy 1"),
            create_test_strategy("strat-2", "Strategy 2"),
        ];

        registry.register_from_plugin("test-plugin", strategies).await.unwrap();

        registry.register(create_test_strategy("builtin-1", "Builtin"), StrategySource::Builtin).await.unwrap();

        assert_eq!(registry.count().await, 3);

        let removed = registry.unregister_plugin("test-plugin").await;
        assert_eq!(removed, 2);
        assert_eq!(registry.count().await, 1);

        assert!(registry.get("builtin-1").await.is_some());
    }


    #[tokio::test]
    async fn test_list_strategies() {
        let registry = StrategyRegistry::new();

        registry.register(create_test_strategy("s1", "S1"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("s2", "S2"), StrategySource::Builtin).await.unwrap();

        let all = registry.list().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_list_enabled_strategies() {
        let registry = StrategyRegistry::new();

        registry.register(create_test_strategy("s1", "S1"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("s2", "S2"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("s3", "S3"), StrategySource::Builtin).await.unwrap();

        registry.disable("s2").await.unwrap();

        let all = registry.list().await;
        assert_eq!(all.len(), 3);

        let enabled = registry.list_enabled().await;
        assert_eq!(enabled.len(), 2);
    }

    #[tokio::test]
    async fn test_get_by_service() {
        let registry = StrategyRegistry::new();

        let mut strategy1 = create_test_strategy("yt-1", "YouTube Strategy");
        strategy1.target_services = vec!["youtube".to_string()];

        let mut strategy2 = create_test_strategy("discord-1", "Discord Strategy");
        strategy2.target_services = vec!["discord".to_string()];

        let mut strategy3 = create_test_strategy("general-1", "General Strategy");
        strategy3.target_services = vec!["youtube".to_string(), "discord".to_string()];

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();
        registry.register(strategy2, StrategySource::Builtin).await.unwrap();
        registry.register(strategy3, StrategySource::Builtin).await.unwrap();

        let youtube_strategies = registry.get_by_service("youtube").await;
        assert_eq!(youtube_strategies.len(), 2);

        let discord_strategies = registry.get_by_service("discord").await;
        assert_eq!(discord_strategies.len(), 2);

        let telegram_strategies = registry.get_by_service("telegram").await;
        assert_eq!(telegram_strategies.len(), 0);
    }

    #[tokio::test]
    async fn test_get_by_service_excludes_disabled() {
        let registry = StrategyRegistry::new();

        let mut strategy1 = create_test_strategy("yt-1", "YouTube Strategy 1");
        strategy1.target_services = vec!["youtube".to_string()];

        let mut strategy2 = create_test_strategy("yt-2", "YouTube Strategy 2");
        strategy2.target_services = vec!["youtube".to_string()];

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();
        registry.register(strategy2, StrategySource::Builtin).await.unwrap();

        registry.disable("yt-1").await.unwrap();

        let youtube_strategies = registry.get_by_service("youtube").await;
        assert_eq!(youtube_strategies.len(), 1);
        assert_eq!(youtube_strategies[0].definition.id, "yt-2");
    }

    #[tokio::test]
    async fn test_get_by_family() {
        let registry = StrategyRegistry::new();

        let mut strategy1 = create_test_strategy("zapret-1", "Zapret Strategy");
        strategy1.family = StrategyFamily::Zapret;

        let mut strategy2 = create_test_strategy("vless-1", "VLESS Strategy");
        strategy2.family = StrategyFamily::Vless;

        let mut strategy3 = create_test_strategy("shadowsocks-1", "Shadowsocks Strategy");
        strategy3.family = StrategyFamily::Shadowsocks;

        registry.register(strategy1, StrategySource::Builtin).await.unwrap();
        registry.register(strategy2, StrategySource::Builtin).await.unwrap();
        registry.register(strategy3, StrategySource::Builtin).await.unwrap();

        let zapret = registry.get_by_family(StrategyFamily::Zapret).await;
        assert_eq!(zapret.len(), 1);
        assert_eq!(zapret[0].definition.id, "zapret-1");

        let vless = registry.get_by_family(StrategyFamily::Vless).await;
        assert_eq!(vless.len(), 1);

        let shadowsocks = registry.get_by_family(StrategyFamily::Shadowsocks).await;
        assert_eq!(shadowsocks.len(), 1);

        let custom = registry.get_by_family(StrategyFamily::Custom).await;
        assert_eq!(custom.len(), 0);
    }

    #[tokio::test]
    async fn test_get_plugin_strategies() {
        let registry = StrategyRegistry::new();

        registry.register(create_test_strategy("builtin-1", "Builtin"), StrategySource::Builtin).await.unwrap();

        registry.register_from_plugin("test-plugin", vec![create_test_strategy("plugin-1", "Plugin Strategy")]).await.unwrap();

        let plugin_strategies = registry.get_plugin_strategies().await;
        assert_eq!(plugin_strategies.len(), 1);
        assert!(matches!(plugin_strategies[0].source, StrategySource::Plugin { .. }));
    }


    #[tokio::test]
    async fn test_get_strategies_from_plugin() {
        let registry = StrategyRegistry::new();

        registry.register_from_plugin("plugin-a", vec![
            create_test_strategy("s1", "S1"),
            create_test_strategy("s2", "S2"),
        ]).await.unwrap();

        registry.register_from_plugin("plugin-b", vec![create_test_strategy("s3", "S3")]).await.unwrap();

        let plugin_a = registry.get_strategies_from_plugin("plugin-a").await;
        assert_eq!(plugin_a.len(), 2);

        let plugin_b = registry.get_strategies_from_plugin("plugin-b").await;
        assert_eq!(plugin_b.len(), 1);

        let plugin_c = registry.get_strategies_from_plugin("plugin-c").await;
        assert_eq!(plugin_c.len(), 0);
    }

    #[tokio::test]
    async fn test_get_builtin_strategies() {
        let registry = StrategyRegistry::new();

        registry.register(create_test_strategy("builtin-1", "Builtin 1"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("builtin-2", "Builtin 2"), StrategySource::Builtin).await.unwrap();

        registry.register_from_plugin("plugin", vec![create_test_strategy("plugin-1", "Plugin")]).await.unwrap();

        let builtin = registry.get_builtin_strategies().await;
        assert_eq!(builtin.len(), 2);
    }

    #[tokio::test]
    async fn test_enable_disable_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = create_test_strategy("test-1", "Test Strategy");

        registry.register(strategy, StrategySource::Builtin).await.unwrap();

        assert!(registry.get("test-1").await.unwrap().enabled);
        assert_eq!(registry.enabled_count().await, 1);

        registry.disable("test-1").await.unwrap();
        assert!(!registry.get("test-1").await.unwrap().enabled);
        assert_eq!(registry.enabled_count().await, 0);

        registry.enable("test-1").await.unwrap();
        assert!(registry.get("test-1").await.unwrap().enabled);
        assert_eq!(registry.enabled_count().await, 1);
    }

    #[tokio::test]
    async fn test_enable_not_found() {
        let registry = StrategyRegistry::new();

        let result = registry.enable("nonexistent").await;
        assert!(matches!(result, Err(RegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_disable_not_found() {
        let registry = StrategyRegistry::new();

        let result = registry.disable("nonexistent").await;
        assert!(matches!(result, Err(RegistryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_clear_strategies() {
        let registry = StrategyRegistry::new();

        registry.register(create_test_strategy("s1", "S1"), StrategySource::Builtin).await.unwrap();
        registry.register(create_test_strategy("s2", "S2"), StrategySource::Builtin).await.unwrap();

        assert_eq!(registry.count().await, 2);

        registry.clear().await;

        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_strategy_family_from_str() {
        assert_eq!(StrategyFamily::from("zapret"), StrategyFamily::Zapret);
        assert_eq!(StrategyFamily::from("ZAPRET"), StrategyFamily::Zapret);
        assert_eq!(StrategyFamily::from("Zapret"), StrategyFamily::Zapret);
        assert_eq!(StrategyFamily::from("vless"), StrategyFamily::Vless);
        assert_eq!(StrategyFamily::from("VLESS"), StrategyFamily::Vless);
        assert_eq!(StrategyFamily::from("shadowsocks"), StrategyFamily::Shadowsocks);
        assert_eq!(StrategyFamily::from("unknown"), StrategyFamily::Custom);
        assert_eq!(StrategyFamily::from(""), StrategyFamily::Custom);
    }

    #[tokio::test]
    async fn test_strategy_family_display() {
        assert_eq!(format!("{}", StrategyFamily::Zapret), "zapret");
        assert_eq!(format!("{}", StrategyFamily::Vless), "vless");
        assert_eq!(format!("{}", StrategyFamily::Shadowsocks), "shadowsocks");
        assert_eq!(format!("{}", StrategyFamily::Custom), "custom");
    }

    #[tokio::test]
    async fn test_strategy_source_variants() {
        let builtin = StrategySource::Builtin;
        let plugin = StrategySource::Plugin { plugin_id: "test-plugin".to_string() };
        let custom = StrategySource::Custom;

        assert_eq!(builtin, StrategySource::Builtin);
        assert_eq!(custom, StrategySource::Custom);
        assert!(matches!(plugin, StrategySource::Plugin { .. }));
    }

    #[tokio::test]
    async fn test_default_registry() {
        let registry = StrategyRegistry::default();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_load_from_plugins_no_manager() {
        let registry = StrategyRegistry::new();

        let result = registry.load_from_plugins().await;
        assert!(matches!(result, Err(RegistryError::NoPluginManager)));
    }
}

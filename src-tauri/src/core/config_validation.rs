//! Configuration validation tests
//! 
//! This module contains tests to validate all YAML configuration files
//! for strategies and services to ensure they are correctly formatted
//! and reference valid resources.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use serde_yaml;
    use crate::core::models::service::Service;
    use crate::core::unified_strategy_loader::UnifiedStrategy;

    /// Get the project root directory (where Cargo.toml is located)
    fn get_project_root() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest_dir)
    }

    /// Get all YAML files in a directory
    fn get_yaml_files(dir: &Path) -> Vec<PathBuf> {
        if !dir.exists() {
            return vec![];
        }

        fs::read_dir(dir)
            .expect(&format!("Failed to read directory: {:?}", dir))
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "yaml" || path.extension()? == "yml" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn test_all_strategy_configs_parse() {
        let root = get_project_root();
        let strategies_dir = root.join("configs").join("strategies");
        
        if !strategies_dir.exists() {
            panic!("Strategies directory not found: {:?}", strategies_dir);
        }

        let yaml_files = get_yaml_files(&strategies_dir);
        assert!(!yaml_files.is_empty(), "No strategy YAML files found");

        let mut errors = Vec::new();
        let mut success_count = 0;

        for file_path in yaml_files {
            let content = fs::read_to_string(&file_path)
                .expect(&format!("Failed to read file: {:?}", file_path));
            
            match serde_yaml::from_str::<UnifiedStrategy>(&content) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{:?}: {}", file_path.file_name().unwrap(), e));
                }
            }
        }

        if !errors.is_empty() {
            panic!(
                "Failed to parse {} strategy configs:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }

        println!("✓ Successfully parsed {} strategy configs", success_count);
    }

    #[test]
    fn test_all_service_configs_parse() {
        let root = get_project_root();
        let services_dir = root.join("configs").join("services");
        
        if !services_dir.exists() {
            panic!("Services directory not found: {:?}", services_dir);
        }

        let yaml_files = get_yaml_files(&services_dir);
        assert!(!yaml_files.is_empty(), "No service YAML files found");

        let mut errors = Vec::new();
        let mut success_count = 0;

        for file_path in yaml_files {
            let content = fs::read_to_string(&file_path)
                .expect(&format!("Failed to read file: {:?}", file_path));
            
            match serde_yaml::from_str::<Service>(&content) {
                Ok(service) => {
                    // Validate service has required fields
                    assert!(!service.id.is_empty(), "Service ID cannot be empty in {:?}", file_path);
                    assert!(!service.name.is_empty(), "Service name cannot be empty in {:?}", file_path);
                    assert!(!service.tests.is_empty(), "Service must have at least one test in {:?}", file_path);
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{:?}: {}", file_path.file_name().unwrap(), e));
                }
            }
        }

        if !errors.is_empty() {
            panic!(
                "Failed to parse {} service configs:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }

        println!("✓ Successfully parsed {} service configs", success_count);
    }

    #[test]
    fn test_split_seqovl_pattern_uses_bin_files() {
        let root = get_project_root();
        let strategies_dir = root.join("configs").join("strategies");
        
        if !strategies_dir.exists() {
            return; // Skip if directory doesn't exist
        }

        let yaml_files = get_yaml_files(&strategies_dir);
        let mut errors = Vec::new();

        for file_path in yaml_files {
            let content = fs::read_to_string(&file_path)
                .expect(&format!("Failed to read file: {:?}", file_path));
            
            // Check if file contains --dpi-desync-split-seqovl-pattern
            if content.contains("--dpi-desync-split-seqovl-pattern") {
                // Find all occurrences
                for line in content.lines() {
                    if line.contains("--dpi-desync-split-seqovl-pattern") {
                        // Should reference a .bin file, not .txt
                        if line.contains(".txt") {
                            errors.push(format!(
                                "{:?}: --dpi-desync-split-seqovl-pattern should use .bin file, not .txt: {}",
                                file_path.file_name().unwrap(),
                                line.trim()
                            ));
                        } else if !line.contains(".bin") {
                            errors.push(format!(
                                "{:?}: --dpi-desync-split-seqovl-pattern should reference a .bin file: {}",
                                file_path.file_name().unwrap(),
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }

        if !errors.is_empty() {
            panic!(
                "Found {} invalid --dpi-desync-split-seqovl-pattern references:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }

        println!("✓ All --dpi-desync-split-seqovl-pattern parameters correctly reference .bin files");
    }

    #[test]
    fn test_google_service_exists() {
        let root = get_project_root();
        let google_service_path = root.join("configs").join("services").join("google.yaml");
        
        assert!(
            google_service_path.exists(),
            "google.yaml service config must exist at {:?}",
            google_service_path
        );

        let content = fs::read_to_string(&google_service_path)
            .expect("Failed to read google.yaml");
        
        let service: Service = serde_yaml::from_str(&content)
            .expect("Failed to parse google.yaml");

        // Validate google service structure
        assert_eq!(service.id, "google", "Service ID must be 'google'");
        assert!(!service.name.is_empty(), "Service name cannot be empty");
        assert!(!service.tests.is_empty(), "Service must have at least one test");
        
        // Check that it has reasonable tests
        assert!(service.tests.len() >= 3, "Google service should have at least 3 tests");

        println!("✓ google.yaml service config is valid");
    }

    #[test]
    fn test_google_hostlist_referenced() {
        let root = get_project_root();
        let strategies_dir = root.join("configs").join("strategies");
        
        if !strategies_dir.exists() {
            return;
        }

        let yaml_files = get_yaml_files(&strategies_dir);
        let mut found_google_reference = false;

        for file_path in yaml_files {
            let content = fs::read_to_string(&file_path)
                .expect(&format!("Failed to read file: {:?}", file_path));
            
            if content.contains("hostlists/google.txt") {
                found_google_reference = true;
                break;
            }
        }

        assert!(
            found_google_reference,
            "At least one strategy should reference hostlists/google.txt"
        );

        println!("✓ google.txt hostlist is referenced by strategies");
    }

    #[test]
    fn test_required_bin_files_declared() {
        let root = get_project_root();
        let strategies_dir = root.join("configs").join("strategies");
        
        if !strategies_dir.exists() {
            return;
        }

        let yaml_files = get_yaml_files(&strategies_dir);
        let mut errors = Vec::new();

        for file_path in yaml_files {
            let content = fs::read_to_string(&file_path)
                .expect(&format!("Failed to read file: {:?}", file_path));
            
            // Parse as UnifiedStrategy to check requirements
            if let Ok(strategy) = serde_yaml::from_str::<UnifiedStrategy>(&content) {
                let binaries = match &strategy {
                    UnifiedStrategy::HighLevel(s) => &s.requirements.binaries,
                    UnifiedStrategy::Zapret(s) => &s.requirements.binaries,
                };

                // Check if strategy uses .bin files in args
                let bin_files_in_args: Vec<&str> = content
                    .lines()
                    .filter(|line| line.contains(".bin"))
                    .filter_map(|line| {
                        // Extract .bin filename
                        if let Some(start) = line.find("binaries/") {
                            if let Some(end) = line[start..].find(".bin") {
                                return Some(&line[start..start + end + 4]);
                            }
                        }
                        None
                    })
                    .collect();

                // Check if all .bin files are declared in requirements
                for bin_file in bin_files_in_args {
                    if !binaries.iter().any(|b| b.contains(bin_file)) {
                        errors.push(format!(
                            "{:?}: Binary file '{}' used in args but not declared in requirements.binaries",
                            file_path.file_name().unwrap(),
                            bin_file
                        ));
                    }
                }
            }
        }

        if !errors.is_empty() {
            panic!(
                "Found {} strategies with undeclared binary dependencies:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }

        println!("✓ All binary files are properly declared in requirements");
    }

    #[test]
    fn test_multisplit_strategies_fixed() {
        let root = get_project_root();
        let strategies_dir = root.join("configs").join("strategies");
        
        if !strategies_dir.exists() {
            return;
        }

        let problematic_files = vec![
            "twitter_multisplit.yaml",
            "ai_multisplit.yaml",
            "meta_multisplit.yaml",
        ];

        for filename in problematic_files {
            let file_path = strategies_dir.join(filename);
            
            if !file_path.exists() {
                continue; // Skip if file doesn't exist
            }

            let content = fs::read_to_string(&file_path)
                .expect(&format!("Failed to read file: {:?}", file_path));
            
            // Check that split-seqovl-pattern uses .bin, not .txt
            let mut has_split_seqovl_pattern = false;
            let mut uses_bin_file = false;
            let mut uses_txt_file = false;

            for line in content.lines() {
                if line.contains("--dpi-desync-split-seqovl-pattern") {
                    has_split_seqovl_pattern = true;
                    if line.contains(".bin") {
                        uses_bin_file = true;
                    }
                    if line.contains(".txt") {
                        uses_txt_file = true;
                    }
                }
            }

            if has_split_seqovl_pattern {
                assert!(
                    uses_bin_file,
                    "{}: --dpi-desync-split-seqovl-pattern must use .bin file",
                    filename
                );
                assert!(
                    !uses_txt_file,
                    "{}: --dpi-desync-split-seqovl-pattern should not use .txt file",
                    filename
                );
            }

            // Check that tls_clienthello_www_google_com.bin is in requirements
            if has_split_seqovl_pattern && uses_bin_file {
                assert!(
                    content.contains("tls_clienthello_www_google_com.bin"),
                    "{}: Must declare tls_clienthello_www_google_com.bin in requirements",
                    filename
                );
            }
        }

        println!("✓ All multisplit strategies correctly use .bin files for split-seqovl-pattern");
    }
}

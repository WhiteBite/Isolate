//! Strategy Combiner for Isolate
//!
//! Combines multiple strategies from different categories into a single
//! unified winws command-line configuration.
//!
//! NOTE: This module is prepared for future multi-strategy support.
//! Currently not used in production but has comprehensive tests.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::core::strategy_combiner::{StrategyCombiner, StrategySelection};
//!
//! let combiner = StrategyCombiner::new(strategies, hostlists_dir, blobs_dir);
//! let selection = StrategySelection {
//!     youtube: Some("youtube_only".to_string()),
//!     discord: Some("discord_only".to_string()),
//!     ..Default::default()
//! };
//! let combined = combiner.combine(&selection)?;
//! // combined.args contains the full winws command line
//! ```

// Public API for future multi-strategy combination feature
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::core::strategy_loader::ZapretStrategy;

// ============================================================================
// Data Structures
// ============================================================================

/// User's strategy selection for combining strategies by category
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategySelection {
    /// Strategy ID for YouTube category
    pub youtube: Option<String>,
    /// Strategy ID for Discord category
    pub discord: Option<String>,
    /// Strategy ID for Telegram category
    pub telegram: Option<String>,
    /// Strategy ID for Games category
    pub games: Option<String>,
    /// Strategy ID for custom/user-defined strategy
    pub custom: Option<String>,
}

impl StrategySelection {
    /// Create new empty selection
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any strategy is selected
    pub fn is_empty(&self) -> bool {
        self.youtube.is_none()
            && self.discord.is_none()
            && self.telegram.is_none()
            && self.games.is_none()
            && self.custom.is_none()
    }

    /// Get all selected strategy IDs
    pub fn get_selected_ids(&self) -> Vec<&str> {
        let mut ids = Vec::new();
        if let Some(ref id) = self.youtube {
            ids.push(id.as_str());
        }
        if let Some(ref id) = self.discord {
            ids.push(id.as_str());
        }
        if let Some(ref id) = self.telegram {
            ids.push(id.as_str());
        }
        if let Some(ref id) = self.games {
            ids.push(id.as_str());
        }
        if let Some(ref id) = self.custom {
            ids.push(id.as_str());
        }
        ids
    }
}

/// Result of combining multiple strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedStrategy {
    /// Generated name for the combined strategy
    pub name: String,
    /// Description of what categories are included
    pub description: String,
    /// Combined winws command-line arguments
    pub args: Vec<String>,
    /// Merged TCP ports string
    pub tcp_ports: String,
    /// Merged UDP ports string
    pub udp_ports: String,
    /// List of active category names
    pub categories: Vec<String>,
}

// ============================================================================
// Strategy Combiner
// ============================================================================

/// Combines multiple strategies into a single winws configuration
pub struct StrategyCombiner {
    /// Available strategies
    strategies: Vec<ZapretStrategy>,
    /// Directory containing hostlist files
    hostlists_dir: PathBuf,
    /// Directory containing binary blob files
    blobs_dir: PathBuf,
}

impl StrategyCombiner {
    /// Create a new strategy combiner
    ///
    /// # Arguments
    /// * `strategies` - List of available strategies
    /// * `hostlists_dir` - Path to hostlists directory
    /// * `blobs_dir` - Path to blobs directory (fake TLS, QUIC files)
    pub fn new(
        strategies: Vec<ZapretStrategy>,
        hostlists_dir: impl AsRef<Path>,
        blobs_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            strategies,
            hostlists_dir: hostlists_dir.as_ref().to_path_buf(),
            blobs_dir: blobs_dir.as_ref().to_path_buf(),
        }
    }

    /// Get available strategies for a specific category
    ///
    /// # Arguments
    /// * `category` - Category name (youtube, discord, telegram, games, custom, general)
    ///
    /// # Returns
    /// * `Vec<&ZapretStrategy>` - Strategies matching the category
    pub fn get_strategies_for_category(&self, category: &str) -> Vec<&ZapretStrategy> {
        let category_lower = category.to_lowercase();
        self.strategies
            .iter()
            .filter(|s| {
                let strategy_category = format!("{:?}", s.category).to_lowercase();
                strategy_category == category_lower
            })
            .collect()
    }

    /// Combine selected strategies into a single configuration
    ///
    /// # Arguments
    /// * `selection` - User's strategy selection by category
    ///
    /// # Returns
    /// * `Ok(CombinedStrategy)` - Combined strategy with merged arguments
    /// * `Err` - Failed to combine strategies
    pub fn combine(&self, selection: &StrategySelection) -> Result<CombinedStrategy> {
        if selection.is_empty() {
            anyhow::bail!("No strategies selected");
        }

        // Collect selected strategies
        let mut selected_strategies: Vec<(&str, &ZapretStrategy)> = Vec::new();

        if let Some(ref id) = selection.youtube {
            if let Some(strategy) = self.find_strategy(id) {
                selected_strategies.push(("YouTube", strategy));
            } else {
                debug!("YouTube strategy '{}' not found", id);
            }
        }

        if let Some(ref id) = selection.discord {
            if let Some(strategy) = self.find_strategy(id) {
                selected_strategies.push(("Discord", strategy));
            } else {
                debug!("Discord strategy '{}' not found", id);
            }
        }

        if let Some(ref id) = selection.telegram {
            if let Some(strategy) = self.find_strategy(id) {
                selected_strategies.push(("Telegram", strategy));
            } else {
                debug!("Telegram strategy '{}' not found", id);
            }
        }

        if let Some(ref id) = selection.games {
            if let Some(strategy) = self.find_strategy(id) {
                selected_strategies.push(("Games", strategy));
            } else {
                debug!("Games strategy '{}' not found", id);
            }
        }

        if let Some(ref id) = selection.custom {
            if let Some(strategy) = self.find_strategy(id) {
                selected_strategies.push(("Custom", strategy));
            } else {
                debug!("Custom strategy '{}' not found", id);
            }
        }

        if selected_strategies.is_empty() {
            anyhow::bail!("None of the selected strategies were found");
        }

        // Extract active categories
        let categories: Vec<String> = selected_strategies
            .iter()
            .map(|(cat, _)| cat.to_string())
            .collect();

        // Merge ports
        let strategies_only: Vec<&ZapretStrategy> =
            selected_strategies.iter().map(|(_, s)| *s).collect();
        let (tcp_ports, udp_ports) = self.merge_ports(&strategies_only);

        // Generate combined arguments
        let args = self.generate_combined_args(&strategies_only, &tcp_ports, &udp_ports)?;

        // Generate name and description
        let name = self.generate_name(&categories);
        let description = self.generate_description(&categories);

        info!(
            "Combined {} strategies: {}",
            selected_strategies.len(),
            categories.join(", ")
        );

        Ok(CombinedStrategy {
            name,
            description,
            args,
            tcp_ports,
            udp_ports,
            categories,
        })
    }

    /// Find a strategy by its ID
    fn find_strategy(&self, id: &str) -> Option<&ZapretStrategy> {
        self.strategies.iter().find(|s| s.id == id)
    }

    /// Merge TCP and UDP ports from multiple strategies
    ///
    /// Combines port specifications, removing duplicates and sorting.
    fn merge_ports(&self, strategies: &[&ZapretStrategy]) -> (String, String) {
        let mut tcp_ports: HashSet<String> = HashSet::new();
        let mut udp_ports: HashSet<String> = HashSet::new();

        for strategy in strategies {
            if let Some(ref tcp) = strategy.ports.tcp {
                // Split by comma and add each port/range
                for port in tcp.split(',') {
                    tcp_ports.insert(port.trim().to_string());
                }
            }
            if let Some(ref udp) = strategy.ports.udp {
                for port in udp.split(',') {
                    udp_ports.insert(port.trim().to_string());
                }
            }
        }

        // Sort ports for consistent output
        let mut tcp_vec: Vec<String> = tcp_ports.into_iter().collect();
        let mut udp_vec: Vec<String> = udp_ports.into_iter().collect();
        
        // Custom sort: numeric ports first, then ranges
        tcp_vec.sort_by(|a, b| Self::compare_ports(a, b));
        udp_vec.sort_by(|a, b| Self::compare_ports(a, b));

        (tcp_vec.join(","), udp_vec.join(","))
    }

    /// Compare ports for sorting (numeric first, then ranges)
    fn compare_ports(a: &str, b: &str) -> std::cmp::Ordering {
        let a_num = a.split('-').next().and_then(|s| s.parse::<u32>().ok());
        let b_num = b.split('-').next().and_then(|s| s.parse::<u32>().ok());
        
        match (a_num, b_num) {
            (Some(a_n), Some(b_n)) => a_n.cmp(&b_n),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    }

    /// Merge profiles from multiple strategies
    fn merge_profiles<'a>(&self, strategies: &[&'a ZapretStrategy]) -> Vec<&'a crate::core::strategy_loader::StrategyProfile> {
        strategies
            .iter()
            .flat_map(|s| s.profiles.iter())
            .collect()
    }

    /// Generate combined winws arguments from multiple strategies
    fn generate_combined_args(
        &self,
        strategies: &[&ZapretStrategy],
        tcp_ports: &str,
        udp_ports: &str,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Add merged port filters at the beginning
        if !tcp_ports.is_empty() {
            args.push("--wf-tcp".to_string());
            args.push(tcp_ports.to_string());
        }

        if !udp_ports.is_empty() {
            args.push("--wf-udp".to_string());
            args.push(udp_ports.to_string());
        }

        // Add profiles from each strategy, separated by --new
        let mut is_first_profile = true;

        for strategy in strategies {
            for profile in &strategy.profiles {
                // Add --new separator between profiles (not before first)
                if !is_first_profile {
                    args.push("--new".to_string());
                }
                is_first_profile = false;

                // Add profile arguments
                self.add_profile_args(&mut args, profile)?;
            }
        }

        debug!("Generated {} combined args", args.len());
        Ok(args)
    }

    /// Add arguments for a single profile
    fn add_profile_args(
        &self,
        args: &mut Vec<String>,
        profile: &crate::core::strategy_loader::StrategyProfile,
    ) -> Result<()> {
        // Filter
        args.push("--filter".to_string());
        args.push(profile.filter.clone());

        // Hostlist
        if let Some(ref hostlist) = profile.hostlist {
            let path = self.hostlists_dir.join(hostlist);
            args.push("--hostlist".to_string());
            args.push(path.display().to_string());
        }

        // Hostlist exclude
        if let Some(ref hostlist_exclude) = profile.hostlist_exclude {
            let path = self.hostlists_dir.join(hostlist_exclude);
            args.push("--hostlist-exclude".to_string());
            args.push(path.display().to_string());
        }

        // Hostlist domains (inline)
        if let Some(ref domains) = profile.hostlist_domains {
            args.push("--hostlist-domains".to_string());
            args.push(domains.clone());
        }

        // IP set
        if let Some(ref ipset) = profile.ipset {
            let path = self.hostlists_dir.join(ipset);
            args.push("--ipset".to_string());
            args.push(path.display().to_string());
        }

        // IP set exclude
        if let Some(ref ipset_exclude) = profile.ipset_exclude {
            let path = self.hostlists_dir.join(ipset_exclude);
            args.push("--ipset-exclude".to_string());
            args.push(path.display().to_string());
        }

        // L7 protocol
        if let Some(ref l7) = profile.l7 {
            args.push("--dpi-desync-l7".to_string());
            args.push(l7.clone());
        }

        // IP ID
        if let Some(ref ip_id) = profile.ip_id {
            args.push("--dpi-desync-ipid".to_string());
            args.push(ip_id.clone());
        }

        // Desync attack type (required)
        args.push("--dpi-desync".to_string());
        args.push(profile.desync.clone());

        // Repeats
        if let Some(repeats) = profile.repeats {
            args.push("--dpi-desync-repeats".to_string());
            args.push(repeats.to_string());
        }

        // Split sequence overlap
        if let Some(split_seqovl) = profile.split_seqovl {
            args.push("--dpi-desync-split-seqovl".to_string());
            args.push(split_seqovl.to_string());
        }

        // Split position
        if let Some(ref split_pos) = profile.split_pos {
            args.push("--dpi-desync-split-pos".to_string());
            args.push(split_pos.clone());
        }

        // Split sequence overlap pattern
        if let Some(ref pattern) = profile.split_seqovl_pattern {
            args.push("--dpi-desync-split-seqovl-pattern".to_string());
            args.push(pattern.clone());
        }

        // Fooling method
        if let Some(ref fooling) = profile.fooling {
            args.push("--dpi-desync-fooling".to_string());
            args.push(fooling.clone());
        }

        // Fake TLS ClientHello
        if let Some(ref fake_tls) = profile.fake_tls {
            let path = self.blobs_dir.join(fake_tls);
            args.push("--dpi-desync-fake-tls".to_string());
            args.push(path.display().to_string());
        }

        // Fake QUIC Initial
        if let Some(ref fake_quic) = profile.fake_quic {
            let path = self.blobs_dir.join(fake_quic);
            args.push("--dpi-desync-fake-quic".to_string());
            args.push(path.display().to_string());
        }

        // Fake TLS modification
        if let Some(ref fake_tls_mod) = profile.fake_tls_mod {
            args.push("--dpi-desync-fake-tls-mod".to_string());
            args.push(fake_tls_mod.clone());
        }

        // Auto TTL
        if let Some(ref autottl) = profile.autottl {
            args.push("--dpi-desync-autottl".to_string());
            args.push(autottl.clone());
        }

        // Cutoff
        if let Some(ref cutoff) = profile.cutoff {
            args.push("--dpi-desync-cutoff".to_string());
            args.push(cutoff.clone());
        }

        Ok(())
    }

    /// Generate a name for the combined strategy
    fn generate_name(&self, active_categories: &[String]) -> String {
        if active_categories.len() == 1 {
            format!("{} Strategy", active_categories[0])
        } else {
            format!("Combined: {}", active_categories.join(" + "))
        }
    }

    /// Generate a description for the combined strategy
    fn generate_description(&self, active_categories: &[String]) -> String {
        match active_categories.len() {
            0 => "No categories selected".to_string(),
            1 => format!("DPI bypass for {}", active_categories[0]),
            2 => format!(
                "Combined DPI bypass for {} and {}",
                active_categories[0], active_categories[1]
            ),
            _ => {
                let last = active_categories.last().unwrap();
                let rest = &active_categories[..active_categories.len() - 1];
                format!("Combined DPI bypass for {} and {}", rest.join(", "), last)
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::strategy_loader::{StrategyCategory, StrategyPorts, StrategyProfile};

    fn create_test_strategy(id: &str, category: StrategyCategory, tcp: &str, udp: &str) -> ZapretStrategy {
        ZapretStrategy {
            id: id.to_string(),
            name: format!("{} Strategy", id),
            description: format!("Test strategy for {}", id),
            category,
            family: "zapret".to_string(),
            author: None,
            label: None,
            ports: StrategyPorts {
                tcp: if tcp.is_empty() { None } else { Some(tcp.to_string()) },
                udp: if udp.is_empty() { None } else { Some(udp.to_string()) },
            },
            profiles: vec![StrategyProfile {
                filter: "tcp".to_string(),
                hostlist: Some(format!("{}.txt", id)),
                hostlist_exclude: None,
                hostlist_domains: None,
                ipset: None,
                ipset_exclude: None,
                l7: Some("http".to_string()),
                ip_id: None,
                desync: "fake,split2".to_string(),
                repeats: Some(6),
                split_seqovl: None,
                split_pos: None,
                split_seqovl_pattern: None,
                fooling: None,
                fake_tls: None,
                fake_quic: None,
                fake_tls_mod: None,
                fake_wireguard: None,
                fake_dht: None,
                fake_unknown_udp: None,
                fake_tcp_mod: None,
                fake_syndata: None,
                ttl: None,
                ttl6: None,
                autottl: None,
                badseq_increment: None,
                badack_increment: None,
                ts_increment: None,
                cutoff: None,
                hostfakesplit_mod: None,
                hostfakesplit_midhost: None,
                fakedsplit_mod: None,
                wsize: None,
                wssize: None,
                wssize_cutoff: None,
                filter_l3: None,
                filter_ssid: None,
                nlm_filter: None,
                dup: None,
                dup_replace: None,
                dup_ttl: None,
                dup_autottl: None,
                dup_fooling: None,
                dup_start: None,
                dup_cutoff: None,
                orig_ttl: None,
                orig_autottl: None,
                orig_tcp_flags_set: None,
                orig_tcp_flags_unset: None,
                orig_mod_start: None,
                orig_mod_cutoff: None,
            }],
        }
    }

    #[test]
    fn test_strategy_selection_default() {
        let selection = StrategySelection::default();
        assert!(selection.is_empty());
        assert!(selection.get_selected_ids().is_empty());
    }

    #[test]
    fn test_strategy_selection_with_values() {
        let selection = StrategySelection {
            youtube: Some("yt-1".to_string()),
            discord: Some("discord-1".to_string()),
            telegram: None,
            games: None,
            custom: None,
        };

        assert!(!selection.is_empty());
        let ids = selection.get_selected_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"yt-1"));
        assert!(ids.contains(&"discord-1"));
    }

    #[test]
    fn test_strategy_selection_with_custom() {
        let selection = StrategySelection {
            youtube: None,
            discord: None,
            telegram: None,
            games: None,
            custom: Some("my-custom".to_string()),
        };

        assert!(!selection.is_empty());
        let ids = selection.get_selected_ids();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"my-custom"));
    }

    #[test]
    fn test_combine_single_strategy() {
        let strategies = vec![
            create_test_strategy("youtube_only", StrategyCategory::YouTube, "80,443", "443"),
        ];

        let combiner = StrategyCombiner::new(strategies, "hostlists", "blobs");
        let selection = StrategySelection {
            youtube: Some("youtube_only".to_string()),
            ..Default::default()
        };

        let combined = combiner.combine(&selection).unwrap();

        assert_eq!(combined.categories.len(), 1);
        assert_eq!(combined.categories[0], "YouTube");
        assert!(combined.tcp_ports.contains("80"));
        assert!(combined.tcp_ports.contains("443"));
        assert_eq!(combined.udp_ports, "443");
        assert!(combined.args.contains(&"--wf-tcp".to_string()));
        assert!(combined.args.contains(&"--dpi-desync".to_string()));
    }

    #[test]
    fn test_combine_multiple_strategies() {
        let strategies = vec![
            create_test_strategy("youtube_only", StrategyCategory::YouTube, "80,443", "443"),
            create_test_strategy("discord_only", StrategyCategory::Discord, "443", "50000-50100"),
        ];

        let combiner = StrategyCombiner::new(strategies, "hostlists", "blobs");
        let selection = StrategySelection {
            youtube: Some("youtube_only".to_string()),
            discord: Some("discord_only".to_string()),
            ..Default::default()
        };

        let combined = combiner.combine(&selection).unwrap();

        assert_eq!(combined.categories.len(), 2);
        assert!(combined.categories.contains(&"YouTube".to_string()));
        assert!(combined.categories.contains(&"Discord".to_string()));

        // Check merged ports (443 should appear once, 80 added)
        assert!(combined.tcp_ports.contains("443"));
        assert!(combined.tcp_ports.contains("80"));
        assert!(combined.udp_ports.contains("443"));
        assert!(combined.udp_ports.contains("50000-50100"));

        // Check --new separator exists between profiles
        assert!(combined.args.contains(&"--new".to_string()));
    }

    #[test]
    fn test_combine_empty_selection() {
        let strategies = vec![
            create_test_strategy("youtube_only", StrategyCategory::YouTube, "443", "443"),
        ];

        let combiner = StrategyCombiner::new(strategies, "hostlists", "blobs");
        let selection = StrategySelection::default();

        let result = combiner.combine(&selection);
        assert!(result.is_err());
    }

    #[test]
    fn test_combine_nonexistent_strategy() {
        let strategies = vec![
            create_test_strategy("youtube_only", StrategyCategory::YouTube, "443", "443"),
        ];

        let combiner = StrategyCombiner::new(strategies, "hostlists", "blobs");
        let selection = StrategySelection {
            youtube: Some("nonexistent".to_string()),
            ..Default::default()
        };

        let result = combiner.combine(&selection);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_ports_deduplication() {
        let strategies = vec![
            create_test_strategy("s1", StrategyCategory::YouTube, "80,443", "443"),
            create_test_strategy("s2", StrategyCategory::Discord, "443,8080", "443,50000"),
        ];

        let combiner = StrategyCombiner::new(strategies.clone(), "hostlists", "blobs");
        let refs: Vec<&ZapretStrategy> = strategies.iter().collect();
        let (tcp, udp) = combiner.merge_ports(&refs);

        // 443 should appear only once
        assert_eq!(tcp.matches("443").count(), 1);
        assert!(tcp.contains("80"));
        assert!(tcp.contains("8080"));

        assert_eq!(udp.matches("443").count(), 1);
        assert!(udp.contains("50000"));
    }

    #[test]
    fn test_merge_ports_sorting() {
        let strategies = vec![
            create_test_strategy("s1", StrategyCategory::YouTube, "8080,443,80", "50000-50100,443"),
        ];

        let combiner = StrategyCombiner::new(strategies.clone(), "hostlists", "blobs");
        let refs: Vec<&ZapretStrategy> = strategies.iter().collect();
        let (tcp, udp) = combiner.merge_ports(&refs);

        // Ports should be sorted numerically
        let tcp_parts: Vec<&str> = tcp.split(',').collect();
        assert_eq!(tcp_parts[0], "80");
        assert_eq!(tcp_parts[1], "443");
        assert_eq!(tcp_parts[2], "8080");
    }

    #[test]
    fn test_get_strategies_for_category() {
        let strategies = vec![
            create_test_strategy("yt-1", StrategyCategory::YouTube, "443", "443"),
            create_test_strategy("yt-2", StrategyCategory::YouTube, "443", "443"),
            create_test_strategy("discord-1", StrategyCategory::Discord, "443", "443"),
        ];

        let combiner = StrategyCombiner::new(strategies, "hostlists", "blobs");

        let youtube_strategies = combiner.get_strategies_for_category("youtube");
        assert_eq!(youtube_strategies.len(), 2);

        let discord_strategies = combiner.get_strategies_for_category("discord");
        assert_eq!(discord_strategies.len(), 1);

        let telegram_strategies = combiner.get_strategies_for_category("telegram");
        assert_eq!(telegram_strategies.len(), 0);
    }

    #[test]
    fn test_get_strategies_for_category_case_insensitive() {
        let strategies = vec![
            create_test_strategy("yt-1", StrategyCategory::YouTube, "443", "443"),
        ];

        let combiner = StrategyCombiner::new(strategies, "hostlists", "blobs");

        // Should work with different cases
        assert_eq!(combiner.get_strategies_for_category("YouTube").len(), 1);
        assert_eq!(combiner.get_strategies_for_category("youtube").len(), 1);
        assert_eq!(combiner.get_strategies_for_category("YOUTUBE").len(), 1);
    }

    #[test]
    fn test_generate_description() {
        let combiner = StrategyCombiner::new(vec![], "hostlists", "blobs");

        let desc1 = combiner.generate_description(&["YouTube".to_string()]);
        assert_eq!(desc1, "DPI bypass for YouTube");

        let desc2 = combiner.generate_description(&["YouTube".to_string(), "Discord".to_string()]);
        assert_eq!(desc2, "Combined DPI bypass for YouTube and Discord");

        let desc3 = combiner.generate_description(&[
            "YouTube".to_string(),
            "Discord".to_string(),
            "Telegram".to_string(),
        ]);
        assert_eq!(desc3, "Combined DPI bypass for YouTube, Discord and Telegram");
    }

    #[test]
    fn test_generate_name() {
        let combiner = StrategyCombiner::new(vec![], "hostlists", "blobs");

        let name1 = combiner.generate_name(&["YouTube".to_string()]);
        assert_eq!(name1, "YouTube Strategy");

        let name2 = combiner.generate_name(&["YouTube".to_string(), "Discord".to_string()]);
        assert_eq!(name2, "Combined: YouTube + Discord");
    }

    #[test]
    fn test_merge_profiles() {
        let strategies = vec![
            create_test_strategy("s1", StrategyCategory::YouTube, "443", "443"),
            create_test_strategy("s2", StrategyCategory::Discord, "443", "443"),
        ];

        let combiner = StrategyCombiner::new(strategies.clone(), "hostlists", "blobs");
        let refs: Vec<&ZapretStrategy> = strategies.iter().collect();
        let profiles = combiner.merge_profiles(&refs);

        // Each strategy has 1 profile, so total should be 2
        assert_eq!(profiles.len(), 2);
    }
}

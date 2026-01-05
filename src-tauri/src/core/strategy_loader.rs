//! JSON Strategy Loader for Isolate
//!
//! Provides functionality for loading and parsing JSON strategy files.
//! Converts JSON strategies to winws command-line arguments.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::core::strategy_loader::StrategyLoader;
//!
//! let loader = StrategyLoader::new("configs/strategies");
//! let strategies = loader.load_all()?;
//!
//! for strategy in &strategies {
//!     let args = loader.to_winws_args(&strategy, hostlists_dir, blobs_dir)?;
//!     println!("Strategy {}: {:?}", strategy.id, args);
//! }
//! ```

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// ============================================================================
// Data Structures
// ============================================================================

/// Root structure of a JSON strategy file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyFile {
    /// File format version
    pub version: String,
    /// List of strategies in the file
    pub strategies: Vec<JsonStrategy>,
}

/// JSON strategy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStrategy {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this strategy does
    pub description: String,
    /// Category (youtube, discord, etc.)
    pub category: StrategyCategory,
    /// Strategy family (zapret, vless, etc.)
    pub family: String,
    /// Author of the strategy
    #[serde(default)]
    pub author: Option<String>,
    /// Label for UI (recommended, experimental, etc.)
    #[serde(default)]
    pub label: Option<String>,
    /// Port filters
    pub ports: StrategyPorts,
    /// DPI bypass profiles
    pub profiles: Vec<StrategyProfile>,
}

/// Strategy category enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum StrategyCategory {
    YouTube,
    Discord,
    Telegram,
    General,
    Games,
    Warp,
    Custom,
}

/// Desync attack mode enumeration
///
/// Represents all available DPI desynchronization modes supported by winws/nfqws.
/// Modes can be combined (e.g., "fake,split2" or "fake,fakedsplit").
///
/// Reference: zapret documentation --dpi-desync parameter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DesyncMode {
    // === Basic modes ===
    /// Send fake packets to confuse DPI
    Fake,
    /// Send fake packets only for known protocols (TLS, HTTP, QUIC)
    #[serde(rename = "fakeknown")]
    FakeKnown,

    // === TCP handshake manipulation ===
    /// Split TCP handshake - send SYN+ACK instead of normal handshake
    #[serde(rename = "synack")]
    SynAck,
    /// Send data in SYN packet (TCP Fast Open style)
    #[serde(rename = "syndata")]
    SynData,

    // === RST-based attacks ===
    /// Send RST packet to reset DPI state
    Rst,
    /// Send RST+ACK packet
    #[serde(rename = "rstack")]
    RstAck,

    // === IPv6 extension headers ===
    /// Add Hop-by-Hop Options header (IPv6)
    #[serde(rename = "hopbyhop")]
    HopByHop,
    /// Add Destination Options header (IPv6)
    #[serde(rename = "destopt")]
    DestOpt,

    // === IP fragmentation ===
    /// IP fragmentation mode 1 - fragment before sending
    #[serde(rename = "ipfrag1")]
    IpFrag1,
    /// IP fragmentation mode 2 - fragment after fake
    #[serde(rename = "ipfrag2")]
    IpFrag2,

    // === TCP segmentation ===
    /// Split TCP stream into multiple segments
    #[serde(rename = "multisplit")]
    MultiSplit,
    /// Send TCP segments in wrong order
    #[serde(rename = "multidisorder")]
    MultiDisorder,
    /// Split with fake data between segments
    #[serde(rename = "fakedsplit")]
    FakedSplit,
    /// Disorder with fake data between segments
    #[serde(rename = "fakeddisorder")]
    FakedDisorder,
    /// Split with fake host header
    #[serde(rename = "hostfakesplit")]
    HostFakeSplit,

    // === UDP manipulation ===
    /// Modify UDP packet length
    #[serde(rename = "udplen")]
    UdpLen,

    // === Packet modification ===
    /// Tamper with packet contents
    Tamper,

    // === Legacy split modes (for backward compatibility) ===
    /// Split into 2 segments (legacy)
    Split,
    /// Split into 2 segments (explicit)
    Split2,
    /// Disorder into 2 segments
    Disorder,
    /// Disorder into 2 segments (explicit)
    Disorder2,
}

impl std::fmt::Display for DesyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesyncMode::Fake => write!(f, "fake"),
            DesyncMode::FakeKnown => write!(f, "fakeknown"),
            DesyncMode::SynAck => write!(f, "synack"),
            DesyncMode::SynData => write!(f, "syndata"),
            DesyncMode::Rst => write!(f, "rst"),
            DesyncMode::RstAck => write!(f, "rstack"),
            DesyncMode::HopByHop => write!(f, "hopbyhop"),
            DesyncMode::DestOpt => write!(f, "destopt"),
            DesyncMode::IpFrag1 => write!(f, "ipfrag1"),
            DesyncMode::IpFrag2 => write!(f, "ipfrag2"),
            DesyncMode::MultiSplit => write!(f, "multisplit"),
            DesyncMode::MultiDisorder => write!(f, "multidisorder"),
            DesyncMode::FakedSplit => write!(f, "fakedsplit"),
            DesyncMode::FakedDisorder => write!(f, "fakeddisorder"),
            DesyncMode::HostFakeSplit => write!(f, "hostfakesplit"),
            DesyncMode::UdpLen => write!(f, "udplen"),
            DesyncMode::Tamper => write!(f, "tamper"),
            DesyncMode::Split => write!(f, "split"),
            DesyncMode::Split2 => write!(f, "split2"),
            DesyncMode::Disorder => write!(f, "disorder"),
            DesyncMode::Disorder2 => write!(f, "disorder2"),
        }
    }
}

/// Desync configuration that supports both single mode and combined modes
///
/// winws supports combining modes like "fake,split2" or "fake,fakedsplit".
/// This struct handles both cases transparently.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DesyncConfig {
    /// Single desync mode
    Single(DesyncMode),
    /// Combined desync modes (e.g., "fake,split2")
    Combined(String),
}

impl Default for DesyncConfig {
    fn default() -> Self {
        DesyncConfig::Single(DesyncMode::Fake)
    }
}

impl std::fmt::Display for DesyncConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesyncConfig::Single(mode) => write!(f, "{}", mode),
            DesyncConfig::Combined(modes) => write!(f, "{}", modes),
        }
    }
}

impl From<&str> for DesyncConfig {
    fn from(s: &str) -> Self {
        // If contains comma, treat as combined modes
        if s.contains(',') {
            DesyncConfig::Combined(s.to_string())
        } else {
            // Try to parse as single mode, fallback to combined for unknown modes
            match s.to_lowercase().as_str() {
                "fake" => DesyncConfig::Single(DesyncMode::Fake),
                "fakeknown" => DesyncConfig::Single(DesyncMode::FakeKnown),
                "synack" => DesyncConfig::Single(DesyncMode::SynAck),
                "syndata" => DesyncConfig::Single(DesyncMode::SynData),
                "rst" => DesyncConfig::Single(DesyncMode::Rst),
                "rstack" => DesyncConfig::Single(DesyncMode::RstAck),
                "hopbyhop" => DesyncConfig::Single(DesyncMode::HopByHop),
                "destopt" => DesyncConfig::Single(DesyncMode::DestOpt),
                "ipfrag1" => DesyncConfig::Single(DesyncMode::IpFrag1),
                "ipfrag2" => DesyncConfig::Single(DesyncMode::IpFrag2),
                "multisplit" => DesyncConfig::Single(DesyncMode::MultiSplit),
                "multidisorder" => DesyncConfig::Single(DesyncMode::MultiDisorder),
                "fakedsplit" => DesyncConfig::Single(DesyncMode::FakedSplit),
                "fakeddisorder" => DesyncConfig::Single(DesyncMode::FakedDisorder),
                "hostfakesplit" => DesyncConfig::Single(DesyncMode::HostFakeSplit),
                "udplen" => DesyncConfig::Single(DesyncMode::UdpLen),
                "tamper" => DesyncConfig::Single(DesyncMode::Tamper),
                "split" => DesyncConfig::Single(DesyncMode::Split),
                "split2" => DesyncConfig::Single(DesyncMode::Split2),
                "disorder" => DesyncConfig::Single(DesyncMode::Disorder),
                "disorder2" => DesyncConfig::Single(DesyncMode::Disorder2),
                _ => DesyncConfig::Combined(s.to_string()),
            }
        }
    }
}

impl From<String> for DesyncConfig {
    fn from(s: String) -> Self {
        DesyncConfig::from(s.as_str())
    }
}

impl std::fmt::Display for StrategyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyCategory::YouTube => write!(f, "YouTube"),
            StrategyCategory::Discord => write!(f, "Discord"),
            StrategyCategory::Telegram => write!(f, "Telegram"),
            StrategyCategory::General => write!(f, "General"),
            StrategyCategory::Games => write!(f, "Games"),
            StrategyCategory::Warp => write!(f, "Warp"),
            StrategyCategory::Custom => write!(f, "Custom"),
        }
    }
}

/// Port configuration for strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyPorts {
    /// TCP port filter (e.g., "80,443")
    #[serde(default)]
    pub tcp: Option<String>,
    /// UDP port filter (e.g., "443,50000-50100")
    #[serde(default)]
    pub udp: Option<String>,
}

/// DPI bypass profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyProfile {
    /// WinDivert filter expression
    pub filter: String,
    /// Path to hostlist file (relative)
    #[serde(default)]
    pub hostlist: Option<String>,
    /// Path to hostlist exclude file
    #[serde(default)]
    pub hostlist_exclude: Option<String>,
    /// Inline hostlist domains (comma-separated)
    #[serde(default)]
    pub hostlist_domains: Option<String>,
    /// Path to IP set file
    #[serde(default)]
    pub ipset: Option<String>,
    /// Path to IP set exclude file
    #[serde(default)]
    pub ipset_exclude: Option<String>,
    /// L7 protocol filter
    #[serde(default)]
    pub l7: Option<String>,
    /// IP identification field value
    #[serde(default)]
    pub ip_id: Option<String>,
    /// Desync attack type
    pub desync: String,
    /// Number of fake packet repeats
    #[serde(default)]
    pub repeats: Option<u32>,
    /// Split sequence overlap
    #[serde(default)]
    pub split_seqovl: Option<u32>,
    /// Split position
    #[serde(default)]
    pub split_pos: Option<String>,
    /// Split sequence overlap pattern
    #[serde(default)]
    pub split_seqovl_pattern: Option<String>,
    /// Fooling method
    #[serde(default)]
    pub fooling: Option<String>,
    /// Path to fake TLS ClientHello file
    #[serde(default)]
    pub fake_tls: Option<String>,
    /// Path to fake QUIC Initial file
    #[serde(default)]
    pub fake_quic: Option<String>,
    /// Fake TLS modification mode
    #[serde(default)]
    pub fake_tls_mod: Option<String>,
    /// Path to fake WireGuard file
    #[serde(default)]
    pub fake_wireguard: Option<String>,
    /// Path to fake DHT file
    #[serde(default)]
    pub fake_dht: Option<String>,
    /// Path to fake unknown UDP file
    #[serde(default)]
    pub fake_unknown_udp: Option<String>,
    /// Fake TCP modification mode
    #[serde(default)]
    pub fake_tcp_mod: Option<String>,
    /// Path to fake syndata file
    #[serde(default)]
    pub fake_syndata: Option<String>,
    /// TTL value for desync packets (IPv4)
    #[serde(default)]
    pub ttl: Option<u8>,
    /// TTL value for desync packets (IPv6 hop limit)
    #[serde(default)]
    pub ttl6: Option<u8>,
    /// Auto TTL configuration (format: "min:max:step" or just value)
    #[serde(default)]
    pub autottl: Option<String>,
    /// Bad sequence number increment for fooling
    #[serde(default)]
    pub badseq_increment: Option<i64>,
    /// Bad acknowledgment number increment for fooling
    #[serde(default)]
    pub badack_increment: Option<i64>,
    /// TCP timestamp increment for fooling
    #[serde(default)]
    pub ts_increment: Option<i64>,
    /// Cutoff configuration
    #[serde(default)]
    pub cutoff: Option<String>,
    /// Hostfakesplit modifier (e.g., host=ya.ru,altorder=1)
    #[serde(default)]
    pub hostfakesplit_mod: Option<String>,
    /// Hostfakesplit middle host
    #[serde(default)]
    pub hostfakesplit_midhost: Option<String>,
    /// Fakedsplit modifier
    #[serde(default)]
    pub fakedsplit_mod: Option<String>,
    /// Window size
    #[serde(default)]
    pub wsize: Option<String>,
    /// Server window size
    #[serde(default)]
    pub wssize: Option<String>,
    /// Cutoff for wssize
    #[serde(default)]
    pub wssize_cutoff: Option<String>,
    /// L3 filter (ipv4|ipv6)
    #[serde(default)]
    pub filter_l3: Option<String>,
    /// WiFi SSID filter
    #[serde(default)]
    pub filter_ssid: Option<String>,
    /// NLM filter
    #[serde(default)]
    pub nlm_filter: Option<String>,
    /// Enable packet duplication
    #[serde(default)]
    pub dup: Option<bool>,
    /// Replace original packet with duplicate
    #[serde(default)]
    pub dup_replace: Option<bool>,
    /// TTL for duplicate packets
    #[serde(default)]
    pub dup_ttl: Option<u8>,
    /// Auto TTL for duplicate packets
    #[serde(default)]
    pub dup_autottl: Option<String>,
    /// Fooling method for duplicate packets
    #[serde(default)]
    pub dup_fooling: Option<String>,
    /// Start position for duplicate application
    #[serde(default)]
    pub dup_start: Option<String>,
    /// Cutoff position for duplicate application
    #[serde(default)]
    pub dup_cutoff: Option<String>,

    // Original packet modification parameters
    /// TTL for original packets
    #[serde(default)]
    pub orig_ttl: Option<u8>,
    /// Auto TTL for original packets
    #[serde(default)]
    pub orig_autottl: Option<String>,
    /// TCP flags to set on original packets
    #[serde(default)]
    pub orig_tcp_flags_set: Option<String>,
    /// TCP flags to unset on original packets
    #[serde(default)]
    pub orig_tcp_flags_unset: Option<String>,
    /// Start of modification range for original packets
    #[serde(default)]
    pub orig_mod_start: Option<String>,
    /// End of modification range for original packets
    #[serde(default)]
    pub orig_mod_cutoff: Option<String>,
}

// ============================================================================
// Strategy Loader
// ============================================================================

/// Loader for JSON strategy files
pub struct StrategyLoader {
    /// Directory containing strategy JSON files
    strategies_dir: PathBuf,
}

impl StrategyLoader {
    /// Create a new strategy loader
    ///
    /// # Arguments
    /// * `strategies_dir` - Path to directory containing JSON strategy files
    pub fn new(strategies_dir: impl AsRef<Path>) -> Self {
        Self {
            strategies_dir: strategies_dir.as_ref().to_path_buf(),
        }
    }

    /// Load all JSON strategy files from the directory
    ///
    /// Scans the strategies directory for .json files and loads all strategies.
    ///
    /// # Returns
    /// * `Ok(Vec<JsonStrategy>)` - All loaded strategies
    /// * `Err` - Failed to read directory or parse files
    pub fn load_all(&self) -> Result<Vec<JsonStrategy>> {
        let mut all_strategies = Vec::new();

        if !self.strategies_dir.exists() {
            warn!(
                "Strategies directory does not exist: {}",
                self.strategies_dir.display()
            );
            return Ok(all_strategies);
        }

        let entries = std::fs::read_dir(&self.strategies_dir)
            .with_context(|| format!("Failed to read strategies directory: {}", self.strategies_dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process .json files
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match self.load_file(&path) {
                    Ok(strategy_file) => {
                        info!(
                            "Loaded {} strategies from {}",
                            strategy_file.strategies.len(),
                            path.display()
                        );
                        all_strategies.extend(strategy_file.strategies);
                    }
                    Err(e) => {
                        warn!("Failed to load strategy file {}: {}", path.display(), e);
                    }
                }
            }
        }

        info!("Loaded {} total strategies", all_strategies.len());
        Ok(all_strategies)
    }

    /// Load a single JSON strategy file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file
    ///
    /// # Returns
    /// * `Ok(StrategyFile)` - Parsed strategy file
    /// * `Err` - Failed to read or parse file
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<StrategyFile> {
        let path = path.as_ref();
        debug!("Loading strategy file: {}", path.display());

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read strategy file: {}", path.display()))?;

        let strategy_file: StrategyFile = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse strategy file: {}", path.display()))?;

        debug!(
            "Parsed {} strategies from {}",
            strategy_file.strategies.len(),
            path.display()
        );

        Ok(strategy_file)
    }

    /// Filter strategies by category
    ///
    /// # Arguments
    /// * `strategies` - List of strategies to filter
    /// * `category` - Category to filter by
    ///
    /// # Returns
    /// * `Vec<JsonStrategy>` - Strategies matching the category
    pub fn filter_by_category(
        &self,
        strategies: &[JsonStrategy],
        category: StrategyCategory,
    ) -> Vec<JsonStrategy> {
        strategies
            .iter()
            .filter(|s| s.category == category)
            .cloned()
            .collect()
    }

    /// Generate winws command-line arguments from a strategy
    ///
    /// Converts a JsonStrategy into a vector of command-line arguments
    /// suitable for launching winws.exe.
    ///
    /// # Arguments
    /// * `strategy` - Strategy to convert
    /// * `hostlists_dir` - Directory containing hostlist files
    /// * `blobs_dir` - Directory containing binary blob files (fake TLS, QUIC, etc.)
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Command-line arguments
    /// * `Err` - Failed to generate arguments
    pub fn to_winws_args(
        &self,
        strategy: &JsonStrategy,
        hostlists_dir: &Path,
        blobs_dir: &Path,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Add port filters
        if let Some(ref tcp) = strategy.ports.tcp {
            args.push("--wf-tcp".to_string());
            args.push(tcp.clone());
        }

        if let Some(ref udp) = strategy.ports.udp {
            args.push("--wf-udp".to_string());
            args.push(udp.clone());
        }

        // Process each profile
        for (i, profile) in strategy.profiles.iter().enumerate() {
            // Add --new separator between profiles (not before first)
            if i > 0 {
                args.push("--new".to_string());
            }

            // Add profile arguments
            self.add_profile_args(&mut args, profile, hostlists_dir, blobs_dir)?;
        }

        debug!(
            "Generated {} args for strategy '{}'",
            args.len(),
            strategy.id
        );

        Ok(args)
    }

    /// Add arguments for a single profile
    fn add_profile_args(
        &self,
        args: &mut Vec<String>,
        profile: &StrategyProfile,
        hostlists_dir: &Path,
        blobs_dir: &Path,
    ) -> Result<()> {
        // Filter
        args.push("--filter".to_string());
        args.push(profile.filter.clone());

        // Hostlist
        if let Some(ref hostlist) = profile.hostlist {
            let path = hostlists_dir.join(hostlist);
            args.push("--hostlist".to_string());
            args.push(path.display().to_string());
        }

        // Hostlist exclude
        if let Some(ref hostlist_exclude) = profile.hostlist_exclude {
            let path = hostlists_dir.join(hostlist_exclude);
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
            let path = hostlists_dir.join(ipset);
            args.push("--ipset".to_string());
            args.push(path.display().to_string());
        }

        // IP set exclude
        if let Some(ref ipset_exclude) = profile.ipset_exclude {
            let path = hostlists_dir.join(ipset_exclude);
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
            let path = blobs_dir.join(fake_tls);
            args.push("--dpi-desync-fake-tls".to_string());
            args.push(path.display().to_string());
        }

        // Fake QUIC Initial
        if let Some(ref fake_quic) = profile.fake_quic {
            let path = blobs_dir.join(fake_quic);
            args.push("--dpi-desync-fake-quic".to_string());
            args.push(path.display().to_string());
        }

        // Fake TLS modification
        if let Some(ref fake_tls_mod) = profile.fake_tls_mod {
            args.push("--dpi-desync-fake-tls-mod".to_string());
            args.push(fake_tls_mod.clone());
        }

        // Fake WireGuard
        if let Some(ref fake_wireguard) = profile.fake_wireguard {
            let path = blobs_dir.join(fake_wireguard);
            args.push("--dpi-desync-fake-wireguard".to_string());
            args.push(path.display().to_string());
        }

        // Fake DHT
        if let Some(ref fake_dht) = profile.fake_dht {
            let path = blobs_dir.join(fake_dht);
            args.push("--dpi-desync-fake-dht".to_string());
            args.push(path.display().to_string());
        }

        // Fake unknown UDP
        if let Some(ref fake_unknown_udp) = profile.fake_unknown_udp {
            let path = blobs_dir.join(fake_unknown_udp);
            args.push("--dpi-desync-fake-unknown-udp".to_string());
            args.push(path.display().to_string());
        }

        // Fake TCP modification
        if let Some(ref fake_tcp_mod) = profile.fake_tcp_mod {
            args.push("--dpi-desync-fake-tcp-mod".to_string());
            args.push(fake_tcp_mod.clone());
        }

        // Fake syndata
        if let Some(ref fake_syndata) = profile.fake_syndata {
            let path = blobs_dir.join(fake_syndata);
            args.push("--dpi-desync-fake-syndata".to_string());
            args.push(path.display().to_string());
        }

        // TTL for desync packets (IPv4)
        if let Some(ttl) = profile.ttl {
            args.push("--dpi-desync-ttl".to_string());
            args.push(ttl.to_string());
        }

        // TTL for desync packets (IPv6)
        if let Some(ttl6) = profile.ttl6 {
            args.push("--dpi-desync-ttl6".to_string());
            args.push(ttl6.to_string());
        }

        // Auto TTL
        if let Some(ref autottl) = profile.autottl {
            args.push("--dpi-desync-autottl".to_string());
            args.push(autottl.clone());
        }

        // Bad sequence number increment
        if let Some(badseq_increment) = profile.badseq_increment {
            args.push("--dpi-desync-badseq-increment".to_string());
            args.push(badseq_increment.to_string());
        }

        // Bad acknowledgment number increment
        if let Some(badack_increment) = profile.badack_increment {
            args.push("--dpi-desync-badack-increment".to_string());
            args.push(badack_increment.to_string());
        }

        // TCP timestamp increment
        if let Some(ts_increment) = profile.ts_increment {
            args.push("--dpi-desync-ts-increment".to_string());
            args.push(ts_increment.to_string());
        }

        // Cutoff
        if let Some(ref cutoff) = profile.cutoff {
            args.push("--dpi-desync-cutoff".to_string());
            args.push(cutoff.clone());
        }

        // Hostfakesplit modifier
        if let Some(ref hostfakesplit_mod) = profile.hostfakesplit_mod {
            args.push("--dpi-desync-hostfakesplit-mod".to_string());
            args.push(hostfakesplit_mod.clone());
        }

        // Hostfakesplit middle host
        if let Some(ref hostfakesplit_midhost) = profile.hostfakesplit_midhost {
            args.push("--dpi-desync-hostfakesplit-midhost".to_string());
            args.push(hostfakesplit_midhost.clone());
        }

        // Fakedsplit modifier
        if let Some(ref fakedsplit_mod) = profile.fakedsplit_mod {
            args.push("--dpi-desync-fakedsplit-mod".to_string());
            args.push(fakedsplit_mod.clone());
        }

        // Window size
        if let Some(ref wsize) = profile.wsize {
            args.push("--wsize".to_string());
            args.push(wsize.clone());
        }

        // Server window size
        if let Some(ref wssize) = profile.wssize {
            args.push("--wssize".to_string());
            args.push(wssize.clone());
        }

        // Wssize cutoff
        if let Some(ref wssize_cutoff) = profile.wssize_cutoff {
            args.push("--wssize-cutoff".to_string());
            args.push(wssize_cutoff.clone());
        }

        // L3 filter (ipv4|ipv6)
        if let Some(ref filter_l3) = profile.filter_l3 {
            args.push("--filter-l3".to_string());
            args.push(filter_l3.clone());
        }

        // WiFi SSID filter
        if let Some(ref filter_ssid) = profile.filter_ssid {
            args.push("--filter-ssid".to_string());
            args.push(filter_ssid.clone());
        }

        // NLM filter
        if let Some(ref nlm_filter) = profile.nlm_filter {
            args.push("--nlm-filter".to_string());
            args.push(nlm_filter.clone());
        }

        // Dup (packet duplication)
        if let Some(dup) = profile.dup {
            if dup {
                args.push("--dpi-desync-dup".to_string());
            }
        }

        // Dup replace
        if let Some(dup_replace) = profile.dup_replace {
            if dup_replace {
                args.push("--dpi-desync-dup-replace".to_string());
            }
        }

        // Dup TTL
        if let Some(dup_ttl) = profile.dup_ttl {
            args.push("--dpi-desync-dup-ttl".to_string());
            args.push(dup_ttl.to_string());
        }

        // Dup Auto TTL
        if let Some(ref dup_autottl) = profile.dup_autottl {
            args.push("--dpi-desync-dup-autottl".to_string());
            args.push(dup_autottl.clone());
        }

        // Dup Fooling
        if let Some(ref dup_fooling) = profile.dup_fooling {
            args.push("--dpi-desync-dup-fooling".to_string());
            args.push(dup_fooling.clone());
        }

        // Dup Start
        if let Some(ref dup_start) = profile.dup_start {
            args.push("--dpi-desync-dup-start".to_string());
            args.push(dup_start.clone());
        }

        // Dup Cutoff
        if let Some(ref dup_cutoff) = profile.dup_cutoff {
            args.push("--dpi-desync-dup-cutoff".to_string());
            args.push(dup_cutoff.clone());
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_strategy() -> JsonStrategy {
        JsonStrategy {
            id: "test-strategy".to_string(),
            name: "Test Strategy".to_string(),
            description: "A test strategy".to_string(),
            category: StrategyCategory::YouTube,
            family: "zapret".to_string(),
            author: Some("Test Author".to_string()),
            label: Some("recommended".to_string()),
            ports: StrategyPorts {
                tcp: Some("80,443".to_string()),
                udp: Some("443".to_string()),
            },
            profiles: vec![
                StrategyProfile {
                    filter: "tcp".to_string(),
                    hostlist: Some("youtube.txt".to_string()),
                    hostlist_exclude: None,
                    hostlist_domains: None,
                    ipset: None,
                    ipset_exclude: None,
                    l7: Some("http".to_string()),
                    ip_id: None,
                    desync: "fake,split2".to_string(),
                    repeats: Some(6),
                    split_seqovl: Some(2),
                    split_pos: Some("1".to_string()),
                    split_seqovl_pattern: None,
                    fooling: Some("md5sig".to_string()),
                    fake_tls: Some("tls_clienthello.bin".to_string()),
                    fake_quic: None,
                    fake_tls_mod: None,
                    fake_wireguard: None,
                    fake_dht: None,
                    fake_unknown_udp: None,
                    fake_tcp_mod: None,
                    fake_syndata: None,
                    ttl: None,
                    ttl6: None,
                    autottl: Some("2".to_string()),
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
                },
            ],
        }
    }

    #[test]
    fn test_strategy_category_display() {
        assert_eq!(format!("{}", StrategyCategory::YouTube), "YouTube");
        assert_eq!(format!("{}", StrategyCategory::Discord), "Discord");
        assert_eq!(format!("{}", StrategyCategory::General), "General");
    }

    #[test]
    fn test_to_winws_args_basic() {
        let loader = StrategyLoader::new("test");
        let strategy = create_test_strategy();
        let hostlists_dir = PathBuf::from("C:\\test\\hostlists");
        let blobs_dir = PathBuf::from("C:\\test\\blobs");

        let args = loader.to_winws_args(&strategy, &hostlists_dir, &blobs_dir).unwrap();

        // Check port filters
        assert!(args.contains(&"--wf-tcp".to_string()));
        assert!(args.contains(&"80,443".to_string()));
        assert!(args.contains(&"--wf-udp".to_string()));
        assert!(args.contains(&"443".to_string()));

        // Check desync
        assert!(args.contains(&"--dpi-desync".to_string()));
        assert!(args.contains(&"fake,split2".to_string()));

        // Check hostlist path
        assert!(args.contains(&"--hostlist".to_string()));
        let hostlist_idx = args.iter().position(|a| a == "--hostlist").unwrap();
        assert!(args[hostlist_idx + 1].contains("youtube.txt"));
    }

    #[test]
    fn test_to_winws_args_multiple_profiles() {
        let loader = StrategyLoader::new("test");
        let mut strategy = create_test_strategy();

        // Add second profile
        strategy.profiles.push(StrategyProfile {
            filter: "udp".to_string(),
            hostlist: None,
            hostlist_exclude: None,
            hostlist_domains: None,
            ipset: None,
            ipset_exclude: None,
            l7: Some("quic".to_string()),
            ip_id: None,
            desync: "fake".to_string(),
            repeats: None,
            split_seqovl: None,
            split_pos: None,
            split_seqovl_pattern: None,
            fooling: None,
            fake_tls: None,
            fake_quic: Some("quic_initial.bin".to_string()),
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
        });

        let hostlists_dir = PathBuf::from("C:\\test\\hostlists");
        let blobs_dir = PathBuf::from("C:\\test\\blobs");

        let args = loader.to_winws_args(&strategy, &hostlists_dir, &blobs_dir).unwrap();

        // Check --new separator exists
        assert!(args.contains(&"--new".to_string()));

        // Count --filter occurrences (should be 2)
        let filter_count = args.iter().filter(|a| *a == "--filter").count();
        assert_eq!(filter_count, 2);
    }

    #[test]
    fn test_filter_by_category() {
        let loader = StrategyLoader::new("test");

        let strategies = vec![
            JsonStrategy {
                id: "yt-1".to_string(),
                name: "YouTube 1".to_string(),
                description: "".to_string(),
                category: StrategyCategory::YouTube,
                family: "zapret".to_string(),
                author: None,
                label: None,
                ports: StrategyPorts::default(),
                profiles: vec![],
            },
            JsonStrategy {
                id: "discord-1".to_string(),
                name: "Discord 1".to_string(),
                description: "".to_string(),
                category: StrategyCategory::Discord,
                family: "zapret".to_string(),
                author: None,
                label: None,
                ports: StrategyPorts::default(),
                profiles: vec![],
            },
            JsonStrategy {
                id: "yt-2".to_string(),
                name: "YouTube 2".to_string(),
                description: "".to_string(),
                category: StrategyCategory::YouTube,
                family: "zapret".to_string(),
                author: None,
                label: None,
                ports: StrategyPorts::default(),
                profiles: vec![],
            },
        ];

        let youtube = loader.filter_by_category(&strategies, StrategyCategory::YouTube);
        assert_eq!(youtube.len(), 2);
        assert!(youtube.iter().all(|s| s.category == StrategyCategory::YouTube));

        let discord = loader.filter_by_category(&strategies, StrategyCategory::Discord);
        assert_eq!(discord.len(), 1);
        assert_eq!(discord[0].id, "discord-1");
    }

    #[test]
    fn test_parse_strategy_json() {
        let json = r#"{
            "version": "1.0",
            "strategies": [
                {
                    "id": "test-1",
                    "name": "Test Strategy",
                    "description": "Test description",
                    "category": "youtube",
                    "family": "zapret",
                    "ports": {
                        "tcp": "443"
                    },
                    "profiles": [
                        {
                            "filter": "tcp",
                            "desync": "fake"
                        }
                    ]
                }
            ]
        }"#;

        let strategy_file: StrategyFile = serde_json::from_str(json).unwrap();
        assert_eq!(strategy_file.version, "1.0");
        assert_eq!(strategy_file.strategies.len(), 1);
        assert_eq!(strategy_file.strategies[0].id, "test-1");
        assert_eq!(strategy_file.strategies[0].category, StrategyCategory::YouTube);
    }

    #[test]
    fn test_strategy_ports_default() {
        let ports = StrategyPorts::default();
        assert!(ports.tcp.is_none());
        assert!(ports.udp.is_none());
    }

    #[test]
    fn test_strategy_deserialization() {
        // Test full strategy deserialization with all fields
        let json = r#"{
            "version": "1.0",
            "strategies": [
                {
                    "id": "youtube-basic",
                    "name": "YouTube Basic",
                    "description": "Basic YouTube bypass strategy",
                    "category": "youtube",
                    "family": "zapret",
                    "author": "Test Author",
                    "label": "recommended",
                    "ports": {
                        "tcp": "80,443",
                        "udp": "443"
                    },
                    "profiles": [
                        {
                            "filter": "tcp",
                            "hostlist": "youtube.txt",
                            "l7": "http",
                            "desync": "fake,split2",
                            "repeats": 6,
                            "split_seqovl": 2,
                            "split_pos": "1",
                            "fooling": "md5sig",
                            "fake_tls": "tls_clienthello.bin",
                            "autottl": "2"
                        }
                    ]
                }
            ]
        }"#;

        let strategy_file: StrategyFile = serde_json::from_str(json).unwrap();
        
        assert_eq!(strategy_file.version, "1.0");
        assert_eq!(strategy_file.strategies.len(), 1);
        
        let strategy = &strategy_file.strategies[0];
        assert_eq!(strategy.id, "youtube-basic");
        assert_eq!(strategy.name, "YouTube Basic");
        assert_eq!(strategy.description, "Basic YouTube bypass strategy");
        assert_eq!(strategy.category, StrategyCategory::YouTube);
        assert_eq!(strategy.family, "zapret");
        assert_eq!(strategy.author, Some("Test Author".to_string()));
        assert_eq!(strategy.label, Some("recommended".to_string()));
        
        // Check ports
        assert_eq!(strategy.ports.tcp, Some("80,443".to_string()));
        assert_eq!(strategy.ports.udp, Some("443".to_string()));
        
        // Check profile
        assert_eq!(strategy.profiles.len(), 1);
        let profile = &strategy.profiles[0];
        assert_eq!(profile.filter, "tcp");
        assert_eq!(profile.hostlist, Some("youtube.txt".to_string()));
        assert_eq!(profile.l7, Some("http".to_string()));
        assert_eq!(profile.desync, "fake,split2");
        assert_eq!(profile.repeats, Some(6));
        assert_eq!(profile.split_seqovl, Some(2));
        assert_eq!(profile.split_pos, Some("1".to_string()));
        assert_eq!(profile.fooling, Some("md5sig".to_string()));
        assert_eq!(profile.fake_tls, Some("tls_clienthello.bin".to_string()));
        assert_eq!(profile.autottl, Some("2".to_string()));
    }

    #[test]
    fn test_strategy_deserialization_minimal() {
        // Test minimal strategy with only required fields
        let json = r#"{
            "version": "1.0",
            "strategies": [
                {
                    "id": "minimal",
                    "name": "Minimal Strategy",
                    "description": "",
                    "category": "general",
                    "family": "zapret",
                    "ports": {},
                    "profiles": [
                        {
                            "filter": "tcp",
                            "desync": "fake"
                        }
                    ]
                }
            ]
        }"#;

        let strategy_file: StrategyFile = serde_json::from_str(json).unwrap();
        let strategy = &strategy_file.strategies[0];
        
        assert_eq!(strategy.id, "minimal");
        assert_eq!(strategy.author, None);
        assert_eq!(strategy.label, None);
        assert!(strategy.ports.tcp.is_none());
        assert!(strategy.ports.udp.is_none());
        
        let profile = &strategy.profiles[0];
        assert!(profile.hostlist.is_none());
        assert!(profile.repeats.is_none());
        assert!(profile.fooling.is_none());
    }

    #[test]
    fn test_profile_to_args() {
        let loader = StrategyLoader::new("test");
        let hostlists_dir = PathBuf::from("/hostlists");
        let blobs_dir = PathBuf::from("/blobs");

        // Create a profile with various options
        let profile = StrategyProfile {
            filter: "tcp".to_string(),
            hostlist: Some("youtube.txt".to_string()),
            hostlist_exclude: Some("exclude.txt".to_string()),
            hostlist_domains: Some("example.com,test.com".to_string()),
            ipset: Some("ips.txt".to_string()),
            ipset_exclude: None,
            l7: Some("http".to_string()),
            ip_id: Some("0x1234".to_string()),
            desync: "fake,split2".to_string(),
            repeats: Some(6),
            split_seqovl: Some(2),
            split_pos: Some("1".to_string()),
            split_seqovl_pattern: Some("GET".to_string()),
            fooling: Some("md5sig".to_string()),
            fake_tls: Some("tls.bin".to_string()),
            fake_quic: Some("quic.bin".to_string()),
            fake_tls_mod: Some("sni=test.com".to_string()),
            fake_wireguard: None,
            fake_dht: None,
            fake_unknown_udp: None,
            fake_tcp_mod: None,
            fake_syndata: None,
            ttl: None,
            ttl6: None,
            autottl: Some("2".to_string()),
            badseq_increment: None,
            badack_increment: None,
            ts_increment: None,
            cutoff: Some("d4".to_string()),
            hostfakesplit_mod: Some("host=ya.ru".to_string()),
            hostfakesplit_midhost: Some("mid.host".to_string()),
            fakedsplit_mod: Some("test".to_string()),
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
        };

        let mut args = Vec::new();
        loader.add_profile_args(&mut args, &profile, &hostlists_dir, &blobs_dir).unwrap();

        // Verify all expected arguments are present
        assert!(args.contains(&"--filter".to_string()));
        assert!(args.contains(&"tcp".to_string()));
        
        assert!(args.contains(&"--hostlist".to_string()));
        assert!(args.iter().any(|a| a.contains("youtube.txt")));
        
        assert!(args.contains(&"--hostlist-exclude".to_string()));
        assert!(args.iter().any(|a| a.contains("exclude.txt")));
        
        assert!(args.contains(&"--hostlist-domains".to_string()));
        assert!(args.contains(&"example.com,test.com".to_string()));
        
        assert!(args.contains(&"--ipset".to_string()));
        assert!(args.iter().any(|a| a.contains("ips.txt")));
        
        assert!(args.contains(&"--dpi-desync-l7".to_string()));
        assert!(args.contains(&"http".to_string()));
        
        assert!(args.contains(&"--dpi-desync-ipid".to_string()));
        assert!(args.contains(&"0x1234".to_string()));
        
        assert!(args.contains(&"--dpi-desync".to_string()));
        assert!(args.contains(&"fake,split2".to_string()));
        
        assert!(args.contains(&"--dpi-desync-repeats".to_string()));
        assert!(args.contains(&"6".to_string()));
        
        assert!(args.contains(&"--dpi-desync-split-seqovl".to_string()));
        assert!(args.contains(&"2".to_string()));
        
        assert!(args.contains(&"--dpi-desync-split-pos".to_string()));
        assert!(args.contains(&"1".to_string()));
        
        assert!(args.contains(&"--dpi-desync-split-seqovl-pattern".to_string()));
        assert!(args.contains(&"GET".to_string()));
        
        assert!(args.contains(&"--dpi-desync-fooling".to_string()));
        assert!(args.contains(&"md5sig".to_string()));
        
        assert!(args.contains(&"--dpi-desync-fake-tls".to_string()));
        assert!(args.iter().any(|a| a.contains("tls.bin")));
        
        assert!(args.contains(&"--dpi-desync-fake-quic".to_string()));
        assert!(args.iter().any(|a| a.contains("quic.bin")));
        
        assert!(args.contains(&"--dpi-desync-autottl".to_string()));
        assert!(args.contains(&"--dpi-desync-cutoff".to_string()));
        assert!(args.contains(&"d4".to_string()));
    }

    #[test]
    fn test_category_display() {
        // Test all category display values
        assert_eq!(StrategyCategory::YouTube.to_string(), "YouTube");
        assert_eq!(StrategyCategory::Discord.to_string(), "Discord");
        assert_eq!(StrategyCategory::Telegram.to_string(), "Telegram");
        assert_eq!(StrategyCategory::General.to_string(), "General");
        assert_eq!(StrategyCategory::Games.to_string(), "Games");
        assert_eq!(StrategyCategory::Warp.to_string(), "Warp");
        assert_eq!(StrategyCategory::Custom.to_string(), "Custom");
    }

    #[test]
    fn test_category_serialization() {
        // Test category serialization/deserialization
        let categories = vec![
            (StrategyCategory::YouTube, "\"youtube\""),
            (StrategyCategory::Discord, "\"discord\""),
            (StrategyCategory::Telegram, "\"telegram\""),
            (StrategyCategory::General, "\"general\""),
            (StrategyCategory::Games, "\"games\""),
            (StrategyCategory::Warp, "\"warp\""),
            (StrategyCategory::Custom, "\"custom\""),
        ];

        for (category, expected_json) in categories {
            let json = serde_json::to_string(&category).unwrap();
            assert_eq!(json, expected_json);
            
            let deserialized: StrategyCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, category);
        }
    }

    // ========================================================================
    // DesyncMode Display trait tests
    // ========================================================================

    #[test]
    fn test_desync_mode_display_basic() {
        assert_eq!(DesyncMode::Fake.to_string(), "fake");
        assert_eq!(DesyncMode::FakeKnown.to_string(), "fakeknown");
    }

    #[test]
    fn test_desync_mode_display_tcp_handshake() {
        assert_eq!(DesyncMode::SynAck.to_string(), "synack");
        assert_eq!(DesyncMode::SynData.to_string(), "syndata");
    }

    #[test]
    fn test_desync_mode_display_rst() {
        assert_eq!(DesyncMode::Rst.to_string(), "rst");
        assert_eq!(DesyncMode::RstAck.to_string(), "rstack");
    }

    #[test]
    fn test_desync_mode_display_ipv6() {
        assert_eq!(DesyncMode::HopByHop.to_string(), "hopbyhop");
        assert_eq!(DesyncMode::DestOpt.to_string(), "destopt");
    }

    #[test]
    fn test_desync_mode_display_ip_fragmentation() {
        assert_eq!(DesyncMode::IpFrag1.to_string(), "ipfrag1");
        assert_eq!(DesyncMode::IpFrag2.to_string(), "ipfrag2");
    }

    #[test]
    fn test_desync_mode_display_tcp_segmentation() {
        assert_eq!(DesyncMode::MultiSplit.to_string(), "multisplit");
        assert_eq!(DesyncMode::MultiDisorder.to_string(), "multidisorder");
        assert_eq!(DesyncMode::FakedSplit.to_string(), "fakedsplit");
        assert_eq!(DesyncMode::FakedDisorder.to_string(), "fakeddisorder");
        assert_eq!(DesyncMode::HostFakeSplit.to_string(), "hostfakesplit");
    }

    #[test]
    fn test_desync_mode_display_udp_and_tamper() {
        assert_eq!(DesyncMode::UdpLen.to_string(), "udplen");
        assert_eq!(DesyncMode::Tamper.to_string(), "tamper");
    }

    #[test]
    fn test_desync_mode_display_legacy_split() {
        assert_eq!(DesyncMode::Split.to_string(), "split");
        assert_eq!(DesyncMode::Split2.to_string(), "split2");
        assert_eq!(DesyncMode::Disorder.to_string(), "disorder");
        assert_eq!(DesyncMode::Disorder2.to_string(), "disorder2");
    }

    #[test]
    fn test_desync_mode_display_all_modes() {
        // Comprehensive test for all DesyncMode variants
        let modes = vec![
            (DesyncMode::Fake, "fake"),
            (DesyncMode::FakeKnown, "fakeknown"),
            (DesyncMode::SynAck, "synack"),
            (DesyncMode::SynData, "syndata"),
            (DesyncMode::Rst, "rst"),
            (DesyncMode::RstAck, "rstack"),
            (DesyncMode::HopByHop, "hopbyhop"),
            (DesyncMode::DestOpt, "destopt"),
            (DesyncMode::IpFrag1, "ipfrag1"),
            (DesyncMode::IpFrag2, "ipfrag2"),
            (DesyncMode::MultiSplit, "multisplit"),
            (DesyncMode::MultiDisorder, "multidisorder"),
            (DesyncMode::FakedSplit, "fakedsplit"),
            (DesyncMode::FakedDisorder, "fakeddisorder"),
            (DesyncMode::HostFakeSplit, "hostfakesplit"),
            (DesyncMode::UdpLen, "udplen"),
            (DesyncMode::Tamper, "tamper"),
            (DesyncMode::Split, "split"),
            (DesyncMode::Split2, "split2"),
            (DesyncMode::Disorder, "disorder"),
            (DesyncMode::Disorder2, "disorder2"),
        ];

        for (mode, expected) in modes {
            assert_eq!(mode.to_string(), expected, "DesyncMode::{:?} should display as '{}'", mode, expected);
        }
    }

    // ========================================================================
    // DesyncConfig tests
    // ========================================================================

    #[test]
    fn test_desync_config_default() {
        let config = DesyncConfig::default();
        assert_eq!(config, DesyncConfig::Single(DesyncMode::Fake));
        assert_eq!(config.to_string(), "fake");
    }

    #[test]
    fn test_desync_config_from_str_single_modes() {
        // Test all known single modes
        let test_cases = vec![
            ("fake", DesyncConfig::Single(DesyncMode::Fake)),
            ("fakeknown", DesyncConfig::Single(DesyncMode::FakeKnown)),
            ("synack", DesyncConfig::Single(DesyncMode::SynAck)),
            ("syndata", DesyncConfig::Single(DesyncMode::SynData)),
            ("rst", DesyncConfig::Single(DesyncMode::Rst)),
            ("rstack", DesyncConfig::Single(DesyncMode::RstAck)),
            ("hopbyhop", DesyncConfig::Single(DesyncMode::HopByHop)),
            ("destopt", DesyncConfig::Single(DesyncMode::DestOpt)),
            ("ipfrag1", DesyncConfig::Single(DesyncMode::IpFrag1)),
            ("ipfrag2", DesyncConfig::Single(DesyncMode::IpFrag2)),
            ("multisplit", DesyncConfig::Single(DesyncMode::MultiSplit)),
            ("multidisorder", DesyncConfig::Single(DesyncMode::MultiDisorder)),
            ("fakedsplit", DesyncConfig::Single(DesyncMode::FakedSplit)),
            ("fakeddisorder", DesyncConfig::Single(DesyncMode::FakedDisorder)),
            ("hostfakesplit", DesyncConfig::Single(DesyncMode::HostFakeSplit)),
            ("udplen", DesyncConfig::Single(DesyncMode::UdpLen)),
            ("tamper", DesyncConfig::Single(DesyncMode::Tamper)),
            ("split", DesyncConfig::Single(DesyncMode::Split)),
            ("split2", DesyncConfig::Single(DesyncMode::Split2)),
            ("disorder", DesyncConfig::Single(DesyncMode::Disorder)),
            ("disorder2", DesyncConfig::Single(DesyncMode::Disorder2)),
        ];

        for (input, expected) in test_cases {
            let config = DesyncConfig::from(input);
            assert_eq!(config, expected, "DesyncConfig::from('{}') should be {:?}", input, expected);
        }
    }

    #[test]
    fn test_desync_config_from_str_case_insensitive() {
        // Test case insensitivity
        assert_eq!(DesyncConfig::from("FAKE"), DesyncConfig::Single(DesyncMode::Fake));
        assert_eq!(DesyncConfig::from("Fake"), DesyncConfig::Single(DesyncMode::Fake));
        assert_eq!(DesyncConfig::from("FaKe"), DesyncConfig::Single(DesyncMode::Fake));
        assert_eq!(DesyncConfig::from("SPLIT2"), DesyncConfig::Single(DesyncMode::Split2));
        assert_eq!(DesyncConfig::from("HopByHop"), DesyncConfig::Single(DesyncMode::HopByHop));
    }

    #[test]
    fn test_desync_config_from_str_combined() {
        // Test combined modes (with comma)
        let combined = DesyncConfig::from("fake,split2");
        assert_eq!(combined, DesyncConfig::Combined("fake,split2".to_string()));

        let combined2 = DesyncConfig::from("fake,fakedsplit");
        assert_eq!(combined2, DesyncConfig::Combined("fake,fakedsplit".to_string()));

        let combined3 = DesyncConfig::from("syndata,fake,split2");
        assert_eq!(combined3, DesyncConfig::Combined("syndata,fake,split2".to_string()));
    }

    #[test]
    fn test_desync_config_from_str_unknown() {
        // Unknown modes should become Combined
        let unknown = DesyncConfig::from("unknownmode");
        assert_eq!(unknown, DesyncConfig::Combined("unknownmode".to_string()));

        let unknown2 = DesyncConfig::from("custom_mode");
        assert_eq!(unknown2, DesyncConfig::Combined("custom_mode".to_string()));
    }

    #[test]
    fn test_desync_config_from_string() {
        // Test From<String> implementation
        let config = DesyncConfig::from(String::from("fake"));
        assert_eq!(config, DesyncConfig::Single(DesyncMode::Fake));

        let config2 = DesyncConfig::from(String::from("fake,split2"));
        assert_eq!(config2, DesyncConfig::Combined("fake,split2".to_string()));
    }

    #[test]
    fn test_desync_config_display_single() {
        let config = DesyncConfig::Single(DesyncMode::Fake);
        assert_eq!(config.to_string(), "fake");

        let config2 = DesyncConfig::Single(DesyncMode::Split2);
        assert_eq!(config2.to_string(), "split2");

        let config3 = DesyncConfig::Single(DesyncMode::FakedSplit);
        assert_eq!(config3.to_string(), "fakedsplit");
    }

    #[test]
    fn test_desync_config_display_combined() {
        let config = DesyncConfig::Combined("fake,split2".to_string());
        assert_eq!(config.to_string(), "fake,split2");

        let config2 = DesyncConfig::Combined("syndata,fake,fakedsplit".to_string());
        assert_eq!(config2.to_string(), "syndata,fake,fakedsplit");
    }

    #[test]
    fn test_desync_config_roundtrip() {
        // Test that from -> display produces consistent results
        let inputs = vec![
            "fake",
            "split2",
            "fakedsplit",
            "fake,split2",
            "syndata,fake,split2",
        ];

        for input in inputs {
            let config = DesyncConfig::from(input);
            let output = config.to_string();
            // For single modes, output should match input (lowercase)
            // For combined modes, output should exactly match input
            if input.contains(',') {
                assert_eq!(output, input);
            } else {
                assert_eq!(output, input.to_lowercase());
            }
        }
    }

    // ========================================================================
    // DesyncMode serialization tests
    // ========================================================================

    #[test]
    fn test_desync_mode_serialization() {
        // Test serialization of DesyncMode
        let modes = vec![
            (DesyncMode::Fake, "\"fake\""),
            (DesyncMode::FakeKnown, "\"fakeknown\""),
            (DesyncMode::SynAck, "\"synack\""),
            (DesyncMode::Split2, "\"split2\""),
            (DesyncMode::FakedSplit, "\"fakedsplit\""),
        ];

        for (mode, expected_json) in modes {
            let json = serde_json::to_string(&mode).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_desync_mode_deserialization() {
        // Test deserialization of DesyncMode
        let test_cases = vec![
            ("\"fake\"", DesyncMode::Fake),
            ("\"fakeknown\"", DesyncMode::FakeKnown),
            ("\"synack\"", DesyncMode::SynAck),
            ("\"split2\"", DesyncMode::Split2),
            ("\"fakedsplit\"", DesyncMode::FakedSplit),
            ("\"hopbyhop\"", DesyncMode::HopByHop),
        ];

        for (json, expected) in test_cases {
            let mode: DesyncMode = serde_json::from_str(json).unwrap();
            assert_eq!(mode, expected);
        }
    }

    // ========================================================================
    // DesyncConfig serialization tests
    // ========================================================================

    #[test]
    fn test_desync_config_serialization_single() {
        let config = DesyncConfig::Single(DesyncMode::Fake);
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "\"fake\"");
    }

    #[test]
    fn test_desync_config_serialization_combined() {
        let config = DesyncConfig::Combined("fake,split2".to_string());
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "\"fake,split2\"");
    }

    #[test]
    fn test_desync_config_deserialization() {
        // Single mode
        let config: DesyncConfig = serde_json::from_str("\"fake\"").unwrap();
        assert!(matches!(config, DesyncConfig::Single(DesyncMode::Fake)));

        // Combined mode (with comma)
        let config2: DesyncConfig = serde_json::from_str("\"fake,split2\"").unwrap();
        assert!(matches!(config2, DesyncConfig::Combined(_)));
        if let DesyncConfig::Combined(s) = config2 {
            assert_eq!(s, "fake,split2");
        }
    }

    // ========================================================================
    // JsonStrategy serialization tests
    // ========================================================================

    #[test]
    fn test_json_strategy_serialization_roundtrip() {
        let strategy = create_test_strategy();
        
        // Serialize
        let json = serde_json::to_string(&strategy).unwrap();
        
        // Deserialize
        let deserialized: JsonStrategy = serde_json::from_str(&json).unwrap();
        
        // Verify fields
        assert_eq!(deserialized.id, strategy.id);
        assert_eq!(deserialized.name, strategy.name);
        assert_eq!(deserialized.description, strategy.description);
        assert_eq!(deserialized.category, strategy.category);
        assert_eq!(deserialized.family, strategy.family);
        assert_eq!(deserialized.author, strategy.author);
        assert_eq!(deserialized.label, strategy.label);
        assert_eq!(deserialized.ports.tcp, strategy.ports.tcp);
        assert_eq!(deserialized.ports.udp, strategy.ports.udp);
        assert_eq!(deserialized.profiles.len(), strategy.profiles.len());
    }

    #[test]
    fn test_strategy_file_serialization_roundtrip() {
        let strategy_file = StrategyFile {
            version: "1.0".to_string(),
            strategies: vec![create_test_strategy()],
        };
        
        // Serialize
        let json = serde_json::to_string_pretty(&strategy_file).unwrap();
        
        // Deserialize
        let deserialized: StrategyFile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.version, strategy_file.version);
        assert_eq!(deserialized.strategies.len(), strategy_file.strategies.len());
        assert_eq!(deserialized.strategies[0].id, strategy_file.strategies[0].id);
    }
}

//! Multi-Strategy Support for winws
//!
//! Provides support for running multiple strategies in a single winws process
//! using the `--new` flag separator.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::core::multi_strategy::{MultiStrategy, MultiStrategyBuilder};
//!
//! // Using builder pattern
//! let multi = MultiStrategyBuilder::new()
//!     .add("youtube", vec!["--wf-tcp=80,443", "--dpi-desync=fake"])
//!     .add("discord", vec!["--wf-udp=443", "--dpi-desync=fake"])
//!     .build();
//!
//! let args = multi.build_args();
//! // Result: ["--wf-tcp=80,443", "--dpi-desync=fake", "--new", "--wf-udp=443", "--dpi-desync=fake"]
//! ```

#![allow(dead_code)] // Public API for multi-strategy support

use serde::{Deserialize, Serialize};

// ============================================================================
// Data Structures
// ============================================================================

/// A single strategy segment within a multi-strategy configuration.
///
/// Each segment represents one strategy with its own arguments that will be
/// separated by `--new` flag when combined with other segments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategySegment {
    /// Unique identifier for this segment (usually strategy ID)
    pub id: String,
    /// Command-line arguments for this segment
    pub args: Vec<String>,
}

impl StrategySegment {
    /// Create a new strategy segment
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this segment
    /// * `args` - Command-line arguments for winws
    pub fn new(id: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            id: id.into(),
            args,
        }
    }

    /// Create a segment from string arguments
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this segment
    /// * `args` - Command-line arguments as string slices
    pub fn from_str_args(id: impl Into<String>, args: &[&str]) -> Self {
        Self {
            id: id.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Check if this segment has any arguments
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Get the number of arguments in this segment
    pub fn arg_count(&self) -> usize {
        self.args.len()
    }
}

// ============================================================================
// Multi-Strategy Configuration
// ============================================================================

/// Combined multi-strategy configuration for winws.
///
/// Manages multiple strategy segments that will be combined into a single
/// winws command using `--new` flag separators.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultiStrategy {
    /// List of strategy segments
    segments: Vec<StrategySegment>,
}

impl MultiStrategy {
    /// Create a new empty multi-strategy configuration
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Create a multi-strategy from existing segments
    pub fn from_segments(segments: Vec<StrategySegment>) -> Self {
        Self { segments }
    }

    /// Add a new strategy segment
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the segment
    /// * `args` - Command-line arguments for this segment
    pub fn add_segment(&mut self, id: impl Into<String>, args: Vec<String>) {
        self.segments.push(StrategySegment::new(id, args));
    }

    /// Add a segment from string arguments
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the segment
    /// * `args` - Command-line arguments as string slices
    pub fn add_segment_from_str(&mut self, id: impl Into<String>, args: &[&str]) {
        self.segments.push(StrategySegment::from_str_args(id, args));
    }

    /// Build the combined command-line arguments with `--new` separators
    ///
    /// # Returns
    /// Combined arguments vector with `--new` between each segment
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut multi = MultiStrategy::new();
    /// multi.add_segment("s1", vec!["--wf-tcp=80".to_string()]);
    /// multi.add_segment("s2", vec!["--wf-udp=443".to_string()]);
    /// let args = multi.build_args();
    /// // ["--wf-tcp=80", "--new", "--wf-udp=443"]
    /// ```
    pub fn build_args(&self) -> Vec<String> {
        let mut result = Vec::new();
        let mut is_first = true;

        for segment in &self.segments {
            if segment.is_empty() {
                continue;
            }

            // Add --new separator between segments (not before first)
            if !is_first {
                result.push("--new".to_string());
            }
            is_first = false;

            // Add segment arguments
            result.extend(segment.args.clone());
        }

        result
    }

    /// Check if this is a multi-strategy (more than one segment)
    pub fn is_multi(&self) -> bool {
        self.segments.len() > 1
    }

    /// Get the number of segments
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Get all strategy IDs in this multi-strategy
    pub fn strategy_ids(&self) -> Vec<&str> {
        self.segments.iter().map(|s| s.id.as_str()).collect()
    }

    /// Get a reference to all segments
    pub fn segments(&self) -> &[StrategySegment] {
        &self.segments
    }

    /// Get a mutable reference to all segments
    pub fn segments_mut(&mut self) -> &mut Vec<StrategySegment> {
        &mut self.segments
    }

    /// Check if the multi-strategy is empty (no segments)
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Remove a segment by ID
    ///
    /// # Returns
    /// `true` if a segment was removed, `false` if not found
    pub fn remove_segment(&mut self, id: &str) -> bool {
        let initial_len = self.segments.len();
        self.segments.retain(|s| s.id != id);
        self.segments.len() < initial_len
    }

    /// Get a segment by ID
    pub fn get_segment(&self, id: &str) -> Option<&StrategySegment> {
        self.segments.iter().find(|s| s.id == id)
    }

    /// Check if a segment with the given ID exists
    pub fn has_segment(&self, id: &str) -> bool {
        self.segments.iter().any(|s| s.id == id)
    }

    /// Clear all segments
    pub fn clear(&mut self) {
        self.segments.clear();
    }
}

// ============================================================================
// Builder Pattern
// ============================================================================

/// Builder for creating MultiStrategy configurations fluently.
///
/// # Example
/// ```rust,ignore
/// let multi = MultiStrategyBuilder::new()
///     .add("youtube", vec!["--wf-tcp=80,443", "--dpi-desync=fake"])
///     .add("discord", vec!["--wf-udp=443", "--dpi-desync=fake"])
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct MultiStrategyBuilder {
    segments: Vec<StrategySegment>,
}

impl MultiStrategyBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Add a strategy segment with owned String arguments
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the segment
    /// * `args` - Command-line arguments as Vec<String>
    pub fn add(mut self, id: impl Into<String>, args: Vec<String>) -> Self {
        self.segments.push(StrategySegment::new(id, args));
        self
    }

    /// Add a strategy segment with string slice arguments
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the segment
    /// * `args` - Command-line arguments as string slices
    pub fn add_str(mut self, id: impl Into<String>, args: &[&str]) -> Self {
        self.segments.push(StrategySegment::from_str_args(id, args));
        self
    }

    /// Add a pre-built segment
    pub fn add_segment(mut self, segment: StrategySegment) -> Self {
        self.segments.push(segment);
        self
    }

    /// Build the final MultiStrategy
    pub fn build(self) -> MultiStrategy {
        MultiStrategy::from_segments(self.segments)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_segment_new() {
        let segment = StrategySegment::new("test", vec!["--arg1".to_string(), "--arg2".to_string()]);
        
        assert_eq!(segment.id, "test");
        assert_eq!(segment.args.len(), 2);
        assert_eq!(segment.args[0], "--arg1");
        assert_eq!(segment.args[1], "--arg2");
        assert!(!segment.is_empty());
        assert_eq!(segment.arg_count(), 2);
    }

    #[test]
    fn test_strategy_segment_from_str_args() {
        let segment = StrategySegment::from_str_args("test", &["--wf-tcp=80", "--dpi-desync=fake"]);
        
        assert_eq!(segment.id, "test");
        assert_eq!(segment.args.len(), 2);
        assert_eq!(segment.args[0], "--wf-tcp=80");
        assert_eq!(segment.args[1], "--dpi-desync=fake");
    }

    #[test]
    fn test_strategy_segment_empty() {
        let segment = StrategySegment::new("empty", vec![]);
        
        assert!(segment.is_empty());
        assert_eq!(segment.arg_count(), 0);
    }

    #[test]
    fn test_multi_strategy_single_segment() {
        let mut multi = MultiStrategy::new();
        multi.add_segment("youtube", vec![
            "--wf-tcp=80,443".to_string(),
            "--dpi-desync=fake".to_string(),
        ]);

        let args = multi.build_args();
        
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "--wf-tcp=80,443");
        assert_eq!(args[1], "--dpi-desync=fake");
        assert!(!multi.is_multi());
        assert_eq!(multi.segment_count(), 1);
    }

    #[test]
    fn test_multi_strategy_multiple_segments_with_new_separator() {
        let mut multi = MultiStrategy::new();
        multi.add_segment("youtube", vec![
            "--wf-tcp=80,443".to_string(),
            "--dpi-desync=fake".to_string(),
        ]);
        multi.add_segment("discord", vec![
            "--wf-udp=443".to_string(),
            "--dpi-desync=fake".to_string(),
        ]);

        let args = multi.build_args();
        
        // Should be: [tcp args] --new [udp args]
        assert_eq!(args.len(), 5);
        assert_eq!(args[0], "--wf-tcp=80,443");
        assert_eq!(args[1], "--dpi-desync=fake");
        assert_eq!(args[2], "--new");
        assert_eq!(args[3], "--wf-udp=443");
        assert_eq!(args[4], "--dpi-desync=fake");
        
        assert!(multi.is_multi());
        assert_eq!(multi.segment_count(), 2);
    }

    #[test]
    fn test_multi_strategy_three_segments() {
        let multi = MultiStrategyBuilder::new()
            .add_str("s1", &["--arg1"])
            .add_str("s2", &["--arg2"])
            .add_str("s3", &["--arg3"])
            .build();

        let args = multi.build_args();
        
        // Should be: --arg1 --new --arg2 --new --arg3
        assert_eq!(args.len(), 5);
        assert_eq!(args[0], "--arg1");
        assert_eq!(args[1], "--new");
        assert_eq!(args[2], "--arg2");
        assert_eq!(args[3], "--new");
        assert_eq!(args[4], "--arg3");
        
        assert!(multi.is_multi());
        assert_eq!(multi.segment_count(), 3);
    }

    #[test]
    fn test_multi_strategy_strategy_ids() {
        let multi = MultiStrategyBuilder::new()
            .add_str("youtube", &["--arg1"])
            .add_str("discord", &["--arg2"])
            .add_str("telegram", &["--arg3"])
            .build();

        let ids = multi.strategy_ids();
        
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], "youtube");
        assert_eq!(ids[1], "discord");
        assert_eq!(ids[2], "telegram");
    }

    #[test]
    fn test_multi_strategy_empty() {
        let multi = MultiStrategy::new();
        
        assert!(multi.is_empty());
        assert!(!multi.is_multi());
        assert_eq!(multi.segment_count(), 0);
        assert!(multi.build_args().is_empty());
        assert!(multi.strategy_ids().is_empty());
    }

    #[test]
    fn test_multi_strategy_skip_empty_segments() {
        let mut multi = MultiStrategy::new();
        multi.add_segment("s1", vec!["--arg1".to_string()]);
        multi.add_segment("empty", vec![]); // Empty segment
        multi.add_segment("s2", vec!["--arg2".to_string()]);

        let args = multi.build_args();
        
        // Empty segment should be skipped, no extra --new
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "--arg1");
        assert_eq!(args[1], "--new");
        assert_eq!(args[2], "--arg2");
    }

    #[test]
    fn test_multi_strategy_remove_segment() {
        let mut multi = MultiStrategyBuilder::new()
            .add_str("s1", &["--arg1"])
            .add_str("s2", &["--arg2"])
            .add_str("s3", &["--arg3"])
            .build();

        assert_eq!(multi.segment_count(), 3);
        
        let removed = multi.remove_segment("s2");
        assert!(removed);
        assert_eq!(multi.segment_count(), 2);
        
        let ids = multi.strategy_ids();
        assert_eq!(ids, vec!["s1", "s3"]);
        
        // Try to remove non-existent
        let not_removed = multi.remove_segment("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_multi_strategy_get_segment() {
        let multi = MultiStrategyBuilder::new()
            .add_str("youtube", &["--wf-tcp=443"])
            .add_str("discord", &["--wf-udp=443"])
            .build();

        let youtube = multi.get_segment("youtube");
        assert!(youtube.is_some());
        assert_eq!(youtube.unwrap().args[0], "--wf-tcp=443");

        let nonexistent = multi.get_segment("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_multi_strategy_has_segment() {
        let multi = MultiStrategyBuilder::new()
            .add_str("youtube", &["--arg"])
            .build();

        assert!(multi.has_segment("youtube"));
        assert!(!multi.has_segment("discord"));
    }

    #[test]
    fn test_multi_strategy_clear() {
        let mut multi = MultiStrategyBuilder::new()
            .add_str("s1", &["--arg1"])
            .add_str("s2", &["--arg2"])
            .build();

        assert_eq!(multi.segment_count(), 2);
        
        multi.clear();
        
        assert!(multi.is_empty());
        assert_eq!(multi.segment_count(), 0);
    }

    #[test]
    fn test_builder_pattern() {
        let multi = MultiStrategyBuilder::new()
            .add("s1".to_string(), vec!["--arg1".to_string()])
            .add_str("s2", &["--arg2", "--arg3"])
            .add_segment(StrategySegment::new("s3", vec!["--arg4".to_string()]))
            .build();

        assert_eq!(multi.segment_count(), 3);
        
        let args = multi.build_args();
        assert_eq!(args.len(), 6); // 1 + --new + 2 + --new + 1
    }

    #[test]
    fn test_real_world_winws_example() {
        // Simulate real winws multi-strategy command
        let multi = MultiStrategyBuilder::new()
            .add_str("youtube_tcp", &[
                "--wf-tcp=80,443",
                "--dpi-desync=fake,split2",
                "--dpi-desync-split-pos=1",
                "--dpi-desync-fooling=badseq",
            ])
            .add_str("youtube_udp", &[
                "--wf-udp=443",
                "--dpi-desync=fake",
                "--dpi-desync-fake-quic=quic_initial.bin",
            ])
            .build();

        let args = multi.build_args();
        
        // Verify structure
        assert!(multi.is_multi());
        assert_eq!(multi.segment_count(), 2);
        
        // Find --new position
        let new_pos = args.iter().position(|a| a == "--new");
        assert!(new_pos.is_some());
        
        // TCP args before --new
        let new_idx = new_pos.unwrap();
        assert!(args[..new_idx].contains(&"--wf-tcp=80,443".to_string()));
        
        // UDP args after --new
        assert!(args[new_idx + 1..].contains(&"--wf-udp=443".to_string()));
    }

    #[test]
    fn test_from_segments() {
        let segments = vec![
            StrategySegment::new("s1", vec!["--arg1".to_string()]),
            StrategySegment::new("s2", vec!["--arg2".to_string()]),
        ];

        let multi = MultiStrategy::from_segments(segments);
        
        assert_eq!(multi.segment_count(), 2);
        assert!(multi.is_multi());
    }

    #[test]
    fn test_segments_mut() {
        let mut multi = MultiStrategyBuilder::new()
            .add_str("s1", &["--arg1"])
            .build();

        // Modify segments directly
        multi.segments_mut().push(StrategySegment::new("s2", vec!["--arg2".to_string()]));
        
        assert_eq!(multi.segment_count(), 2);
    }

    #[test]
    fn test_serialization() {
        let multi = MultiStrategyBuilder::new()
            .add_str("youtube", &["--wf-tcp=443"])
            .add_str("discord", &["--wf-udp=443"])
            .build();

        // Serialize to JSON
        let json = serde_json::to_string(&multi).unwrap();
        assert!(json.contains("youtube"));
        assert!(json.contains("discord"));

        // Deserialize back
        let deserialized: MultiStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.segment_count(), 2);
        assert_eq!(deserialized.strategy_ids(), vec!["youtube", "discord"]);
    }
}

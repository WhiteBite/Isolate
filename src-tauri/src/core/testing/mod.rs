//! Testing module for strategy validation
//!
//! This module provides HTTP probing, scoring, and endpoint management
//! for testing DPI bypass strategies.
//!
//! # Components
//!
//! - [`endpoints`] - Test endpoint registry and management
//! - [`prober`] - HTTP probing with direct and SOCKS5 support
//! - [`scorer`] - Strategy score calculation based on probe results

pub mod endpoints;
pub mod prober;
pub mod scorer;

// Re-export main types
pub use endpoints::{EndpointRegistry, TestEndpoint};
pub use prober::{HttpProber, ProbeConfig};
#[allow(unused_imports)]
pub use prober::ProbeResult;
pub use scorer::{ScoreCalculator, StrategyScore};
#[allow(unused_imports)]
pub use scorer::ScoreWeights;

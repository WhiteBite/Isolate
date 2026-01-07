//! Data models for Isolate
//!
//! This module contains all data structures used throughout the application.
//! Models are organized by domain:
//! - `strategy` - Strategy definitions and scoring
//! - `service` - Service definitions and test results
//! - `config` - Application settings and configuration
//! - `proxy` - Proxy configurations and routing
//! - `diagnostic` - DPI diagnostics and error types
//! - `subscription` - Proxy subscription management

pub mod config;
pub mod diagnostic;
pub mod proxy;
pub mod service;
pub mod strategy;
pub mod subscription;

// Re-export all types for backward compatibility
pub use config::*;
pub use diagnostic::*;
pub use proxy::*;
pub use service::*;
pub use strategy::*;
pub use subscription::*;

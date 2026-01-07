//! Services module for Isolate
//!
//! Provides service registry and availability checking functionality.
//! Services are loaded from plugin.json files in the plugins directory.

pub mod registry;

// Re-export checker from core module (unified implementation)
pub use crate::core::checker::{
    CheckResult, CheckerError, EndpointChecker, ServiceChecker, ServiceStatus,
    Endpoint, HttpMethod,
};
pub use registry::{Service, ServiceEndpoint, ServiceRegistry};

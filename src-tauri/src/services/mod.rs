//! Services module for Isolate
//!
//! Provides service registry and availability checking functionality.
//! Services are loaded from plugin.json files in the plugins directory.

pub mod checker;
pub mod registry;

pub use checker::{CheckResult, ServiceChecker, ServiceStatus};
pub use registry::{Service, ServiceEndpoint, ServiceRegistry};

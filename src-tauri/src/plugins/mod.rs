//! Plugin System for Isolate

pub mod checker;
pub mod js_loader;
pub mod manifest;

pub use js_loader::{get_all_plugins, get_all_services, load_manifest, scan_plugins, PluginLoaderError};
pub use manifest::{LoadedPluginInfo, PluginManifest, ServiceDefinition, ServiceEndpoint};

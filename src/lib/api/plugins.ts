/**
 * Plugin API Functions
 * 
 * Functions for interacting with the plugin system, including:
 * - Plugin hostlists
 * - Plugin services
 * - Plugin strategies
 */

import { invoke } from '@tauri-apps/api/core';
import type { 
  RegisteredHostlist, 
  HostlistRegistryStats,
  PluginManifest,
  ServiceDefinition 
} from '$lib/types/plugin';

// ============================================================================
// Plugin Hostlist API
// ============================================================================

/**
 * Get all registered hostlists from plugins
 */
export async function getPluginHostlists(): Promise<RegisteredHostlist[]> {
  return invoke<RegisteredHostlist[]>('get_plugin_hostlists');
}

/**
 * Get a specific hostlist by ID
 */
export async function getPluginHostlist(id: string): Promise<RegisteredHostlist> {
  return invoke<RegisteredHostlist>('get_plugin_hostlist', { id });
}

/**
 * Get domains from a specific hostlist
 */
export async function getPluginHostlistDomains(id: string): Promise<string[]> {
  return invoke<string[]>('get_plugin_hostlist_domains', { id });
}

/**
 * Merge multiple hostlists into a single domain list
 */
export async function mergePluginHostlists(ids: string[]): Promise<string[]> {
  return invoke<string[]>('merge_plugin_hostlists', { ids });
}

/**
 * Get all domains from all enabled hostlists
 */
export async function getAllPluginDomains(): Promise<string[]> {
  return invoke<string[]>('get_all_plugin_domains');
}

/**
 * Check if a domain matches any registered hostlist
 */
export async function checkDomainInHostlists(domain: string): Promise<boolean> {
  return invoke<boolean>('check_domain_in_hostlists', { domain });
}

/**
 * Find which hostlists match a specific domain
 */
export async function findMatchingHostlists(domain: string): Promise<string[]> {
  return invoke<string[]>('find_matching_hostlists', { domain });
}

/**
 * Get hostlists by category
 */
export async function getHostlistsByCategory(category: string): Promise<RegisteredHostlist[]> {
  return invoke<RegisteredHostlist[]>('get_hostlists_by_category', { category });
}

/**
 * Get hostlists by plugin ID
 */
export async function getHostlistsByPlugin(pluginId: string): Promise<RegisteredHostlist[]> {
  return invoke<RegisteredHostlist[]>('get_hostlists_by_plugin', { pluginId });
}

/**
 * Enable or disable a hostlist
 */
export async function setHostlistEnabled(id: string, enabled: boolean): Promise<void> {
  return invoke<void>('set_hostlist_enabled', { id, enabled });
}

/**
 * Reload a hostlist (re-fetch domains from file/URL)
 */
export async function reloadPluginHostlist(id: string): Promise<void> {
  return invoke<void>('reload_plugin_hostlist', { id });
}

/**
 * Get hostlist registry statistics
 */
export async function getHostlistRegistryStats(): Promise<HostlistRegistryStats> {
  return invoke<HostlistRegistryStats>('get_hostlist_registry_stats');
}

// ============================================================================
// Plugin Services API
// ============================================================================

/**
 * Get all services from service-checker plugins
 */
export async function getPluginServices(): Promise<ServiceDefinition[]> {
  return invoke<ServiceDefinition[]>('get_plugin_services');
}

// ============================================================================
// Loaded Plugin Info (matches Rust LoadedPluginInfo)
// ============================================================================

export interface LoadedPluginInfo {
  manifest: PluginManifest;
  enabled: boolean;
  path: string;
  error?: string;
}

/**
 * Get all loaded plugins
 */
export async function getAllPlugins(): Promise<LoadedPluginInfo[]> {
  return invoke<LoadedPluginInfo[]>('get_all_plugins_cmd');
}

/**
 * Reload all plugins (hot reload)
 */
export interface ReloadPluginsResult {
  plugins_loaded: number;
  hostlists_loaded: number;
  strategies_loaded: number;
  services_loaded: number;
}

export async function reloadPlugins(): Promise<ReloadPluginsResult> {
  return invoke<ReloadPluginsResult>('reload_plugins');
}

/**
 * Reload a single plugin by ID
 */
export async function reloadPlugin(pluginId: string): Promise<boolean> {
  return invoke<boolean>('reload_plugin', { pluginId });
}

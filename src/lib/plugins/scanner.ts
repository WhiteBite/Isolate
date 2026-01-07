/**
 * Plugin Scanner
 * 
 * Scans plugin directories and loads plugin manifests.
 * Works with both local plugins and installed plugins.
 */

import type { PluginManifest } from '$lib/types/plugin';
import { registerUIPlugin, unregisterUIPlugin } from './loader';
import { registerBuiltinPlugins } from './builtin';

// ============================================================================
// Plugin Discovery
// ============================================================================

/**
 * Get all plugin manifests using Vite's glob import
 */
const pluginManifests = import.meta.glob('/plugins/*/plugin.json', { eager: true });

/**
 * Parse and validate a plugin manifest
 */
function parseManifest(raw: unknown, path: string): PluginManifest | null {
  try {
    const manifest = raw as PluginManifest;
    
    // Basic validation
    if (!manifest.id || !manifest.name || !manifest.version || !manifest.type) {
      console.warn(`[Scanner] Invalid manifest at ${path}: missing required fields`);
      return null;
    }
    
    // Validate type
    const validTypes = ['service-checker', 'ui-plugin', 'script-plugin', 'hostlist-provider', 'strategy-config'];
    if (!validTypes.includes(manifest.type)) {
      console.warn(`[Scanner] Invalid plugin type "${manifest.type}" at ${path}`);
      return null;
    }
    
    return manifest;
  } catch (error) {
    console.error(`[Scanner] Failed to parse manifest at ${path}:`, error);
    return null;
  }
}

/**
 * Discover all plugins in the plugins directory
 */
export function discoverPlugins(): PluginManifest[] {
  const manifests: PluginManifest[] = [];
  
  for (const [path, module] of Object.entries(pluginManifests)) {
    const manifest = parseManifest((module as { default?: unknown }).default ?? module, path);
    if (manifest) {
      manifests.push(manifest);
    }
  }
  
  console.log(`[Scanner] Discovered ${manifests.length} plugins`);
  return manifests;
}

/**
 * Get plugin path from manifest path
 */
function getPluginPath(manifestPath: string): string {
  // /plugins/speed-widget/plugin.json -> plugins/speed-widget
  return manifestPath.replace('/plugin.json', '').replace(/^\//, '');
}

// ============================================================================
// Plugin Loading
// ============================================================================

/**
 * Load all discovered UI plugins
 */
export async function loadDiscoveredPlugins(): Promise<void> {
  console.log('[Scanner] Loading discovered plugins...');
  
  const manifests = discoverPlugins();
  const uiPlugins = manifests.filter(m => m.type === 'ui-plugin');
  
  for (const manifest of uiPlugins) {
    try {
      // Find the path for this manifest
      const manifestPath = Object.keys(pluginManifests).find(path => {
        const mod = pluginManifests[path] as { default?: PluginManifest } | PluginManifest;
        const m = (mod as { default?: PluginManifest }).default ?? mod;
        return (m as PluginManifest).id === manifest.id;
      });
      
      if (manifestPath) {
        const pluginPath = getPluginPath(manifestPath);
        await registerUIPlugin(manifest, pluginPath);
      }
    } catch (error) {
      console.error(`[Scanner] Failed to load plugin ${manifest.id}:`, error);
    }
  }
}

/**
 * Reload a specific plugin
 */
export async function reloadPlugin(pluginId: string): Promise<void> {
  console.log(`[Scanner] Reloading plugin: ${pluginId}`);
  
  // Unregister first
  unregisterUIPlugin(pluginId);
  
  // Find and reload
  const manifests = discoverPlugins();
  const manifest = manifests.find(m => m.id === pluginId);
  
  if (manifest && manifest.type === 'ui-plugin') {
    const manifestPath = Object.keys(pluginManifests).find(path => {
      const mod = pluginManifests[path] as { default?: PluginManifest } | PluginManifest;
      const m = (mod as { default?: PluginManifest }).default ?? mod;
      return (m as PluginManifest).id === pluginId;
    });
    
    if (manifestPath) {
      const pluginPath = getPluginPath(manifestPath);
      await registerUIPlugin(manifest, pluginPath);
    }
  }
}

// ============================================================================
// Initialization
// ============================================================================

/**
 * Initialize the plugin system
 * - Register builtin plugins
 * - Discover and load external plugins
 */
export async function initializePlugins(): Promise<void> {
  console.log('[Scanner] Initializing plugin system...');
  
  // Register builtin plugins first
  await registerBuiltinPlugins();
  
  // Then load discovered plugins
  await loadDiscoveredPlugins();
  
  console.log('[Scanner] Plugin system initialized');
}

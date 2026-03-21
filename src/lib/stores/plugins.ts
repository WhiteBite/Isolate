import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { PluginSlotLocation } from '$lib/types/plugin';
import { logger } from '$lib/utils/logger';

// Re-export PluginSlotLocation for components that import from this module
export type { PluginSlotLocation } from '$lib/types/plugin';

export type PluginType = 'service-checker' | 'strategy-provider' | 'hostlist-provider' | 'ui-widget' | 'script-plugin';

export type PluginLevel = 1 | 2 | 3;

export interface PluginUI {
  locations: PluginSlotLocation[];
  component?: string;
}

export type PluginSettingValue = string | number | boolean;

export interface PluginSetting {
  id: string;
  label: string;
  type: 'toggle' | 'text' | 'select' | 'number';
  value: PluginSettingValue;
  options?: { value: string; label: string }[];
  description?: string;
  min?: number;
  max?: number;
  placeholder?: string;
}

export interface PluginContributes {
  services?: number;
  hostlists?: number;
  strategies?: number;
  widgets?: number;
  scripts?: number;
}

export interface PluginPermissions {
  http?: string[];
  storage?: boolean;
  events?: string[];
  timeout?: number;
  memory?: number;
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  icon: string;
  type: PluginType;
  enabled: boolean;
  installed?: boolean;
  // Level info (1 = declarative, 2 = UI, 3 = scripts)
  level: PluginLevel;
  // Contributions
  contributes?: PluginContributes;
  // Permissions
  permissions?: PluginPermissions;
  // UI
  route?: string;
  ui?: PluginUI;
  settings?: PluginSetting[];
  // Marketplace info
  downloads?: number;
  rating?: number;
  featured?: boolean;
  category?: 'strategies' | 'services' | 'tools';
  // Source info (for Level 3)
  sourceUrl?: string;
  changelog?: string[];
}

// Helper function to determine plugin level from type
export function getPluginLevel(type: PluginType): PluginLevel {
  switch (type) {
    case 'service-checker':
    case 'hostlist-provider':
    case 'strategy-provider':
      return 1;
    case 'ui-widget':
      return 2;
    case 'script-plugin':
      return 3;
    default:
      return 1;
  }
}

// Helper function to get level label
export function getLevelLabel(level: PluginLevel): string {
  switch (level) {
    case 1: return 'Declarative';
    case 2: return 'UI Components';
    case 3: return 'Scripts';
    default: return 'Unknown';
  }
}

// Helper function to get level color
export function getLevelColor(level: PluginLevel): string {
  switch (level) {
    case 1: return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
    case 2: return 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20';
    case 3: return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
    default: return 'bg-zinc-500/10 text-zinc-400 border-zinc-500/20';
  }
}

// Helper function to get type label
export function getTypeLabel(type: PluginType): string {
  const labels: Record<PluginType, string> = {
    'service-checker': 'Service Checker',
    'strategy-provider': 'Strategy Provider',
    'hostlist-provider': 'Hostlist Provider',
    'ui-widget': 'UI Widget',
    'script-plugin': 'Script'
  };
  return labels[type] || 'Unknown';
}

// Helper function to get type icon
export function getTypeIcon(type: PluginType): string {
  const icons: Record<PluginType, string> = {
    'service-checker': '📡',
    'strategy-provider': '🎯',
    'hostlist-provider': '📋',
    'ui-widget': '🎨',
    'script-plugin': '📜'
  };
  return icons[type] || '📦';
}

// Start with empty array - plugins are loaded from backend
export const installedPlugins = writable<PluginInfo[]>([]);

// Track loading state for UI indicators
export const pluginsLoading = writable<boolean>(false);
export const pluginsError = writable<string | null>(null);

// Track initialization state
let storeInitialized = false;
let eventListenerUnlisten: (() => void) | null = null;

/**
 * Initialize the plugins store
 * Loads plugins from backend and sets up event listeners
 */
export async function initPluginsStore(): Promise<void> {
  // Prevent multiple initializations
  if (storeInitialized) {
    logger.log('Plugins', 'Store already initialized, skipping');
    return;
  }
  
  // Set loading state
  pluginsLoading.set(true);
  pluginsError.set(null);
  
  try {
    logger.log('Plugins', 'Initializing plugins store...');
    
    // Load plugins from backend
    const plugins = await loadPluginsFromBackend();
    if (plugins.length > 0) {
      installedPlugins.set(plugins);
      logger.log('Plugins', `Initialized with ${plugins.length} plugins`);
    } else {
      logger.log('Plugins', 'No plugins loaded from backend');
    }
    
    // Set up event listener for plugin state changes
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      try {
        // Clean up previous listener if exists
        if (eventListenerUnlisten) {
          eventListenerUnlisten();
          eventListenerUnlisten = null;
        }
        
        eventListenerUnlisten = await listen<{ plugin_id: string; enabled: boolean }>('plugin:state_changed', (event) => {
          logger.log('Plugins', `State changed event received for ${event.payload.plugin_id}: ${event.payload.enabled}`);
          
          installedPlugins.update(plugins => {
            return plugins.map(plugin => {
              if (plugin.id === event.payload.plugin_id) {
                return { ...plugin, enabled: event.payload.enabled };
              }
              return plugin;
            });
          });
        });
        logger.log('Plugins', 'Event listener registered for plugin:state_changed');
      } catch (error) {
        logger.warn('Plugins', 'Failed to setup event listener:', error);
      }
    }
    
    // Mark as initialized
    storeInitialized = true;
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    logger.error('Plugins', 'Failed to initialize store:', errorMsg);
    pluginsError.set(errorMsg);
    // Don't throw - allow app to continue with empty plugins
  } finally {
    pluginsLoading.set(false);
  }
}

/**
 * Cleanup the plugins store
 * Removes event listeners and resets state
 */
export function cleanupPluginsStore(): void {
  if (eventListenerUnlisten) {
    eventListenerUnlisten();
    eventListenerUnlisten = null;
    logger.log('Plugins', 'Event listener cleaned up');
  }
  storeInitialized = false;
}

/**
 * Check if plugins store has been initialized
 */
export function isPluginsStoreInitialized(): boolean {
  return storeInitialized;
}

// Function to update plugin settings
export function updatePluginSettings(pluginId: string, settings: PluginSetting[]) {
  installedPlugins.update(plugins => {
    return plugins.map(plugin => {
      if (plugin.id === pluginId) {
        return { ...plugin, settings };
      }
      return plugin;
    });
  });
}

// Backend plugin info structure
interface BackendPluginInfo {
  manifest: {
    id: string;
    name: string;
    version: string;
    author: string;
    description?: string;
    type: string;  // Note: Rust serializes plugin_type as "type"
    service?: {
      icon?: string;
      name?: string;
    };
  };
  enabled: boolean;
  path: string;
  error?: string;
}

// Function to load plugins from backend
export async function loadPluginsFromBackend(): Promise<PluginInfo[]> {
  try {
    // Check if backend is ready
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      logger.log('Plugins', 'Backend not ready');
      return [];
    }
    
    // Get plugins from backend - enabled states are already applied by backend
    const backendPlugins = await invoke<BackendPluginInfo[]>('get_all_plugins_cmd');
    logger.log('Plugins', 'Raw backend data:', JSON.stringify(backendPlugins, null, 2));
    
    // Map backend structure to frontend PluginInfo
    // Filter out plugins with errors or invalid data
    const mapped = backendPlugins
      .filter(bp => {
        // Skip plugins with errors
        if (bp.error) {
          logger.log('Plugins', `Skipping ${bp.manifest?.id || 'unknown'}: has error - ${bp.error}`);
          return false;
        }
        // Skip plugins without valid manifest
        if (!bp.manifest || !bp.manifest.id || !bp.manifest.name) {
          logger.log('Plugins', 'Skipping: invalid manifest');
          return false;
        }
        // Skip plugins with placeholder names
        if (bp.manifest.name === 'Invalid Plugin' || bp.manifest.name === 'Unknown') {
          logger.log('Plugins', `Skipping ${bp.manifest.id}: placeholder name`);
          return false;
        }
        return true;
      })
      .map(bp => {
        const typeMap: Record<string, PluginType> = {
          'service-checker': 'service-checker',
          'strategy-provider': 'strategy-provider',
          'hostlist-provider': 'hostlist-provider',
          'ui-widget': 'ui-widget',
          'script-plugin': 'script-plugin',
        };
        const rawType = bp.manifest.type;
        const pluginType = typeMap[rawType] || 'service-checker';
        
        // Get icon from service.icon or use default
        const icon = bp.manifest.service?.icon || getDefaultIcon(pluginType);
        
        // Use enabled state from backend (already includes saved states)
        const enabled = bp.enabled;
        
        logger.log('Plugins', `Mapping ${bp.manifest.id}: type=${rawType} -> ${pluginType}, icon=${icon}, enabled=${enabled}`);
        
        return {
          id: bp.manifest.id,
          name: bp.manifest.name,
          version: bp.manifest.version || '1.0.0',
          author: bp.manifest.author || 'Unknown',
          description: bp.manifest.description || '',
          icon,
          type: pluginType,
          level: getPluginLevel(pluginType),
          enabled,
          installed: true,
        };
      });
    
    logger.log('Plugins', `Loaded ${mapped.length} plugins from backend`);
    return mapped;
  } catch (error) {
    logger.warn('Plugins', 'Failed to load from backend:', error);
    return [];
  }
}

// Get default icon for plugin type
function getDefaultIcon(type: PluginType): string {
  const icons: Record<PluginType, string> = {
    'service-checker': '📡',
    'strategy-provider': '🎯',
    'hostlist-provider': '📋',
    'ui-widget': '🎨',
    'script-plugin': '📜'
  };
  return icons[type] || '📦';
}

// Function to set plugin enabled state (synced with backend)
export async function setPluginEnabled(pluginId: string, enabled: boolean): Promise<void> {
  try {
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      throw new Error('Backend not ready');
    }
    
    // Call backend command to persist state
    // Backend will emit 'plugin:state_changed' event which updates the store
    await invoke('set_plugin_enabled', { pluginId, enabled });
    
    // Also update local store immediately for responsive UI
    // (event listener will also update, but this ensures immediate feedback)
    installedPlugins.update(plugins => {
      return plugins.map(plugin => {
        if (plugin.id === pluginId) {
          return { ...plugin, enabled };
        }
        return plugin;
      });
    });
    
    logger.log('Plugins', `Plugin ${pluginId} ${enabled ? 'enabled' : 'disabled'}`);
  } catch (error) {
    logger.error('Plugins', `Failed to set plugin enabled state:`, error);
    throw error;
  }
}

// Function to toggle plugin enabled state
export async function togglePlugin(pluginId: string): Promise<void> {
  const plugins = await new Promise<PluginInfo[]>(resolve => {
    const unsubscribe = installedPlugins.subscribe(value => {
      unsubscribe();
      resolve(value);
    });
  });
  
  const plugin = plugins.find(p => p.id === pluginId);
  if (plugin) {
    await setPluginEnabled(pluginId, !plugin.enabled);
  }
}

// Function to install a plugin
export function installPlugin(plugin: PluginInfo) {
  installedPlugins.update(plugins => {
    const exists = plugins.find(p => p.id === plugin.id);
    if (exists) {
      return plugins.map(p => p.id === plugin.id ? { ...plugin, installed: true, enabled: true } : p);
    }
    return [...plugins, { ...plugin, installed: true, enabled: true }];
  });
}

// Function to uninstall a plugin
export function uninstallPlugin(pluginId: string) {
  installedPlugins.update(plugins => {
    return plugins.filter(p => p.id !== pluginId);
  });
}

// Result of reloading all plugins
export interface ReloadPluginsResult {
  plugins_loaded: number;
  hostlists_loaded: number;
  strategies_loaded: number;
  services_loaded: number;
}

// Function to reload all plugins (hot reload)
export async function reloadAllPlugins(): Promise<ReloadPluginsResult> {
  try {
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      throw new Error('Backend not ready');
    }
    
    const result = await invoke<ReloadPluginsResult>('reload_plugins');
    
    // Refresh the plugins list after reload
    const backendPlugins = await loadPluginsFromBackend();
    if (backendPlugins.length > 0) {
      installedPlugins.set(backendPlugins);
    }
    
    return result;
  } catch (error) {
    logger.error('Plugins', 'Failed to reload plugins:', error);
    throw error;
  }
}

// Function to reload a single plugin
export async function reloadPlugin(pluginId: string): Promise<boolean> {
  try {
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      throw new Error('Backend not ready');
    }
    
    const result = await invoke<boolean>('reload_plugin', { pluginId });
    
    // Refresh the plugins list after reload
    const backendPlugins = await loadPluginsFromBackend();
    if (backendPlugins.length > 0) {
      installedPlugins.set(backendPlugins);
    }
    
    return result;
  } catch (error) {
    logger.error('Plugins', `Failed to reload plugin ${pluginId}:`, error);
    throw error;
  }
}

// ============================================================================
// Plugin Services
// ============================================================================

export interface PluginServiceEndpoint {
  id: string;
  name: string;
  url: string;
  method: string;
}

export interface PluginServiceDefinition {
  id: string;
  name: string;
  icon: string;
  category: string;
  description?: string;
  endpoints: PluginServiceEndpoint[];
}

export interface PluginServiceStatus {
  service_id: string;
  accessible: boolean;
  avg_latency_ms?: number;
  endpoint_results: Array<{
    endpoint_id: string;
    success: boolean;
    latency_ms?: number;
    error?: string;
  }>;
}

// Function to get all services from plugins
export async function getPluginServices(): Promise<PluginServiceDefinition[]> {
  try {
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      logger.warn('Plugins', 'Backend not ready for getPluginServices');
      return [];
    }
    
    const services = await invoke<PluginServiceDefinition[]>('get_plugin_services');
    logger.log('Plugins', `Loaded ${services.length} services from plugins`);
    return services;
  } catch (error) {
    logger.error('Plugins', 'Failed to get plugin services:', error);
    return [];
  }
}

// Function to check a specific plugin service
export async function checkPluginService(serviceId: string): Promise<PluginServiceStatus | null> {
  try {
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      throw new Error('Backend not ready');
    }
    
    const status = await invoke<PluginServiceStatus>('check_plugin_service', { serviceId });
    return status;
  } catch (error) {
    logger.error('Plugins', `Failed to check plugin service ${serviceId}:`, error);
    throw error;
  }
}

// Function to check all plugin services
export async function checkAllPluginServices(): Promise<PluginServiceStatus[]> {
  try {
    const ready = await invoke<boolean>('is_backend_ready');
    if (!ready) {
      throw new Error('Backend not ready');
    }
    
    const results = await invoke<PluginServiceStatus[]>('check_all_plugin_services');
    logger.log('Plugins', `Checked ${results.length} plugin services`);
    return results;
  } catch (error) {
    logger.error('Plugins', 'Failed to check all plugin services:', error);
    throw error;
  }
}

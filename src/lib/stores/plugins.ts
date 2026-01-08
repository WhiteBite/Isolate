import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
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

// Auto-initialize store from backend when module loads
// This ensures plugins are available as soon as possible
if (typeof window !== 'undefined') {
  loadPluginsFromBackend().then(plugins => {
    installedPlugins.set(plugins);
  }).catch(err => {
    logger.warn('Plugins', 'Auto-init failed:', err);
    // Keep empty array as fallback
  });
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
    
    // Try to get plugins from backend
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
        
        logger.log('Plugins', `Mapping ${bp.manifest.id}: type=${rawType} -> ${pluginType}, icon=${icon}`);
        
        return {
          id: bp.manifest.id,
          name: bp.manifest.name,
          version: bp.manifest.version || '1.0.0',
          author: bp.manifest.author || 'Unknown',
          description: bp.manifest.description || '',
          icon,
          type: pluginType,
          level: getPluginLevel(pluginType),
          enabled: bp.enabled,
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

// Function to toggle plugin enabled state
export function togglePlugin(pluginId: string) {
  installedPlugins.update(plugins => {
    return plugins.map(plugin => {
      if (plugin.id === pluginId) {
        return { ...plugin, enabled: !plugin.enabled };
      }
      return plugin;
    });
  });
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

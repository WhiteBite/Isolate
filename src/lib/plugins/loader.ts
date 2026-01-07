/**
 * Plugin Loader
 * 
 * Handles dynamic loading of Svelte components for Level 2 UI plugins.
 * Uses Vite's import.meta.glob for development and dynamic imports for production.
 */

import type { Component } from 'svelte';
import type { 
  PluginManifest, 
  WidgetDefinition, 
  LoadedUIPlugin,
  PluginContext 
} from '$lib/types/plugin';
import { createPluginContext, destroyPluginContext } from './context';
import { writable, derived, get } from 'svelte/store';

// ============================================================================
// Types
// ============================================================================

export interface LoadedComponent {
  component: Component;
  widgetDef: WidgetDefinition;
  pluginId: string;
  context: PluginContext;
}

export interface PluginLoadError {
  pluginId: string;
  componentPath: string;
  error: string;
}

// ============================================================================
// Component Cache
// ============================================================================

/** Cache of loaded Svelte components */
const componentCache = new Map<string, Component>();

/** Cache of loaded plugins */
const loadedPlugins = new Map<string, LoadedUIPlugin>();

// ============================================================================
// Stores
// ============================================================================

/** Store of all loaded UI plugins */
export const uiPlugins = writable<LoadedUIPlugin[]>([]);

/** Store of loading errors */
export const pluginErrors = writable<PluginLoadError[]>([]);

/** Derived store of all widgets by slot */
export const widgetsBySlot = derived(uiPlugins, ($plugins) => {
  const slots: Record<string, LoadedComponent[]> = {
    dashboard: [],
    sidebar: [],
    settings: [],
    toolbar: []
  };
  
  for (const plugin of $plugins) {
    if (!plugin.enabled || !plugin.manifest.contributes.widgets) continue;
    
    const context = createPluginContext(plugin.manifest);
    
    for (const widget of plugin.manifest.contributes.widgets) {
      const cacheKey = `${plugin.manifest.id}:${widget.component}`;
      const component = componentCache.get(cacheKey);
      
      if (component && slots[widget.slot]) {
        slots[widget.slot].push({
          component,
          widgetDef: widget,
          pluginId: plugin.manifest.id,
          context
        });
      }
    }
  }
  
  // Sort by order
  for (const slot of Object.keys(slots)) {
    slots[slot].sort((a, b) => (a.widgetDef.order ?? 0) - (b.widgetDef.order ?? 0));
  }
  
  return slots;
});

// ============================================================================
// Dynamic Import Helpers
// ============================================================================

/**
 * Get all plugin components using Vite's glob import
 * This creates a map of all .svelte files in the plugins directory
 */
const pluginModules = import.meta.glob('/plugins/**/ui/*.svelte');

/**
 * Get all builtin plugin components
 */
const builtinModules = import.meta.glob('/src/lib/plugins/builtin/**/ui/*.svelte');

/**
 * Resolve component path to module loader
 */
function resolveComponentModule(pluginId: string, componentPath: string): (() => Promise<unknown>) | undefined {
  // Try plugins directory first
  const pluginPath = `/plugins/${pluginId}/${componentPath}`;
  if (pluginModules[pluginPath]) {
    return pluginModules[pluginPath];
  }
  
  // Try builtin plugins
  const builtinPath = `/src/lib/plugins/builtin/${pluginId}/${componentPath}`;
  if (builtinModules[builtinPath]) {
    return builtinModules[builtinPath];
  }
  
  return undefined;
}

// ============================================================================
// Component Loading
// ============================================================================

/**
 * Load a single Svelte component for a plugin
 */
export async function loadPluginComponent(
  pluginId: string,
  componentPath: string
): Promise<Component | null> {
  const cacheKey = `${pluginId}:${componentPath}`;
  
  // Return cached component
  if (componentCache.has(cacheKey)) {
    return componentCache.get(cacheKey)!;
  }
  
  try {
    const moduleLoader = resolveComponentModule(pluginId, componentPath);
    
    if (!moduleLoader) {
      console.warn(`[PluginLoader] Component not found: ${pluginId}/${componentPath}`);
      return null;
    }
    
    // Dynamic import
    const module = await moduleLoader() as { default: Component };
    const component = module.default;
    
    // Cache the component
    componentCache.set(cacheKey, component);
    
    console.log(`[PluginLoader] Loaded component: ${pluginId}/${componentPath}`);
    return component;
  } catch (error) {
    console.error(`[PluginLoader] Failed to load ${pluginId}/${componentPath}:`, error);
    
    pluginErrors.update(errors => [
      ...errors,
      {
        pluginId,
        componentPath,
        error: error instanceof Error ? error.message : String(error)
      }
    ]);
    
    return null;
  }
}

/**
 * Load all components for a plugin
 */
export async function loadPluginComponents(manifest: PluginManifest, pluginPath: string): Promise<LoadedUIPlugin> {
  const plugin: LoadedUIPlugin = {
    manifest,
    path: pluginPath,
    enabled: true,
    components: new Map()
  };
  
  const widgets = manifest.contributes.widgets ?? [];
  const settings = manifest.contributes.settings ?? [];
  
  // Load all widget components
  for (const widget of widgets) {
    const component = await loadPluginComponent(manifest.id, widget.component);
    if (component) {
      plugin.components.set(widget.component, component);
    }
  }
  
  // Load all settings components
  for (const setting of settings) {
    const component = await loadPluginComponent(manifest.id, setting.component);
    if (component) {
      plugin.components.set(setting.component, component);
    }
  }
  
  return plugin;
}

// ============================================================================
// Plugin Management
// ============================================================================

/**
 * Register and load a UI plugin
 */
export async function registerUIPlugin(manifest: PluginManifest, pluginPath: string = ''): Promise<LoadedUIPlugin> {
  // Check if already loaded
  if (loadedPlugins.has(manifest.id)) {
    return loadedPlugins.get(manifest.id)!;
  }
  
  // Load all components
  const plugin = await loadPluginComponents(manifest, pluginPath);
  
  // Store in cache
  loadedPlugins.set(manifest.id, plugin);
  
  // Update store
  uiPlugins.update(plugins => [...plugins, plugin]);
  
  console.log(`[PluginLoader] Registered plugin: ${manifest.id}`);
  return plugin;
}

/**
 * Unregister a UI plugin
 */
export function unregisterUIPlugin(pluginId: string): void {
  const plugin = loadedPlugins.get(pluginId);
  if (!plugin) return;
  
  // Clear component cache
  for (const componentPath of plugin.components.keys()) {
    componentCache.delete(`${pluginId}:${componentPath}`);
  }
  
  // Destroy context
  destroyPluginContext(pluginId);
  
  // Remove from cache
  loadedPlugins.delete(pluginId);
  
  // Update store
  uiPlugins.update(plugins => plugins.filter(p => p.manifest.id !== pluginId));
  
  console.log(`[PluginLoader] Unregistered plugin: ${pluginId}`);
}

/**
 * Enable/disable a plugin
 */
export function setPluginEnabled(pluginId: string, enabled: boolean): void {
  uiPlugins.update(plugins => 
    plugins.map(p => 
      p.manifest.id === pluginId ? { ...p, enabled } : p
    )
  );
}

/**
 * Get a loaded plugin by ID
 */
export function getLoadedPlugin(pluginId: string): LoadedUIPlugin | undefined {
  return loadedPlugins.get(pluginId);
}

/**
 * Get component for a specific widget
 */
export function getWidgetComponent(pluginId: string, componentPath: string): Component | undefined {
  const cacheKey = `${pluginId}:${componentPath}`;
  return componentCache.get(cacheKey);
}

/**
 * Clear all loaded plugins and cache
 */
export function clearAllPlugins(): void {
  componentCache.clear();
  loadedPlugins.clear();
  uiPlugins.set([]);
  pluginErrors.set([]);
}

// ============================================================================
// Initialization
// ============================================================================

/**
 * Initialize plugin loader and load builtin plugins
 */
export async function initializePluginLoader(): Promise<void> {
  console.log('[PluginLoader] Initializing...');
  
  // Load builtin plugins
  // These will be registered from the builtin directory
  
  console.log('[PluginLoader] Ready');
}

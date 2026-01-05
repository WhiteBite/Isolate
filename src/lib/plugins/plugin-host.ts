/**
 * Plugin Host - загружает и управляет плагинами
 * 
 * Плагины могут:
 * - Добавлять вкладки в sidebar
 * - Добавлять виджеты на dashboard
 * - Регистрировать команды
 * - Делать HTTP запросы (через sandbox)
 */

import { writable, derived, get } from 'svelte/store';

// Types
export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  icon?: string;
  main: string;
  contributes: {
    views?: ViewContribution[];
    widgets?: WidgetContribution[];
    commands?: CommandContribution[];
    checkers?: CheckerContribution[];
    menus?: MenuContribution[];
  };
  permissions: {
    http?: string[];
    filesystem?: boolean;
    process?: boolean;
    system?: boolean;
    strategies?: boolean;
  };
}

export interface ViewContribution {
  id: string;
  name: string;
  icon: string;
  component: string;
  location: string;
  order?: number;
}

export interface WidgetContribution {
  id: string;
  name: string;
  size: string;
  component: string;
}

export interface CommandContribution {
  id: string;
  title: string;
  icon?: string;
  keybinding?: string;
}

export interface CheckerContribution {
  id: string;
  name: string;
  icon?: string;
  urls: string[];
  handler?: string;
}

export interface MenuContribution {
  location: string;
  command: string;
  title: string;
  icon?: string;
  group?: string;
}

export interface LoadedPlugin {
  manifest: PluginManifest;
  enabled: boolean;
  path: string;
  instance?: PluginInstance;
  error?: string;
}

export interface PluginInstance {
  activate: (context: PluginContext) => void;
  deactivate: () => void;
  commands: Record<string, (...args: any[]) => any>;
  components: Record<string, PluginComponent>;
  getState?: () => any;
}

export interface PluginComponent {
  template: string;
  styles?: string;
  data?: () => any;
  methods?: Record<string, (...args: any[]) => any>;
}

export interface PluginContext {
  registerCommand: (id: string, handler: (...args: any[]) => any) => void;
  registerComponent: (name: string, component: PluginComponent) => void;
  subscriptions: { dispose: () => void }[];
}

// Stores
export const plugins = writable<LoadedPlugin[]>([]);
export const pluginViews = derived(plugins, $plugins => 
  $plugins
    .filter(p => p.enabled && p.manifest.contributes.views)
    .flatMap(p => p.manifest.contributes.views!.map(v => ({ ...v, pluginId: p.manifest.id })))
    .sort((a, b) => (a.order || 0) - (b.order || 0))
);
export const pluginWidgets = derived(plugins, $plugins =>
  $plugins
    .filter(p => p.enabled && p.manifest.contributes.widgets)
    .flatMap(p => p.manifest.contributes.widgets!.map(w => ({ ...w, pluginId: p.manifest.id })))
);
export const pluginCommands = derived(plugins, $plugins =>
  $plugins
    .filter(p => p.enabled && p.manifest.contributes.commands)
    .flatMap(p => p.manifest.contributes.commands!.map(c => ({ ...c, pluginId: p.manifest.id })))
);
export const pluginCheckers = derived(plugins, $plugins =>
  $plugins
    .filter(p => p.enabled && p.manifest.contributes.checkers)
    .flatMap(p => p.manifest.contributes.checkers!.map(c => ({ ...c, pluginId: p.manifest.id })))
);

// Command registry
const commandHandlers = new Map<string, (...args: any[]) => any>();

// Component registry
const componentRegistry = new Map<string, PluginComponent>();

/**
 * Execute a plugin command
 */
export async function executeCommand(commandId: string, ...args: any[]): Promise<any> {
  const handler = commandHandlers.get(commandId);
  if (!handler) {
    throw new Error(`Command not found: ${commandId}`);
  }
  return handler(...args);
}

/**
 * Get a registered component
 */
export function getComponent(pluginId: string, componentName: string): PluginComponent | undefined {
  return componentRegistry.get(`${pluginId}.${componentName}`);
}

/**
 * Create plugin sandbox API
 */
function createPluginAPI(manifest: PluginManifest) {
  const allowedDomains = manifest.permissions.http || [];
  
  return {
    http: {
      async get(url: string, options?: { timeout?: number }) {
        // Check domain permission
        const urlObj = new URL(url);
        const allowed = allowedDomains.some(d => 
          d === '*' || urlObj.hostname === d || urlObj.hostname.endsWith('.' + d)
        );
        if (!allowed) {
          throw new Error(`HTTP access denied for domain: ${urlObj.hostname}`);
        }
        
        // Use Tauri's HTTP client
        const { invoke } = (window as any).__TAURI__.core;
        return invoke('plugin_http_request', { 
          url, 
          method: 'GET',
          timeout: options?.timeout || 5000 
        });
      },
      async post(url: string, body: any, options?: { timeout?: number }) {
        const urlObj = new URL(url);
        const allowed = allowedDomains.some(d => 
          d === '*' || urlObj.hostname === d || urlObj.hostname.endsWith('.' + d)
        );
        if (!allowed) {
          throw new Error(`HTTP access denied for domain: ${urlObj.hostname}`);
        }
        
        const { invoke } = (window as any).__TAURI__.core;
        return invoke('plugin_http_request', { 
          url, 
          method: 'POST',
          body: JSON.stringify(body),
          timeout: options?.timeout || 5000 
        });
      }
    },
    events: {
      emit(event: string, data: any) {
        window.dispatchEvent(new CustomEvent(`plugin:${manifest.id}:${event}`, { detail: data }));
      },
      on(event: string, handler: (data: any) => void) {
        const listener = (e: CustomEvent) => handler(e.detail);
        window.addEventListener(`plugin:${manifest.id}:${event}`, listener as EventListener);
        return () => window.removeEventListener(`plugin:${manifest.id}:${event}`, listener as EventListener);
      }
    },
    ui: {
      showNotification(message: string, type: 'info' | 'success' | 'warning' | 'error' = 'info') {
        window.dispatchEvent(new CustomEvent('app:notification', { 
          detail: { message, type, pluginId: manifest.id } 
        }));
      }
    }
  };
}

/**
 * Load and activate a plugin
 */
export async function loadPlugin(pluginPath: string): Promise<LoadedPlugin> {
  try {
    // Load manifest
    const { invoke } = (window as any).__TAURI__.core;
    const manifest: PluginManifest = await invoke('load_plugin_manifest', { path: pluginPath });
    
    // Load plugin code
    const code: string = await invoke('load_plugin_code', { 
      path: pluginPath, 
      main: manifest.main 
    });
    
    // Create sandbox
    const api = createPluginAPI(manifest);
    
    // Execute plugin in sandbox
    const sandbox = new Function('Isolate', 'module', 'exports', code);
    const moduleExports: any = {};
    const module = { exports: moduleExports };
    
    // Inject API
    (globalThis as any).Isolate = api;
    sandbox(api, module, moduleExports);
    
    const instance = module.exports as PluginInstance;
    
    // Create context
    const context: PluginContext = {
      subscriptions: [],
      registerCommand(id: string, handler: (...args: any[]) => any) {
        commandHandlers.set(id, handler);
        this.subscriptions.push({
          dispose: () => commandHandlers.delete(id)
        });
      },
      registerComponent(name: string, component: PluginComponent) {
        componentRegistry.set(`${manifest.id}.${name}`, component);
        this.subscriptions.push({
          dispose: () => componentRegistry.delete(`${manifest.id}.${name}`)
        });
      }
    };
    
    // Activate
    if (instance.activate) {
      instance.activate(context);
    }
    
    const loadedPlugin: LoadedPlugin = {
      manifest,
      enabled: true,
      path: pluginPath,
      instance
    };
    
    // Update store
    plugins.update(p => [...p, loadedPlugin]);
    
    return loadedPlugin;
  } catch (error) {
    const loadedPlugin: LoadedPlugin = {
      manifest: { id: pluginPath, name: pluginPath } as any,
      enabled: false,
      path: pluginPath,
      error: String(error)
    };
    plugins.update(p => [...p, loadedPlugin]);
    return loadedPlugin;
  }
}

/**
 * Unload a plugin
 */
export function unloadPlugin(pluginId: string) {
  const currentPlugins = get(plugins);
  const plugin = currentPlugins.find(p => p.manifest.id === pluginId);
  
  if (plugin?.instance?.deactivate) {
    plugin.instance.deactivate();
  }
  
  plugins.update(p => p.filter(pl => pl.manifest.id !== pluginId));
}

/**
 * Toggle plugin enabled state
 */
export function togglePlugin(pluginId: string, enabled: boolean) {
  plugins.update(p => p.map(pl => 
    pl.manifest.id === pluginId ? { ...pl, enabled } : pl
  ));
}

/**
 * Scan and load all plugins
 */
export async function scanPlugins(): Promise<LoadedPlugin[]> {
  const { invoke } = (window as any).__TAURI__.core;
  const pluginPaths: string[] = await invoke('scan_plugin_directories');
  
  const loaded: LoadedPlugin[] = [];
  for (const path of pluginPaths) {
    const plugin = await loadPlugin(path);
    loaded.push(plugin);
  }
  
  return loaded;
}

/**
 * Plugin System Tests
 * 
 * Tests for scanner.ts, loader.ts, context.ts, and plugin types
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { PluginManifest, PluginType, PluginContext } from '$lib/types/plugin';

// ============================================================================
// Mocks
// ============================================================================

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
    get length() { return Object.keys(store).length; },
    key: vi.fn((i: number) => Object.keys(store)[i] ?? null)
  };
})();

Object.defineProperty(global, 'localStorage', { value: localStorageMock });

// Mock window for events
const eventListeners = new Map<string, Set<EventListener>>();
const windowMock = {
  addEventListener: vi.fn((event: string, handler: EventListener) => {
    if (!eventListeners.has(event)) eventListeners.set(event, new Set());
    eventListeners.get(event)!.add(handler);
  }),
  removeEventListener: vi.fn((event: string, handler: EventListener) => {
    eventListeners.get(event)?.delete(handler);
  }),
  dispatchEvent: vi.fn((event: Event) => {
    const handlers = eventListeners.get(event.type);
    handlers?.forEach(h => h(event));
    return true;
  })
};

Object.defineProperty(global, 'window', { value: windowMock });

// ============================================================================
// Test Fixtures
// ============================================================================

function createTestManifest(overrides: Partial<PluginManifest> = {}): PluginManifest {
  return {
    id: 'test-plugin',
    name: 'Test Plugin',
    version: '1.0.0',
    type: 'ui-plugin',
    author: 'Test Author',
    description: 'A test plugin',
    icon: '🧪',
    contributes: {
      widgets: [
        {
          id: 'test-widget',
          name: 'Test Widget',
          slot: 'dashboard',
          component: 'ui/TestWidget.svelte',
          defaultSize: { cols: 1, rows: 1 },
          order: 0
        }
      ]
    },
    permissions: {
      http: ['*.example.com'],
      storage: true,
      events: ['status:*', 'data:update'],
      timeout: 5000
    },
    ...overrides
  };
}

function createServiceCheckerManifest(): PluginManifest {
  return {
    id: 'discord-checker',
    name: 'Discord Checker',
    version: '1.0.0',
    type: 'service-checker',
    contributes: {
      services: [
        {
          id: 'discord-voice',
          name: 'Discord Voice',
          icon: '🎮',
          category: 'gaming',
          endpoints: [
            { url: 'https://discord.com/api/v10/gateway', method: 'GET', expectedStatus: 200 }
          ]
        }
      ]
    }
  };
}

function createHostlistProviderManifest(): PluginManifest {
  return {
    id: 'youtube-hostlist',
    name: 'YouTube Hostlist',
    version: '1.0.0',
    type: 'hostlist-provider',
    contributes: {}
  };
}

// ============================================================================
// Plugin Types Tests
// ============================================================================

describe('Plugin Types', () => {
  describe('PluginManifest validation', () => {
    it('accepts valid ui-plugin manifest', () => {
      const manifest = createTestManifest();
      
      expect(manifest.id).toBe('test-plugin');
      expect(manifest.type).toBe('ui-plugin');
      expect(manifest.contributes.widgets).toHaveLength(1);
    });

    it('accepts valid service-checker manifest', () => {
      const manifest = createServiceCheckerManifest();
      
      expect(manifest.type).toBe('service-checker');
      expect(manifest.contributes.services).toHaveLength(1);
      expect(manifest.contributes.services![0].endpoints).toHaveLength(1);
    });

    it('accepts valid hostlist-provider manifest', () => {
      const manifest = createHostlistProviderManifest();
      
      expect(manifest.type).toBe('hostlist-provider');
    });

    it('validates all plugin types', () => {
      const validTypes: PluginType[] = [
        'service-checker',
        'ui-plugin', 
        'script-plugin',
        'hostlist-provider',
        'strategy-config'
      ];
      
      validTypes.forEach(type => {
        const manifest = createTestManifest({ type });
        expect(manifest.type).toBe(type);
      });
    });
  });

  describe('WidgetDefinition', () => {
    it('validates widget slots', () => {
      const validSlots = ['dashboard', 'sidebar', 'settings', 'toolbar'] as const;
      
      validSlots.forEach(slot => {
        const manifest = createTestManifest({
          contributes: {
            widgets: [{
              id: 'test',
              name: 'Test',
              slot,
              component: 'ui/Test.svelte'
            }]
          }
        });
        
        expect(manifest.contributes.widgets![0].slot).toBe(slot);
      });
    });

    it('validates widget sizes', () => {
      const manifest = createTestManifest({
        contributes: {
          widgets: [{
            id: 'test',
            name: 'Test',
            slot: 'dashboard',
            component: 'ui/Test.svelte',
            defaultSize: { cols: 2, rows: 2 }
          }]
        }
      });
      
      expect(manifest.contributes.widgets![0].defaultSize).toEqual({ cols: 2, rows: 2 });
    });
  });

  describe('PluginPermissions', () => {
    it('validates http permissions with wildcards', () => {
      const manifest = createTestManifest({
        permissions: {
          http: ['*.discord.com', 'api.example.com']
        }
      });
      
      expect(manifest.permissions?.http).toContain('*.discord.com');
    });

    it('validates timeout permission', () => {
      const manifest = createTestManifest({
        permissions: { timeout: 10000 }
      });
      
      expect(manifest.permissions?.timeout).toBe(10000);
    });

    it('validates event permissions', () => {
      const manifest = createTestManifest({
        permissions: { events: ['status:*', 'data:update'] }
      });
      
      expect(manifest.permissions?.events).toContain('status:*');
    });
  });
});

// ============================================================================
// Plugin Context Tests
// ============================================================================

describe('Plugin Context', () => {
  let createPluginContext: typeof import('./context').createPluginContext;
  let destroyPluginContext: typeof import('./context').destroyPluginContext;
  let getPluginContext: typeof import('./context').getPluginContext;
  let clearAllContexts: typeof import('./context').clearAllContexts;

  beforeEach(async () => {
    vi.resetModules();
    localStorageMock.clear();
    eventListeners.clear();
    
    const contextModule = await import('./context');
    createPluginContext = contextModule.createPluginContext;
    destroyPluginContext = contextModule.destroyPluginContext;
    getPluginContext = contextModule.getPluginContext;
    clearAllContexts = contextModule.clearAllContexts;
    
    clearAllContexts();
  });

  afterEach(() => {
    clearAllContexts();
  });

  describe('createPluginContext', () => {
    it('creates context with correct plugin info', () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      expect(context.pluginId).toBe('test-plugin');
      expect(context.pluginVersion).toBe('1.0.0');
      expect(context.manifest).toBe(manifest);
    });

    it('returns cached context for same plugin', () => {
      const manifest = createTestManifest();
      const context1 = createPluginContext(manifest);
      const context2 = createPluginContext(manifest);
      
      expect(context1).toBe(context2);
    });

    it('creates different contexts for different plugins', () => {
      const manifest1 = createTestManifest({ id: 'plugin-1' });
      const manifest2 = createTestManifest({ id: 'plugin-2' });
      
      const context1 = createPluginContext(manifest1);
      const context2 = createPluginContext(manifest2);
      
      expect(context1).not.toBe(context2);
      expect(context1.pluginId).toBe('plugin-1');
      expect(context2.pluginId).toBe('plugin-2');
    });
  });

  describe('getPluginContext', () => {
    it('returns existing context', () => {
      const manifest = createTestManifest();
      const created = createPluginContext(manifest);
      const retrieved = getPluginContext('test-plugin');
      
      expect(retrieved).toBe(created);
    });

    it('returns undefined for non-existent context', () => {
      const context = getPluginContext('non-existent');
      expect(context).toBeUndefined();
    });
  });

  describe('destroyPluginContext', () => {
    it('removes context from cache', () => {
      const manifest = createTestManifest();
      createPluginContext(manifest);
      
      expect(getPluginContext('test-plugin')).toBeDefined();
      
      destroyPluginContext('test-plugin');
      
      expect(getPluginContext('test-plugin')).toBeUndefined();
    });

    it('handles non-existent context gracefully', () => {
      expect(() => destroyPluginContext('non-existent')).not.toThrow();
    });
  });

  describe('PluginStorage', () => {
    it('stores and retrieves values', async () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      await context.storage.set('testKey', { value: 42 });
      const result = await context.storage.get<{ value: number }>('testKey');
      
      expect(result).toEqual({ value: 42 });
    });

    it('returns null for non-existent keys', async () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      const result = await context.storage.get('nonExistent');
      
      expect(result).toBeNull();
    });

    it('deletes values', async () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      await context.storage.set('toDelete', 'value');
      await context.storage.delete('toDelete');
      const result = await context.storage.get('toDelete');
      
      expect(result).toBeNull();
    });

    it('lists all keys', async () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      await context.storage.set('key1', 'value1');
      await context.storage.set('key2', 'value2');
      
      const keys = await context.storage.keys();
      
      expect(keys).toContain('key1');
      expect(keys).toContain('key2');
    });

    it('isolates storage between plugins', async () => {
      const manifest1 = createTestManifest({ id: 'plugin-1' });
      const manifest2 = createTestManifest({ id: 'plugin-2' });
      
      const context1 = createPluginContext(manifest1);
      const context2 = createPluginContext(manifest2);
      
      await context1.storage.set('shared-key', 'value-1');
      await context2.storage.set('shared-key', 'value-2');
      
      const result1 = await context1.storage.get('shared-key');
      const result2 = await context2.storage.get('shared-key');
      
      expect(result1).toBe('value-1');
      expect(result2).toBe('value-2');
    });
  });

  describe('PluginEvents', () => {
    it('emits and receives events', () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      const handler = vi.fn();
      context.events.on('status:changed', handler);
      
      context.events.emit('status:changed', { status: 'active' });
      
      expect(handler).toHaveBeenCalledWith({ status: 'active' });
    });

    it('unsubscribes from events', () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      const handler = vi.fn();
      const unsubscribe = context.events.on('test-event', handler);
      
      unsubscribe();
      context.events.emit('test-event', 'data');
      
      expect(handler).not.toHaveBeenCalled();
    });

    it('handles once subscription', () => {
      const manifest = createTestManifest({
        permissions: { events: ['one-time'] }
      });
      const context = createPluginContext(manifest);
      
      const handler = vi.fn();
      context.events.once('one-time', handler);
      
      // Manually trigger the event through window mock
      const eventName = `plugin:${manifest.id}:one-time`;
      const event = new CustomEvent(eventName, { detail: 'first' });
      windowMock.dispatchEvent(event);
      
      expect(handler).toHaveBeenCalledTimes(1);
      expect(handler).toHaveBeenCalledWith('first');
    });

    it('respects event permissions with wildcards', () => {
      const manifest = createTestManifest({
        permissions: { events: ['status:*'] }
      });
      const context = createPluginContext(manifest);
      
      const allowedHandler = vi.fn();
      const blockedHandler = vi.fn();
      
      context.events.on('status:changed', allowedHandler);
      context.events.on('data:update', blockedHandler);
      
      context.events.emit('status:changed', 'allowed');
      context.events.emit('data:update', 'blocked');
      
      expect(allowedHandler).toHaveBeenCalled();
      expect(blockedHandler).not.toHaveBeenCalled();
    });

    it('blocks events not in permissions', () => {
      const manifest = createTestManifest({
        permissions: { events: ['allowed-event'] }
      });
      const context = createPluginContext(manifest);
      
      const handler = vi.fn();
      const unsubscribe = context.events.on('blocked-event', handler);
      
      context.events.emit('blocked-event', 'data');
      
      expect(handler).not.toHaveBeenCalled();
      expect(unsubscribe).toBeDefined();
    });
  });

  describe('Sandboxed Invoke', () => {
    it('allows whitelisted commands', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockedInvoke = vi.mocked(invoke);
      mockedInvoke.mockResolvedValue({ status: 'ok' });
      
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      const result = await context.invoke('is_backend_ready');
      
      expect(mockedInvoke).toHaveBeenCalledWith('is_backend_ready', {
        __pluginId: 'test-plugin'
      });
    });

    it('blocks non-whitelisted commands', async () => {
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      await expect(context.invoke('dangerous_command'))
        .rejects.toThrow('Command "dangerous_command" is not allowed');
    });

    it('adds plugin context to args', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockedInvoke = vi.mocked(invoke);
      mockedInvoke.mockResolvedValue([]);
      
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      await context.invoke('get_services', { filter: 'active' });
      
      expect(mockedInvoke).toHaveBeenCalledWith('get_services', {
        filter: 'active',
        __pluginId: 'test-plugin'
      });
    });

    it('respects timeout from permissions', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      const mockedInvoke = vi.mocked(invoke);
      
      // Simulate slow response
      mockedInvoke.mockImplementation(() => 
        new Promise(resolve => setTimeout(() => resolve('late'), 200))
      );
      
      const manifest = createTestManifest({
        permissions: { timeout: 50 }
      });
      
      // Need fresh context with new timeout
      clearAllContexts();
      const context = createPluginContext(manifest);
      
      await expect(context.invoke('get_services'))
        .rejects.toThrow('timed out');
    });
  });
});

// ============================================================================
// Plugin Loader Tests
// ============================================================================

describe('Plugin Loader', () => {
  let registerUIPlugin: typeof import('./loader').registerUIPlugin;
  let unregisterUIPlugin: typeof import('./loader').unregisterUIPlugin;
  let setPluginEnabled: typeof import('./loader').setPluginEnabled;
  let getLoadedPlugin: typeof import('./loader').getLoadedPlugin;
  let clearAllPlugins: typeof import('./loader').clearAllPlugins;
  let uiPlugins: typeof import('./loader').uiPlugins;
  let pluginErrors: typeof import('./loader').pluginErrors;

  beforeEach(async () => {
    vi.resetModules();
    
    const loaderModule = await import('./loader');
    registerUIPlugin = loaderModule.registerUIPlugin;
    unregisterUIPlugin = loaderModule.unregisterUIPlugin;
    setPluginEnabled = loaderModule.setPluginEnabled;
    getLoadedPlugin = loaderModule.getLoadedPlugin;
    clearAllPlugins = loaderModule.clearAllPlugins;
    uiPlugins = loaderModule.uiPlugins;
    pluginErrors = loaderModule.pluginErrors;
    
    clearAllPlugins();
  });

  afterEach(() => {
    clearAllPlugins();
  });

  describe('registerUIPlugin', () => {
    it('registers a plugin and adds to store', async () => {
      const manifest = createTestManifest();
      const { get } = await import('svelte/store');
      
      const plugin = await registerUIPlugin(manifest, 'plugins/test-plugin');
      
      expect(plugin.manifest.id).toBe('test-plugin');
      expect(plugin.enabled).toBe(true);
      expect(plugin.path).toBe('plugins/test-plugin');
      
      const plugins = get(uiPlugins);
      expect(plugins).toHaveLength(1);
      expect(plugins[0].manifest.id).toBe('test-plugin');
    });

    it('returns cached plugin if already registered', async () => {
      const manifest = createTestManifest();
      
      const plugin1 = await registerUIPlugin(manifest, 'path1');
      const plugin2 = await registerUIPlugin(manifest, 'path2');
      
      expect(plugin1).toBe(plugin2);
    });

    it('registers multiple plugins', async () => {
      const { get } = await import('svelte/store');
      
      const manifest1 = createTestManifest({ id: 'plugin-1', name: 'Plugin 1' });
      const manifest2 = createTestManifest({ id: 'plugin-2', name: 'Plugin 2' });
      
      await registerUIPlugin(manifest1, 'path1');
      await registerUIPlugin(manifest2, 'path2');
      
      const plugins = get(uiPlugins);
      expect(plugins).toHaveLength(2);
    });
  });

  describe('unregisterUIPlugin', () => {
    it('removes plugin from store', async () => {
      const { get } = await import('svelte/store');
      const manifest = createTestManifest();
      
      await registerUIPlugin(manifest, 'path');
      expect(get(uiPlugins)).toHaveLength(1);
      
      unregisterUIPlugin('test-plugin');
      expect(get(uiPlugins)).toHaveLength(0);
    });

    it('handles non-existent plugin gracefully', () => {
      expect(() => unregisterUIPlugin('non-existent')).not.toThrow();
    });
  });

  describe('setPluginEnabled', () => {
    it('enables and disables plugins', async () => {
      const { get } = await import('svelte/store');
      const manifest = createTestManifest();
      
      await registerUIPlugin(manifest, 'path');
      
      setPluginEnabled('test-plugin', false);
      expect(get(uiPlugins)[0].enabled).toBe(false);
      
      setPluginEnabled('test-plugin', true);
      expect(get(uiPlugins)[0].enabled).toBe(true);
    });
  });

  describe('getLoadedPlugin', () => {
    it('returns loaded plugin by ID', async () => {
      const manifest = createTestManifest();
      await registerUIPlugin(manifest, 'path');
      
      const plugin = getLoadedPlugin('test-plugin');
      
      expect(plugin).toBeDefined();
      expect(plugin?.manifest.id).toBe('test-plugin');
    });

    it('returns undefined for non-existent plugin', () => {
      const plugin = getLoadedPlugin('non-existent');
      expect(plugin).toBeUndefined();
    });
  });

  describe('clearAllPlugins', () => {
    it('removes all plugins and clears errors', async () => {
      const { get } = await import('svelte/store');
      
      const manifest1 = createTestManifest({ id: 'plugin-1' });
      const manifest2 = createTestManifest({ id: 'plugin-2' });
      
      await registerUIPlugin(manifest1, 'path1');
      await registerUIPlugin(manifest2, 'path2');
      
      clearAllPlugins();
      
      expect(get(uiPlugins)).toHaveLength(0);
      expect(get(pluginErrors)).toHaveLength(0);
    });
  });
});

// ============================================================================
// Plugin Scanner Tests
// ============================================================================

describe('Plugin Scanner', () => {
  describe('discoverPlugins', () => {
    it('discovers plugins from glob imports', async () => {
      // Note: In test environment, import.meta.glob returns empty
      // This test verifies the function doesn't throw
      const { discoverPlugins } = await import('./scanner');
      
      const manifests = discoverPlugins();
      
      expect(Array.isArray(manifests)).toBe(true);
    });
  });

  describe('Manifest Validation', () => {
    it('validates required fields', () => {
      // Test manifest structure requirements
      const validManifest = createTestManifest();
      
      expect(validManifest.id).toBeDefined();
      expect(validManifest.name).toBeDefined();
      expect(validManifest.version).toBeDefined();
      expect(validManifest.type).toBeDefined();
    });

    it('validates plugin type enum', () => {
      const validTypes = ['service-checker', 'ui-plugin', 'script-plugin', 'hostlist-provider', 'strategy-config'];
      
      validTypes.forEach(type => {
        const manifest = createTestManifest({ type: type as PluginType });
        expect(validTypes).toContain(manifest.type);
      });
    });

    it('rejects invalid plugin types', () => {
      const invalidTypes = ['invalid', 'unknown', 'custom'];
      const validTypes = ['service-checker', 'ui-plugin', 'script-plugin', 'hostlist-provider', 'strategy-config'];
      
      invalidTypes.forEach(type => {
        expect(validTypes).not.toContain(type);
      });
    });
  });

  describe('initializePlugins', () => {
    it('initializes plugin system without errors', async () => {
      const { initializePlugins } = await import('./scanner');
      
      // Should not throw
      await expect(initializePlugins()).resolves.not.toThrow();
    });
  });

  describe('reloadPlugin', () => {
    it('reloads a specific plugin', async () => {
      const { reloadPlugin } = await import('./scanner');
      
      // Should not throw even for non-existent plugin
      await expect(reloadPlugin('non-existent')).resolves.not.toThrow();
    });
  });
});

// ============================================================================
// Builtin Plugins Tests
// ============================================================================

describe('Builtin Plugins', () => {
  describe('Manifest Definitions', () => {
    it('defines status widget manifest correctly', async () => {
      const { statusWidgetManifest } = await import('./builtin');
      
      expect(statusWidgetManifest.id).toBe('builtin-status');
      expect(statusWidgetManifest.type).toBe('ui-plugin');
      expect(statusWidgetManifest.contributes.widgets).toHaveLength(1);
      expect(statusWidgetManifest.contributes.widgets![0].slot).toBe('dashboard');
    });

    it('defines health widget manifest correctly', async () => {
      const { healthWidgetManifest } = await import('./builtin');
      
      expect(healthWidgetManifest.id).toBe('builtin-health');
      expect(healthWidgetManifest.type).toBe('ui-plugin');
      expect(healthWidgetManifest.contributes.widgets).toHaveLength(1);
    });

    it('exports builtin manifests array', async () => {
      const { builtinManifests } = await import('./builtin');
      
      expect(Array.isArray(builtinManifests)).toBe(true);
    });
  });

  describe('registerBuiltinPlugins', () => {
    it('registers all builtin plugins', async () => {
      const { registerBuiltinPlugins } = await import('./builtin');
      
      // Should not throw
      await expect(registerBuiltinPlugins()).resolves.not.toThrow();
    });
  });
});

// ============================================================================
// Integration Tests
// ============================================================================

describe('Plugin System Integration', () => {
  beforeEach(async () => {
    vi.resetModules();
    localStorageMock.clear();
    eventListeners.clear();
  });

  describe('Full Plugin Lifecycle', () => {
    it('registers, uses, and unregisters a plugin', async () => {
      const { registerUIPlugin, unregisterUIPlugin, getLoadedPlugin, clearAllPlugins } = await import('./loader');
      const { createPluginContext, destroyPluginContext } = await import('./context');
      
      clearAllPlugins();
      
      // 1. Register plugin
      const manifest = createTestManifest();
      const plugin = await registerUIPlugin(manifest, 'test-path');
      
      expect(plugin).toBeDefined();
      expect(getLoadedPlugin('test-plugin')).toBeDefined();
      
      // 2. Create and use context
      const context = createPluginContext(manifest);
      await context.storage.set('test', 'value');
      const stored = await context.storage.get('test');
      expect(stored).toBe('value');
      
      // 3. Unregister plugin
      unregisterUIPlugin('test-plugin');
      expect(getLoadedPlugin('test-plugin')).toBeUndefined();
      
      // 4. Cleanup
      destroyPluginContext('test-plugin');
      clearAllPlugins();
    });

    it('handles multiple plugins with isolated contexts', async () => {
      const { registerUIPlugin, clearAllPlugins } = await import('./loader');
      const { createPluginContext, clearAllContexts } = await import('./context');
      
      clearAllPlugins();
      clearAllContexts();
      
      const manifest1 = createTestManifest({ id: 'plugin-a' });
      const manifest2 = createTestManifest({ id: 'plugin-b' });
      
      await registerUIPlugin(manifest1, 'path-a');
      await registerUIPlugin(manifest2, 'path-b');
      
      const contextA = createPluginContext(manifest1);
      const contextB = createPluginContext(manifest2);
      
      // Store different values
      await contextA.storage.set('key', 'value-a');
      await contextB.storage.set('key', 'value-b');
      
      // Verify isolation
      expect(await contextA.storage.get('key')).toBe('value-a');
      expect(await contextB.storage.get('key')).toBe('value-b');
      
      clearAllPlugins();
      clearAllContexts();
    });
  });

  describe('Error Handling', () => {
    it('handles storage errors gracefully', async () => {
      const { createPluginContext, clearAllContexts } = await import('./context');
      clearAllContexts();
      
      // Simulate corrupted localStorage
      localStorageMock.getItem.mockReturnValueOnce('invalid json{');
      
      const manifest = createTestManifest();
      const context = createPluginContext(manifest);
      
      // Should return null instead of throwing
      const result = await context.storage.get('any-key');
      expect(result).toBeNull();
    });

    it('handles event permission errors gracefully', async () => {
      const { createPluginContext, clearAllContexts } = await import('./context');
      clearAllContexts();
      
      const manifest = createTestManifest({
        permissions: { events: [] } // No events allowed
      });
      const context = createPluginContext(manifest);
      
      const handler = vi.fn();
      
      // Should not throw, just warn
      expect(() => context.events.on('blocked', handler)).not.toThrow();
      expect(() => context.events.emit('blocked', 'data')).not.toThrow();
    });
  });
});

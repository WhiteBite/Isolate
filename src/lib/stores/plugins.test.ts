import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock @tauri-apps/api/core before importing the store
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

import { invoke } from '@tauri-apps/api/core';
import {
  installedPlugins,
  getPluginLevel,
  getLevelLabel,
  getLevelColor,
  getTypeLabel,
  getTypeIcon,
  togglePlugin,
  installPlugin,
  uninstallPlugin,
  loadPluginsFromBackend,
  type PluginInfo,
  type PluginType,
  type PluginLevel
} from './plugins';

const mockedInvoke = vi.mocked(invoke);

// Helper to create test plugin with all required fields
function createTestPlugin(overrides: Partial<PluginInfo> & { id: string; name: string; icon: string }): PluginInfo {
  return {
    version: '1.0.0',
    author: 'Test Author',
    description: 'Test plugin',
    type: 'ui-plugin',
    enabled: true,
    level: 2,
    ...overrides,
  };
}

describe('installedPlugins store', () => {
  beforeEach(() => {
    // Reset store to empty state (default)
    installedPlugins.set([]);
    vi.clearAllMocks();
  });

  describe('initial state', () => {
    it('starts with empty array by default', () => {
      const plugins = get(installedPlugins);
      expect(plugins).toEqual([]);
    });
  });

  describe('set method', () => {
    it('sets plugins array', () => {
      const testPlugins: PluginInfo[] = [
        createTestPlugin({ id: 'test-plugin', name: 'Test Plugin', icon: '🧪' })
      ];
      
      installedPlugins.set(testPlugins);
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(1);
      expect(plugins[0].id).toBe('test-plugin');
    });

    it('can set empty array', () => {
      installedPlugins.set([
        createTestPlugin({ id: 'test', name: 'Test', icon: '🧪' })
      ]);
      installedPlugins.set([]);
      
      expect(get(installedPlugins)).toEqual([]);
    });

    it('replaces all plugins', () => {
      installedPlugins.set([
        createTestPlugin({ id: 'old-plugin', name: 'Old Plugin', icon: '📦' })
      ]);
      
      const newPlugins: PluginInfo[] = [
        createTestPlugin({ id: 'new-plugin', name: 'New Plugin', icon: '🆕' })
      ];
      
      installedPlugins.set(newPlugins);
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(1);
      expect(plugins[0].id).toBe('new-plugin');
    });
  });

  describe('update method', () => {
    it('adds new plugin', () => {
      installedPlugins.set([
        createTestPlugin({ id: 'existing', name: 'Existing', icon: '📦' })
      ]);
      
      installedPlugins.update(plugins => [
        ...plugins,
        createTestPlugin({ id: 'new-plugin', name: 'New Plugin', icon: '🆕', route: '/plugins/new' })
      ]);
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(2);
      expect(plugins[1].id).toBe('new-plugin');
    });

    it('removes plugin by id', () => {
      installedPlugins.set([
        createTestPlugin({ id: 'plugin-1', name: 'Plugin 1', icon: '🔌' }),
        createTestPlugin({ id: 'plugin-2', name: 'Plugin 2', icon: '🔧' })
      ]);
      
      installedPlugins.update(plugins => 
        plugins.filter(p => p.id !== 'plugin-1')
      );
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(1);
      expect(plugins[0].id).toBe('plugin-2');
    });

    it('updates existing plugin', () => {
      installedPlugins.set([
        createTestPlugin({ id: 'plugin-1', name: 'Old Name', icon: '📦' })
      ]);
      
      installedPlugins.update(plugins => 
        plugins.map(p => 
          p.id === 'plugin-1' 
            ? { ...p, name: 'Updated Name' }
            : p
        )
      );
      
      const plugins = get(installedPlugins);
      const plugin = plugins.find(p => p.id === 'plugin-1');
      expect(plugin?.name).toBe('Updated Name');
    });
  });

  describe('subscription', () => {
    it('notifies subscribers on changes', () => {
      const callback = vi.fn();
      const unsubscribe = installedPlugins.subscribe(callback);
      
      // Initial call with current value
      expect(callback).toHaveBeenCalledTimes(1);
      
      installedPlugins.update(plugins => [
        ...plugins,
        createTestPlugin({ id: 'test', name: 'Test', icon: '🧪' })
      ]);
      
      expect(callback).toHaveBeenCalledTimes(2);
      
      unsubscribe();
    });

    it('stops notifying after unsubscribe', () => {
      const callback = vi.fn();
      const unsubscribe = installedPlugins.subscribe(callback);
      
      unsubscribe();
      
      installedPlugins.set([createTestPlugin({ id: 'test', name: 'Test', icon: '🧪' })]);
      
      // Should still be 1 (only initial call)
      expect(callback).toHaveBeenCalledTimes(1);
    });
  });

  describe('plugin without route', () => {
    it('allows plugins without route property', () => {
      installedPlugins.set([
        createTestPlugin({ id: 'no-route', name: 'No Route Plugin', icon: '📦' })
      ]);
      
      const plugins = get(installedPlugins);
      expect(plugins[0].route).toBeUndefined();
    });
  });
});

describe('loadPluginsFromBackend', () => {
  beforeEach(() => {
    installedPlugins.set([]);
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('returns empty array when backend is not ready', async () => {
    mockedInvoke.mockResolvedValueOnce(false); // is_backend_ready returns false
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toEqual([]);
    expect(mockedInvoke).toHaveBeenCalledWith('is_backend_ready');
  });

  it('returns empty array on invoke error (fallback)', async () => {
    mockedInvoke.mockRejectedValueOnce(new Error('Connection failed'));
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toEqual([]);
  });

  it('returns empty array when get_all_plugins_cmd fails', async () => {
    mockedInvoke
      .mockResolvedValueOnce(true) // is_backend_ready
      .mockRejectedValueOnce(new Error('Command not found'));
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toEqual([]);
  });

  it('returns empty array when backend returns empty plugins', async () => {
    mockedInvoke
      .mockResolvedValueOnce(true) // is_backend_ready
      .mockResolvedValueOnce([]); // get_all_plugins_cmd
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toEqual([]);
  });

  it('maps backend plugins to frontend format', async () => {
    const backendPlugins = [
      {
        manifest: {
          id: 'test-service',
          name: 'Test Service',
          version: '1.2.0',
          author: 'Test Author',
          description: 'A test service checker',
          type: 'service-checker',
          service: { icon: '🔍' }
        },
        enabled: true,
        path: '/plugins/test-service'
      }
    ];
    
    mockedInvoke
      .mockResolvedValueOnce(true) // is_backend_ready
      .mockResolvedValueOnce(backendPlugins); // get_all_plugins_cmd
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({
      id: 'test-service',
      name: 'Test Service',
      version: '1.2.0',
      author: 'Test Author',
      description: 'A test service checker',
      icon: '🔍',
      type: 'service-checker',
      level: 1,
      enabled: true,
      installed: true
    });
  });

  it('filters out plugins with errors', async () => {
    const backendPlugins = [
      {
        manifest: { id: 'good-plugin', name: 'Good Plugin', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: true,
        path: '/plugins/good'
      },
      {
        manifest: { id: 'bad-plugin', name: 'Bad Plugin', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: false,
        path: '/plugins/bad',
        error: 'Failed to load'
      }
    ];
    
    mockedInvoke
      .mockResolvedValueOnce(true)
      .mockResolvedValueOnce(backendPlugins);
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe('good-plugin');
  });

  it('filters out plugins with invalid manifest', async () => {
    const backendPlugins = [
      {
        manifest: { id: 'valid', name: 'Valid Plugin', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: true,
        path: '/plugins/valid'
      },
      {
        manifest: { id: '', name: '', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: true,
        path: '/plugins/invalid'
      },
      {
        manifest: null,
        enabled: true,
        path: '/plugins/null'
      }
    ];
    
    mockedInvoke
      .mockResolvedValueOnce(true)
      .mockResolvedValueOnce(backendPlugins);
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe('valid');
  });

  it('filters out plugins with placeholder names', async () => {
    const backendPlugins = [
      {
        manifest: { id: 'real', name: 'Real Plugin', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: true,
        path: '/plugins/real'
      },
      {
        manifest: { id: 'invalid', name: 'Invalid Plugin', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: true,
        path: '/plugins/invalid'
      },
      {
        manifest: { id: 'unknown', name: 'Unknown', version: '1.0.0', author: 'Author', type: 'ui-plugin' },
        enabled: true,
        path: '/plugins/unknown'
      }
    ];
    
    mockedInvoke
      .mockResolvedValueOnce(true)
      .mockResolvedValueOnce(backendPlugins);
    
    const result = await loadPluginsFromBackend();
    
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe('real');
  });

  it('maps plugin types correctly', async () => {
    const backendPlugins = [
      { manifest: { id: 'p1', name: 'P1', version: '1.0.0', author: 'A', type: 'service-checker' }, enabled: true, path: '' },
      { manifest: { id: 'p2', name: 'P2', version: '1.0.0', author: 'A', type: 'strategy-provider' }, enabled: true, path: '' },
      { manifest: { id: 'p3', name: 'P3', version: '1.0.0', author: 'A', type: 'hostlist-provider' }, enabled: true, path: '' },
      { manifest: { id: 'p4', name: 'P4', version: '1.0.0', author: 'A', type: 'ui-widget' }, enabled: true, path: '' },
      { manifest: { id: 'p5', name: 'P5', version: '1.0.0', author: 'A', type: 'script-plugin' }, enabled: true, path: '' },
      { manifest: { id: 'p6', name: 'P6', version: '1.0.0', author: 'A', type: 'unknown-type' }, enabled: true, path: '' },
    ];
    
    mockedInvoke
      .mockResolvedValueOnce(true)
      .mockResolvedValueOnce(backendPlugins);
    
    const result = await loadPluginsFromBackend();
    
    expect(result.find(p => p.id === 'p1')?.type).toBe('service-checker');
    expect(result.find(p => p.id === 'p2')?.type).toBe('strategy-config');
    expect(result.find(p => p.id === 'p3')?.type).toBe('hostlist-provider');
    expect(result.find(p => p.id === 'p4')?.type).toBe('ui-plugin');
    expect(result.find(p => p.id === 'p5')?.type).toBe('script-plugin');
    expect(result.find(p => p.id === 'p6')?.type).toBe('service-checker'); // fallback
  });

  it('uses default icon when service.icon is not provided', async () => {
    const backendPlugins = [
      {
        manifest: { id: 'no-icon', name: 'No Icon', version: '1.0.0', author: 'A', type: 'service-checker' },
        enabled: true,
        path: ''
      }
    ];
    
    mockedInvoke
      .mockResolvedValueOnce(true)
      .mockResolvedValueOnce(backendPlugins);
    
    const result = await loadPluginsFromBackend();
    
    expect(result[0].icon).toBe('📡'); // default for service-checker
  });
});

describe('getPluginLevel helper', () => {
  it('returns level 1 for service-checker', () => {
    expect(getPluginLevel('service-checker')).toBe(1);
  });

  it('returns level 1 for hostlist-provider', () => {
    expect(getPluginLevel('hostlist-provider')).toBe(1);
  });

  it('returns level 1 for strategy-config', () => {
    expect(getPluginLevel('strategy-config')).toBe(1);
  });

  it('returns level 2 for ui-plugin', () => {
    expect(getPluginLevel('ui-plugin')).toBe(2);
  });

  it('returns level 3 for script-plugin', () => {
    expect(getPluginLevel('script-plugin')).toBe(3);
  });

  it('returns level 1 for unknown type', () => {
    expect(getPluginLevel('unknown-type' as PluginType)).toBe(1);
  });
});

describe('getLevelLabel helper', () => {
  it('returns "Declarative" for level 1', () => {
    expect(getLevelLabel(1)).toBe('Declarative');
  });

  it('returns "UI Components" for level 2', () => {
    expect(getLevelLabel(2)).toBe('UI Components');
  });

  it('returns "Scripts" for level 3', () => {
    expect(getLevelLabel(3)).toBe('Scripts');
  });

  it('returns "Unknown" for invalid level', () => {
    expect(getLevelLabel(99 as PluginLevel)).toBe('Unknown');
  });
});

describe('getLevelColor helper', () => {
  it('returns emerald color classes for level 1', () => {
    const color = getLevelColor(1);
    expect(color).toContain('emerald');
    expect(color).toContain('bg-');
    expect(color).toContain('text-');
    expect(color).toContain('border-');
  });

  it('returns indigo color classes for level 2', () => {
    const color = getLevelColor(2);
    expect(color).toContain('indigo');
    expect(color).toContain('bg-');
    expect(color).toContain('text-');
    expect(color).toContain('border-');
  });

  it('returns amber color classes for level 3', () => {
    const color = getLevelColor(3);
    expect(color).toContain('amber');
    expect(color).toContain('bg-');
    expect(color).toContain('text-');
    expect(color).toContain('border-');
  });

  it('returns zinc color classes for invalid level', () => {
    const color = getLevelColor(99 as PluginLevel);
    expect(color).toContain('zinc');
  });
});

describe('getTypeLabel helper', () => {
  it('returns "Service Checker" for service-checker', () => {
    expect(getTypeLabel('service-checker')).toBe('Service Checker');
  });

  it('returns "Hostlist Provider" for hostlist-provider', () => {
    expect(getTypeLabel('hostlist-provider')).toBe('Hostlist Provider');
  });

  it('returns "Strategy Config" for strategy-config', () => {
    expect(getTypeLabel('strategy-config')).toBe('Strategy Config');
  });

  it('returns "UI Plugin" for ui-plugin', () => {
    expect(getTypeLabel('ui-plugin')).toBe('UI Plugin');
  });

  it('returns "Script" for script-plugin', () => {
    expect(getTypeLabel('script-plugin')).toBe('Script');
  });

  it('returns "Unknown" for unknown type', () => {
    expect(getTypeLabel('unknown-type' as PluginType)).toBe('Unknown');
  });
});

describe('getTypeIcon helper', () => {
  it('returns 📡 for service-checker', () => {
    expect(getTypeIcon('service-checker')).toBe('📡');
  });

  it('returns 📋 for hostlist-provider', () => {
    expect(getTypeIcon('hostlist-provider')).toBe('📋');
  });

  it('returns 🎯 for strategy-config', () => {
    expect(getTypeIcon('strategy-config')).toBe('🎯');
  });

  it('returns 🎨 for ui-plugin', () => {
    expect(getTypeIcon('ui-plugin')).toBe('🎨');
  });

  it('returns 📜 for script-plugin', () => {
    expect(getTypeIcon('script-plugin')).toBe('📜');
  });

  it('returns 📦 for unknown type', () => {
    expect(getTypeIcon('unknown-type' as PluginType)).toBe('📦');
  });
});

describe('togglePlugin action', () => {
  beforeEach(() => {
    installedPlugins.set([
      createTestPlugin({ id: 'plugin-1', name: 'Plugin 1', icon: '🔌', enabled: true }),
      createTestPlugin({ id: 'plugin-2', name: 'Plugin 2', icon: '🔧', enabled: false }),
    ]);
  });

  it('toggles enabled plugin to disabled', () => {
    togglePlugin('plugin-1');
    
    const plugins = get(installedPlugins);
    const plugin = plugins.find(p => p.id === 'plugin-1');
    expect(plugin?.enabled).toBe(false);
  });

  it('toggles disabled plugin to enabled', () => {
    togglePlugin('plugin-2');
    
    const plugins = get(installedPlugins);
    const plugin = plugins.find(p => p.id === 'plugin-2');
    expect(plugin?.enabled).toBe(true);
  });

  it('does not affect other plugins', () => {
    togglePlugin('plugin-1');
    
    const plugins = get(installedPlugins);
    const plugin2 = plugins.find(p => p.id === 'plugin-2');
    expect(plugin2?.enabled).toBe(false);
  });

  it('does nothing for non-existent plugin', () => {
    const before = get(installedPlugins);
    togglePlugin('non-existent');
    const after = get(installedPlugins);
    
    expect(after).toEqual(before);
  });

  it('can toggle same plugin multiple times', () => {
    togglePlugin('plugin-1');
    expect(get(installedPlugins).find(p => p.id === 'plugin-1')?.enabled).toBe(false);
    
    togglePlugin('plugin-1');
    expect(get(installedPlugins).find(p => p.id === 'plugin-1')?.enabled).toBe(true);
    
    togglePlugin('plugin-1');
    expect(get(installedPlugins).find(p => p.id === 'plugin-1')?.enabled).toBe(false);
  });
});

describe('installPlugin action', () => {
  beforeEach(() => {
    installedPlugins.set([]);
  });

  it('installs new plugin to empty store', () => {
    const newPlugin = createTestPlugin({ id: 'new-plugin', name: 'New Plugin', icon: '🆕' });
    
    installPlugin(newPlugin);
    
    const plugins = get(installedPlugins);
    expect(plugins).toHaveLength(1);
    expect(plugins[0].id).toBe('new-plugin');
    expect(plugins[0].installed).toBe(true);
    expect(plugins[0].enabled).toBe(true);
  });

  it('installs new plugin to existing store', () => {
    installedPlugins.set([
      createTestPlugin({ id: 'existing', name: 'Existing', icon: '📦' })
    ]);
    
    const newPlugin = createTestPlugin({ id: 'new-plugin', name: 'New Plugin', icon: '🆕' });
    installPlugin(newPlugin);
    
    const plugins = get(installedPlugins);
    expect(plugins).toHaveLength(2);
    expect(plugins[1].id).toBe('new-plugin');
  });

  it('updates existing plugin if already installed', () => {
    installedPlugins.set([
      createTestPlugin({ id: 'plugin-1', name: 'Old Name', icon: '📦', enabled: false })
    ]);
    
    const updatedPlugin = createTestPlugin({ id: 'plugin-1', name: 'New Name', icon: '🆕' });
    installPlugin(updatedPlugin);
    
    const plugins = get(installedPlugins);
    expect(plugins).toHaveLength(1);
    expect(plugins[0].name).toBe('New Name');
    expect(plugins[0].installed).toBe(true);
    expect(plugins[0].enabled).toBe(true);
  });

  it('sets installed and enabled flags on install', () => {
    const plugin = createTestPlugin({ 
      id: 'test', 
      name: 'Test', 
      icon: '🧪',
      installed: false,
      enabled: false 
    });
    
    installPlugin(plugin);
    
    const installed = get(installedPlugins).find(p => p.id === 'test');
    expect(installed?.installed).toBe(true);
    expect(installed?.enabled).toBe(true);
  });
});

describe('uninstallPlugin action', () => {
  beforeEach(() => {
    installedPlugins.set([
      createTestPlugin({ id: 'plugin-1', name: 'Plugin 1', icon: '🔌' }),
      createTestPlugin({ id: 'plugin-2', name: 'Plugin 2', icon: '🔧' }),
      createTestPlugin({ id: 'plugin-3', name: 'Plugin 3', icon: '⚙️' }),
    ]);
  });

  it('removes plugin by id', () => {
    uninstallPlugin('plugin-2');
    
    const plugins = get(installedPlugins);
    expect(plugins).toHaveLength(2);
    expect(plugins.find(p => p.id === 'plugin-2')).toBeUndefined();
  });

  it('keeps other plugins intact', () => {
    uninstallPlugin('plugin-2');
    
    const plugins = get(installedPlugins);
    expect(plugins.find(p => p.id === 'plugin-1')).toBeDefined();
    expect(plugins.find(p => p.id === 'plugin-3')).toBeDefined();
  });

  it('does nothing for non-existent plugin', () => {
    uninstallPlugin('non-existent');
    
    const plugins = get(installedPlugins);
    expect(plugins).toHaveLength(3);
  });

  it('can uninstall all plugins one by one', () => {
    uninstallPlugin('plugin-1');
    expect(get(installedPlugins)).toHaveLength(2);
    
    uninstallPlugin('plugin-2');
    expect(get(installedPlugins)).toHaveLength(1);
    
    uninstallPlugin('plugin-3');
    expect(get(installedPlugins)).toHaveLength(0);
  });

  it('handles uninstall from empty store', () => {
    installedPlugins.set([]);
    
    uninstallPlugin('any-id');
    
    expect(get(installedPlugins)).toEqual([]);
  });
});

describe('plugin type and level consistency', () => {
  it('all plugin types have corresponding labels', () => {
    const types: PluginType[] = [
      'service-checker',
      'hostlist-provider',
      'strategy-config',
      'ui-plugin',
      'script-plugin'
    ];
    
    types.forEach(type => {
      expect(getTypeLabel(type)).not.toBe('Unknown');
    });
  });

  it('all plugin types have corresponding icons', () => {
    const types: PluginType[] = [
      'service-checker',
      'hostlist-provider',
      'strategy-config',
      'ui-plugin',
      'script-plugin'
    ];
    
    types.forEach(type => {
      expect(getTypeIcon(type)).not.toBe('📦');
    });
  });

  it('all levels have corresponding labels', () => {
    const levels: PluginLevel[] = [1, 2, 3];
    
    levels.forEach(level => {
      expect(getLevelLabel(level)).not.toBe('Unknown');
    });
  });

  it('all levels have corresponding colors', () => {
    const levels: PluginLevel[] = [1, 2, 3];
    
    levels.forEach(level => {
      expect(getLevelColor(level)).not.toContain('zinc');
    });
  });

  it('getPluginLevel returns valid levels for all types', () => {
    const types: PluginType[] = [
      'service-checker',
      'hostlist-provider',
      'strategy-config',
      'ui-plugin',
      'script-plugin'
    ];
    
    types.forEach(type => {
      const level = getPluginLevel(type);
      expect([1, 2, 3]).toContain(level);
    });
  });
});

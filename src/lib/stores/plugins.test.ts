import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { installedPlugins, type PluginInfo } from './plugins';

describe('installedPlugins store', () => {
  const defaultPlugins: PluginInfo[] = [
    { id: 'discord-fix', name: 'Discord Fix', icon: '🎮', route: '/plugins/discord' },
    { id: 'speed-test', name: 'Speed Test', icon: '⚡', route: '/plugins/speed' },
  ];

  beforeEach(() => {
    // Reset to default state
    installedPlugins.set(defaultPlugins);
  });

  describe('initial state', () => {
    it('has default plugins', () => {
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(2);
      expect(plugins[0].id).toBe('discord-fix');
      expect(plugins[1].id).toBe('speed-test');
    });

    it('plugins have correct structure', () => {
      const plugins = get(installedPlugins);
      
      expect(plugins[0]).toEqual({
        id: 'discord-fix',
        name: 'Discord Fix',
        icon: '🎮',
        route: '/plugins/discord'
      });
    });
  });

  describe('set method', () => {
    it('replaces all plugins', () => {
      const newPlugins: PluginInfo[] = [
        { id: 'new-plugin', name: 'New Plugin', icon: '🆕' }
      ];
      
      installedPlugins.set(newPlugins);
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(1);
      expect(plugins[0].id).toBe('new-plugin');
    });

    it('can set empty array', () => {
      installedPlugins.set([]);
      
      expect(get(installedPlugins)).toEqual([]);
    });
  });

  describe('update method', () => {
    it('adds new plugin', () => {
      installedPlugins.update(plugins => [
        ...plugins,
        { id: 'new-plugin', name: 'New Plugin', icon: '🆕', route: '/plugins/new' }
      ]);
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(3);
      expect(plugins[2].id).toBe('new-plugin');
    });

    it('removes plugin by id', () => {
      installedPlugins.update(plugins => 
        plugins.filter(p => p.id !== 'discord-fix')
      );
      
      const plugins = get(installedPlugins);
      expect(plugins).toHaveLength(1);
      expect(plugins[0].id).toBe('speed-test');
    });

    it('updates existing plugin', () => {
      installedPlugins.update(plugins => 
        plugins.map(p => 
          p.id === 'discord-fix' 
            ? { ...p, name: 'Updated Discord Fix' }
            : p
        )
      );
      
      const plugins = get(installedPlugins);
      const discordPlugin = plugins.find(p => p.id === 'discord-fix');
      expect(discordPlugin?.name).toBe('Updated Discord Fix');
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
        { id: 'test', name: 'Test', icon: '🧪' }
      ]);
      
      expect(callback).toHaveBeenCalledTimes(2);
      
      unsubscribe();
    });

    it('stops notifying after unsubscribe', () => {
      const callback = vi.fn();
      const unsubscribe = installedPlugins.subscribe(callback);
      
      unsubscribe();
      
      installedPlugins.set([]);
      
      // Should still be 1 (only initial call)
      expect(callback).toHaveBeenCalledTimes(1);
    });
  });

  describe('plugin without route', () => {
    it('allows plugins without route property', () => {
      installedPlugins.set([
        { id: 'no-route', name: 'No Route Plugin', icon: '📦' }
      ]);
      
      const plugins = get(installedPlugins);
      expect(plugins[0].route).toBeUndefined();
    });
  });
});

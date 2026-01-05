import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { pluginSlots, type PluginSlotEntry, type PluginSlots } from './pluginSlots';

// Mock Svelte component
const MockComponent = {} as any;
const MockComponent2 = {} as any;

describe('pluginSlots store', () => {
  beforeEach(() => {
    // Reset all slots to empty
    // Since there's no reset method, we unregister all known entries
    const slots = get(pluginSlots);
    Object.keys(slots).forEach(slotName => {
      slots[slotName as keyof PluginSlots].forEach(entry => {
        pluginSlots.unregister(slotName as keyof PluginSlots, entry.pluginId);
      });
    });
  });

  describe('initial state', () => {
    it('has all slot types initialized as empty arrays', () => {
      const slots = get(pluginSlots);
      
      expect(slots).toHaveProperty('sidebar');
      expect(slots).toHaveProperty('dashboard');
      expect(slots).toHaveProperty('settings');
      expect(slots).toHaveProperty('contextMenu');
      
      expect(slots.sidebar).toEqual([]);
      expect(slots.dashboard).toEqual([]);
      expect(slots.settings).toEqual([]);
      expect(slots.contextMenu).toEqual([]);
    });
  });

  describe('register method', () => {
    it('adds entry to sidebar slot', () => {
      const entry: PluginSlotEntry = {
        pluginId: 'test-plugin',
        component: MockComponent,
        props: { foo: 'bar' }
      };
      
      pluginSlots.register('sidebar', entry);
      
      const slots = get(pluginSlots);
      expect(slots.sidebar).toHaveLength(1);
      expect(slots.sidebar[0]).toEqual(entry);
    });

    it('adds entry to dashboard slot', () => {
      const entry: PluginSlotEntry = {
        pluginId: 'dashboard-plugin',
        component: MockComponent
      };
      
      pluginSlots.register('dashboard', entry);
      
      const slots = get(pluginSlots);
      expect(slots.dashboard).toHaveLength(1);
      expect(slots.dashboard[0].pluginId).toBe('dashboard-plugin');
    });

    it('adds entry to settings slot', () => {
      const entry: PluginSlotEntry = {
        pluginId: 'settings-plugin',
        component: MockComponent
      };
      
      pluginSlots.register('settings', entry);
      
      const slots = get(pluginSlots);
      expect(slots.settings).toHaveLength(1);
    });

    it('adds entry to contextMenu slot', () => {
      const entry: PluginSlotEntry = {
        pluginId: 'context-plugin',
        component: MockComponent
      };
      
      pluginSlots.register('contextMenu', entry);
      
      const slots = get(pluginSlots);
      expect(slots.contextMenu).toHaveLength(1);
    });

    it('adds multiple entries to same slot', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-1',
        component: MockComponent
      });
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-2',
        component: MockComponent2
      });
      
      const slots = get(pluginSlots);
      expect(slots.sidebar).toHaveLength(2);
    });

    it('sorts entries by order', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-c',
        component: MockComponent,
        order: 30
      });
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-a',
        component: MockComponent,
        order: 10
      });
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-b',
        component: MockComponent,
        order: 20
      });
      
      const slots = get(pluginSlots);
      expect(slots.sidebar[0].pluginId).toBe('plugin-a');
      expect(slots.sidebar[1].pluginId).toBe('plugin-b');
      expect(slots.sidebar[2].pluginId).toBe('plugin-c');
    });

    it('treats undefined order as 0', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-with-order',
        component: MockComponent,
        order: 10
      });
      pluginSlots.register('sidebar', {
        pluginId: 'plugin-no-order',
        component: MockComponent
        // no order specified
      });
      
      const slots = get(pluginSlots);
      expect(slots.sidebar[0].pluginId).toBe('plugin-no-order');
      expect(slots.sidebar[1].pluginId).toBe('plugin-with-order');
    });
  });

  describe('unregister method', () => {
    it('removes entry by pluginId', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'to-remove',
        component: MockComponent
      });
      pluginSlots.register('sidebar', {
        pluginId: 'to-keep',
        component: MockComponent2
      });
      
      pluginSlots.unregister('sidebar', 'to-remove');
      
      const slots = get(pluginSlots);
      expect(slots.sidebar).toHaveLength(1);
      expect(slots.sidebar[0].pluginId).toBe('to-keep');
    });

    it('does nothing if pluginId not found', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'existing',
        component: MockComponent
      });
      
      pluginSlots.unregister('sidebar', 'non-existing');
      
      const slots = get(pluginSlots);
      expect(slots.sidebar).toHaveLength(1);
    });

    it('only removes from specified slot', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'multi-slot',
        component: MockComponent
      });
      pluginSlots.register('dashboard', {
        pluginId: 'multi-slot',
        component: MockComponent
      });
      
      pluginSlots.unregister('sidebar', 'multi-slot');
      
      const slots = get(pluginSlots);
      expect(slots.sidebar).toHaveLength(0);
      expect(slots.dashboard).toHaveLength(1);
    });
  });

  describe('subscription', () => {
    it('notifies subscribers on register', () => {
      const callback = vi.fn();
      const unsubscribe = pluginSlots.subscribe(callback);
      
      // Initial call
      expect(callback).toHaveBeenCalledTimes(1);
      
      pluginSlots.register('sidebar', {
        pluginId: 'test',
        component: MockComponent
      });
      
      expect(callback).toHaveBeenCalledTimes(2);
      
      unsubscribe();
    });

    it('notifies subscribers on unregister', () => {
      pluginSlots.register('sidebar', {
        pluginId: 'test',
        component: MockComponent
      });
      
      const callback = vi.fn();
      const unsubscribe = pluginSlots.subscribe(callback);
      
      pluginSlots.unregister('sidebar', 'test');
      
      expect(callback).toHaveBeenCalledTimes(2); // initial + unregister
      
      unsubscribe();
    });
  });

  describe('props handling', () => {
    it('stores props with entry', () => {
      const props = { 
        title: 'Test',
        count: 42,
        nested: { key: 'value' }
      };
      
      pluginSlots.register('dashboard', {
        pluginId: 'with-props',
        component: MockComponent,
        props
      });
      
      const slots = get(pluginSlots);
      expect(slots.dashboard[0].props).toEqual(props);
    });

    it('allows entry without props', () => {
      pluginSlots.register('dashboard', {
        pluginId: 'no-props',
        component: MockComponent
      });
      
      const slots = get(pluginSlots);
      expect(slots.dashboard[0].props).toBeUndefined();
    });
  });
});

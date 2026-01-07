/**
 * Unit tests for layout.ts
 * Tests localStorage persistence of UI layout state
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';

// Mock $app/environment before importing the module
vi.mock('$app/environment', () => ({
  browser: true,
}));

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
    get _store() {
      return store;
    },
  };
})();

Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

// Import after mocks are set up
import {
  loadLayoutState,
  saveLayoutState,
  saveSidebarState,
  saveTerminalState,
  saveLastRoute,
  getLastRoute,
  saveWidgetOrder,
  getWidgetOrder,
  isFirstLaunch,
  type LayoutState,
} from './layout';

const STORAGE_KEY = 'isolate-layout';
const WIDGET_ORDER_KEY = 'isolate:dashboard:widget-order';
const DEFAULT_WIDGET_ORDER = ['status', 'health', 'method', 'actions', 'network'];

describe('layout store', () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('loadLayoutState', () => {
    it('should return default state when localStorage is empty', () => {
      const state = loadLayoutState();

      expect(state).toEqual({
        sidebar: { collapsed: false },
        terminal: { isOpen: false, height: 200 },
        navigation: { lastRoute: '/' },
        dashboard: { widgetOrder: DEFAULT_WIDGET_ORDER },
      });
    });

    it('should load and merge stored state with defaults', () => {
      const partialState = {
        sidebar: { collapsed: true },
        terminal: { isOpen: true },
      };
      localStorageMock.setItem(STORAGE_KEY, JSON.stringify(partialState));

      const state = loadLayoutState();

      expect(state.sidebar.collapsed).toBe(true);
      expect(state.terminal.isOpen).toBe(true);
      expect(state.terminal.height).toBe(200); // default value
      expect(state.navigation.lastRoute).toBe('/'); // default value
    });

    it('should handle corrupted JSON gracefully', () => {
      localStorageMock.setItem(STORAGE_KEY, 'invalid-json{');
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const state = loadLayoutState();

      expect(state).toEqual({
        sidebar: { collapsed: false },
        terminal: { isOpen: false, height: 200 },
        navigation: { lastRoute: '/' },
        dashboard: { widgetOrder: DEFAULT_WIDGET_ORDER },
      });
      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });

    it('should migrate old widget order from separate key', () => {
      const customOrder = ['health', 'status', 'actions', 'method', 'network'];
      localStorageMock.setItem(WIDGET_ORDER_KEY, JSON.stringify(customOrder));

      const state = loadLayoutState();

      expect(state.dashboard.widgetOrder).toEqual(customOrder);
    });

    it('should not migrate invalid widget order (missing required widgets)', () => {
      const invalidOrder = ['health', 'status']; // missing 'method' and 'actions'
      localStorageMock.setItem(WIDGET_ORDER_KEY, JSON.stringify(invalidOrder));

      const state = loadLayoutState();

      expect(state.dashboard.widgetOrder).toEqual(DEFAULT_WIDGET_ORDER);
    });

    it('should prefer main storage over old widget order key', () => {
      const mainState = {
        dashboard: { widgetOrder: ['actions', 'method', 'health', 'status'] },
      };
      const oldOrder = ['health', 'status', 'actions', 'method'];
      localStorageMock.setItem(STORAGE_KEY, JSON.stringify(mainState));
      localStorageMock.setItem(WIDGET_ORDER_KEY, JSON.stringify(oldOrder));

      const state = loadLayoutState();

      expect(state.dashboard.widgetOrder).toEqual(['actions', 'method', 'health', 'status']);
    });
  });

  describe('saveLayoutState', () => {
    it('should save partial state merged with current state', () => {
      saveLayoutState({ sidebar: { collapsed: true } });

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.sidebar.collapsed).toBe(true);
      expect(stored.terminal.isOpen).toBe(false);
      expect(stored.terminal.height).toBe(200);
    });

    it('should preserve existing state when saving new values', () => {
      saveLayoutState({ sidebar: { collapsed: true } });
      saveLayoutState({ terminal: { isOpen: true, height: 300 } });

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.sidebar.collapsed).toBe(true);
      expect(stored.terminal.isOpen).toBe(true);
      expect(stored.terminal.height).toBe(300);
    });

    it('should handle localStorage errors gracefully', () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      localStorageMock.setItem.mockImplementationOnce(() => {
        throw new Error('QuotaExceeded');
      });

      // Should not throw
      expect(() => saveLayoutState({ sidebar: { collapsed: true } })).not.toThrow();
      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('saveSidebarState', () => {
    it('should save collapsed state as true', () => {
      saveSidebarState(true);

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.sidebar.collapsed).toBe(true);
    });

    it('should save collapsed state as false', () => {
      saveSidebarState(true);
      saveSidebarState(false);

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.sidebar.collapsed).toBe(false);
    });
  });

  describe('saveTerminalState', () => {
    it('should save terminal isOpen and height', () => {
      saveTerminalState(true, 350);

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.terminal.isOpen).toBe(true);
      expect(stored.terminal.height).toBe(350);
    });

    it('should save terminal closed state', () => {
      saveTerminalState(false, 200);

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.terminal.isOpen).toBe(false);
      expect(stored.terminal.height).toBe(200);
    });
  });

  describe('saveLastRoute', () => {
    it('should save regular route', () => {
      saveLastRoute('/services');

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.navigation.lastRoute).toBe('/services');
    });

    it('should NOT save /onboarding route', () => {
      saveLastRoute('/services');
      saveLastRoute('/onboarding');

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.navigation.lastRoute).toBe('/services');
    });

    it('should NOT save routes starting with /_', () => {
      saveLastRoute('/services');
      saveLastRoute('/_internal');

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.navigation.lastRoute).toBe('/services');
    });

    it('should save nested routes', () => {
      saveLastRoute('/plugins/my-plugin');

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.navigation.lastRoute).toBe('/plugins/my-plugin');
    });
  });

  describe('getLastRoute', () => {
    it('should return "/" when no route is saved', () => {
      const route = getLastRoute();
      expect(route).toBe('/');
    });

    it('should return saved route', () => {
      saveLastRoute('/strategies');

      const route = getLastRoute();
      expect(route).toBe('/strategies');
    });

    it('should return "/" when navigation.lastRoute is empty', () => {
      localStorageMock.setItem(STORAGE_KEY, JSON.stringify({
        navigation: { lastRoute: '' },
      }));

      const route = getLastRoute();
      expect(route).toBe('/');
    });
  });

  describe('saveWidgetOrder', () => {
    it('should save valid widget order', () => {
      const order = ['health', 'status', 'actions', 'method', 'network'];
      saveWidgetOrder(order);

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.dashboard.widgetOrder).toEqual(order);
    });

    it('should also save to old key for backward compatibility', () => {
      const order = ['health', 'status', 'actions', 'method', 'network'];
      saveWidgetOrder(order);

      const oldKeyStored = JSON.parse(localStorageMock._store[WIDGET_ORDER_KEY]);
      expect(oldKeyStored).toEqual(order);
    });

    it('should NOT save invalid order (missing required widgets)', () => {
      const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      const invalidOrder = ['health', 'status']; // missing 'method' and 'actions'

      saveWidgetOrder(invalidOrder);

      expect(localStorageMock._store[STORAGE_KEY]).toBeUndefined();
      expect(consoleSpy).toHaveBeenCalledWith('Invalid widget order, missing required widgets');
      consoleSpy.mockRestore();
    });

    it('should accept order with extra widgets', () => {
      const order = ['status', 'health', 'method', 'actions', 'network', 'custom-widget'];
      saveWidgetOrder(order);

      const stored = JSON.parse(localStorageMock._store[STORAGE_KEY]);
      expect(stored.dashboard.widgetOrder).toEqual(order);
    });
  });

  describe('getWidgetOrder', () => {
    it('should return default order when nothing is saved', () => {
      const order = getWidgetOrder();
      expect(order).toEqual(DEFAULT_WIDGET_ORDER);
    });

    it('should return saved order', () => {
      const customOrder = ['actions', 'method', 'health', 'status', 'network'];
      saveWidgetOrder(customOrder);

      const order = getWidgetOrder();
      expect(order).toEqual(customOrder);
    });

    it('should return default when widgetOrder is empty', () => {
      localStorageMock.setItem(STORAGE_KEY, JSON.stringify({
        dashboard: { widgetOrder: null },
      }));

      const order = getWidgetOrder();
      expect(order).toEqual(DEFAULT_WIDGET_ORDER);
    });
  });

  describe('isFirstLaunch', () => {
    it('should return true when localStorage is empty', () => {
      expect(isFirstLaunch()).toBe(true);
    });

    it('should return false when state exists', () => {
      saveLayoutState({ sidebar: { collapsed: false } });

      expect(isFirstLaunch()).toBe(false);
    });

    it('should return false even with minimal saved state', () => {
      localStorageMock.setItem(STORAGE_KEY, '{}');

      expect(isFirstLaunch()).toBe(false);
    });
  });
});
